use lingonberry_core::{
    build_runtime_capability_manifest, build_runtime_storage_backend, export_archive,
    import_archive, promote_quarantine_batch, promote_quarantine_record, quarantine_record_json,
    quarantine_resolution_json, runtime_state_dir, QuarantineBatchReport,
    QuarantinePromotionOutcome, QuarantineStore, StorageBackend,
};
use lingonberry_indexer::IndexSnapshot;
use lingonberry_protocol::{
    build_capability_manifest, derive_identity_key, detect_shape, read_json_file,
    to_canonical_json, JsonValue, CARRIER_KIND_HTTP, DEFAULT_ACCESS_SCOPE, DEFAULT_RETENTION_HINT,
};
use lingonberry_validation::{
    evaluate_acceptance, finalize_knowledge_object_full, validate_knowledge_object_full,
    validate_publish_request_full, AcceptanceDecision, AcceptancePolicy, IdentityValidationStatus,
    ValidationReport,
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
        return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay|rebuild-index|relation-graph|lineage-graph|provenance-graph|quarantine-list|quarantine-get|quarantine-promote|quarantine-promote-batch|quarantine-resolutions|capabilities|ready|export-archive|import-archive|serve-http> <json-file|id|type|archive-dir|addr>".to_string());
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
        "quarantine-list" => handle_quarantine_list(),
        "quarantine-get" => {
            let id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry quarantine-get <quarantine-id>".to_string())?;
            handle_quarantine_get(id)
        }
        "quarantine-promote" => {
            let id = args.get(1).ok_or_else(|| {
                "usage: lingonberry quarantine-promote <quarantine-id>".to_string()
            })?;
            handle_quarantine_promote(id, &backend)
        }
        "quarantine-promote-batch" => {
            let limit = parse_batch_limit(args.get(1).map(String::as_str))?;
            let dry_run = args.iter().any(|arg| arg == "--dry-run");
            handle_quarantine_promote_batch(limit, dry_run, &backend)
        }
        "quarantine-resolutions" => handle_quarantine_resolutions(),
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
    let report = match detect_shape(&loaded.value) {
        "publish-request" => validate_publish_request_full(&loaded.value),
        _ => validate_knowledge_object_full(&loaded.value),
    };
    if !report.is_valid() {
        return Err(format_validation_error(
            "validation failed",
            &report.combined_errors(),
        ));
    }
    println!(
        "{}",
        to_canonical_json(&json_object(vec![("ok", JsonValue::Bool(true))]))
    );
    Ok(())
}

