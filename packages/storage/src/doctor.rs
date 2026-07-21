use crate::{
    inspect_storage, runtime_storage_layout, StorageFormatState, StorageRuntimeConfig,
    MIGRATION_JOURNAL_FILE,
};
use lingonberry_core::resolve_quarantine_active_generation;
use lingonberry_indexer::{verify_index, IndexConsistencyStatus, IndexSnapshot};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorSeverity {
    Ok,
    Warning,
    Failed,
}

impl DoctorSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub name: &'static str,
    pub severity: DoctorSeverity,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub severity: DoctorSeverity,
    pub checks: Vec<DoctorCheck>,
}

impl DoctorReport {
    pub fn has_failures(&self) -> bool {
        self.severity == DoctorSeverity::Failed
    }
}

pub fn run_storage_doctor(config: &StorageRuntimeConfig) -> DoctorReport {
    let layout = runtime_storage_layout(config);
    let checks = vec![
        check_config(config),
        check_directory("state_directory", &config.state_dir),
        check_directory("data_directory", &config.data_dir),
        check_directory("backup_directory", &config.backup_dir),
        check_directory("temporary_directory", &config.temp_dir),
        check_storage_format(config),
        check_migration_journal(config),
        check_regular_file("raw_log", &layout.raw_log_path),
        check_regular_file("catalog", &layout.catalog_path),
        check_generation_pointer(config),
        check_index(config),
        check_backup_inventory(config),
        check_operational_workspace(config),
        check_disk_capacity(config),
    ];

    DoctorReport {
        severity: aggregate_severity(&checks),
        checks,
    }
}

fn check_config(config: &StorageRuntimeConfig) -> DoctorCheck {
    if config.data_dir.as_os_str().is_empty()
        || config.state_dir.as_os_str().is_empty()
        || config.backup_dir.as_os_str().is_empty()
        || config.temp_dir.as_os_str().is_empty()
    {
        return failed(
            "configuration",
            "LB_DOCTOR_INVALID_CONFIG",
            "one or more required directory paths are empty",
        );
    }
    ok(
        "configuration",
        "LB_DOCTOR_CONFIG_OK",
        "effective storage configuration is structurally valid",
    )
}

fn check_directory(name: &'static str, path: &Path) -> DoctorCheck {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => failed(
            name,
            "LB_DOCTOR_SYMLINK_REJECTED",
            format!("{} is a symbolic link", path.display()),
        ),
        Ok(metadata) if !metadata.is_dir() => failed(
            name,
            "LB_DOCTOR_NOT_DIRECTORY",
            format!("{} is not a directory", path.display()),
        ),
        Ok(metadata) if metadata.permissions().readonly() => warning(
            name,
            "LB_DOCTOR_DIRECTORY_READ_ONLY",
            format!("{} is marked read-only", path.display()),
        ),
        Ok(_) => ok(
            name,
            "LB_DOCTOR_DIRECTORY_OK",
            format!("{} is an accessible directory", path.display()),
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => warning(
            name,
            "LB_DOCTOR_DIRECTORY_MISSING",
            format!("{} does not exist", path.display()),
        ),
        Err(error) => failed(
            name,
            "LB_DOCTOR_DIRECTORY_METADATA",
            format!("cannot inspect {}: {error}", path.display()),
        ),
    }
}

