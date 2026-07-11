from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text()
    if old not in content:
        raise SystemExit(f"expected text not found in {path}: {old[:120]!r}")
    target.write_text(content.replace(old, new, 1))


replace_once(
    "packages/core/src/lib.rs",
    '''pub use quarantine::{quarantine_record_json, QuarantineRecord, QuarantineStore};
''',
    '''pub use quarantine::{
    quarantine_record_json, quarantine_resolution_json, QuarantineRecord,
    QuarantineResolution, QuarantineStore,
};
''',
)

insert_after = '''pub fn build_runtime_capability_manifest() -> JsonValue {
    build_capability_manifest(
        lingonberry_protocol::CARRIER_KIND_RELAY,
        DEFAULT_ACCESS_SCOPE,
        DEFAULT_RETENTION_HINT,
    )
}
'''
addition = r'''

#[derive(Debug, Clone)]
pub enum QuarantinePromotionOutcome {
    Promoted {
        quarantine_id: String,
        canonical_id: String,
        duplicate: bool,
    },
    AlreadyPromoted {
        quarantine_id: String,
        canonical_id: String,
        duplicate: bool,
    },
    StillDeferred {
        quarantine_id: String,
        code: &'static str,
        errors: Vec<String>,
    },
    Rejected {
        quarantine_id: String,
        code: &'static str,
        errors: Vec<String>,
    },
}

pub fn promote_quarantine_record(
    quarantine_id: &str,
    backend: &impl StorageBackend,
) -> Result<QuarantinePromotionOutcome, StoreError> {
    let store = QuarantineStore::new(runtime_state_dir());
    if let Some(resolution) = store.get_resolution(quarantine_id)? {
        return Ok(QuarantinePromotionOutcome::AlreadyPromoted {
            quarantine_id: quarantine_id.to_string(),
            canonical_id: resolution.canonical_id,
            duplicate: resolution.duplicate,
        });
    }

    let record = store.get(quarantine_id)?.ok_or_else(|| {
        store_error(
            "LB_QUARANTINE_NOT_FOUND",
            format!("quarantine record not found: {quarantine_id}"),
        )
    })?;
    let request = parse_json(&record.request_json)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
    let request_map = as_object(&request).ok_or_else(|| {
        store_error(
            "LB_QUARANTINE_CORRUPT",
            "quarantine request is not a publish request",
        )
    })?;
    let object = request_map.get("object").ok_or_else(|| {
        store_error(
            "LB_QUARANTINE_CORRUPT",
            "quarantine publish request missing object",
        )
    })?;

    let report = validate_knowledge_object_full(object);
    let policy = AcceptancePolicy::from_env()
        .map_err(|error| store_error("LB_ACCEPTANCE_POLICY", error))?;
    match evaluate_acceptance(&report, &policy) {
        AcceptanceDecision::Reject { code, errors } => {
            return Ok(QuarantinePromotionOutcome::Rejected {
                quarantine_id: quarantine_id.to_string(),
                code,
                errors,
            })
        }
        AcceptanceDecision::Defer { code, errors } => {
            return Ok(QuarantinePromotionOutcome::StillDeferred {
                quarantine_id: quarantine_id.to_string(),
                code,
                errors,
            })
        }
        AcceptanceDecision::Accept => {}
    }

    let finalized = finalize_knowledge_object_full(object).map_err(|report| {
        store_error(
            "LB_QUARANTINE_PROMOTION",
            report.combined_errors().join("; "),
        )
    })?;
    let outcome = backend.append_publish_request(&record.request_json, &finalized)?;
    store.append_resolution(
        quarantine_id,
        &outcome.canonical_id,
        outcome.duplicate,
    )?;
    Ok(QuarantinePromotionOutcome::Promoted {
        quarantine_id: quarantine_id.to_string(),
        canonical_id: outcome.canonical_id,
        duplicate: outcome.duplicate,
    })
}
'''
replace_once("packages/core/src/lib.rs", insert_after, insert_after + addition)

