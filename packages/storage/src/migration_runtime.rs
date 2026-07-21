use crate::{
    inspect_storage, plan_migration, read_migration_journal, verify_source_binding,
    write_migration_journal, write_storage_manifest, MigrationJournal, MigrationPlan,
    MigrationStage, StorageFormatManifest, StorageFormatState, MIGRATION_JOURNAL_FILE,
    STORAGE_MANIFEST_FILE,
};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationPreparation {
    pub plan: MigrationPlan,
    pub journal: MigrationJournal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedMigrationBackup {
    pub plan_id: String,
    pub backup_dir: PathBuf,
    pub evidence_digest: String,
}

pub fn prepare_migration(data_dir: impl AsRef<Path>) -> Result<MigrationPreparation, String> {
    let data_dir = data_dir.as_ref();
    if data_dir.join(MIGRATION_JOURNAL_FILE).exists() {
        return Err("migration journal already exists; resume or rollback it first".to_string());
    }
    let inspection = inspect_storage(data_dir)?;
    let plan = plan_migration(&inspection)?;
    let journal = MigrationJournal::from_plan(&plan);
    write_migration_journal(data_dir, &journal)?;
    Ok(MigrationPreparation { plan, journal })
}

pub fn create_verified_migration_backup(
    data_dir: impl AsRef<Path>,
    backup_root: impl AsRef<Path>,
) -> Result<VerifiedMigrationBackup, String> {
    let data_dir = data_dir.as_ref();
    let backup_root = backup_root.as_ref();
    let mut journal = read_migration_journal(data_dir)?;
    if journal.stage != MigrationStage::Planned {
        return Err(format!(
            "backup requires planned stage, found {:?}",
            journal.stage
        ));
    }
    verify_source_binding(data_dir, &journal)?;
    let backup_dir = backup_root.join(&journal.plan_id);
    if backup_dir.exists() {
        return Err(format!(
            "migration backup destination already exists: {}",
            backup_dir.display()
        ));
    }
    fs::create_dir_all(&backup_dir).map_err(|error| error.to_string())?;
    if let Err(error) = copy_durable_tree(data_dir, data_dir, &backup_dir) {
        let _ = fs::remove_dir_all(&backup_dir);
        return Err(error);
    }
    sync_tree(&backup_dir)?;
    let source = inspect_storage(data_dir)?;
    let backup = inspect_storage(&backup_dir)?;
    if source.inventory_digest != journal.source_inventory_digest {
        let _ = fs::remove_dir_all(&backup_dir);
        return Err("source storage changed while backup was being created".to_string());
    }
    if backup.inventory_digest != journal.source_inventory_digest {
        let _ = fs::remove_dir_all(&backup_dir);
        return Err(format!(
            "migration backup verification failed: expected {}, found {}",
            journal.source_inventory_digest, backup.inventory_digest
        ));
    }
    let evidence_digest = backup.inventory_digest;
    journal.advance(
        MigrationStage::BackupVerified,
        Some(evidence_digest.clone()),
    )?;
    write_migration_journal(data_dir, &journal)?;
    Ok(VerifiedMigrationBackup {
        plan_id: journal.plan_id,
        backup_dir,
        evidence_digest,
    })
}

pub fn apply_migration(
    data_dir: impl AsRef<Path>,
    created_by: impl Into<String>,
) -> Result<MigrationJournal, String> {
    let data_dir = data_dir.as_ref();
    let mut journal = read_migration_journal(data_dir)?;
    let inspection = inspect_storage(data_dir)?;
    let requires_backup = matches!(
        inspection.state,
        StorageFormatState::LegacyUnversioned { .. }
    );
    match journal.stage {
        MigrationStage::Planned if requires_backup => {
            return Err("migration requires a verified backup before apply".to_string());
        }
        MigrationStage::Planned | MigrationStage::BackupVerified => {}
        MigrationStage::Migrating => return resume_migration(data_dir, created_by),
        MigrationStage::Verified | MigrationStage::Committed => return Ok(journal),
        MigrationStage::RollingBack | MigrationStage::RolledBack => {
            return Err("rolled-back migration cannot be applied".to_string());
        }
    }
    verify_source_binding(data_dir, &journal)?;
    journal.advance(MigrationStage::Migrating, None)?;
    write_migration_journal(data_dir, &journal)?;
    write_storage_manifest(data_dir, &StorageFormatManifest::current(created_by, None))?;
    verify_migrated_storage(data_dir, &journal)?;
    journal.advance(MigrationStage::Verified, None)?;
    write_migration_journal(data_dir, &journal)?;
    Ok(journal)
}

pub fn commit_migration(data_dir: impl AsRef<Path>) -> Result<MigrationJournal, String> {
    let data_dir = data_dir.as_ref();
    let mut journal = read_migration_journal(data_dir)?;
    if journal.stage == MigrationStage::Committed {
        return Ok(journal);
    }
    if journal.stage != MigrationStage::Verified {
        return Err(format!(
            "migration commit requires verified stage, found {:?}",
            journal.stage
        ));
    }
    verify_migrated_storage(data_dir, &journal)?;
    journal.advance(MigrationStage::Committed, None)?;
    write_migration_journal(data_dir, &journal)?;
    Ok(journal)
}

pub fn resume_migration(
    data_dir: impl AsRef<Path>,
    created_by: impl Into<String>,
) -> Result<MigrationJournal, String> {
    let data_dir = data_dir.as_ref();
    let created_by = created_by.into();
    let journal = read_migration_journal(data_dir)?;
    match journal.stage {
        MigrationStage::Planned | MigrationStage::BackupVerified => {
            apply_migration(data_dir, created_by)
        }
        MigrationStage::Migrating => {
            if !data_dir.join(STORAGE_MANIFEST_FILE).exists() {
                write_storage_manifest(
                    data_dir,
                    &StorageFormatManifest::current(created_by, None),
                )?;
            }
            verify_migrated_storage(data_dir, &journal)?;
            let mut resumed = journal;
            resumed.advance(MigrationStage::Verified, None)?;
            write_migration_journal(data_dir, &resumed)?;
            Ok(resumed)
        }
        MigrationStage::Verified => commit_migration(data_dir),
        MigrationStage::Committed => Ok(journal),
        MigrationStage::RollingBack | MigrationStage::RolledBack => {
            Err("rolled-back migration cannot be resumed".to_string())
        }
    }
}

pub fn rollback_migration(data_dir: impl AsRef<Path>) -> Result<MigrationJournal, String> {
    let data_dir = data_dir.as_ref();
    let mut journal = read_migration_journal(data_dir)?;
    if journal.stage == MigrationStage::Committed {
        return Err("committed migration cannot be rolled back".to_string());
    }
    if journal.stage == MigrationStage::RolledBack {
        return Ok(journal);
    }
    if journal.stage != MigrationStage::RollingBack {
        journal.advance(MigrationStage::RollingBack, None)?;
        write_migration_journal(data_dir, &journal)?;
    }
    let manifest_path = data_dir.join(STORAGE_MANIFEST_FILE);
    if manifest_path.exists() {
        fs::remove_file(&manifest_path).map_err(|error| error.to_string())?;
        sync_directory(data_dir)?;
    }
    journal.advance(MigrationStage::RolledBack, None)?;
    write_migration_journal(data_dir, &journal)?;
    Ok(journal)
}

fn verify_migrated_storage(data_dir: &Path, journal: &MigrationJournal) -> Result<(), String> {
    let inspection = inspect_storage(data_dir)?;
    if inspection.inventory_digest != journal.source_inventory_digest {
        return Err(format!(
            "migrated storage inventory changed: expected {}, found {}",
            journal.source_inventory_digest, inspection.inventory_digest
        ));
    }
    match inspection.state {
        StorageFormatState::Supported(manifest)
            if manifest.format_version == journal.target_format_version =>
        {
            Ok(())
        }
        other => Err(format!(
            "migration verification did not find the target storage format: {:?}",
            other
        )),
    }
}

fn copy_durable_tree(root: &Path, current: &Path, destination: &Path) -> Result<(), String> {
    let mut entries = fs::read_dir(current)
        .map_err(|error| format!("failed to read {}: {error}", current.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let source = entry.path();
        let relative = source
            .strip_prefix(root)
            .map_err(|error| error.to_string())?;
        if relative == Path::new(STORAGE_MANIFEST_FILE)
            || relative == Path::new(MIGRATION_JOURNAL_FILE)
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&source).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symlink is not allowed in migration backup: {}",
                relative.display()
            ));
        }
        let target = destination.join(relative);
        if metadata.is_dir() {
            fs::create_dir_all(&target).map_err(|error| error.to_string())?;
            copy_durable_tree(root, &source, destination)?;
        } else if metadata.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            fs::copy(&source, &target).map_err(|error| error.to_string())?;
            File::open(&target)
                .and_then(|file| file.sync_all())
                .map_err(|error| error.to_string())?;
        } else {
            return Err(format!(
                "unsupported entry in migration backup: {}",
                relative.display()
            ));
        }
    }
    Ok(())
}