fn check_storage_format(config: &StorageRuntimeConfig) -> DoctorCheck {
    match inspect_storage(&config.data_dir) {
        Ok(inspection) => match inspection.state {
            StorageFormatState::Empty => warning(
                "storage_format",
                "LB_DOCTOR_STORAGE_EMPTY",
                "storage is empty and has no current-format manifest",
            ),
            StorageFormatState::LegacyUnversioned { .. } => warning(
                "storage_format",
                "LB_DOCTOR_STORAGE_LEGACY",
                "storage is legacy and requires an explicit migration",
            ),
            StorageFormatState::Supported(manifest) => ok(
                "storage_format",
                "LB_DOCTOR_STORAGE_SUPPORTED",
                format!(
                    "storage format {} with layout {} is supported",
                    manifest.format_version, manifest.layout_id
                ),
            ),
            StorageFormatState::UnknownNewer { format_version } => failed(
                "storage_format",
                "LB_DOCTOR_STORAGE_UNKNOWN_NEWER",
                format!("storage format {format_version} is newer than this binary supports"),
            ),
            StorageFormatState::Corrupt { reason } => {
                failed("storage_format", "LB_DOCTOR_STORAGE_CORRUPT", reason)
            }
        },
        Err(error) => failed("storage_format", "LB_DOCTOR_STORAGE_INSPECTION", error),
    }
}

fn check_migration_journal(config: &StorageRuntimeConfig) -> DoctorCheck {
    let path = config.data_dir.join(MIGRATION_JOURNAL_FILE);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => failed(
            "migration_journal",
            "LB_DOCTOR_JOURNAL_SYMLINK",
            format!("{} is a symbolic link", path.display()),
        ),
        Ok(metadata) if !metadata.is_file() => failed(
            "migration_journal",
            "LB_DOCTOR_JOURNAL_NOT_FILE",
            format!("{} is not a regular file", path.display()),
        ),
        Ok(_) => warning(
            "migration_journal",
            "LB_DOCTOR_JOURNAL_PRESENT",
            "a durable migration journal is present; inspect migration status before normal operation",
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ok(
            "migration_journal",
            "LB_DOCTOR_JOURNAL_ABSENT",
            "no active migration journal is present",
        ),
        Err(error) => failed(
            "migration_journal",
            "LB_DOCTOR_JOURNAL_METADATA",
            format!("cannot inspect {}: {error}", path.display()),
        ),
    }
}

fn check_regular_file(name: &'static str, path: &Path) -> DoctorCheck {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => failed(
            name,
            "LB_DOCTOR_FILE_SYMLINK",
            format!("{} is a symbolic link", path.display()),
        ),
        Ok(metadata) if !metadata.is_file() => failed(
            name,
            "LB_DOCTOR_NOT_REGULAR_FILE",
            format!("{} is not a regular file", path.display()),
        ),
        Ok(_) => ok(
            name,
            "LB_DOCTOR_FILE_OK",
            format!("{} is a regular file", path.display()),
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => warning(
            name,
            "LB_DOCTOR_FILE_MISSING",
            format!("{} does not exist", path.display()),
        ),
        Err(error) => failed(
            name,
            "LB_DOCTOR_FILE_METADATA",
            format!("cannot inspect {}: {error}", path.display()),
        ),
    }
}

fn check_generation_pointer(config: &StorageRuntimeConfig) -> DoctorCheck {
    match resolve_quarantine_active_generation(&config.state_dir) {
        Ok(generation) => match generation.transaction_id {
            Some(transaction_id) => ok(
                "generation_pointer",
                "LB_DOCTOR_GENERATION_POINTER_OK",
                format!(
                    "current quarantine generation {transaction_id} is bound to verified metadata"
                ),
            ),
            None => ok(
                "generation_pointer",
                "LB_DOCTOR_GENERATION_POINTER_LEGACY_ROOT",
                "no generation pointer is present; quarantine uses the state root",
            ),
        },
        Err(error) => failed(
            "generation_pointer",
            "LB_DOCTOR_GENERATION_POINTER_INVALID",
            error.to_string(),
        ),
    }
}

