use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{acquire_quarantine_lock, resolve_quarantine_active_path, store_error, StoreError};

pub const QUARANTINE_BACKUP_VERSION: &str = "lingonberry-quarantine-backup/v1";
pub const QUARANTINE_BACKUP_MANIFEST: &str = "quarantine-backup-manifest.json";
pub const QUARANTINE_BACKUP_FILES: [&str; 6] = [
    "quarantine.jsonl",
    "quarantine-resolutions.jsonl",
    "quarantine-annotations.jsonl",
    "quarantine-dismissals.jsonl",
    "quarantine-rejections.jsonl",
    "admin-auth-audit.jsonl",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineBackupFile {
    pub name: String,
    pub present: bool,
    pub bytes: u64,
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineBackupManifest {
    pub version: String,
    pub created_at: String,
    pub source_state_dir: String,
    pub files: Vec<QuarantineBackupFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineBackupReport {
    pub backup_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub present_files: usize,
    pub total_bytes: u64,
}

pub fn export_quarantine_backup(
    state_dir: impl AsRef<Path>,
    backup_dir: impl AsRef<Path>,
) -> Result<QuarantineBackupReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let backup_dir = backup_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-backup-export")?;
    prepare_empty_backup_dir(backup_dir)?;

    let mut entries = Vec::new();
    for name in QUARANTINE_BACKUP_FILES {
        validate_managed_name(name)?;
        let source = resolve_quarantine_active_path(state_dir, name)?;
        let destination = backup_dir.join(name);
        if !source.exists() {
            entries.push(QuarantineBackupFile {
                name: name.to_string(),
                present: false,
                bytes: 0,
                digest: None,
            });
            continue;
        }
        if !source.is_file() {
            return Err(store_error(
                "LB_QUARANTINE_BACKUP",
                format!(
                    "managed state path is not a regular file: {}",
                    source.display()
                ),
            ));
        }
        let before = fs::read(&source)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let before_digest = integrity_digest(&before);
        let temporary = backup_dir.join(format!(".{name}.tmp"));
        fs::write(&temporary, &before)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        fs::rename(&temporary, &destination)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;

        let after = fs::read(&source)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        if before.len() != after.len() || before_digest != integrity_digest(&after) {
            remove_invalid_manifest(backup_dir);
            return Err(store_error(
                "LB_QUARANTINE_BACKUP_CHANGED",
                format!("source changed during backup: {name}"),
            ));
        }
        entries.push(QuarantineBackupFile {
            name: name.to_string(),
            present: true,
            bytes: before.len() as u64,
            digest: Some(before_digest),
        });
    }

    let manifest = QuarantineBackupManifest {
        version: QUARANTINE_BACKUP_VERSION.to_string(),
        created_at: timestamp()?,
        source_state_dir: state_dir.to_string_lossy().to_string(),
        files: entries,
    };
    let manifest_path = backup_dir.join(QUARANTINE_BACKUP_MANIFEST);
    let temporary_manifest = backup_dir.join(format!(".{QUARANTINE_BACKUP_MANIFEST}.tmp"));
    fs::write(
        &temporary_manifest,
        to_canonical_json(&quarantine_backup_manifest_json(&manifest)),
    )
    .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    fs::rename(&temporary_manifest, &manifest_path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;

    verify_quarantine_backup(backup_dir)
}

pub fn verify_quarantine_backup(
    backup_dir: impl AsRef<Path>,
) -> Result<QuarantineBackupReport, StoreError> {
    let backup_dir = backup_dir.as_ref();
    let manifest_path = backup_dir.join(QUARANTINE_BACKUP_MANIFEST);
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|error| {
        store_error(
            "LB_QUARANTINE_BACKUP_INVALID",
            format!("failed to read backup manifest: {error}"),
        )
    })?;
    let manifest = parse_manifest(&manifest_text)?;
    validate_manifest(&manifest)?;

    let listed = manifest
        .files
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<BTreeSet<_>>();
    for directory_entry in fs::read_dir(backup_dir)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?
    {
        let directory_entry =
            directory_entry.map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let name = directory_entry.file_name().to_string_lossy().to_string();
        if name == QUARANTINE_BACKUP_MANIFEST || name.starts_with('.') {
            continue;
        }
        if QUARANTINE_BACKUP_FILES.contains(&name.as_str()) && !listed.contains(name.as_str()) {
            return Err(store_error(
                "LB_QUARANTINE_BACKUP_INVALID",
                format!("unlisted managed file in backup: {name}"),
            ));
        }
    }

    let mut present_files = 0;
    let mut total_bytes = 0;
    for entry in &manifest.files {
        validate_managed_name(&entry.name)?;
        let path = backup_dir.join(&entry.name);
        if !entry.present {
            if path.exists() {
                return Err(store_error(
                    "LB_QUARANTINE_BACKUP_INVALID",
                    format!("manifest marks file absent but it exists: {}", entry.name),
                ));
            }
            continue;
        }
        let bytes = fs::read(&path).map_err(|error| {
            store_error(
                "LB_QUARANTINE_BACKUP_INVALID",
                format!("failed to read backup file {}: {error}", entry.name),
            )
        })?;
        if bytes.len() as u64 != entry.bytes {
            return Err(store_error(
                "LB_QUARANTINE_BACKUP_INVALID",
                format!("byte length mismatch for {}", entry.name),
            ));
        }
        let expected = entry.digest.as_deref().ok_or_else(|| {
            store_error(
                "LB_QUARANTINE_BACKUP_INVALID",
                format!("present file missing digest: {}", entry.name),
            )
        })?;
        if integrity_digest(&bytes) != expected {
            return Err(store_error(
                "LB_QUARANTINE_BACKUP_INVALID",
                format!("integrity digest mismatch for {}", entry.name),
            ));
        }
        present_files += 1;
        total_bytes += entry.bytes;
    }

    Ok(QuarantineBackupReport {
        backup_dir: backup_dir.to_path_buf(),
        manifest_path,
        present_files,
        total_bytes,
    })
}

pub fn restore_quarantine_backup(
    backup_dir: impl AsRef<Path>,
    destination_state_dir: impl AsRef<Path>,
) -> Result<QuarantineBackupReport, StoreError> {
    let backup_dir = backup_dir.as_ref();
    let destination = destination_state_dir.as_ref();
    let verified = verify_quarantine_backup(backup_dir)?;
    let manifest_text = fs::read_to_string(backup_dir.join(QUARANTINE_BACKUP_MANIFEST))
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    let manifest = parse_manifest(&manifest_text)?;

    let _lock = acquire_quarantine_lock(destination, "quarantine-backup-restore")?;
    fs::create_dir_all(destination)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    for name in QUARANTINE_BACKUP_FILES {
        if destination.join(name).exists() {
            return Err(store_error(
                "LB_QUARANTINE_RESTORE_CONFLICT",
                format!("destination already contains managed file: {name}"),
            ));
        }
    }

    for entry in manifest.files.iter().filter(|entry| entry.present) {
        let source = backup_dir.join(&entry.name);
        let target = destination.join(&entry.name);
        let temporary = destination.join(format!(".{}.restore-tmp", entry.name));
        let bytes =
            fs::read(source).map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        fs::write(&temporary, bytes)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        fs::rename(&temporary, &target)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    }

    Ok(QuarantineBackupReport {
        backup_dir: verified.backup_dir,
        manifest_path: verified.manifest_path,
        present_files: verified.present_files,
        total_bytes: verified.total_bytes,
    })
}

pub fn quarantine_backup_report_json(report: &QuarantineBackupReport) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "backupDir".to_string(),
            JsonValue::String(report.backup_dir.to_string_lossy().to_string()),
        ),
        (
            "manifestPath".to_string(),
            JsonValue::String(report.manifest_path.to_string_lossy().to_string()),
        ),
        (
            "presentFiles".to_string(),
            JsonValue::Number(report.present_files.to_string()),
        ),
        (
            "totalBytes".to_string(),
            JsonValue::Number(report.total_bytes.to_string()),
        ),
    ]))
}

