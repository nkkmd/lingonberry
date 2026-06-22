use lingonberry_core::{
    build_runtime_capability_manifest, build_runtime_storage_backend, export_archive,
    import_archive, StorageBackend,
};
use lingonberry_indexer::IndexSnapshot;
use lingonberry_protocol::{
    build_capability_manifest, derive_identity_key, detect_shape, finalize_knowledge_object,
    read_json_file, to_canonical_json, validate_knowledge_object, validate_publish_request,
    JsonValue, CARRIER_KIND_HTTP, DEFAULT_ACCESS_SCOPE, DEFAULT_RETENTION_HINT,
};
use std::collections::BTreeMap;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{}", error);
        process::exit(exit_code_for_error(&error));
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay|rebuild-index|relation-graph|lineage-graph|provenance-graph|capabilities|ready|export-archive|import-archive|serve-http> <json-file|id|type|archive-dir|addr>".to_string());
    };
    let backend = build_runtime_storage_backend();

    match command {
        "validate" => {
            let pathname = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry validate <json-file>".to_string())?;
            handle_validate(pathname)
        }
        "publish" => {
            let pathname = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry publish <json-file>".to_string())?;
            handle_publish(pathname, &backend)
        }
        "identity-key" => {
            let pathname = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry identity-key <json-file>".to_string())?;
            handle_identity_key(pathname)
        }
        "get" => {
            let canonical_id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry get <canonical-id>".to_string())?;
            handle_get(canonical_id, &backend)
        }
        "raw" => {
            let canonical_id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry raw <canonical-id>".to_string())?;
            handle_raw(canonical_id, &backend)
        }
        "list" => handle_list(&backend),
        "subscribe" => handle_subscribe(args.get(1).map(String::as_str), &backend),
        "replay" => handle_replay(&backend),
        "rebuild-index" => handle_rebuild_index(&backend),
        "relation-graph" => {
            let canonical_id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry relation-graph <canonical-id>".to_string())?;
            handle_relation_graph(canonical_id, &backend)
        }
        "lineage-graph" => {
            let canonical_id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry lineage-graph <canonical-id>".to_string())?;
            handle_lineage_graph(canonical_id, &backend)
        }
        "provenance-graph" => {
            let protocol = args.get(1).ok_or_else(|| {
                "usage: lingonberry provenance-graph <protocol> <source-id>".to_string()
            })?;
            let source_id = args.get(2).ok_or_else(|| {
                "usage: lingonberry provenance-graph <protocol> <source-id>".to_string()
            })?;
            handle_provenance_graph(protocol, source_id, &backend)
        }
        "capabilities" => handle_capabilities(),
        "ready" => handle_ready(),
        "export-archive" => {
            let archive_dir = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry export-archive <archive-dir>".to_string())?;
            handle_export_archive(archive_dir, &backend)
        }
        "import-archive" => {
            let archive_dir = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry import-archive <archive-dir>".to_string())?;
            handle_import_archive(archive_dir, &backend)
        }
        "serve-http" => {
            let addr = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:8787");
            handle_serve_http(addr, &backend)
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
    println!(
        "{}",
        to_canonical_json(&json_object(vec![("ok", JsonValue::Bool(true))]))
    );
    Ok(())
}

fn handle_publish(pathname: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let loaded = read_json_file(pathname)?;
    let errors = validate_publish_request(&loaded.value);
    if !errors.is_empty() {
        return Err(format_validation_error("validation failed", &errors));
    }

    let request =
        as_object(&loaded.value).ok_or_else(|| "publish request must be an object".to_string())?;
    let object = request
        .get("object")
        .ok_or_else(|| "publish request missing object".to_string())?;
    let finalized = finalize_knowledge_object(object)
        .map_err(|errors| format_validation_error("validation failed", &errors))?;
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
        (
            "storedAt",
            match stored_at {
                Some(value) => JsonValue::String(value),
                None => JsonValue::Null,
            },
        ),
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
        (
            "carrierIdentity",
            JsonValue::String(record.carrier_identity),
        ),
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
        (
            "carrierIdentity",
            JsonValue::String(record.carrier_identity),
        ),
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

fn handle_subscribe(
    object_type: Option<&str>,
    backend: &impl StorageBackend,
) -> Result<(), String> {
    let records = backend
        .subscribe(object_type)
        .map_err(|error| error.to_string())?;
    let count = records.len();
    let objects = records
        .into_iter()
        .map(|record| {
            json_object(vec![
                ("canonicalId", JsonValue::String(record.canonical_id)),
                (
                    "carrierIdentity",
                    JsonValue::String(record.carrier_identity),
                ),
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
                (
                    "carrierIdentity",
                    JsonValue::String(record.carrier_identity),
                ),
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
            JsonValue::Array(
                fragment
                    .outbound
                    .into_iter()
                    .map(relation_edge_json)
                    .collect(),
            ),
        ),
        (
            "inbound",
            JsonValue::Array(
                fragment
                    .inbound
                    .into_iter()
                    .map(relation_edge_json)
                    .collect(),
            ),
        ),
        (
            "relatedIds",
            JsonValue::Array(
                fragment
                    .related_ids
                    .into_iter()
                    .map(JsonValue::String)
                    .collect(),
            ),
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
            JsonValue::Array(
                fragment
                    .outbound
                    .into_iter()
                    .map(lineage_edge_json)
                    .collect(),
            ),
        ),
        (
            "inbound",
            JsonValue::Array(
                fragment
                    .inbound
                    .into_iter()
                    .map(lineage_edge_json)
                    .collect(),
            ),
        ),
        (
            "relatedIds",
            JsonValue::Array(
                fragment
                    .related_ids
                    .into_iter()
                    .map(JsonValue::String)
                    .collect(),
            ),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_provenance_graph(
    protocol: &str,
    source_id: &str,
    backend: &impl StorageBackend,
) -> Result<(), String> {
    let snapshot = IndexSnapshot::from_backend(backend).map_err(|error| error.to_string())?;
    let fragment = snapshot
        .provenance_graph(protocol, source_id)
        .ok_or_else(|| format!("provenance source not found: {} / {}", protocol, source_id))?;
    let output = json_object(vec![
        ("protocol", JsonValue::String(fragment.protocol)),
        ("sourceId", JsonValue::String(fragment.source_id)),
        (
            "canonicalIds",
            JsonValue::Array(
                fragment
                    .canonical_ids
                    .into_iter()
                    .map(JsonValue::String)
                    .collect(),
            ),
        ),
        (
            "entries",
            JsonValue::Array(
                fragment
                    .entries
                    .into_iter()
                    .map(provenance_entry_json)
                    .collect(),
            ),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_capabilities() -> Result<(), String> {
    let output = build_runtime_capability_manifest();
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_ready() -> Result<(), String> {
    let output = json_object(vec![
        ("status", JsonValue::String("ok".to_string())),
        ("service", JsonValue::String("relay".to_string())),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_serve_http(addr: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let listener =
        TcpListener::bind(addr).map_err(|error| format!("failed to bind {}: {}", addr, error))?;
    eprintln!("listening on http://{}", addr);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_http_connection(stream, backend) {
                    eprintln!("{}", error);
                }
            }
            Err(error) => eprintln!("accept error: {}", error),
        }
    }
    Ok(())
}

fn handle_export_archive(archive_dir: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let report = export_archive(backend, archive_dir).map_err(|error| error.to_string())?;
    let output = json_object(vec![
        ("ok", JsonValue::Bool(true)),
        (
            "archiveDir",
            JsonValue::String(report.archive_dir.to_string_lossy().to_string()),
        ),
        (
            "manifestPath",
            JsonValue::String(report.manifest_path.to_string_lossy().to_string()),
        ),
        (
            "wireLogPath",
            JsonValue::String(report.wire_log_path.to_string_lossy().to_string()),
        ),
        (
            "catalogPath",
            JsonValue::String(report.catalog_path.to_string_lossy().to_string()),
        ),
        (
            "recordCount",
            JsonValue::Number(report.record_count.to_string()),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_import_archive(archive_dir: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let report = import_archive(backend, archive_dir).map_err(|error| error.to_string())?;
    let output = json_object(vec![
        ("ok", JsonValue::Bool(true)),
        (
            "archiveDir",
            JsonValue::String(report.archive_dir.to_string_lossy().to_string()),
        ),
        (
            "recordCount",
            JsonValue::Number(report.record_count.to_string()),
        ),
        (
            "duplicateCount",
            JsonValue::Number(report.duplicate_count.to_string()),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_http_connection(
    mut stream: TcpStream,
    backend: &impl StorageBackend,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|error| error.to_string())?);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|error| error.to_string())?;
    if request_line.trim().is_empty() {
        return Ok(());
    }
    let (method, path, _version) = parse_http_request_line(&request_line)?;
    let headers = read_http_headers(&mut reader)?;
    let body = read_http_body(&mut reader, &headers)?;
    let (status_code, status_text, response_body) =
        route_http_request(&method, &path, &body, backend)?;
    write_http_response(&mut stream, status_code, status_text, &response_body)
        .map_err(|error| error.to_string())
}

fn route_http_request(
    method: &str,
    path: &str,
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    match (method, path) {
        ("GET", "/v1/ready") => Ok((
            200,
            "OK",
            json_object(vec![
                ("status", JsonValue::String("ok".to_string())),
                ("service", JsonValue::String("relay".to_string())),
            ]),
        )),
        ("GET", "/v1/capabilities") => Ok((
            200,
            "OK",
            build_capability_manifest(
                CARRIER_KIND_HTTP,
                DEFAULT_ACCESS_SCOPE,
                DEFAULT_RETENTION_HINT,
            ),
        )),
        ("POST", "/v1/objects") => handle_http_publish(body, backend),
        ("GET", path) if path.starts_with("/v1/objects/") => {
            let canonical_id = path.trim_start_matches("/v1/objects/");
            handle_http_get(canonical_id, backend)
        }
        _ => Ok((404, "Not Found", http_error("not_found", "route not found"))),
    }
}

fn handle_http_publish(
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    if body.trim().is_empty() {
        return Ok((
            400,
            "Bad Request",
            http_error("validation_error", "request body is empty"),
        ));
    }
    let value = lingonberry_protocol::parse_json(body).map_err(|error| error.to_string())?;
    let errors = validate_publish_request(&value);
    if !errors.is_empty() {
        return Ok((
            400,
            "Bad Request",
            http_error("validation_error", &errors.join("; ")),
        ));
    }
    let Some(request_map) = as_object(&value) else {
        return Ok((
            400,
            "Bad Request",
            http_error("validation_error", "publish request must be an object"),
        ));
    };
    let Some(object) = request_map.get("object") else {
        return Ok((
            400,
            "Bad Request",
            http_error("validation_error", "publish request missing object"),
        ));
    };
    let finalized = finalize_knowledge_object(object)
        .map_err(|errors| format_validation_error("validation failed", &errors))?;
    match backend.append_publish_request(body, &finalized) {
        Ok(outcome) => {
            let raw_ref = as_object(object)
                .and_then(|map| map.get("rawRef"))
                .cloned()
                .unwrap_or(JsonValue::Null);
            let response = json_object(vec![
                ("status", JsonValue::String("ok".to_string())),
                ("id", JsonValue::String(outcome.canonical_id)),
                ("identityKey", JsonValue::String(finalized.identity_key)),
                (
                    "storedAt",
                    match outcome.stored_at {
                        Some(value) => JsonValue::String(value),
                        None => JsonValue::Null,
                    },
                ),
                ("duplicate", JsonValue::Bool(outcome.duplicate)),
                ("canonical", outcome.object),
                ("rawRef", raw_ref),
            ]);
            Ok((200, "OK", response))
        }
        Err(error) if error.code == "LB_OBJECT_CONFLICT" => {
            Ok((409, "Conflict", http_error("conflict", &error.message)))
        }
        Err(error) => Ok((
            500,
            "Internal Server Error",
            http_error("storage_error", &error.to_string()),
        )),
    }
}

fn handle_http_get(
    canonical_id: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    if canonical_id.trim().is_empty() {
        return Ok((
            400,
            "Bad Request",
            http_error("validation_error", "missing canonical id"),
        ));
    }
    let record = backend
        .get(canonical_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "not found".to_string())?;
    let raw_ref = as_object(&record.object)
        .and_then(|map| map.get("rawRef"))
        .cloned()
        .unwrap_or(JsonValue::Null);
    let identity_key = derive_identity_key(&record.object);
    let response = json_object(vec![
        ("status", JsonValue::String("ok".to_string())),
        ("id", JsonValue::String(record.canonical_id)),
        ("identityKey", JsonValue::String(identity_key)),
        ("storedAt", JsonValue::String(record.stored_at)),
        (
            "carrierIdentity",
            JsonValue::String(record.carrier_identity),
        ),
        ("canonical", record.object),
        ("rawRef", raw_ref),
    ]);
    Ok((200, "OK", response))
}

fn read_http_headers(
    reader: &mut BufReader<TcpStream>,
) -> Result<BTreeMap<String, String>, String> {
    let mut headers = BTreeMap::new();
    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    Ok(headers)
}

fn read_http_body(
    reader: &mut BufReader<TcpStream>,
    headers: &BTreeMap<String, String>,
) -> Result<String, String> {
    let Some(content_length) = headers.get("content-length") else {
        return Ok(String::new());
    };
    let length: usize = content_length
        .parse()
        .map_err(|_| "invalid content-length".to_string())?;
    let mut buffer = vec![0u8; length];
    reader
        .read_exact(&mut buffer)
        .map_err(|error| error.to_string())?;
    String::from_utf8(buffer).map_err(|error| error.to_string())
}

fn parse_http_request_line(line: &str) -> Result<(String, String, String), String> {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    let mut parts = trimmed.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "invalid HTTP request line".to_string())?;
    let path = parts
        .next()
        .ok_or_else(|| "invalid HTTP request line".to_string())?;
    let version = parts
        .next()
        .ok_or_else(|| "invalid HTTP request line".to_string())?;
    Ok((method.to_string(), path.to_string(), version.to_string()))
}

fn write_http_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    body: &JsonValue,
) -> Result<(), std::io::Error> {
    let body_json = to_canonical_json(body);
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status_code,
        status_text,
        body_json.len(),
        body_json
    );
    stream.write_all(response.as_bytes())
}

fn http_error(kind: &str, message: &str) -> JsonValue {
    json_object(vec![
        ("status", JsonValue::String("error".to_string())),
        (
            "error",
            json_object(vec![
                ("type", JsonValue::String(kind.to_string())),
                ("message", JsonValue::String(message.to_string())),
            ]),
        ),
    ])
}

fn handle_rebuild_index(backend: &impl StorageBackend) -> Result<(), String> {
    let snapshot =
        IndexSnapshot::rebuild_from_backend(backend).map_err(|error| error.to_string())?;
    let output = json_object(vec![
        ("ok", JsonValue::Bool(true)),
        (
            "recordCount",
            JsonValue::Number(snapshot.record_count().to_string()),
        ),
        (
            "typeCount",
            JsonValue::Number(snapshot.list_types().len().to_string()),
        ),
        (
            "relationEdgeCount",
            JsonValue::Number(snapshot.relation_edges().len().to_string()),
        ),
        (
            "lineageEdgeCount",
            JsonValue::Number(snapshot.lineage_edges().len().to_string()),
        ),
        (
            "provenanceSourceCount",
            JsonValue::Number(snapshot.provenance_source_count().to_string()),
        ),
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

fn exit_code_for_error(error: &str) -> i32 {
    if error.starts_with("usage:") {
        64
    } else if error.contains("not found") {
        66
    } else if error.contains("failed to bind") {
        78
    } else if error.contains("validation failed") {
        65
    } else if error.contains("LB_") {
        70
    } else {
        1
    }
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
