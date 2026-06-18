use lingonberry_core::{default_state_dir, SqliteStorageBackend, StorageBackend};
use lingonberry_indexer::IndexSnapshot;
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
        return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay|rebuild-index|relation-graph|lineage-graph|provenance-graph> <json-file|id|type>".to_string());
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
        "rebuild-index" => handle_rebuild_index(&backend),
        "relation-graph" => {
            let canonical_id = args.get(1).ok_or_else(|| "usage: lingonberry relation-graph <canonical-id>".to_string())?;
            handle_relation_graph(canonical_id, &backend)
        }
        "lineage-graph" => {
            let canonical_id = args.get(1).ok_or_else(|| "usage: lingonberry lineage-graph <canonical-id>".to_string())?;
            handle_lineage_graph(canonical_id, &backend)
        }
        "provenance-graph" => {
            let protocol = args.get(1).ok_or_else(|| "usage: lingonberry provenance-graph <protocol> <source-id>".to_string())?;
            let source_id = args.get(2).ok_or_else(|| "usage: lingonberry provenance-graph <protocol> <source-id>".to_string())?;
            handle_provenance_graph(protocol, source_id, &backend)
        }
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

fn handle_relation_graph(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let snapshot = IndexSnapshot::from_backend(backend).map_err(|error| error.to_string())?;
    let fragment = snapshot
        .relation_graph(canonical_id)
        .ok_or_else(|| format!("object not found: {}", canonical_id))?;
    let output = json_object(vec![
        ("canonicalId", JsonValue::String(fragment.canonical_id)),
        (
            "outbound",
            JsonValue::Array(fragment.outbound.into_iter().map(relation_edge_json).collect()),
        ),
        (
            "inbound",
            JsonValue::Array(fragment.inbound.into_iter().map(relation_edge_json).collect()),
        ),
        (
            "relatedIds",
            JsonValue::Array(fragment.related_ids.into_iter().map(JsonValue::String).collect()),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_lineage_graph(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let snapshot = IndexSnapshot::from_backend(backend).map_err(|error| error.to_string())?;
    let fragment = snapshot
        .lineage_graph(canonical_id)
        .ok_or_else(|| format!("object not found: {}", canonical_id))?;
    let output = json_object(vec![
        ("canonicalId", JsonValue::String(fragment.canonical_id)),
        (
            "outbound",
            JsonValue::Array(fragment.outbound.into_iter().map(lineage_edge_json).collect()),
        ),
        (
            "inbound",
            JsonValue::Array(fragment.inbound.into_iter().map(lineage_edge_json).collect()),
        ),
        (
            "relatedIds",
            JsonValue::Array(fragment.related_ids.into_iter().map(JsonValue::String).collect()),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_provenance_graph(protocol: &str, source_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let snapshot = IndexSnapshot::from_backend(backend).map_err(|error| error.to_string())?;
    let fragment = snapshot
        .provenance_graph(protocol, source_id)
        .ok_or_else(|| format!("provenance source not found: {} / {}", protocol, source_id))?;
    let output = json_object(vec![
        ("protocol", JsonValue::String(fragment.protocol)),
        ("sourceId", JsonValue::String(fragment.source_id)),
        (
            "canonicalIds",
            JsonValue::Array(fragment.canonical_ids.into_iter().map(JsonValue::String).collect()),
        ),
        (
            "entries",
            JsonValue::Array(fragment.entries.into_iter().map(provenance_entry_json).collect()),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_rebuild_index(backend: &impl StorageBackend) -> Result<(), String> {
    let snapshot = IndexSnapshot::rebuild_from_backend(backend).map_err(|error| error.to_string())?;
    let output = json_object(vec![
        ("ok", JsonValue::Bool(true)),
        ("recordCount", JsonValue::Number(snapshot.record_count().to_string())),
        ("typeCount", JsonValue::Number(snapshot.list_types().len().to_string())),
        ("relationEdgeCount", JsonValue::Number(snapshot.relation_edges().len().to_string())),
        ("lineageEdgeCount", JsonValue::Number(snapshot.lineage_edges().len().to_string())),
        ("provenanceSourceCount", JsonValue::Number(snapshot.provenance_source_count().to_string())),
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

fn relation_edge_json(edge: lingonberry_indexer::RelationEdge) -> JsonValue {
    let mut entries = vec![
        ("canonicalId", JsonValue::String(edge.canonical_id)),
        ("source", JsonValue::String(edge.source)),
        ("target", JsonValue::String(edge.target)),
    ];
    if let Some(kind) = edge.kind {
        entries.push(("kind", JsonValue::String(kind)));
    }
    json_object(entries)
}

fn lineage_edge_json(edge: lingonberry_indexer::LineageEdge) -> JsonValue {
    json_object(vec![
        ("canonicalId", JsonValue::String(edge.canonical_id)),
        ("edgeType", JsonValue::String(edge.edge_type)),
        ("target", JsonValue::String(edge.target)),
    ])
}

fn provenance_entry_json(entry: lingonberry_indexer::ProvenanceGraphEntry) -> JsonValue {
    let mut entries = vec![
        ("canonicalId", JsonValue::String(entry.canonical_id)),
        ("protocol", JsonValue::String(entry.protocol)),
        ("sourceId", JsonValue::String(entry.source_id)),
    ];
    if let Some(author_id) = entry.author_id {
        entries.push(("authorId", JsonValue::String(author_id)));
    }
    if let Some(observed_at) = entry.observed_at {
        entries.push(("observedAt", JsonValue::String(observed_at)));
    }
    json_object(entries)
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
