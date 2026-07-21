use lingonberry_storage::{
    apply_migration, commit_migration, create_verified_migration_backup, inspect_storage,
    prepare_migration, read_migration_journal, resume_migration, rollback_migration,
    runtime_storage_config, MigrationJournal, MigrationPlan, MigrationStage, StorageFormatState,
};
use std::env;
use std::process;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        process::exit(exit_code(&error));
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage());
    };
    let config = runtime_storage_config()?;
    match command {
        "inspect" => {
            let inspection = inspect_storage(&config.data_dir)?;
            println!(
                "state={} inventoryDigest={} entries={}",
                state_name(&inspection.state),
                inspection.inventory_digest,
                inspection.inventory.len()
            );
            Ok(())
        }
        "plan" => {
            let prepared = prepare_migration(&config.data_dir)?;
            print_plan(&prepared.plan);
            Ok(())
        }
        "backup" => {
            let backup =
                create_verified_migration_backup(&config.data_dir, &config.backup_dir)?;
            println!(
                "planId={} backupDir={} evidenceDigest={}",
                backup.plan_id,
                backup.backup_dir.display(),
                backup.evidence_digest
            );
            Ok(())
        }
        "apply" => {
            let journal = apply_migration(&config.data_dir, env!("CARGO_PKG_VERSION"))?;
            print_journal(&journal);
            Ok(())
        }
        "verify" => {
            let journal = read_migration_journal(&config.data_dir)?;
            if journal.stage != MigrationStage::Verified
                && journal.stage != MigrationStage::Committed
            {
                return Err(format!(
                    "migration is not verified: current stage {:?}",
                    journal.stage
                ));
            }
            let inspection = inspect_storage(&config.data_dir)?;
            if inspection.inventory_digest != journal.source_inventory_digest {
                return Err("verified migration inventory no longer matches its source binding"
                    .to_string());
            }
            println!(
                "planId={} stage={:?} inventoryDigest={}",
                journal.plan_id, journal.stage, inspection.inventory_digest
            );
            Ok(())
        }
        "commit" => {
            let journal = commit_migration(&config.data_dir)?;
            print_journal(&journal);
            Ok(())
        }
        "resume" => {
            let journal = resume_migration(&config.data_dir, env!("CARGO_PKG_VERSION"))?;
            print_journal(&journal);
            Ok(())
        }
        "rollback" => {
            let journal = rollback_migration(&config.data_dir)?;
            print_journal(&journal);
            Ok(())
        }
        "status" => {
            let journal = read_migration_journal(&config.data_dir)?;
            print_journal(&journal);
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: lingonberry-storage-migrate <inspect|plan|backup|apply|verify|commit|resume|rollback|status>"
        .to_string()
}

fn print_plan(plan: &MigrationPlan) {
    println!(
        "planId={} sourceInventoryDigest={} sourceFormatVersion={} targetFormatVersion={} requiresVerifiedBackup={} steps={:?}",
        plan.plan_id,
        plan.source_inventory_digest,
        plan.source_format_version
            .map(|version| version.to_string())
            .unwrap_or_else(|| "legacy".to_string()),
        plan.target_format_version,
        plan.requires_verified_backup,
        plan.steps
    );
}

fn print_journal(journal: &MigrationJournal) {
    println!(
        "planId={} stage={:?} sourceInventoryDigest={} targetFormatVersion={} backupEvidenceDigest={}",
        journal.plan_id,
        journal.stage,
        journal.source_inventory_digest,
        journal.target_format_version,
        journal
            .backup_evidence_digest
            .as_deref()
            .unwrap_or("none")
    );
}

fn state_name(state: &StorageFormatState) -> &'static str {
    match state {
        StorageFormatState::Empty => "empty",
        StorageFormatState::LegacyUnversioned { .. } => "legacy_unversioned",
        StorageFormatState::Supported(_) => "supported",
        StorageFormatState::UnknownNewer { .. } => "unknown_newer",
        StorageFormatState::Corrupt { .. } => "corrupt",
    }
}

fn exit_code(error: &str) -> i32 {
    if error.starts_with("usage:") {
        64
    } else if error.contains("not found") {
        66
    } else if error.contains("refusing")
        || error.contains("corrupt")
        || error.contains("unknown newer")
    {
        65
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_names_are_stable() {
        assert_eq!(state_name(&StorageFormatState::Empty), "empty");
        assert_eq!(
            state_name(&StorageFormatState::UnknownNewer { format_version: 2 }),
            "unknown_newer"
        );
    }
}