fn sync_tree(root: &Path) -> Result<(), String> {
    let mut directories = vec![root.to_path_buf()];
    let mut index = 0;
    while index < directories.len() {
        let current = directories[index].clone();
        index += 1;
        for entry in fs::read_dir(&current).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();
            if path.is_dir() {
                directories.push(path);
            }
        }
    }
    for directory in directories.into_iter().rev() {
        sync_directory(&directory)?;
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), String> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("lingonberry-{name}-{nonce}"))
    }

    #[test]
    fn legacy_upgrade_requires_verified_backup_and_commits() {
        let data_dir = temp_dir("migration-runtime");
        let backup_root = temp_dir("migration-backup");
        fs::create_dir_all(&data_dir).expect("create data dir");
        fs::write(data_dir.join("relay-wire-log.jsonl"), b"legacy\n").expect("write fixture");
        let prepared = prepare_migration(&data_dir).expect("prepare");
        assert!(prepared.plan.requires_verified_backup);
        assert!(apply_migration(&data_dir, "test").is_err());
        let backup = create_verified_migration_backup(&data_dir, &backup_root).expect("backup");
        assert_eq!(
            backup.evidence_digest,
            prepared.plan.source_inventory_digest
        );
        let verified = apply_migration(&data_dir, "0.7.0").expect("apply");
        assert_eq!(verified.stage, MigrationStage::Verified);
        let committed = commit_migration(&data_dir).expect("commit");
        assert_eq!(committed.stage, MigrationStage::Committed);
        assert!(matches!(
            inspect_storage(&data_dir).expect("inspect").state,
            StorageFormatState::Supported(_)
        ));
        let _ = fs::remove_dir_all(data_dir);
        let _ = fs::remove_dir_all(backup_root);
    }

    #[test]
    fn interrupted_migration_resumes_and_rollback_is_idempotent() {
        let data_dir = temp_dir("migration-resume");
        fs::create_dir_all(&data_dir).expect("create data dir");
        let prepared = prepare_migration(&data_dir).expect("prepare");
        assert!(!prepared.plan.requires_verified_backup);
        let mut journal = prepared.journal;
        journal
            .advance(MigrationStage::Migrating, None)
            .expect("advance");
        write_migration_journal(&data_dir, &journal).expect("journal");
        let resumed = resume_migration(&data_dir, "0.7.0").expect("resume");
        assert_eq!(resumed.stage, MigrationStage::Verified);
        let rolled_back = rollback_migration(&data_dir).expect("rollback");
        assert_eq!(rolled_back.stage, MigrationStage::RolledBack);
        assert_eq!(
            rollback_migration(&data_dir)
                .expect("repeat rollback")
                .stage,
            MigrationStage::RolledBack
        );
        assert!(!data_dir.join(STORAGE_MANIFEST_FILE).exists());
        let _ = fs::remove_dir_all(data_dir);
    }
}
