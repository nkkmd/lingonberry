use crate::{AppendOutcome, QuarantineStore, StorageBackend};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::{
    evaluate_acceptance, finalize_knowledge_object_full, validate_publish_request_full,
    AcceptanceDecision, AcceptancePolicy,
};
use std::collections::BTreeMap;

pub const PUBLISH_INGESTION_CONTRACT_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishIngestionStatus {
    Stored,
    Duplicate,
    Deferred,
    Rejected,
    Conflict,
    Failed,
}

impl PublishIngestionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stored => "stored",
            Self::Duplicate => "duplicate",
            Self::Deferred => "deferred",
            Self::Rejected => "rejected",
            Self::Conflict => "conflict",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublishIngestionResult {
    pub contract_version: &'static str,
    pub status: PublishIngestionStatus,
    pub code: String,
    pub errors: Vec<String>,
    pub canonical_id: Option<String>,
    pub identity_key: Option<String>,
    pub carrier_identity: Option<String>,
    pub stored_at: Option<String>,
    pub object: Option<JsonValue>,
    pub quarantine_id: Option<String>,
}

impl PublishIngestionResult {
    pub fn stored(&self) -> bool {
        matches!(self.status, PublishIngestionStatus::Stored | PublishIngestionStatus::Duplicate)
    }

    pub fn duplicate(&self) -> bool {
        self.status == PublishIngestionStatus::Duplicate
    }
}

pub fn ingest_publish_request(
    request_json: &str,
    backend: &impl StorageBackend,
    quarantine: &QuarantineStore,
    policy: &AcceptancePolicy,
) -> PublishIngestionResult {
    if request_json.trim().is_empty() {
        return terminal_result(
            PublishIngestionStatus::Rejected,
            "LB_EMPTY_REQUEST",
            vec!["publish request body is empty".to_string()],
        );
    }

    let request = match parse_json(request_json) {
        Ok(value) => value,
        Err(error) => {
            return terminal_result(
                PublishIngestionStatus::Rejected,
                "LB_INVALID_JSON",
                vec![error.to_string()],
            )
        }
    };

    let report = validate_publish_request_full(&request);
    match evaluate_acceptance(&report, policy) {
        AcceptanceDecision::Reject { code, errors } => {
            return terminal_result(PublishIngestionStatus::Rejected, code, errors)
        }
        AcceptanceDecision::Defer { code, errors } => {
            return match quarantine.append(request_json, code, &errors) {
                Ok(record) => PublishIngestionResult {
                    contract_version: PUBLISH_INGESTION_CONTRACT_VERSION,
                    status: PublishIngestionStatus::Deferred,
                    code: code.to_string(),
                    errors,
                    canonical_id: None,
                    identity_key: None,
                    carrier_identity: None,
                    stored_at: None,
                    object: None,
                    quarantine_id: Some(record.id),
                },
                Err(error) => terminal_result(
                    PublishIngestionStatus::Failed,
                    error.code,
                    vec![error.message],
                ),
            }
        }
        AcceptanceDecision::Accept => {}
    }

    let object = match as_object(&request).and_then(|map| map.get("object")) {
        Some(object) => object,
        None => {
            return terminal_result(
                PublishIngestionStatus::Rejected,
                "LB_PUBLISH_REQUEST_OBJECT_MISSING",
                vec!["publish request missing object".to_string()],
            )
        }
    };

    let finalized = match finalize_knowledge_object_full(object) {
        Ok(finalized) => finalized,
        Err(report) => {
            return terminal_result(
                PublishIngestionStatus::Rejected,
                "LB_VALIDATION_FAILED",
                report.combined_errors(),
            )
        }
    };

    match backend.append_publish_request(request_json, &finalized) {
        Ok(outcome) => successful_result(finalized.identity_key, outcome),
        Err(error) if error.code == "LB_OBJECT_CONFLICT" => terminal_result(
            PublishIngestionStatus::Conflict,
            error.code,
            vec![error.message],
        ),
        Err(error) => terminal_result(
            PublishIngestionStatus::Failed,
            error.code,
            vec![error.message],
        ),
    }
}

