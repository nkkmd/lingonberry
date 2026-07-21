use crate::{build_storage_backend_at, StorageRuntimeConfig};
use lingonberry_core::{export_archive, import_archive, StorageBackend};
use lingonberry_indexer::{
    index_rebuild_result_json, rebuild_index, verify_index, IndexConsistencyStatus, IndexSnapshot,
};
use lingonberry_protocol::{to_canonical_json, JsonValue};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn handle_backup(
    config: &StorageRuntimeConfig,
    backend: &impl StorageBackend,
    args: &[String],
) -> Result<(), String> {
    let Some(action) = args.first().map(String::as_str) else {
        return Err("usage: lingonberry-storage backup <create|verify> [archive-dir]".to_string());
    };
    match action {
        "create" => {
            let archive_dir = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
                config
                    .backup_dir
                    .join(format!("archive-{}", unique_nonce()))
            });
            refuse_symlink_path(&archive_dir)?;
            if archive_dir.exists()
                && fs::read_dir(&archive_dir)
                    .map_err(|error| error.to_string())?
                    .next()
                    .is_some()
            {
                return Err(format!(
                    "backup destination is not empty: {}",
                    archive_dir.display()
                ));
            }
            let report =
                export_archive(backend, &archive_dir).map_err(|error| error.to_string())?;
            verify_archive_isolated(config, &archive_dir)?;
            print_json(json_object(vec![
                ("status", JsonValue::String("verified".to_string())),
                (
                    "archiveDir",
                    JsonValue::String(report.archive_dir.to_string_lossy().to_string()),
                ),
                (
                    "recordCount",
                    JsonValue::Number(report.record_count.to_string()),
                ),
            ]));
            Ok(())
        }
        "verify" => {
            let archive_dir = required_path(
                args.get(1),
                "usage: lingonberry-storage backup verify <archive-dir>",
            )?;
            let count = verify_archive_isolated(config, &archive_dir)?;
            print_json(json_object(vec![
                ("status", JsonValue::String("verified".to_string())),
                (
                    "archiveDir",
                    JsonValue::String(archive_dir.to_string_lossy().to_string()),
                ),
                ("recordCount", JsonValue::Number(count.to_string())),
            ]));
            Ok(())
        }
        _ => Err("usage: lingonberry-storage backup <create|verify> [archive-dir]".to_string()),
    }
}

pub fn handle_restore(config: &StorageRuntimeConfig, args: &[String]) -> Result<(), String> {
    let Some(action) = args.first().map(String::as_str) else {
        return Err(
            "usage: lingonberry-storage restore <plan|apply> <archive-dir> <target-dir>"
                .to_string(),
        );
    };
    let archive_dir = required_path(
        args.get(1),
        "usage: lingonberry-storage restore <plan|apply> <archive-dir> <target-dir>",
    )?;
    let target_dir = required_path(
        args.get(2),
        "usage: lingonberry-storage restore <plan|apply> <archive-dir> <target-dir>",
    )?;
    validate_restore_target(config, &target_dir)?;
    match action {
        "plan" => {
            let count = verify_archive_isolated(config, &archive_dir)?;
            print_json(json_object(vec![
                ("status", JsonValue::String("planned".to_string())),
                ("readOnlyTarget", JsonValue::Bool(true)),
                (
                    "archiveDir",
                    JsonValue::String(archive_dir.to_string_lossy().to_string()),
                ),
                (
                    "targetDir",
                    JsonValue::String(target_dir.to_string_lossy().to_string()),
                ),
                ("recordCount", JsonValue::Number(count.to_string())),
            ]));
            Ok(())
        }
        "apply" => {
            ensure_empty_target(&target_dir)?;
            let target = build_storage_backend_at(&target_dir);
            let report =
                import_archive(&target, &archive_dir).map_err(|error| error.to_string())?;
            verify_restored_read_path(&target, report.record_count)?;
            verify_index_consistency(&target, "restored")?;
            print_json(json_object(vec![
                ("status", JsonValue::String("restored".to_string())),
                (
                    "archiveDir",
                    JsonValue::String(archive_dir.to_string_lossy().to_string()),
                ),
                (
                    "targetDir",
                    JsonValue::String(target_dir.to_string_lossy().to_string()),
                ),
                (
                    "recordCount",
                    JsonValue::Number(report.record_count.to_string()),
                ),
                (
                    "duplicateCount",
                    JsonValue::Number(report.duplicate_count.to_string()),
                ),
                ("readVerified", JsonValue::Bool(true)),
            ]));
            Ok(())
        }
        _ => Err(
            "usage: lingonberry-storage restore <plan|apply> <archive-dir> <target-dir>"
                .to_string(),
        ),
    }
}

