use lingonberry_core::{runtime_state_dir, SqliteStorageBackend};
use lingonberry_protocol::{read_json_file, JsonValue};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct StorageRuntimeConfig {
    pub config_path: Option<PathBuf>,
    pub state_dir: PathBuf,
    pub data_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub temp_dir: PathBuf,
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
    let state_dir = runtime_state_dir();
    let explicit_config_path = env::var_os("LINGONBERRY_STORAGE_CONFIG").map(PathBuf::from);
    let config_path = explicit_config_path
        .clone()
        .unwrap_or_else(|| state_dir.join("storage-config.json"));
    let mut config = StorageRuntimeConfig {
        config_path: Some(config_path.clone()),
        state_dir: state_dir.clone(),
        data_dir: state_dir.clone(),
        backup_dir: state_dir.join("backup"),
        temp_dir: state_dir.join("tmp"),
    };
    if config_path.exists() {
        let loaded = read_json_file(&config_path)?;
        apply_storage_config(&mut config, &loaded.value)?;
    } else if explicit_config_path.is_some() {
        return Err(format!("storage config file not found: {}", config_path.display()));
    }
    Ok(config)
}

pub fn runtime_storage_layout(config: &StorageRuntimeConfig) -> StorageRuntimeLayout {
    StorageRuntimeLayout {
        raw_log_path: config.data_dir.join("relay-wire-log.jsonl"),
        catalog_path: config.data_dir.join("canonical-catalog.sqlite3"),
    }
}

fn apply_storage_config(config: &mut StorageRuntimeConfig, value: &JsonValue) -> Result<(), String> {
    let Some(map) = as_object(value) else {
        return Err("storage config must be an object".to_string());
    };
    if let Some(path) = map.get("stateDir").and_then(as_string) {
        config.state_dir = PathBuf::from(path);
    }
    if let Some(path) = map.get("dataDir").and_then(as_string) {
        config.data_dir = PathBuf::from(path);
    }
    if let Some(path) = map.get("backupDir").and_then(as_string) {
        config.backup_dir = PathBuf::from(path);
    }
    if let Some(path) = map.get("tempDir").and_then(as_string) {
        config.temp_dir = PathBuf::from(path);
    }
    Ok(())
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn as_string(value: &JsonValue) -> Option<&str> {
    match value {
        JsonValue::String(value) => Some(value.as_str()),
        _ => None,
    }
}
