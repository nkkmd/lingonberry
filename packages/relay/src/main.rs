use lingonberry_core::{default_state_dir, SqliteStorageBackend, StorageBackend};
use lingonberry_protocol::{
    detect_shape, derive_identity_key, finalize_knowledge_object, read_json_file, to_canonical_json,
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
        return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay> <json-file|id|type>".to_string());
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
        "identity-key" => {
            let pathname = args.get(1).ok_or_else(|| "usage: lingonberry identity-key <json-file>".to_string())?;
            handle_identity_key(pathname)
        }
        "get" => {
            let canonical_id = args.get(1).ok_or_else(|| "usage: lingonberry get <canonical-id>".to_string())?;
            handle_get(canonical_id, &backend)
        }
        "raw" => {
            let canonical_id = args.get(1).ok_or_else(|| "usage: lingonberry raw <canonical-id>".to_string())?;
            handle_raw(canonical_id, &backend)
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
        ("identityKey", JsonValue::String(finalized.identity_key)),
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
    let identity_key = derive_identity_key(&record.object);
    let output = json_object(vec![
        ("canonicalId", JsonValue::String(record.canonical_id)),
        ("carrierIdentity", JsonValue::String(record.carrier_identity)),
        ("identityKey", JsonValue::String(identity_key)),
        ("storedAt", JsonValue::String(record.stored_at)),
        ("object", record.object),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_identity_key(pathname: &str) -> Result<(), String> {
    let loaded = read_json_file(pathname)?;
    let object = match detect_shape(&loaded.value) {
        "publish-request" => {
            let errors = validate_publish_request(&loaded.value);
            if !errors.is_empty() {
                return Err(format_validation_error("validation failed", &errors));
            }
            as_object(&loaded.value)
                .and_then(|request| request.get("object"))
                .cloned()
                .ok_or_else(|| "publish request missing object".to_string())?
        }
        _ => {
            let errors = validate_knowledge_object(&loaded.value);
            if !errors.is_empty() {
                return Err(format_validation_error("validation failed", &errors));
            }
            loaded.value
        }
    };
    let identity_key = derive_identity_key(&object);
    let canonical_id = as_object(&object)
        .and_then(|map| map.get("id"))
        .and_then(as_string)
        .unwrap_or_default()
        .to_string();
    let output = json_object(vec![
        ("canonicalId", JsonValue::String(canonical_id)),
        ("identityKey", JsonValue::String(identity_key)),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_raw(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let record = backend
        .get_raw_request(canonical_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("raw request not found: {}", canonical_id))?;
    let output = json_object(vec![
        ("canonicalId", JsonValue::String(record.canonical_id)),
        ("carrierIdentity", JsonValue::String(record.carrier_identity)),
        ("requestJson", JsonValue::String(record.request_json)),
        ("storedAt", JsonValue::String(record.stored_at)),
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

fn as_string(value: &JsonValue) -> Option<&str> {
    match value {
        JsonValue::String(value) => Some(value.as_str()),
        _ => None,
    }
}