pub fn quarantine_backup_manifest_json(manifest: &QuarantineBackupManifest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "createdAt".to_string(),
            JsonValue::String(manifest.created_at.clone()),
        ),
        (
            "files".to_string(),
            JsonValue::Array(
                manifest
                    .files
                    .iter()
                    .map(|entry| {
                        JsonValue::Object(BTreeMap::from([
                            (
                                "bytes".to_string(),
                                JsonValue::Number(entry.bytes.to_string()),
                            ),
                            (
                                "digest".to_string(),
                                entry
                                    .digest
                                    .as_ref()
                                    .map(|value| JsonValue::String(value.clone()))
                                    .unwrap_or(JsonValue::Null),
                            ),
                            ("name".to_string(), JsonValue::String(entry.name.clone())),
                            ("present".to_string(), JsonValue::Bool(entry.present)),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "sourceStateDir".to_string(),
            JsonValue::String(manifest.source_state_dir.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(manifest.version.clone()),
        ),
    ]))
}

fn prepare_empty_backup_dir(path: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(path).map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    if fs::read_dir(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?
        .next()
        .is_some()
    {
        return Err(store_error(
            "LB_QUARANTINE_BACKUP_CONFLICT",
            "backup directory must be empty",
        ));
    }
    Ok(())
}

fn validate_manifest(manifest: &QuarantineBackupManifest) -> Result<(), StoreError> {
    if manifest.version != QUARANTINE_BACKUP_VERSION {
        return Err(invalid(&format!(
            "unsupported backup version: {}",
            manifest.version
        )));
    }
    let names = manifest
        .files
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<BTreeSet<_>>();
    let expected = QUARANTINE_BACKUP_FILES.into_iter().collect::<BTreeSet<_>>();
    if names != expected || manifest.files.len() != QUARANTINE_BACKUP_FILES.len() {
        return Err(invalid(
            "manifest must contain the exact supported managed file set",
        ));
    }
    for entry in &manifest.files {
        validate_managed_name(&entry.name)?;
        if entry.present != entry.digest.is_some() {
            return Err(invalid(&format!(
                "invalid presence/digest combination for {}",
                entry.name
            )));
        }
        if !entry.present && entry.bytes != 0 {
            return Err(invalid(&format!(
                "absent file has non-zero bytes: {}",
                entry.name
            )));
        }
    }
    Ok(())
}

fn validate_managed_name(name: &str) -> Result<(), StoreError> {
    let path = Path::new(name);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || !QUARANTINE_BACKUP_FILES.contains(&name)
    {
        return Err(invalid(&format!("invalid managed file name: {name}")));
    }
    Ok(())
}

fn parse_manifest(text: &str) -> Result<QuarantineBackupManifest, StoreError> {
    let map = object(parse_json(text).map_err(|error| invalid(&error.to_string()))?)?;
    let files = match map.get("files") {
        Some(JsonValue::Array(files)) => files
            .iter()
            .map(|value| {
                let value = object(value.clone())?;
                Ok(QuarantineBackupFile {
                    name: string(&value, "name")?,
                    present: boolean(&value, "present")?,
                    bytes: number(&value, "bytes")?,
                    digest: optional_string(&value, "digest")?,
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?,
        _ => return Err(invalid("manifest missing files")),
    };
    Ok(QuarantineBackupManifest {
        version: string(&map, "version")?,
        created_at: string(&map, "createdAt")?,
        source_state_dir: string(&map, "sourceStateDir")?,
        files,
    })
}

fn object(value: JsonValue) -> Result<BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(invalid("expected JSON object")),
    }
}

fn string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(invalid(&format!("missing string field: {name}"))),
    }
}

fn optional_string(
    map: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<Option<String>, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(invalid(&format!("invalid optional string field: {name}"))),
    }
}

fn boolean(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<bool, StoreError> {
    match map.get(name) {
        Some(JsonValue::Bool(value)) => Ok(*value),
        _ => Err(invalid(&format!("missing boolean field: {name}"))),
    }
}

fn number(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<u64, StoreError> {
    match map.get(name) {
        Some(JsonValue::Number(value)) => value
            .parse()
            .map_err(|_| invalid(&format!("invalid number field: {name}"))),
        _ => Err(invalid(&format!("missing number field: {name}"))),
    }
}

fn invalid(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_BACKUP_INVALID", message)
}

fn remove_invalid_manifest(backup_dir: &Path) {
    let _ = fs::remove_file(backup_dir.join(QUARANTINE_BACKUP_MANIFEST));
    let _ = fs::remove_file(backup_dir.join(format!(".{QUARANTINE_BACKUP_MANIFEST}.tmp")));
}

fn timestamp() -> Result<String, StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    Ok(format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()))
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn exports_verifies_and_restores_sparse_snapshot() {
        let state = temp_dir("backup-state");
        let backup = temp_dir("backup-output");
        let restore = temp_dir("backup-restore");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), "one\n").unwrap();
        let exported = export_quarantine_backup(&state, &backup).unwrap();
        assert_eq!(exported.present_files, 1);
        assert_eq!(verify_quarantine_backup(&backup).unwrap(), exported);
        restore_quarantine_backup(&backup, &restore).unwrap();
        assert_eq!(
            fs::read_to_string(restore.join("quarantine.jsonl")).unwrap(),
            "one\n"
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(restore);
    }

    #[test]
    fn export_and_restore_respect_operation_lock() {
        let state = temp_dir("locked-state");
        let backup = temp_dir("locked-backup");
        let destination = temp_dir("locked-destination");
        fs::create_dir_all(&state).unwrap();
        let source_guard = acquire_quarantine_lock(&state, "test-holder").unwrap();
        assert_eq!(
            export_quarantine_backup(&state, &backup).unwrap_err().code,
            "LB_QUARANTINE_BUSY"
        );
        drop(source_guard);
        export_quarantine_backup(&state, &backup).unwrap();
        let destination_guard = acquire_quarantine_lock(&destination, "test-holder").unwrap();
        assert_eq!(
            restore_quarantine_backup(&backup, &destination)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_BUSY"
        );
        drop(destination_guard);
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(destination);
    }
}