pub fn handle_index(backend: &impl StorageBackend, args: &[String]) -> Result<(), String> {
    let Some(action) = args.first().map(String::as_str) else {
        return Err("usage: lingonberry-storage index <verify|rebuild>".to_string());
    };
    let result = match action {
        "rebuild" => rebuild_index(backend),
        "verify" => {
            let snapshot =
                IndexSnapshot::from_backend(backend).map_err(|error| error.to_string())?;
            verify_index(backend, snapshot)
        }
        _ => return Err("usage: lingonberry-storage index <verify|rebuild>".to_string()),
    };
    print_json(index_rebuild_result_json(&result));
    if result.status == IndexConsistencyStatus::Consistent {
        Ok(())
    } else {
        Err(format!("index operation failed: {}", result.message))
    }
}

pub fn handle_drill(config: &StorageRuntimeConfig, args: &[String]) -> Result<(), String> {
    if args.first().map(String::as_str) != Some("restore") {
        return Err("usage: lingonberry-storage drill restore <archive-dir>".to_string());
    }
    let archive_dir = required_path(
        args.get(1),
        "usage: lingonberry-storage drill restore <archive-dir>",
    )?;
    let count = verify_archive_isolated(config, &archive_dir)?;
    print_json(json_object(vec![
        ("status", JsonValue::String("passed".to_string())),
        ("isolated", JsonValue::Bool(true)),
        ("readVerified", JsonValue::Bool(true)),
        ("writeVerified", JsonValue::Bool(true)),
        ("cleanupVerified", JsonValue::Bool(true)),
        (
            "archiveDir",
            JsonValue::String(archive_dir.to_string_lossy().to_string()),
        ),
        ("recordCount", JsonValue::Number(count.to_string())),
    ]));
    Ok(())
}

fn verify_archive_isolated(
    config: &StorageRuntimeConfig,
    archive_dir: &Path,
) -> Result<usize, String> {
    refuse_symlink_path(archive_dir)?;
    with_isolated_restore_target(config, |target_dir| {
        let target = build_storage_backend_at(target_dir);
        let report = import_archive(&target, archive_dir).map_err(|error| error.to_string())?;
        verify_restored_read_path(&target, report.record_count)?;
        verify_index_consistency(&target, "isolated restore")?;

        let before_ids = target.list_ids().map_err(|error| error.to_string())?;
        let second_import =
            import_archive(&target, archive_dir).map_err(|error| error.to_string())?;
        let after_ids = target.list_ids().map_err(|error| error.to_string())?;
        if second_import.record_count != 0
            || second_import.duplicate_count != before_ids.len()
            || before_ids != after_ids
        {
            return Err(
                "isolated restore write verification failed: duplicate-safe re-import changed logical storage"
                    .to_string(),
            );
        }
        verify_index_consistency(&target, "isolated restore re-import")?;
        Ok(report.record_count + report.duplicate_count)
    })
}

