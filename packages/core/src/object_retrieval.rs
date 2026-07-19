use crate::{StorageBackend, StoredCatalogRecord};
use lingonberry_protocol::{derive_identity_key, JsonValue};
use std::collections::BTreeMap;

pub const OBJECT_RETRIEVAL_CONTRACT_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectRetrievalStatus {
    Found,
    InvalidRequest,
    NotFound,
    Failed,
}

impl ObjectRetrievalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Found => "found",
            Self::InvalidRequest => "invalid-request",
            Self::NotFound => "not-found",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectRetrievalResult {
    pub contract_version: &'static str,
    pub status: ObjectRetrievalStatus,
    pub code: &'static str,
    pub message: String,
    pub record: Option<StoredCatalogRecord>,
}

pub fn retrieve_object(canonical_id: &str, backend: &impl StorageBackend) -> ObjectRetrievalResult {
    if canonical_id.trim().is_empty() {
        return ObjectRetrievalResult {
            contract_version: OBJECT_RETRIEVAL_CONTRACT_VERSION,
            status: ObjectRetrievalStatus::InvalidRequest,
            code: "LB_CANONICAL_ID_REQUIRED",
            message: "missing canonical id".to_string(),
            record: None,
        };
    }

    match backend.get(canonical_id) {
        Ok(Some(record)) => ObjectRetrievalResult {
            contract_version: OBJECT_RETRIEVAL_CONTRACT_VERSION,
            status: ObjectRetrievalStatus::Found,
            code: "LB_OBJECT_FOUND",
            message: "object found".to_string(),
            record: Some(record),
        },
        Ok(None) => ObjectRetrievalResult {
            contract_version: OBJECT_RETRIEVAL_CONTRACT_VERSION,
            status: ObjectRetrievalStatus::NotFound,
            code: "LB_OBJECT_NOT_FOUND",
            message: "object not found".to_string(),
            record: None,
        },
        Err(error) => ObjectRetrievalResult {
            contract_version: OBJECT_RETRIEVAL_CONTRACT_VERSION,
            status: ObjectRetrievalStatus::Failed,
            code: error.code,
            message: error.message,
            record: None,
        },
    }
}

pub fn object_retrieval_result_json(result: &ObjectRetrievalResult) -> JsonValue {
    let mut entries = vec![
        (
            "contractVersion",
            JsonValue::String(result.contract_version.to_string()),
        ),
        (
            "status",
            JsonValue::String(result.status.as_str().to_string()),
        ),
        ("code", JsonValue::String(result.code.to_string())),
        ("message", JsonValue::String(result.message.clone())),
    ];

    if let Some(record) = &result.record {
        entries.extend([
            (
                "canonicalId",
                JsonValue::String(record.canonical_id.clone()),
            ),
            (
                "identityKey",
                JsonValue::String(derive_identity_key(&record.object)),
            ),
            ("storedAt", JsonValue::String(record.stored_at.clone())),
            (
                "carrierIdentity",
                JsonValue::String(record.carrier_identity.clone()),
            ),
            ("canonical", record.object.clone()),
        ]);
    }

    json_object(entries)
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}
