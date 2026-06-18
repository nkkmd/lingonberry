use lingonberry_core::{default_state_dir, SqliteStorageBackend, StorageBackend};
use lingonberry_protocol::{
    detect_shape, finalize_knowledge_object, read_json_file, to_canonical_json,
    validate_knowledge_object, validate_publish_request, JsonValue,
};
use std::collections::BTreeMap;
use std::env;
use std::process;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{}", error);
        process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("usage: lingonberry <validate|publish|get|list|subscribe|replay> <json-file|id|type>".to_string());
    };
    let backend = SqliteStorageBackend::new(default_state_dir());

    match command {
        "validate" => {
            let pathname = args.get(1).ok_or_else(|| "usage: lingonberry validate <json-file>".to_string())?;
            handle_validate(pathname)
        }
        "publish" => {
            let pathname = args.get(1).ok_or_else(|| "usage: lingonberry publish <json-file>".to_string())?;
            handle_publish(pathname, &backend)
        }
        "get" => {
            let canonical_id = args.get(1).ok_or_else(|| "usage: lingonberry get <canonical-id>".to_string())?;
            handle_get(canonical_id, &backend)
        }
        "list" => handle_list(&backend),
        "subscribe" => handle_subscribe(args.get(1).map(String::as_str), &backend),
        "replay" => handle_replay(&backend),
        _ => Err(format!("unknown command: {}", command)),
    }
}

fn handle_validate(pathname: &str) -> Result<(), String> {
    let loaded = read_json_file(pathname)?;
    let errors = match detect_shape(&loaded.value) {
        "publish-request" => validate_publish_request(&loaded.value),
        _ => validate_knowledge_object(&loaded.value),
    };
    if !errors.is_empty() {
        return Err(format_validation_error("validation failed", &errors));
    }
    println!("{}", to_canonical_json(&json_object(vec![("ok", JsonValue::Bool(true))])));
    Ok(())
}

fn handle_publish(pathname: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let loaded = read_json_file(pathname)?;
    let errors = validate_publish_request(&loaded.value);
    if !errors.is_empty() {
        return Err(format_validation_error("validation failed", &errors));
    }

    let request = as_object(&loaded.value).ok_or_else(|| "publish request must be an object".to_string())?;
    let object = request
        .get("object")
        .ok_or_else(|| "publish request missing object".to_string())?;
    let finalized = finalize_knowledge_object(object).map_err(|errors| format_validation_error("validation failed", &errors))?;
    let outcome = backend
        .append_publish_request(&loaded.raw, &finalized)
        .map_err(|error| error.to_string())?;
    let lingonberry_core::AppendOutcome {
        stored_at,
        canonical_id,
        carrier_identity,
        object,
        duplicate,
    } = outcome;

    let output = json_object(vec![
        ("canonicalId", JsonValue::String(canonical_id)),
        ("carrierIdentity", JsonValue::String(carrier_identity)),
        ("storedAt", match stored_at {
            Some(value) => JsonValue::String(value),
            None => JsonValue::Null,
        }),
        ("duplicate", JsonValue::Bool(duplicate)),
        ("object", object),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_get(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let record = backend
        .get(canonical_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("object not found: {}", canonical_id))?;
    let output = json_object(vec![
        ("canonicalId", JsonValue::String(record.canonical_id)),
        ("carrierIdentity", JsonValue::String(record.carrier_identity)),
        ("storedAt", JsonValue::String(record.stored_at)),
        ("object", record.object),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_list(backend: &impl StorageBackend) -> Result<(), String> {
    let ids = backend.list_ids().map_err(|error| error.to_string())?;
    let output = json_object(vec![(
        "ids",
        JsonValue::Array(ids.into_iter().map(JsonValue::String).collect()),
    )]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_subscribe(object_type: Option<&str>, backend: &impl StorageBackend) -> Result<(), String> {
    let records = backend
        .subscribe(object_type)
        .map_err(|error| error.to_string())?;
    let count = records.len();
    let objects = records
        .into_iter()
        .map(|record| {
            json_object(vec![
                ("canonicalId", JsonValue::String(record.canonical_id)),
                ("carrierIdentity", JsonValue::String(record.carrier_identity)),
                ("storedAt", JsonValue::String(record.stored_at)),
                ("object", record.object),
            ])
        })
        .collect();
    let output = json_object(vec![
        ("count", JsonValue::Number(count.to_string())),
        (
            "filter",
            match object_type {
                Some(value) => json_object(vec![("type", JsonValue::String(value.to_string()))]),
                None => json_object(vec![]),
            },
        ),
        ("objects", JsonValue::Array(objects)),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_replay(backend: &impl StorageBackend) -> Result<(), String> {
    let records = backend.replay().map_err(|error| error.to_string())?;
    let count = records.len();
    let objects = records
        .into_iter()
        .map(|record| {
            json_object(vec![
                ("canonicalId", JsonValue::String(record.canonical_id)),
                ("carrierIdentity", JsonValue::String(record.carrier_identity)),
                ("storedAt", JsonValue::String(record.stored_at)),
                ("object", record.object),
            ])
        })
        .collect();
    let output = json_object(vec![
        ("count", JsonValue::Number(count.to_string())),
        ("objects", JsonValue::Array(objects)),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn format_validation_error(message: &str, errors: &[String]) -> String {
    let suffix = if errors.is_empty() {
        String::new()
    } else {
        format!("\n- {}", errors.join("\n- "))
    };
    format!("{}{}", message, suffix)
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    let mut map = BTreeMap::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    JsonValue::Object(map)
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}
