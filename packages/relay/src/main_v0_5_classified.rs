mod existing_v0_5 {
    include!("main_v0_5.rs");

    pub fn run_main() {
        main();
    }
}

use lingonberry_core::{
    build_runtime_storage_backend, import_archive_classified,
    promote_quarantine_batch_classified, promote_quarantine_record_classified,
    ArchiveImportReport, QuarantineBatchReport, QuarantinePromotionOutcome,
};
use lingonberry_protocol::{to_canonical_json, JsonValue};
use std::env;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("quarantine-promote") => handle_quarantine_promote(&args),
        Some("quarantine-promote-batch") => handle_quarantine_promote_batch(&args),
        Some("import-archive") => handle_import_archive(&args),
        _ => {
            existing_v0_5::run_main();
            return;
        }
    };

    if let Err(error) = result {
        eprintln!("{error}");
        process::exit(if error.starts_with("usage:") { 64 } else { 70 });
    }
}

fn handle_quarantine_promote(args: &[String]) -> Result<(), String> {
    let quarantine_id = args
        .get(1)
        .ok_or_else(|| "usage: lingonberry quarantine-promote <quarantine-id>".to_string())?;
    let backend = build_runtime_storage_backend();
    let outcome = promote_quarantine_record_classified(quarantine_id, &backend)
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&promotion_outcome_json(outcome)));
    Ok(())
}

fn handle_quarantine_promote_batch(args: &[String]) -> Result<(), String> {
    let limit = parse_batch_limit(args.get(1).map(String::as_str))?;
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    let backend = build_runtime_storage_backend();
    let report = promote_quarantine_batch_classified(limit, dry_run, &backend)
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&batch_report_json(report)));
    Ok(())
}

fn handle_import_archive(args: &[String]) -> Result<(), String> {
    let archive_dir = args
        .get(1)
        .ok_or_else(|| "usage: lingonberry import-archive <archive-dir>".to_string())?;
    let backend = build_runtime_storage_backend();
    let report = import_archive_classified(&backend, archive_dir)
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&archive_import_report_json(report)));
    Ok(())
}

fn archive_import_report_json(report: ArchiveImportReport) -> JsonValue {
    json_object(vec![
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
    ])
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

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}
