mod doctor;
mod migration;
mod migration_runtime;

pub use doctor::{run_storage_doctor, DoctorCheck, DoctorReport, DoctorSeverity};
pub use migration::{
    inspect_storage, plan_migration, read_migration_journal, verify_source_binding,
    write_migration_journal, write_storage_manifest, MigrationJournal, MigrationPlan,
    MigrationStage, MigrationStep, StorageFormatManifest, StorageFormatState, StorageInspection,
    CURRENT_LAYOUT_ID, CURRENT_STORAGE_FORMAT_VERSION, MIGRATION_JOURNAL_FILE,
    STORAGE_MANIFEST_FILE,
};
pub use migration_runtime::{
    apply_migration, commit_migration, create_verified_migration_backup, prepare_migration,
    resume_migration, rollback_migration, MigrationPreparation, VerifiedMigrationBackup,
};

use lingonberry_core::{runtime_state_dir, SqliteStorageBackend};
use lingonberry_protocol::{read_json_file, JsonValue};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

pub const STORAGE_CONFIG_PRECEDENCE: [&str; 4] = ["defaults", "config_file", "environment", "cli"];

#[derive(Debug, Clone)]
pub struct StorageRuntimeConfig {
    pub config_path: Option<PathBuf>,
    pub state_dir: PathBuf,
    pub data_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub temp_dir: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageRuntimeConfigOverrides {
    pub config_path: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub backup_dir: Option<PathBuf>,
    pub temp_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct StorageRuntimeLayout {
    pub raw_log_path: PathBuf,
    pub catalog_path: PathBuf,
}

pub fn build_storage_backend() -> SqliteStorageBackend {
    SqliteStorageBackend::new(runtime_storage_config().expect("storage config").data_dir)
}

pub fn build_storage_backend_at(base_dir: impl AsRef<Path>) -> SqliteStorageBackend {
    SqliteStorageBackend::new(base_dir)
}

pub fn runtime_storage_config() -> Result<StorageRuntimeConfig, String> {
    runtime_storage_config_with_overrides(&StorageRuntimeConfigOverrides::default())
}

pub fn runtime_storage_config_with_overrides(
    cli: &StorageRuntimeConfigOverrides,
) -> Result<StorageRuntimeConfig, String> {
    validate_overrides(cli, "CLI")?;
    let default_state_dir = runtime_state_dir();
    let environment = environment_overrides()?;
    let explicit_config_path = cli
        .config_path
        .clone()
        .or_else(|| environment.config_path.clone());
    let config_path = explicit_config_path
        .clone()
        .unwrap_or_else(|| default_state_dir.join("storage-config.json"));
    let mut config = StorageRuntimeConfig {
        config_path: Some(config_path.clone()),
        state_dir: default_state_dir.clone(),
        data_dir: default_state_dir.clone(),
        backup_dir: default_state_dir.join("backup"),
        temp_dir: default_state_dir.join("tmp"),
    };
    let mut assigned = StorageConfigAssignments::default();

    if config_path.exists() {
        let loaded = read_json_file(&config_path)?;
        assigned.merge(apply_storage_config(&mut config, &loaded.value)?);
    } else if explicit_config_path.is_some() {
        return Err(format!(
            "storage config file not found: {}",
            config_path.display()
        ));
    }

    assigned.merge(apply_overrides(&mut config, &environment));
    assigned.merge(apply_overrides(&mut config, cli));

    if !assigned.state_dir {
        config.state_dir = default_state_dir;
    }
    if !assigned.data_dir {
        config.data_dir = config.state_dir.clone();
    }
    if !assigned.backup_dir {
        config.backup_dir = config.state_dir.join("backup");
    }
    if !assigned.temp_dir {
        config.temp_dir = config.state_dir.join("tmp");
    }
    Ok(config)
}

pub fn runtime_storage_layout(config: &StorageRuntimeConfig) -> StorageRuntimeLayout {
    StorageRuntimeLayout {
        raw_log_path: config.data_dir.join("relay-wire-log.jsonl"),
        catalog_path: config.data_dir.join("canonical-catalog.sqlite3"),
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct StorageConfigAssignments {
    state_dir: bool,
    data_dir: bool,
    backup_dir: bool,
    temp_dir: bool,
}

impl StorageConfigAssignments {
    fn merge(&mut self, other: Self) {
        self.state_dir |= other.state_dir;
        self.data_dir |= other.data_dir;
        self.backup_dir |= other.backup_dir;
        self.temp_dir |= other.temp_dir;
    }
}

fn environment_overrides() -> Result<StorageRuntimeConfigOverrides, String> {
    let overrides = StorageRuntimeConfigOverrides {
        config_path: environment_path("LINGONBERRY_STORAGE_CONFIG")?,
        state_dir: environment_path("LINGONBERRY_STORAGE_STATE_DIR")?,
        data_dir: environment_path("LINGONBERRY_STORAGE_DATA_DIR")?,
        backup_dir: environment_path("LINGONBERRY_STORAGE_BACKUP_DIR")?,
        temp_dir: environment_path("LINGONBERRY_STORAGE_TEMP_DIR")?,
    };
    validate_overrides(&overrides, "environment")?;
    Ok(overrides)
}

fn environment_path(name: &str) -> Result<Option<PathBuf>, String> {
    match env::var_os(name) {
        Some(value) if value.is_empty() => Err(format!("{name} must not be empty")),
        Some(value) => Ok(Some(PathBuf::from(value))),
        None => Ok(None),
    }
}

fn validate_overrides(
    overrides: &StorageRuntimeConfigOverrides,
    source: &str,
) -> Result<(), String> {
    for (name, value) in [
        ("config path", overrides.config_path.as_ref()),
        ("state directory", overrides.state_dir.as_ref()),
        ("data directory", overrides.data_dir.as_ref()),
        ("backup directory", overrides.backup_dir.as_ref()),
        ("temporary directory", overrides.temp_dir.as_ref()),
    ] {
        if value.is_some_and(|path| path.as_os_str().is_empty()) {
            return Err(format!("{source} {name} must not be empty"));
        }
    }
    Ok(())
}

fn apply_overrides(
    config: &mut StorageRuntimeConfig,
    overrides: &StorageRuntimeConfigOverrides,
) -> StorageConfigAssignments {
    let mut assigned = StorageConfigAssignments::default();
    if let Some(path) = &overrides.state_dir {
        config.state_dir = path.clone();
        assigned.state_dir = true;
    }
    if let Some(path) = &overrides.data_dir {
        config.data_dir = path.clone();
        assigned.data_dir = true;
    }
    if let Some(path) = &overrides.backup_dir {
        config.backup_dir = path.clone();
        assigned.backup_dir = true;
    }
    if let Some(path) = &overrides.temp_dir {
        config.temp_dir = path.clone();
        assigned.temp_dir = true;
    }
    assigned
}

fn apply_storage_config(
    config: &mut StorageRuntimeConfig,
    value: &JsonValue,
) -> Result<StorageConfigAssignments, String> {
    let Some(map) = as_object(value) else {
        return Err("storage config must be an object".to_string());
    };
    for key in map.keys() {
        if !matches!(
            key.as_str(),
            "stateDir" | "dataDir" | "backupDir" | "tempDir"
        ) {
            return Err(format!("storage config contains unknown field: {key}"));
        }
    }
    let mut assigned = StorageConfigAssignments::default();
    if let Some(path) = config_path_field(map, "stateDir")? {
        config.state_dir = path;
        assigned.state_dir = true;
    }
    if let Some(path) = config_path_field(map, "dataDir")? {
        config.data_dir = path;
        assigned.data_dir = true;
    }
    if let Some(path) = config_path_field(map, "backupDir")? {
        config.backup_dir = path;
        assigned.backup_dir = true;
    }
    if let Some(path) = config_path_field(map, "tempDir")? {
        config.temp_dir = path;
        assigned.temp_dir = true;
    }
    Ok(assigned)
}

fn config_path_field(
    map: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<Option<PathBuf>, String> {
    match map.get(name) {
        Some(JsonValue::String(value)) if value.is_empty() => {
            Err(format!("storage config field {name} must not be empty"))
        }
        Some(JsonValue::String(value)) => Ok(Some(PathBuf::from(value))),
        Some(_) => Err(format!("storage config field {name} must be a string")),
        None => Ok(None),
    }
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        env::temp_dir().join(format!("lingonberry-config-{name}-{nonce}"))
    }

    #[test]
    fn cli_overrides_config_file_and_derived_paths_follow_effective_state_dir() {
        let root = temporary_directory("precedence");
        fs::create_dir_all(&root).expect("create root");
        let config_path = root.join("storage-config.json");
        fs::write(
            &config_path,
            r#"{"stateDir":"from-file","dataDir":"file-data"}"#,
        )
        .expect("write config");
        let cli_state = root.join("from-cli");
        let config = runtime_storage_config_with_overrides(&StorageRuntimeConfigOverrides {
            config_path: Some(config_path),
            state_dir: Some(cli_state.clone()),
            data_dir: None,
            backup_dir: None,
            temp_dir: None,
        })
        .expect("resolve config");
        assert_eq!(config.state_dir, cli_state.clone());
        assert_eq!(config.data_dir, PathBuf::from("file-data"));
        assert_eq!(config.backup_dir, cli_state.join("backup"));
        assert_eq!(config.temp_dir, cli_state.join("tmp"));
        fs::remove_dir_all(root).expect("remove root");
    }

    #[test]
    fn config_file_rejects_unknown_fields() {
        let root = temporary_directory("unknown-field");
        fs::create_dir_all(&root).expect("create root");
        let config_path = root.join("storage-config.json");
        fs::write(&config_path, r#"{"stateDir":"state","secret":"no"}"#).expect("write config");
        let error = runtime_storage_config_with_overrides(&StorageRuntimeConfigOverrides {
            config_path: Some(config_path),
            ..StorageRuntimeConfigOverrides::default()
        })
        .expect_err("unknown field must fail");
        assert!(error.contains("unknown field"));
        fs::remove_dir_all(root).expect("remove root");
    }
}
