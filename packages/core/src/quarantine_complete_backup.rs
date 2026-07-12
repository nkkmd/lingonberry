use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    acquire_quarantine_lock, restore_quarantine_backup, store_error,
    verify_quarantine_backup, verify_quarantine_segments, QuarantineBackupReport, StoreError,
    QUARANTINE_BACKUP_FILES, QUARANTINE_BACKUP_MANIFEST,
    QUARANTINE_SEGMENT_ARCHIVE_DIR, QUARANTINE_SEGMENT_MANIFEST_FILE,
};

pub const QUARANTINE_COMPLETE_BACKUP_VERSION: &str = "lingonberry-quarantine-backup/v2";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompleteBackupEntry {
    path: String,
    present: bool,
    bytes: u64,
    digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompleteBackupManifest {
    version: String,
    created_at: String,
    source_state_dir: String,
    files: Vec<CompleteBackupEntry>,
}

pub fn export_complete_quarantine_backup(
    state_dir: impl AsRef<Path>,
    backup_dir: impl AsRef<Path>,
) -> Result<QuarantineBackupReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let backup_dir = backup_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-backup-export-v2")?;
    prepare_empty_dir(backup_dir)?;
    verify_quarantine_segments(state_dir)?;

    let mut paths = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let segment_manifest = state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE);
    if segment_manifest.exists() {
        paths.push(QUARANTINE_SEGMENT_MANIFEST_FILE.to_string());
        let archive_dir = state_dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR);
        if archive_dir.exists() {
            let mut archive_files = fs::read_dir(&archive_dir)
                .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?
                .map(|entry| {
                    entry
                        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                })
                .collect::<Result<Vec<_>, StoreError>>()?;
            archive_files.sort();
            for name in archive_files {
                if name.starts_with('.') {
                    continue;
                }
                paths.push(format!("{QUARANTINE_SEGMENT_ARCHIVE_DIR}/{name}"));
            }
        }
    }

    let mut entries = Vec::new();
    for relative in paths {
        validate_relative_path(&relative)?;
        let source = state_dir.join(&relative);
        if !source.exists() {
            entries.push(CompleteBackupEntry {
                path: relative,
                present: false,
                bytes: 0,
                digest: None,
            });
            continue;
        }
        if !source.is_file() {
            return Err(store_error(
                "LB_QUARANTINE_BACKUP",
                format!("backup source is not a regular file: {}", source.display()),
            ));
        }
        let before = fs::read(&source)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let digest = integrity_digest(&before);
        let destination = backup_dir.join(&relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        }
        let temporary = destination.with_extension("backup-tmp");
        fs::write(&temporary, &before)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        fs::rename(&temporary, &destination)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        let after = fs::read(&source)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        if before.len() != after.len() || digest != integrity_digest(&after) {
            return Err(store_error(
                "LB_QUARANTINE_BACKUP_CHANGED",
                format!("source changed during backup: {relative}"),
            ));
        }
        entries.push(CompleteBackupEntry {
            path: relative,
            present: true,
            bytes: before.len() as u64,
            digest: Some(digest),
        });
    }

    let manifest = CompleteBackupManifest {
        version: QUARANTINE_COMPLETE_BACKUP_VERSION.to_string(),
        created_at: timestamp()?,
        source_state_dir: state_dir.to_string_lossy().to_string(),
        files: entries,
    };
    let manifest_path = backup_dir.join(QUARANTINE_BACKUP_MANIFEST);
    let temporary = backup_dir.join(format!(".{QUARANTINE_BACKUP_MANIFEST}.tmp"));
    fs::write(&temporary, to_canonical_json(&manifest_json(&manifest)))
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    fs::rename(&temporary, &manifest_path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    verify_any_quarantine_backup(backup_dir)
}

pub fn verify_any_quarantine_backup(
    backup_dir: impl AsRef<Path>,
) -> Result<QuarantineBackupReport, StoreError> {
    let backup_dir = backup_dir.as_ref();
    let text = fs::read_to_string(backup_dir.join(QUARANTINE_BACKUP_MANIFEST))
        .map_err(|error| store_error("LB_QUARANTINE_BACKUP_INVALID", error.to_string()))?;
    let version = manifest_version(&text)?;
    if version != QUARANTINE_COMPLETE_BACKUP_VERSION {
        return verify_quarantine_backup(backup_dir);
    }
    let manifest = parse_manifest(&text)?;
    validate_manifest(&manifest)?;
    verify_backup_tree(backup_dir, &manifest)?;
    verify_quarantine_segments(backup_dir)?;
    Ok(report(backup_dir, &manifest))
}