fn check_index(config: &StorageRuntimeConfig) -> DoctorCheck {
    let catalog_path = runtime_storage_layout(config).catalog_path;
    match fs::symlink_metadata(&catalog_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return failed(
                "index",
                "LB_DOCTOR_INDEX_SYMLINK_REJECTED",
                format!("{} is a symbolic link", catalog_path.display()),
            )
        }
        Ok(metadata) if !metadata.is_file() => {
            return failed(
                "index",
                "LB_DOCTOR_INDEX_NOT_REGULAR_FILE",
                format!("{} is not a regular index file", catalog_path.display()),
            )
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return warning(
                "index",
                "LB_DOCTOR_INDEX_MISSING",
                format!("{} does not exist", catalog_path.display()),
            )
        }
        Err(error) => {
            return failed("index", "LB_DOCTOR_INDEX_METADATA", error.to_string())
        }
    }

    let backend = crate::build_storage_backend_at(&config.data_dir);
    let snapshot = match IndexSnapshot::from_backend(&backend) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return failed(
                "index",
                "LB_DOCTOR_INDEX_SNAPSHOT_FAILED",
                error.to_string(),
            )
        }
    };
    let result = verify_index(&backend, snapshot);
    match result.status {
        IndexConsistencyStatus::Consistent => {
            ok("index", "LB_DOCTOR_INDEX_CONSISTENT", result.message)
        }
        _ => failed("index", "LB_DOCTOR_INDEX_INCONSISTENT", result.message),
    }
}

fn check_backup_inventory(config: &StorageRuntimeConfig) -> DoctorCheck {
    let entries = match fs::read_dir(&config.backup_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return warning(
                "backup_inventory",
                "LB_DOCTOR_BACKUP_DIRECTORY_MISSING",
                format!("{} does not exist", config.backup_dir.display()),
            )
        }
        Err(error) => {
            return failed(
                "backup_inventory",
                "LB_DOCTOR_BACKUP_DIRECTORY_UNREADABLE",
                error.to_string(),
            )
        }
    };

    let mut archive_count = 0usize;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                return failed(
                    "backup_inventory",
                    "LB_DOCTOR_BACKUP_ENTRY_UNREADABLE",
                    error.to_string(),
                )
            }
        };
        let metadata = match fs::symlink_metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(error) => {
                return failed(
                    "backup_inventory",
                    "LB_DOCTOR_BACKUP_ENTRY_METADATA",
                    error.to_string(),
                )
            }
        };
        if metadata.file_type().is_symlink() {
            return failed(
                "backup_inventory",
                "LB_DOCTOR_BACKUP_SYMLINK_REJECTED",
                format!("{} is a symbolic link", entry.path().display()),
            );
        }
        if !metadata.is_dir() {
            return failed(
                "backup_inventory",
                "LB_DOCTOR_BACKUP_ENTRY_NOT_DIRECTORY",
                format!("{} is not an archive directory", entry.path().display()),
            );
        }
        let manifest = entry.path().join("manifest.json");
        match fs::symlink_metadata(&manifest) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                archive_count += 1;
            }
            Ok(_) => {
                return failed(
                    "backup_inventory",
                    "LB_DOCTOR_BACKUP_MANIFEST_INVALID",
                    format!("{} is not a regular manifest", manifest.display()),
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return warning(
                    "backup_inventory",
                    "LB_DOCTOR_BACKUP_PARTIAL_ARCHIVE",
                    format!("{} has no manifest.json", entry.path().display()),
                )
            }
            Err(error) => {
                return failed(
                    "backup_inventory",
                    "LB_DOCTOR_BACKUP_MANIFEST_METADATA",
                    error.to_string(),
                )
            }
        }
    }

    if archive_count == 0 {
        warning(
            "backup_inventory",
            "LB_DOCTOR_BACKUP_NONE",
            "no structurally complete backup archive is present",
        )
    } else {
        ok(
            "backup_inventory",
            "LB_DOCTOR_BACKUP_INVENTORY_OK",
            format!("{archive_count} structurally complete backup archive(s) found"),
        )
    }
}