replace_once(
    "packages/relay/src/main.rs",
    '''use lingonberry_core::{
    build_runtime_capability_manifest, build_runtime_storage_backend, export_archive,
    import_archive, quarantine_record_json, runtime_state_dir, QuarantineStore, StorageBackend,
};
''',
    '''use lingonberry_core::{
    build_runtime_capability_manifest, build_runtime_storage_backend, export_archive,
    import_archive, promote_quarantine_record, quarantine_record_json,
    quarantine_resolution_json, runtime_state_dir, QuarantinePromotionOutcome,
    QuarantineStore, StorageBackend,
};
''',
)
replace_once(
    "packages/relay/src/main.rs",
    'return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay|rebuild-index|relation-graph|lineage-graph|provenance-graph|quarantine-list|quarantine-get|capabilities|ready|export-archive|import-archive|serve-http> <json-file|id|type|archive-dir|addr>".to_string());',
    'return Err("usage: lingonberry <validate|publish|identity-key|get|raw|list|subscribe|replay|rebuild-index|relation-graph|lineage-graph|provenance-graph|quarantine-list|quarantine-get|quarantine-promote|quarantine-resolutions|capabilities|ready|export-archive|import-archive|serve-http> <json-file|id|type|archive-dir|addr>".to_string());',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        "quarantine-get" => {
            let id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry quarantine-get <quarantine-id>".to_string())?;
            handle_quarantine_get(id)
        }
''',
    '''        "quarantine-get" => {
            let id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry quarantine-get <quarantine-id>".to_string())?;
            handle_quarantine_get(id)
        }
        "quarantine-promote" => {
            let id = args
                .get(1)
                .ok_or_else(|| "usage: lingonberry quarantine-promote <quarantine-id>".to_string())?;
            handle_quarantine_promote(id, &backend)
        }
        "quarantine-resolutions" => handle_quarantine_resolutions(),
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''fn handle_raw(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
''',
    r'''fn promotion_outcome_json(outcome: QuarantinePromotionOutcome) -> JsonValue {
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
            ("errors", JsonValue::Array(errors.into_iter().map(JsonValue::String).collect())),
        ]),
        QuarantinePromotionOutcome::Rejected {
            quarantine_id,
            code,
            errors,
        } => json_object(vec![
            ("status", JsonValue::String("rejected".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("code", JsonValue::String(code.to_string())),
            ("errors", JsonValue::Array(errors.into_iter().map(JsonValue::String).collect())),
        ]),
    }
}

fn handle_quarantine_promote(
    id: &str,
    backend: &impl StorageBackend,
) -> Result<(), String> {
    let outcome = promote_quarantine_record(id, backend).map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&promotion_outcome_json(outcome)));
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
            JsonValue::Array(
                resolutions
                    .iter()
                    .map(quarantine_resolution_json)
                    .collect(),
            ),
        ),
    ]);
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_raw(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        ("GET", "/v1/quarantine") => handle_http_quarantine_list(),
''',
    '''        ("GET", "/v1/quarantine") => handle_http_quarantine_list(),
        ("GET", "/v1/quarantine-resolutions") => handle_http_quarantine_resolutions(),
        ("POST", path) if path.starts_with("/v1/quarantine/") && path.ends_with("/promote") => {
            let id = path
                .trim_start_matches("/v1/quarantine/")
                .trim_end_matches("/promote")
                .trim_end_matches('/');
            handle_http_quarantine_promote(id, backend)
        }
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''fn handle_http_get(
''',
    r'''fn handle_http_quarantine_promote(
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

fn handle_http_quarantine_resolutions(
) -> Result<(u16, &'static str, JsonValue), String> {
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
                JsonValue::Array(
                    resolutions
                        .iter()
                        .map(quarantine_resolution_json)
                        .collect(),
                ),
            ),
        ]),
    ))
}

fn handle_http_get(
''',
)

policy = Path("docs/operations/ACCEPTANCE_POLICY.md")
policy.write_text(policy.read_text() + '''

## Revalidation and promotion

Quarantined records can be revalidated against the current implementation and acceptance policy.

```bash
lingonberry quarantine-promote <quarantine-id>
lingonberry quarantine-resolutions
```

HTTP equivalents:

```text
POST /v1/quarantine/<quarantine-id>/promote
GET /v1/quarantine-resolutions
```

Promotion is allowed only when the record now evaluates to `Accept`. Successful and duplicate promotions are recorded in the append-only `quarantine-resolutions.jsonl` ledger. The original quarantine record is retained for auditability, and repeated promotion requests return the existing resolution instead of writing again.
''')
