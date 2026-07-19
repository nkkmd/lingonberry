use crate::{
    append_publish_request_classified, as_object, as_string, read_lines, runtime_state_dir,
    store_error, validate_archive_manifest, ArchiveImportReport, QuarantineStore, StorageBackend,
    StoreError,
};
use lingonberry_protocol::parse_json;
use lingonberry_validation::{
    evaluate_acceptance, finalize_knowledge_object_full, validate_knowledge_object_full,
    AcceptanceDecision, AcceptancePolicy,
};
use std::fs;
use std::path::Path;

pub fn import_archive_classified(
    backend: &impl StorageBackend,
    archive_dir: impl AsRef<Path>,
) -> Result<ArchiveImportReport, StoreError> {
    let archive_dir = archive_dir.as_ref().to_path_buf();
    let manifest_path = archive_dir.join("manifest.json");
    let wire_log_path = archive_dir.join("wire-log.jsonl");
    let manifest_raw = fs::read_to_string(&manifest_path)
        .map_err(|error| store_error("LB_IO_ERROR", error.to_string()))?;
    let manifest_value = parse_json(&manifest_raw)
        .map_err(|error| store_error("LB_ARCHIVE_IMPORT", error.to_string()))?;
    validate_archive_manifest(&manifest_value)?;

    let lines = read_lines(&wire_log_path)?;
    let mut imported = 0usize;
    let mut duplicates = 0usize;
    for line in lines {
        let value = parse_json(&line)
            .map_err(|error| store_error("LB_ARCHIVE_IMPORT", error.to_string()))?;
        let Some(map) = as_object(&value) else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "wire-log record must be an object",
            ));
        };
        let Some(request_json) = map.get("requestJson").and_then(as_string) else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "wire-log record missing requestJson",
            ));
        };
        let request_value = parse_json(request_json)
            .map_err(|error| store_error("LB_ARCHIVE_IMPORT", error.to_string()))?;
        let Some(request_map) = as_object(&request_value) else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "requestJson is not a publish request",
            ));
        };
        let Some(object_value) = request_map.get("object") else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "publish request missing object",
            ));
        };
        let report = validate_knowledge_object_full(object_value);
        let policy = AcceptancePolicy::from_env()
            .map_err(|error| store_error("LB_ACCEPTANCE_POLICY", error))?;
        match evaluate_acceptance(&report, &policy) {
            AcceptanceDecision::Accept => {}
            AcceptanceDecision::Reject { code, errors } => {
                return Err(store_error(code, errors.join("; ")))
            }
            AcceptanceDecision::Defer { code, errors } => {
                let record = QuarantineStore::new(runtime_state_dir()).append(
                    request_json,
                    code,
                    &errors,
                )?;
                return Err(store_error(
                    code,
                    format!("{}; quarantineId={}", errors.join("; "), record.id),
                ));
            }
        }
        let finalized = finalize_knowledge_object_full(object_value).map_err(|report| {
            store_error("LB_ARCHIVE_IMPORT", report.combined_errors().join("; "))
        })?;
        let outcome = append_publish_request_classified(backend, request_json, &finalized)?;
        if outcome.duplicate {
            duplicates += 1;
        } else {
            imported += 1;
        }
    }

    Ok(ArchiveImportReport {
        archive_dir,
        record_count: imported,
        duplicate_count: duplicates,
    })
}
