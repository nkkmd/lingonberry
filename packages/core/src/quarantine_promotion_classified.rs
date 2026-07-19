use crate::{
    append_publish_request_classified, runtime_state_dir, store_error, QuarantinePromotionOutcome,
    QuarantineStore, StorageBackend, StoreError,
};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::{
    evaluate_acceptance, finalize_knowledge_object_full, validate_knowledge_object_full,
    AcceptanceDecision, AcceptancePolicy,
};
use std::collections::BTreeMap;

pub fn promote_quarantine_record_classified(
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
    let policy =
        AcceptancePolicy::from_env().map_err(|error| store_error("LB_ACCEPTANCE_POLICY", error))?;
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
    let outcome = append_publish_request_classified(backend, &record.request_json, &finalized)?;
    store.append_resolution(quarantine_id, &outcome.canonical_id, outcome.duplicate)?;
    Ok(QuarantinePromotionOutcome::Promoted {
        quarantine_id: quarantine_id.to_string(),
        canonical_id: outcome.canonical_id,
        duplicate: outcome.duplicate,
    })
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}
