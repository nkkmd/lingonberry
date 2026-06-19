use lingonberry_core::StorageBackend;
use lingonberry_protocol::{
    finalize_knowledge_object, read_json_file, to_canonical_json, validate_knowledge_object,
    validate_publish_request, JsonValue,
};
use lingonberry_storage::{
    build_storage_backend_at, runtime_storage_config, runtime_storage_layout, StorageRuntimeConfig,
};
use std::collections::BTreeMap;
use std::env;
use std::process;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{}", error);
        process::exit(exit_code_for_error(&error));
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("usage: lingonberry-storage <capabilities|config|ready|run|append|retrieve|replay|list> <json-file|canonical-id>".to_string());
    };

    let config = runtime_storage_config()?;
    let backend = build_storage_backend_at(&config.data_dir);

    match command {
        "capabilities" => {
            println!("{}", to_canonical_json(&json_object(vec![
                ("status", JsonValue::String("ok".to_string())),
                ("service", JsonValue::String("storage".to_string())),
                (
                    "operations",
                    JsonValue::Array(vec![
                        JsonValue::String("append".to_string()),
                        JsonValue::String("retrieve".to_string()),
                        JsonValue::String("replay".to_string()),
                        JsonValue::String("list".to_string()),
                        JsonValue::String("config".to_string()),
                        JsonValue::String("ready".to_string()),
                        JsonValue::String("run".to_string()),
                    ]),
                ),
            ])));
            Ok(())
        }
        "config" => {
            print_config(&config);
            Ok(())
        }
        "ready" => {
            print_runtime_status(&config);
            Ok(())
        }
        "run" => {
            print_runtime_status(&config);
            Ok(())
        }
        "append" => {
            let pathname = args.get(1).ok_or_else(|| "usage: lingonberry-storage append <json-file>".to_string())?;
            handle_append(pathname, &backend)
        }
        "retrieve" => {
            let canonical_id = args.get(1).ok_or_else(|| "usage: lingonberry-storage retrieve <canonical-id>".to_string())?;
            handle_retrieve(canonical_id, &backend)
        }
        "replay" => {
            handle_replay(&backend)
        }
        "list" => {
            handle_list(&backend)
        }
        _ => Err(format!("unknown command: {}", command)),
    }
}

fn handle_append(pathname: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let loaded = read_json_file(pathname)?;
    let errors = validate_publish_request(&loaded.value);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let Some(request_map) = as_object(&loaded.value) else {
        return Err("publish request must be an object".to_string());
    };
    let Some(object) = request_map.get("object") else {
        return Err("publish request missing object".to_string());
    };
    let object_errors = validate_knowledge_object(object);
    if !object_errors.is_empty() {
        return Err(object_errors.join("; "));
    }
    let finalized = finalize_knowledge_object(object).map_err(|errors| errors.join("; "))?;
    let outcome = backend.append_publish_request(&loaded.raw, &finalized).map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&json_object(vec![
        ("canonicalId", JsonValue::String(outcome.canonical_id)),
        ("carrierIdentity", JsonValue::String(outcome.carrier_identity)),
        ("duplicate", JsonValue::Bool(outcome.duplicate)),
        (
            "storedAt",
            match outcome.stored_at {
                Some(value) => JsonValue::String(value),
                None => JsonValue::Null,
            },
        ),
        ("object", outcome.object),
    ])));
    Ok(())
}

fn handle_retrieve(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let record = backend
        .get(canonical_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("object not found: {}", canonical_id))?;
    println!("{}", to_canonical_json(&json_object(vec![
        ("canonicalId", JsonValue::String(record.canonical_id)),
        ("carrierIdentity", JsonValue::String(record.carrier_identity)),
        ("storedAt", JsonValue::String(record.stored_at)),
        ("object", record.object),
    ])));
    Ok(())
}

fn handle_replay(backend: &impl StorageBackend) -> Result<(), String> {
    let records = backend.replay().map_err(|error| error.to_string())?;
    let objects: Vec<JsonValue> = records
        .into_iter()
        .map(|record| json_object(vec![
            ("canonicalId", JsonValue::String(record.canonical_id)),
            ("carrierIdentity", JsonValue::String(record.carrier_identity)),
            ("storedAt", JsonValue::String(record.stored_at)),
            ("object", record.object),
        ]))
        .collect();
    println!("{}", to_canonical_json(&json_object(vec![
        ("count", JsonValue::Number(objects.len().to_string())),
        ("objects", JsonValue::Array(objects)),
    ])));
    Ok(())
}

fn handle_list(backend: &impl StorageBackend) -> Result<(), String> {
    let ids = backend.list_ids().map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&json_object(vec![
        ("ids", JsonValue::Array(ids.into_iter().map(JsonValue::String).collect())),
    ])));
    Ok(())
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    let mut map = BTreeMap::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    JsonValue::Object(map)
}

fn print_config(config: &StorageRuntimeConfig) {
    let layout = runtime_storage_layout(config);
    println!("{}", to_canonical_json(&json_object(vec![
        ("configPath", path_value(config.config_path.as_ref())),
        ("stateDir", JsonValue::String(config.state_dir.to_string_lossy().to_string())),
        ("dataDir", JsonValue::String(config.data_dir.to_string_lossy().to_string())),
        ("backupDir", JsonValue::String(config.backup_dir.to_string_lossy().to_string())),
        ("tempDir", JsonValue::String(config.temp_dir.to_string_lossy().to_string())),
        ("rawLogPath", JsonValue::String(layout.raw_log_path.to_string_lossy().to_string())),
        ("catalogPath", JsonValue::String(layout.catalog_path.to_string_lossy().to_string())),
    ])));
}

fn print_runtime_status(config: &StorageRuntimeConfig) {
    let layout = runtime_storage_layout(config);
    println!("{}", to_canonical_json(&json_object(vec![
        ("status", JsonValue::String("ok".to_string())),
        ("service", JsonValue::String("storage".to_string())),
        ("configPath", path_value(config.config_path.as_ref())),
        ("stateDir", JsonValue::String(config.state_dir.to_string_lossy().to_string())),
        ("dataDir", JsonValue::String(config.data_dir.to_string_lossy().to_string())),
        ("backupDir", JsonValue::String(config.backup_dir.to_string_lossy().to_string())),
        ("tempDir", JsonValue::String(config.temp_dir.to_string_lossy().to_string())),
        ("rawLogPath", JsonValue::String(layout.raw_log_path.to_string_lossy().to_string())),
        ("catalogPath", JsonValue::String(layout.catalog_path.to_string_lossy().to_string())),
    ])));
}

fn path_value(path: Option<&std::path::PathBuf>) -> JsonValue {
    match path {
        Some(path) => JsonValue::String(path.to_string_lossy().to_string()),
        None => JsonValue::Null,
    }
}

fn exit_code_for_error(error: &str) -> i32 {
    if error.starts_with("usage:") {
        64
    } else if error.contains("not found") {
        66
    } else if error.contains("config") || error.contains("failed to bind") {
        78
    } else if error.contains("validation failed") {
        65
    } else if error.contains("LB_") {
        70
    } else {
        1
    }
}