pub fn restore_any_quarantine_backup(
    backup_dir: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> Result<QuarantineBackupReport, StoreError> {
    let backup_dir = backup_dir.as_ref();
    let destination = destination.as_ref();
    let text = fs::read_to_string(backup_dir.join(QUARANTINE_BACKUP_MANIFEST))
        .map_err(|error| store_error("LB_QUARANTINE_BACKUP_INVALID", error.to_string()))?;
    let version = manifest_version(&text)?;
    if version != QUARANTINE_COMPLETE_BACKUP_VERSION {
        return restore_quarantine_backup(backup_dir, destination);
    }
    let manifest = parse_manifest(&text)?;
    verify_any_quarantine_backup(backup_dir)?;
    let _lock = acquire_quarantine_lock(destination, "quarantine-backup-restore-v2")?;
    reject_destination_conflicts(destination)?;
    fs::create_dir_all(destination)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;

    let mut written = Vec::new();
    for entry in manifest.files.iter().filter(|entry| entry.present) {
        let source = backup_dir.join(&entry.path);
        let target = destination.join(&entry.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        }
        let temporary = target.with_extension("restore-tmp");
        let bytes = fs::read(&source)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
        if let Err(error) = fs::write(&temporary, bytes)
            .and_then(|_| fs::rename(&temporary, &target))
        {
            rollback_written(&written);
            return Err(store_error("LB_QUARANTINE_IO", error.to_string()));
        }
        written.push(target);
    }
    if let Err(error) = verify_quarantine_segments(destination) {
        rollback_written(&written);
        return Err(error);
    }
    Ok(report(backup_dir, &manifest))
}

fn verify_backup_tree(
    backup_dir: &Path,
    manifest: &CompleteBackupManifest,
) -> Result<(), StoreError> {
    let listed = manifest
        .files
        .iter()
        .filter(|entry| entry.present)
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    for entry in &manifest.files {
        validate_relative_path(&entry.path)?;
        let path = backup_dir.join(&entry.path);
        if !entry.present {
            if path.exists() {
                return Err(invalid(&format!(
                    "manifest marks path absent but it exists: {}",
                    entry.path
                )));
            }
            continue;
        }
        let bytes = fs::read(&path)
            .map_err(|error| invalid(&format!("failed to read {}: {error}", entry.path)))?;
        if bytes.len() as u64 != entry.bytes
            || entry.digest.as_deref() != Some(integrity_digest(&bytes).as_str())
        {
            return Err(invalid(&format!("backup metadata mismatch: {}", entry.path)));
        }
    }
    let archive_dir = backup_dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR);
    if archive_dir.exists() {
        for item in fs::read_dir(&archive_dir)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?
        {
            let item = item
                .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
            let relative = format!(
                "{QUARANTINE_SEGMENT_ARCHIVE_DIR}/{}",
                item.file_name().to_string_lossy()
            );
            if !listed.contains(relative.as_str()) {
                return Err(invalid(&format!("unlisted archive file: {relative}")));
            }
        }
    }
    Ok(())
}

fn validate_manifest(manifest: &CompleteBackupManifest) -> Result<(), StoreError> {
    if manifest.version != QUARANTINE_COMPLETE_BACKUP_VERSION {
        return Err(invalid("unsupported complete backup version"));
    }
    let names = manifest
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    for required in QUARANTINE_BACKUP_FILES {
        if !names.contains(required) {
            return Err(invalid(&format!("missing active ledger entry: {required}")));
        }
    }
    if names.len() != manifest.files.len() {
        return Err(invalid("duplicate backup path"));
    }
    for entry in &manifest.files {
        validate_relative_path(&entry.path)?;
        if entry.present != entry.digest.is_some() {
            return Err(invalid(&format!("invalid digest presence: {}", entry.path)));
        }
        if !entry.present && entry.bytes != 0 {
            return Err(invalid(&format!("absent path has bytes: {}", entry.path)));
        }
        let allowed = QUARANTINE_BACKUP_FILES.contains(&entry.path.as_str())
            || entry.path == QUARANTINE_SEGMENT_MANIFEST_FILE
            || entry
                .path
                .starts_with(&format!("{QUARANTINE_SEGMENT_ARCHIVE_DIR}/"));
        if !allowed {
            return Err(invalid(&format!("unsupported backup path: {}", entry.path)));
        }
    }
    Ok(())
}