fn check_operational_workspace(config: &StorageRuntimeConfig) -> DoctorCheck {
    let entries = match fs::read_dir(&config.state_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return warning(
                "operational_workspace",
                "LB_DOCTOR_WORKSPACE_STATE_MISSING",
                "state directory does not exist",
            )
        }
        Err(error) => {
            return failed(
                "operational_workspace",
                "LB_DOCTOR_WORKSPACE_UNREADABLE",
                error.to_string(),
            )
        }
    };

    let mut workspace_count = 0usize;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                return failed(
                    "operational_workspace",
                    "LB_DOCTOR_WORKSPACE_ENTRY_UNREADABLE",
                    error.to_string(),
                )
            }
        };
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !(name.contains("replacement")
            || name.contains("cleanup")
            || name.contains("generation")
            || name.contains("migration"))
        {
            continue;
        }
        workspace_count += 1;
        match fs::symlink_metadata(entry.path()) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return failed(
                    "operational_workspace",
                    "LB_DOCTOR_WORKSPACE_SYMLINK_REJECTED",
                    format!("{} is a symbolic link", entry.path().display()),
                )
            }
            Ok(metadata) if !(metadata.is_dir() || metadata.is_file()) => {
                return failed(
                    "operational_workspace",
                    "LB_DOCTOR_WORKSPACE_SPECIAL_FILE",
                    format!("{} is a special file", entry.path().display()),
                )
            }
            Ok(_) => {}
            Err(error) => {
                return failed(
                    "operational_workspace",
                    "LB_DOCTOR_WORKSPACE_METADATA",
                    error.to_string(),
                )
            }
        }
    }

    ok(
        "operational_workspace",
        "LB_DOCTOR_WORKSPACE_STRUCTURAL_OK",
        format!("{workspace_count} maintenance workspace entry or entries inspected"),
    )
}

fn check_disk_capacity(config: &StorageRuntimeConfig) -> DoctorCheck {
    let target = if config.data_dir.exists() {
        &config.data_dir
    } else {
        &config.state_dir
    };
    let output = match Command::new("df").arg("-Pk").arg(target).output() {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            return warning(
                "disk_capacity",
                "LB_DOCTOR_DISK_CAPACITY_UNAVAILABLE",
                format!("df exited with status {}", output.status),
            )
        }
        Err(error) => {
            return warning(
                "disk_capacity",
                "LB_DOCTOR_DISK_CAPACITY_UNAVAILABLE",
                error.to_string(),
            )
        }
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let Some(line) = text.lines().next_back() else {
        return warning(
            "disk_capacity",
            "LB_DOCTOR_DISK_CAPACITY_UNAVAILABLE",
            "df returned no filesystem row",
        );
    };
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 6 {
        return warning(
            "disk_capacity",
            "LB_DOCTOR_DISK_CAPACITY_UNAVAILABLE",
            "df returned an unsupported row",
        );
    }
    let available_kib = match fields[3].parse::<u64>() {
        Ok(value) => value,
        Err(_) => {
            return warning(
                "disk_capacity",
                "LB_DOCTOR_DISK_CAPACITY_UNAVAILABLE",
                "df available capacity is not numeric",
            )
        }
    };
    let capacity_kib = fields[1].parse::<u64>().unwrap_or_default();
    let available_percent = available_kib
        .saturating_mul(100)
        .checked_div(capacity_kib)
        .unwrap_or_default();

    if available_kib < 64 * 1024 || available_percent < 2 {
        failed(
            "disk_capacity",
            "LB_DOCTOR_DISK_CRITICAL",
            format!("only {available_kib} KiB ({available_percent}%) is available"),
        )
    } else if available_kib < 512 * 1024 || available_percent < 10 {
        warning(
            "disk_capacity",
            "LB_DOCTOR_DISK_LOW",
            format!("{available_kib} KiB ({available_percent}%) is available"),
        )
    } else {
        ok(
            "disk_capacity",
            "LB_DOCTOR_DISK_OK",
            format!("{available_kib} KiB ({available_percent}%) is available"),
        )
    }
}