pub fn publish_ingestion_result_json(result: &PublishIngestionResult) -> JsonValue {
    let mut entries = vec![
        (
            "contractVersion",
            JsonValue::String(result.contract_version.to_string()),
        ),
        (
            "status",
            JsonValue::String(result.status.as_str().to_string()),
        ),
        ("code", JsonValue::String(result.code.clone())),
        ("stored", JsonValue::Bool(result.stored())),
        ("duplicate", JsonValue::Bool(result.duplicate())),
        (
            "errors",
            JsonValue::Array(result.errors.iter().cloned().map(JsonValue::String).collect()),
        ),
    ];
    push_optional_string(&mut entries, "canonicalId", &result.canonical_id);
    push_optional_string(&mut entries, "identityKey", &result.identity_key);
    push_optional_string(&mut entries, "carrierIdentity", &result.carrier_identity);
    push_optional_string(&mut entries, "storedAt", &result.stored_at);
    push_optional_string(&mut entries, "quarantineId", &result.quarantine_id);
    if let Some(object) = &result.object {
        entries.push(("object", object.clone()));
    }
    json_object(entries)
}

fn successful_result(identity_key: String, outcome: AppendOutcome) -> PublishIngestionResult {
    PublishIngestionResult {
        contract_version: PUBLISH_INGESTION_CONTRACT_VERSION,
        status: if outcome.duplicate {
            PublishIngestionStatus::Duplicate
        } else {
            PublishIngestionStatus::Stored
        },
        code: if outcome.duplicate {
            "LB_OBJECT_DUPLICATE".to_string()
        } else {
            "LB_OBJECT_STORED".to_string()
        },
        errors: Vec::new(),
        canonical_id: Some(outcome.canonical_id),
        identity_key: Some(identity_key),
        carrier_identity: Some(outcome.carrier_identity),
        stored_at: outcome.stored_at,
        object: Some(outcome.object),
        quarantine_id: None,
    }
}

fn terminal_result(
    status: PublishIngestionStatus,
    code: impl Into<String>,
    errors: Vec<String>,
) -> PublishIngestionResult {
    PublishIngestionResult {
        contract_version: PUBLISH_INGESTION_CONTRACT_VERSION,
        status,
        code: code.into(),
        errors,
        canonical_id: None,
        identity_key: None,
        carrier_identity: None,
        stored_at: None,
        object: None,
        quarantine_id: None,
    }
}

fn push_optional_string<'a>(
    entries: &mut Vec<(&'a str, JsonValue)>,
    key: &'a str,
    value: &Option<String>,
) {
    if let Some(value) = value {
        entries.push((key, JsonValue::String(value.clone())));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_request_is_versioned_rejection() {
        let result = terminal_result(
            PublishIngestionStatus::Rejected,
            "LB_EMPTY_REQUEST",
            vec!["publish request body is empty".to_string()],
        );
        let json = publish_ingestion_result_json(&result);
        let map = as_object(&json).expect("result must be an object");
        assert_eq!(
            map.get("contractVersion"),
            Some(&JsonValue::String("1".to_string()))
        );
        assert_eq!(
            map.get("status"),
            Some(&JsonValue::String("rejected".to_string()))
        );
        assert_eq!(map.get("stored"), Some(&JsonValue::Bool(false)));
    }

    #[test]
    fn duplicate_is_successful_but_explicit() {
        let result = successful_result(
            "identity-key".to_string(),
            AppendOutcome {
                stored_at: Some("2026-07-19T00:00:00Z".to_string()),
                canonical_id: "object-1".to_string(),
                carrier_identity: "carrier-1".to_string(),
                object: JsonValue::Object(BTreeMap::new()),
                duplicate: true,
            },
        );
        assert!(result.stored());
        assert!(result.duplicate());
        assert_eq!(result.status, PublishIngestionStatus::Duplicate);
        assert_eq!(result.code, "LB_OBJECT_DUPLICATE");
    }
}