fn handle_publish(pathname: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let loaded = read_json_file(pathname)?;
    let report = validate_publish_request_full(&loaded.value);
    let policy = AcceptancePolicy::from_env()?;
    match evaluate_acceptance(&report, &policy) {
        AcceptanceDecision::Accept => {}
        AcceptanceDecision::Reject { code, errors } => {
            return Err(format!("{}: {}", code, errors.join("; ")))
        }
        AcceptanceDecision::Defer { code, errors } => {
            let record = QuarantineStore::new(runtime_state_dir())
                .append(&loaded.raw, code, &errors)
                .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&json_object(vec![
                    ("status", JsonValue::String("deferred".to_string())),
                    ("stored", JsonValue::Bool(false)),
                    ("quarantineId", JsonValue::String(record.id)),
                    ("reason", JsonValue::String(errors.join("; "))),
                ]))
            );
            return Ok(());
        }
    }

    let request =
        as_object(&loaded.value).ok_or_else(|| "publish request must be an object".to_string())?;
    let object = request
        .get("object")
        .ok_or_else(|| "publish request missing object".to_string())?;
    let finalized = finalize_knowledge_object_full(object).map_err(|report| {
        format_validation_error("validation failed", &report.combined_errors())
    })?;
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
            let report = validate_publish_request_full(&loaded.value);
            if !report.is_valid() {
                return Err(format_validation_error(
                    "validation failed",
                    &report.combined_errors(),
                ));
            }
            as_object(&loaded.value)
                .and_then(|request| request.get("object"))
                .cloned()
                .ok_or_else(|| "publish request missing object".to_string())?
        }
        _ => {
            let report = validate_knowledge_object_full(&loaded.value);
            if !report.is_valid() {
                return Err(format_validation_error(
                    "validation failed",
                    &report.combined_errors(),
                ));
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

fn handle_quarantine_list() -> Result<(), String> {
    let records = QuarantineStore::new(runtime_state_dir())
        .list()
        .map_err(|error| error.to_string())?;
    let output = json_object(vec![
        ("count", JsonValue::Number(records.len().to_string())),
        (
            "records",
            JsonValue::Array(records.iter().map(quarantine_record_json).collect()),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_quarantine_get(id: &str) -> Result<(), String> {
    let record = QuarantineStore::new(runtime_state_dir())
        .get(id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("quarantine record not found: {}", id))?;
    println!("{}", to_canonical_json(&quarantine_record_json(&record)));
    Ok(())
}

fn promotion_outcome_json(outcome: QuarantinePromotionOutcome) -> JsonValue {
    match outcome {
        QuarantinePromotionOutcome::Promoted {
            quarantine_id,
            canonical_id,
            duplicate,
        } => json_object(vec![
            ("status", JsonValue::String("promoted".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("canonicalId", JsonValue::String(canonical_id)),
            ("duplicate", JsonValue::Bool(duplicate)),
        ]),
        QuarantinePromotionOutcome::AlreadyPromoted {
            quarantine_id,
            canonical_id,
            duplicate,
        } => json_object(vec![
            ("status", JsonValue::String("already-promoted".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("canonicalId", JsonValue::String(canonical_id)),
            ("duplicate", JsonValue::Bool(duplicate)),
        ]),
        QuarantinePromotionOutcome::StillDeferred {
            quarantine_id,
            code,
            errors,
        } => json_object(vec![
            ("status", JsonValue::String("deferred".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("code", JsonValue::String(code.to_string())),
            (
                "errors",
                JsonValue::Array(errors.into_iter().map(JsonValue::String).collect()),
            ),
        ]),
        QuarantinePromotionOutcome::Rejected {
            quarantine_id,
            code,
            errors,
        } => json_object(vec![
            ("status", JsonValue::String("rejected".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("code", JsonValue::String(code.to_string())),
            (
                "errors",
                JsonValue::Array(errors.into_iter().map(JsonValue::String).collect()),
            ),
        ]),
    }
}

fn handle_quarantine_promote(id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let outcome = promote_quarantine_record(id, backend).map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&promotion_outcome_json(outcome)));
    Ok(())
}

fn parse_batch_limit(value: Option<&str>) -> Result<usize, String> {
    match value {
        None | Some("--dry-run") => Ok(100),
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| "batch limit must be a positive integer".to_string())
            .and_then(|limit| {
                if limit == 0 || limit > 1000 {
                    Err("batch limit must be between 1 and 1000".to_string())
                } else {
                    Ok(limit)
                }
            }),
    }
}

fn batch_report_json(report: QuarantineBatchReport) -> JsonValue {
    json_object(vec![
        ("dryRun", JsonValue::Bool(report.dry_run)),
        ("limit", JsonValue::Number(report.limit.to_string())),
        ("scanned", JsonValue::Number(report.scanned.to_string())),
        ("promoted", JsonValue::Number(report.promoted.to_string())),
        (
            "alreadyPromoted",
            JsonValue::Number(report.already_promoted.to_string()),
        ),
        ("deferred", JsonValue::Number(report.deferred.to_string())),
        ("rejected", JsonValue::Number(report.rejected.to_string())),
        (
            "outcomes",
            JsonValue::Array(
                report
                    .outcomes
                    .into_iter()
                    .map(promotion_outcome_json)
                    .collect(),
            ),
        ),
    ])
}

fn handle_quarantine_promote_batch(
    limit: usize,
    dry_run: bool,
    backend: &impl StorageBackend,
) -> Result<(), String> {
    let report =
        promote_quarantine_batch(limit, dry_run, backend).map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&batch_report_json(report)));
    Ok(())
}

fn handle_quarantine_resolutions() -> Result<(), String> {
    let resolutions = QuarantineStore::new(runtime_state_dir())
        .list_resolutions()
        .map_err(|error| error.to_string())?;
    let output = json_object(vec![
        ("count", JsonValue::Number(resolutions.len().to_string())),
        (
            "resolutions",
            JsonValue::Array(resolutions.iter().map(quarantine_resolution_json).collect()),
        ),
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
        ("GET", "/v1/quarantine") => handle_http_quarantine_list(),
        ("GET", "/v1/quarantine-resolutions") => handle_http_quarantine_resolutions(),
        ("POST", "/v1/quarantine/promote-batch") => {
            handle_http_quarantine_promote_batch(body, backend)
        }
        ("POST", path) if path.starts_with("/v1/quarantine/") && path.ends_with("/promote") => {
            let id = path
                .trim_start_matches("/v1/quarantine/")
                .trim_end_matches("/promote")
                .trim_end_matches('/');
            handle_http_quarantine_promote(id, backend)
        }
        ("GET", path) if path.starts_with("/v1/quarantine/") => {
            let id = path.trim_start_matches("/v1/quarantine/");
            handle_http_quarantine_get(id)
        }
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
    let report = validate_publish_request_full(&value);
    let policy = AcceptancePolicy::from_env()?;
    match evaluate_acceptance(&report, &policy) {
        AcceptanceDecision::Accept => {}
        AcceptanceDecision::Reject { code, errors } => {
            let kind = match code {
                "LB_IDENTITY_CLAIM_REQUIRED" => "identity_claim_required",
                "LB_UNSUPPORTED_IDENTITY_RULE" => "unsupported_identity_rule",
                _ => "validation_error",
            };
            let status = if code == "LB_UNSUPPORTED_IDENTITY_RULE" {
                422
            } else {
                400
            };
            let text = if status == 422 {
                "Unprocessable Entity"
            } else {
                "Bad Request"
            };
            return Ok((status, text, http_error(kind, &errors.join("; "))));
        }
        AcceptanceDecision::Defer { code, errors } => {
            let record = QuarantineStore::new(runtime_state_dir())
                .append(body, code, &errors)
                .map_err(|error| error.to_string())?;
            return Ok((
                202,
                "Accepted",
                json_object(vec![
                    ("status", JsonValue::String("deferred".to_string())),
                    ("reason", JsonValue::String(errors.join("; "))),
                    ("stored", JsonValue::Bool(false)),
                    ("quarantineId", JsonValue::String(record.id)),
                ]),
            ));
        }
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
    let finalized = finalize_knowledge_object_full(object).map_err(|report| {
        format_validation_error("validation failed", &report.combined_errors())
    })?;
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

fn handle_http_quarantine_list() -> Result<(u16, &'static str, JsonValue), String> {
    let records = QuarantineStore::new(runtime_state_dir())
        .list()
        .map_err(|error| error.to_string())?;
    Ok((
        200,
        "OK",
        json_object(vec![
            ("count", JsonValue::Number(records.len().to_string())),
            (
                "records",
                JsonValue::Array(records.iter().map(quarantine_record_json).collect()),
            ),
        ]),
    ))
}

fn handle_http_quarantine_get(id: &str) -> Result<(u16, &'static str, JsonValue), String> {
    match QuarantineStore::new(runtime_state_dir())
        .get(id)
        .map_err(|error| error.to_string())?
    {
        Some(record) => Ok((200, "OK", quarantine_record_json(&record))),
        None => Ok((
            404,
            "Not Found",
            http_error("not_found", "quarantine record not found"),
        )),
    }
}

fn handle_http_quarantine_promote(
    id: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    let outcome = promote_quarantine_record(id, backend).map_err(|error| error.to_string())?;
    let status = match outcome {
        QuarantinePromotionOutcome::Promoted { .. }
        | QuarantinePromotionOutcome::AlreadyPromoted { .. } => 200,
        QuarantinePromotionOutcome::StillDeferred { .. } => 202,
        QuarantinePromotionOutcome::Rejected { .. } => 422,
    };
    let text = match status {
        200 => "OK",
        202 => "Accepted",
        _ => "Unprocessable Entity",
    };
    Ok((status, text, promotion_outcome_json(outcome)))
}

fn handle_http_quarantine_promote_batch(
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    let (limit, dry_run) = if body.trim().is_empty() {
        (100, false)
    } else {
        let value = lingonberry_protocol::parse_json(body).map_err(|error| error.to_string())?;
        let map = as_object(&value).ok_or_else(|| "batch request must be an object".to_string())?;
        let limit = match map.get("limit") {
            None => 100,
            Some(JsonValue::Number(value)) => parse_batch_limit(Some(value))?,
            _ => return Err("batch limit must be a number".to_string()),
        };
        let dry_run = match map.get("dryRun") {
            None => false,
            Some(JsonValue::Bool(value)) => *value,
            _ => return Err("dryRun must be a boolean".to_string()),
        };
        (limit, dry_run)
    };
    let report =
        promote_quarantine_batch(limit, dry_run, backend).map_err(|error| error.to_string())?;
    Ok((200, "OK", batch_report_json(report)))
}

fn handle_http_quarantine_resolutions() -> Result<(u16, &'static str, JsonValue), String> {
    let resolutions = QuarantineStore::new(runtime_state_dir())
        .list_resolutions()
        .map_err(|error| error.to_string())?;
    Ok((
        200,
        "OK",
        json_object(vec![
            ("count", JsonValue::Number(resolutions.len().to_string())),
            (
                "resolutions",
                JsonValue::Array(resolutions.iter().map(quarantine_resolution_json).collect()),
            ),
        ]),
    ))
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

fn validation_http_error(report: &ValidationReport) -> (u16, &'static str, &'static str) {
    if report.identity_status == IdentityValidationStatus::Unsupported {
        (422, "Unprocessable Entity", "unsupported_identity_rule")
    } else {
        (400, "Bad Request", "validation_error")
    }
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