fn aggregate_severity(checks: &[DoctorCheck]) -> DoctorSeverity {
    if checks
        .iter()
        .any(|check| check.severity == DoctorSeverity::Failed)
    {
        DoctorSeverity::Failed
    } else if checks
        .iter()
        .any(|check| check.severity == DoctorSeverity::Warning)
    {
        DoctorSeverity::Warning
    } else {
        DoctorSeverity::Ok
    }
}

fn ok(name: &'static str, code: &'static str, message: impl Into<String>) -> DoctorCheck {
    DoctorCheck {
        name,
        severity: DoctorSeverity::Ok,
        code,
        message: message.into(),
    }
}

fn warning(name: &'static str, code: &'static str, message: impl Into<String>) -> DoctorCheck {
    DoctorCheck {
        name,
        severity: DoctorSeverity::Warning,
        code,
        message: message.into(),
    }
}

fn failed(name: &'static str, code: &'static str, message: impl Into<String>) -> DoctorCheck {
    DoctorCheck {
        name,
        severity: DoctorSeverity::Failed,
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{write_storage_manifest, StorageFormatManifest};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("lingonberry-doctor-{name}-{nonce}"))
    }

    #[test]
    fn doctor_is_read_only_and_reports_current_storage() {
        let root = temporary_directory("supported");
        fs::create_dir_all(&root).expect("create root");
        write_storage_manifest(&root, &StorageFormatManifest::current("doctor-test", None))
            .expect("write manifest");
        let config = StorageRuntimeConfig {
            config_path: None,
            state_dir: root.clone(),
            data_dir: root.clone(),
            backup_dir: root.join("backup"),
            temp_dir: root.join("tmp"),
        };
        let before = fs::read_dir(&root).expect("read before").count();
        let report = run_storage_doctor(&config);
        let after = fs::read_dir(&root).expect("read after").count();
        assert_eq!(before, after);
        assert!(!report.has_failures());
        assert!(report.checks.iter().any(|check| {
            check.name == "storage_format" && check.code == "LB_DOCTOR_STORAGE_SUPPORTED"
        }));
        assert!(report
            .checks
            .iter()
            .any(|check| check.name == "index" && check.code == "LB_DOCTOR_INDEX_MISSING"));
        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn doctor_fails_closed_for_invalid_generation_pointer() {
        let root = temporary_directory("generation");
        fs::create_dir_all(&root).expect("create root");
        fs::write(
            root.join("quarantine-current-generation.json"),
            r#"{"version":"invalid"}"#,
        )
        .expect("write pointer");
        let config = StorageRuntimeConfig {
            config_path: None,
            state_dir: root.clone(),
            data_dir: root.clone(),
            backup_dir: root.join("backup"),
            temp_dir: root.join("tmp"),
        };
        let report = run_storage_doctor(&config);
        assert!(report.has_failures());
        assert!(report.checks.iter().any(|check| {
            check.name == "generation_pointer"
                && check.code == "LB_DOCTOR_GENERATION_POINTER_INVALID"
        }));
        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn doctor_fails_closed_for_symlinked_data_directory() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let root = temporary_directory("symlink");
            let target = root.join("target");
            let linked = root.join("linked");
            fs::create_dir_all(&target).expect("create target");
            symlink(&target, &linked).expect("create symlink");
            let config = StorageRuntimeConfig {
                config_path: None,
                state_dir: root.clone(),
                data_dir: linked,
                backup_dir: root.join("backup"),
                temp_dir: root.join("tmp"),
            };
            let report = run_storage_doctor(&config);
            assert!(report.has_failures());
            assert!(report.checks.iter().any(|check| {
                check.name == "data_directory" && check.code == "LB_DOCTOR_SYMLINK_REJECTED"
            }));
            fs::remove_dir_all(root).expect("remove root");
        }
    }
}