fn with_isolated_restore_target<T>(
    config: &StorageRuntimeConfig,
    operation: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<T, String> {
    let target_dir = config
        .temp_dir
        .join(format!("restore-drill-{}", unique_nonce()));
    if target_dir.exists() {
        return Err(format!(
            "isolated restore target already exists: {}",
            target_dir.display()
        ));
    }
    fs::create_dir_all(&target_dir).map_err(|error| error.to_string())?;
    let outcome = operation(&target_dir);
    let cleanup = fs::remove_dir_all(&target_dir).map_err(|error| error.to_string());
    match (outcome, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(format!(
            "{error}; isolated restore cleanup also failed: {cleanup_error}"
        )),
        (Ok(_), Err(error)) => Err(format!("isolated restore cleanup failed: {error}")),
    }
}

fn verify_restored_read_path(
    backend: &impl StorageBackend,
    expected_record_count: usize,
) -> Result<(), String> {
    let ids = backend.list_ids().map_err(|error| error.to_string())?;
    if ids.len() != expected_record_count {
        return Err(format!(
            "restored read verification failed: expected {expected_record_count} records, found {}",
            ids.len()
        ));
    }
    for canonical_id in ids {
        let record = backend
            .get(&canonical_id)
            .map_err(|error| error.to_string())?;
        if record.is_none() {
            return Err(format!(
                "restored read verification failed: record is not retrievable: {canonical_id}"
            ));
        }
    }
    Ok(())
}

fn verify_index_consistency(
    backend: &impl StorageBackend,
    operation_name: &str,
) -> Result<(), String> {
    let verification = rebuild_index(backend);
    if verification.status != IndexConsistencyStatus::Consistent {
        return Err(format!(
            "{operation_name} index verification failed: {}",
            verification.message
        ));
    }
    Ok(())
}

fn validate_restore_target(config: &StorageRuntimeConfig, target: &Path) -> Result<(), String> {
    refuse_symlink_path(target)?;
    if target == config.data_dir || target == config.state_dir {
        return Err(
            "restore target must be isolated from the active state and data directories"
                .to_string(),
        );
    }
    Ok(())
}

fn ensure_empty_target(target: &Path) -> Result<(), String> {
    if target.exists() {
        let metadata = fs::symlink_metadata(target).map_err(|error| error.to_string())?;
        if !metadata.is_dir() {
            return Err(format!(
                "restore target is not a directory: {}",
                target.display()
            ));
        }
        if fs::read_dir(target)
            .map_err(|error| error.to_string())?
            .next()
            .is_some()
        {
            return Err(format!("restore target is not empty: {}", target.display()));
        }
    } else {
        fs::create_dir_all(target).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn refuse_symlink_path(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("refusing symbolic link path: {}", path.display()))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot inspect {}: {error}", path.display())),
    }
}

fn required_path(value: Option<&String>, usage: &str) -> Result<PathBuf, String> {
    value.map(PathBuf::from).ok_or_else(|| usage.to_string())
}

fn unique_nonce() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn print_json(value: JsonValue) {
    println!("{}", to_canonical_json(&value));
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(root: &Path) -> StorageRuntimeConfig {
        StorageRuntimeConfig {
            config_path: None,
            state_dir: root.join("state"),
            data_dir: root.join("data"),
            backup_dir: root.join("backup"),
            temp_dir: root.join("tmp"),
        }
    }

    #[test]
    fn active_data_directory_cannot_be_a_restore_target() {
        let config = StorageRuntimeConfig {
            config_path: None,
            state_dir: PathBuf::from("state"),
            data_dir: PathBuf::from("data"),
            backup_dir: PathBuf::from("backup"),
            temp_dir: PathBuf::from("tmp"),
        };
        assert!(validate_restore_target(&config, Path::new("data")).is_err());
    }

    #[test]
    fn interrupted_isolated_restore_removes_partial_target() {
        let root = std::env::temp_dir().join(format!("lingonberry-restore-failure-{}", unique_nonce()));
        let config = test_config(&root);
        let observed_target = std::cell::RefCell::new(None::<PathBuf>);
        let result = with_isolated_restore_target(&config, |target| {
            *observed_target.borrow_mut() = Some(target.to_path_buf());
            fs::write(target.join("partial-state"), b"partial").map_err(|error| error.to_string())?;
            Err::<(), _>("injected restore interruption".to_string())
        });
        assert!(result.is_err());
        let target = observed_target.into_inner().expect("target recorded");
        assert!(!target.exists());
        let _ = fs::remove_dir_all(root);
    }
}