fn reject_destination_conflicts(destination: &Path) -> Result<(), StoreError> {
    let conflicts = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| destination.join(name))
        .chain([
            destination.join(QUARANTINE_SEGMENT_MANIFEST_FILE),
            destination.join(QUARANTINE_SEGMENT_ARCHIVE_DIR),
            destination.join("quarantine-ledger-index.json"),
            destination.join(".quarantine-operation.lock"),
        ]);
    for path in conflicts {
        if path.exists() {
            return Err(store_error(
                "LB_QUARANTINE_RESTORE_CONFLICT",
                format!("destination contains conflicting path: {}", path.display()),
            ));
        }
    }
    Ok(())
}

fn rollback_written(paths: &[PathBuf]) {
    for path in paths.iter().rev() {
        let _ = fs::remove_file(path);
    }
}

fn prepare_empty_dir(path: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
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

fn validate_relative_path(path: &str) -> Result<(), StoreError> {
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        return Err(invalid(&format!("invalid backup path: {path}")));
    }
    Ok(())
}

fn manifest_version(text: &str) -> Result<String, StoreError> {
    let map = object(parse_json(text).map_err(|error| invalid(&error.to_string()))?)?;
    string(&map, "version")
}

fn manifest_json(manifest: &CompleteBackupManifest) -> JsonValue {
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
                            ("bytes".to_string(), JsonValue::Number(entry.bytes.to_string())),
                            (
                                "digest".to_string(),
                                entry
                                    .digest
                                    .as_ref()
                                    .map(|value| JsonValue::String(value.clone()))
                                    .unwrap_or(JsonValue::Null),
                            ),
                            ("path".to_string(), JsonValue::String(entry.path.clone())),
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

fn parse_manifest(text: &str) -> Result<CompleteBackupManifest, StoreError> {
    let map = object(parse_json(text).map_err(|error| invalid(&error.to_string()))?)?;
    let files = match map.get("files") {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let map = object(value.clone())?;
                Ok(CompleteBackupEntry {
                    path: string(&map, "path")?,
                    present: boolean(&map, "present")?,
                    bytes: number(&map, "bytes")?,
                    digest: optional_string(&map, "digest")?,
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?,
        _ => return Err(invalid("backup manifest missing files")),
    };
    Ok(CompleteBackupManifest {
        version: string(&map, "version")?,
        created_at: string(&map, "createdAt")?,
        source_state_dir: string(&map, "sourceStateDir")?,
        files,
    })
}

fn report(backup_dir: &Path, manifest: &CompleteBackupManifest) -> QuarantineBackupReport {
    QuarantineBackupReport {
        backup_dir: backup_dir.to_path_buf(),
        manifest_path: backup_dir.join(QUARANTINE_BACKUP_MANIFEST),
        present_files: manifest.files.iter().filter(|entry| entry.present).count(),
        total_bytes: manifest.files.iter().map(|entry| entry.bytes).sum(),
    }
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

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn timestamp() -> Result<String, StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    Ok(format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()))
}

fn invalid(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_BACKUP_INVALID", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_quarantine_ledger_index, read_managed_ledger_lines, rotate_quarantine_ledger,
    };

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn exports_and_restores_rotated_state() {
        let state = temp_dir("complete-backup-state");
        let backup = temp_dir("complete-backup-output");
        let restored = temp_dir("complete-backup-restore");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), "{\"n\":1}\n").unwrap();
        build_quarantine_ledger_index(&state).unwrap();
        rotate_quarantine_ledger(&state, "quarantine.jsonl").unwrap();
        fs::write(state.join("quarantine.jsonl"), "{\"n\":2}\n").unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        restore_any_quarantine_backup(&backup, &restored).unwrap();
        assert_eq!(
            read_managed_ledger_lines(&restored, "quarantine.jsonl").unwrap(),
            vec!["{\"n\":1}", "{\"n\":2}"]
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(restored);
    }

    #[test]
    fn rejects_tampered_archive_segment() {
        let state = temp_dir("tampered-state");
        let backup = temp_dir("tampered-backup");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), "{\"n\":1}\n").unwrap();
        build_quarantine_ledger_index(&state).unwrap();
        rotate_quarantine_ledger(&state, "quarantine.jsonl").unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        let archive = fs::read_dir(backup.join(QUARANTINE_SEGMENT_ARCHIVE_DIR))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        fs::write(archive, "{\"tampered\":true}\n").unwrap();
        assert_eq!(
            verify_any_quarantine_backup(&backup).unwrap_err().code,
            "LB_QUARANTINE_BACKUP_INVALID"
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
    }
}