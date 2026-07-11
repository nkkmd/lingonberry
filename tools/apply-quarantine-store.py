from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text()
    if old not in content:
        raise SystemExit(f"expected text not found in {path}: {old[:120]!r}")
    target.write_text(content.replace(old, new, 1))


replace_once(
    "packages/core/src/lib.rs",
    "mod sqlite;\npub use sqlite::SqliteStorageBackend;\n",
    "mod quarantine;\nmod sqlite;\npub use quarantine::{quarantine_record_json, QuarantineRecord, QuarantineStore};\npub use sqlite::SqliteStorageBackend;\n",
)
replace_once(
    "packages/core/src/lib.rs",
    '''            AcceptanceDecision::Defer { code, errors } => {
                return Err(store_error(code, errors.join("; ")))
            }
''',
    '''            AcceptanceDecision::Defer { code, errors } => {
                let record = QuarantineStore::new(runtime_state_dir())
                    .append(request_json, code, &errors)?;
                return Err(store_error(
                    code,
                    format!("{}; quarantineId={}", errors.join("; "), record.id),
                ));
            }
''',
)

replace_once(
    "packages/relay/src/main.rs",
    '''use lingonberry_core::{
    build_runtime_capability_manifest, build_runtime_storage_backend, export_archive,
    import_archive, StorageBackend,
};
''',
    '''use lingonberry_core::{
    build_runtime_capability_manifest, build_runtime_storage_backend, export_archive,
    import_archive, quarantine_record_json, runtime_state_dir, QuarantineStore, StorageBackend,
};
''',
)
replace_once(
    "packages/relay/src/main.rs",
    'return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay|rebuild-index|relation-graph|lineage-graph|provenance-graph|capabilities|ready|export-archive|import-archive|serve-http> <json-file|id|type|archive-dir|addr>".to_string());',
    'return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay|rebuild-index|relation-graph|lineage-graph|provenance-graph|quarantine-list|quarantine-get|capabilities|ready|export-archive|import-archive|serve-http> <json-file|id|type|archive-dir|addr>".to_string());',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        "provenance-graph" => {
''',
    '''        "quarantine-list" => handle_quarantine_list(),
        "quarantine-get" => {
            let id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry quarantine-get <quarantine-id>".to_string())?;
            handle_quarantine_get(id)
        }
        "provenance-graph" => {
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        AcceptanceDecision::Defer { code, errors } => {
            return Err(format!("{}: {}", code, errors.join("; ")))
        }
''',
    '''        AcceptanceDecision::Defer { code, errors } => {
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
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        ("GET", "/v1/capabilities") => Ok((
''',
    '''        ("GET", "/v1/quarantine") => handle_http_quarantine_list(),
        ("GET", path) if path.starts_with("/v1/quarantine/") => {
            let id = path.trim_start_matches("/v1/quarantine/");
            handle_http_quarantine_get(id)
        }
        ("GET", "/v1/capabilities") => Ok((
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        AcceptanceDecision::Defer { errors, .. } => {
            return Ok((
                202,
                "Accepted",
                json_object(vec![
                    ("status", JsonValue::String("deferred".to_string())),
                    ("reason", JsonValue::String(errors.join("; "))),
                    ("stored", JsonValue::Bool(false)),
                ]),
            ));
        }
''',
    '''        AcceptanceDecision::Defer { code, errors } => {
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
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''fn handle_raw(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
''',
    '''fn handle_quarantine_list() -> Result<(), String> {
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

fn handle_raw(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''fn handle_http_get(
''',
    '''fn handle_http_quarantine_list() -> Result<(u16, &'static str, JsonValue), String> {
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

fn handle_http_quarantine_get(
    id: &str,
) -> Result<(u16, &'static str, JsonValue), String> {
    match QuarantineStore::new(runtime_state_dir())
        .get(id)
        .map_err(|error| error.to_string())?
    {
        Some(record) => Ok((200, "OK", quarantine_record_json(&record))),
        None => Ok((404, "Not Found", http_error("not_found", "quarantine record not found"))),
    }
}

fn handle_http_get(
''',
)

policy = Path("docs/operations/ACCEPTANCE_POLICY.md")
policy.write_text(policy.read_text().replace(
    '`defer` is intentionally non-persistent in this version. A durable quarantine store can be added separately without weakening the guarantee that unverified objects never enter the canonical catalog.',
    '`defer` persists the original publish request and validation reasons to `<state-dir>/quarantine.jsonl`. Quarantined objects remain outside the canonical catalog. Use `quarantine-list`, `quarantine-get <id>`, `GET /v1/quarantine`, or `GET /v1/quarantine/<id>` to inspect them.'
))
