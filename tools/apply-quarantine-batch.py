from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text()
    if old not in content:
        raise SystemExit(f"expected text not found in {path}: {old[:120]!r}")
    target.write_text(content.replace(old, new, 1))


marker = '''pub fn export_archive(
'''
addition = r'''
#[derive(Debug, Clone)]
pub struct QuarantineBatchReport {
    pub dry_run: bool,
    pub limit: usize,
    pub scanned: usize,
    pub promoted: usize,
    pub already_promoted: usize,
    pub deferred: usize,
    pub rejected: usize,
    pub outcomes: Vec<QuarantinePromotionOutcome>,
}

pub fn preview_quarantine_record(
    quarantine_id: &str,
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
        AcceptanceDecision::Reject { code, errors } => Ok(QuarantinePromotionOutcome::Rejected {
            quarantine_id: quarantine_id.to_string(),
            code,
            errors,
        }),
        AcceptanceDecision::Defer { code, errors } => Ok(QuarantinePromotionOutcome::StillDeferred {
            quarantine_id: quarantine_id.to_string(),
            code,
            errors,
        }),
        AcceptanceDecision::Accept => {
            let finalized = finalize_knowledge_object_full(object).map_err(|report| {
                store_error(
                    "LB_QUARANTINE_PROMOTION",
                    report.combined_errors().join("; "),
                )
            })?;
            Ok(QuarantinePromotionOutcome::Promoted {
                quarantine_id: quarantine_id.to_string(),
                canonical_id: finalized.canonical_id,
                duplicate: false,
            })
        }
    }
}

pub fn promote_quarantine_batch(
    limit: usize,
    dry_run: bool,
    backend: &impl StorageBackend,
) -> Result<QuarantineBatchReport, StoreError> {
    if limit == 0 {
        return Err(store_error(
            "LB_QUARANTINE_BATCH",
            "limit must be greater than zero",
        ));
    }
    let store = QuarantineStore::new(runtime_state_dir());
    let resolved = store
        .list_resolutions()?
        .into_iter()
        .map(|resolution| resolution.quarantine_id)
        .collect::<BTreeSet<_>>();
    let ids = store
        .list()?
        .into_iter()
        .filter(|record| !resolved.contains(&record.id))
        .map(|record| record.id)
        .take(limit)
        .collect::<Vec<_>>();

    let mut report = QuarantineBatchReport {
        dry_run,
        limit,
        scanned: ids.len(),
        promoted: 0,
        already_promoted: 0,
        deferred: 0,
        rejected: 0,
        outcomes: Vec::new(),
    };
    for id in ids {
        let outcome = if dry_run {
            preview_quarantine_record(&id)?
        } else {
            promote_quarantine_record(&id, backend)?
        };
        match &outcome {
            QuarantinePromotionOutcome::Promoted { .. } => report.promoted += 1,
            QuarantinePromotionOutcome::AlreadyPromoted { .. } => report.already_promoted += 1,
            QuarantinePromotionOutcome::StillDeferred { .. } => report.deferred += 1,
            QuarantinePromotionOutcome::Rejected { .. } => report.rejected += 1,
        }
        report.outcomes.push(outcome);
    }
    Ok(report)
}

'''
replace_once("packages/core/src/lib.rs", marker, addition + marker)

replace_once(
    "packages/relay/src/main.rs",
    '''    import_archive, promote_quarantine_record, quarantine_record_json,
    quarantine_resolution_json, runtime_state_dir, QuarantinePromotionOutcome, QuarantineStore,
    StorageBackend,
''',
    '''    import_archive, promote_quarantine_batch, promote_quarantine_record,
    quarantine_record_json, quarantine_resolution_json, runtime_state_dir,
    QuarantineBatchReport, QuarantinePromotionOutcome, QuarantineStore, StorageBackend,
''',
)
replace_once(
    "packages/relay/src/main.rs",
    'quarantine-list|quarantine-get|quarantine-promote|quarantine-resolutions|capabilities',
    'quarantine-list|quarantine-get|quarantine-promote|quarantine-promote-batch|quarantine-resolutions|capabilities',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        "quarantine-resolutions" => handle_quarantine_resolutions(),
''',
    '''        "quarantine-promote-batch" => {
            let limit = parse_batch_limit(args.get(1).map(String::as_str))?;
            let dry_run = args.iter().any(|arg| arg == "--dry-run");
            handle_quarantine_promote_batch(limit, dry_run, &backend)
        }
        "quarantine-resolutions" => handle_quarantine_resolutions(),
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''fn handle_quarantine_resolutions() -> Result<(), String> {
''',
    r'''fn parse_batch_limit(value: Option<&str>) -> Result<usize, String> {
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
    let report = promote_quarantine_batch(limit, dry_run, backend)
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&batch_report_json(report)));
    Ok(())
}

fn handle_quarantine_resolutions() -> Result<(), String> {
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''        ("GET", "/v1/quarantine-resolutions") => handle_http_quarantine_resolutions(),
''',
    '''        ("GET", "/v1/quarantine-resolutions") => handle_http_quarantine_resolutions(),
        ("POST", "/v1/quarantine/promote-batch") => {
            handle_http_quarantine_promote_batch(body, backend)
        }
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''fn handle_http_quarantine_resolutions() -> Result<(u16, &'static str, JsonValue), String> {
''',
    r'''fn handle_http_quarantine_promote_batch(
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    let (limit, dry_run) = if body.trim().is_empty() {
        (100, false)
    } else {
        let value = lingonberry_protocol::parse_json(body).map_err(|error| error.to_string())?;
        let map = as_object(&value)
            .ok_or_else(|| "batch request must be an object".to_string())?;
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
    let report = promote_quarantine_batch(limit, dry_run, backend)
        .map_err(|error| error.to_string())?;
    Ok((200, "OK", batch_report_json(report)))
}

fn handle_http_quarantine_resolutions() -> Result<(u16, &'static str, JsonValue), String> {
''',
)

policy = Path("docs/operations/ACCEPTANCE_POLICY.md")
policy.write_text(policy.read_text() + '''

## Batch revalidation

Unresolved quarantine records can be processed in bounded batches.

```bash
lingonberry quarantine-promote-batch [limit] [--dry-run]
```

The default limit is 100 and the maximum is 1000. `--dry-run` evaluates records without writing to canonical storage or the resolution ledger.

HTTP equivalent:

```text
POST /v1/quarantine/promote-batch
```

Example body:

```json
{
  "limit": 100,
  "dryRun": true
}
```

Only records without an existing resolution are selected. The response includes aggregate counts and the outcome for each scanned record, making the command suitable for a scheduler or periodic maintenance job.
''')
