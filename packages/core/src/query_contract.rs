use crate::{StorageBackend, StoredCatalogRecord};
use lingonberry_protocol::JsonValue;
use std::collections::BTreeMap;

pub const BASIC_QUERY_CONTRACT_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicQueryStatus {
    Success,
    Empty,
    InvalidRequest,
    Failed,
}

impl BasicQueryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Empty => "empty",
            Self::InvalidRequest => "invalid-request",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BasicQueryResult {
    pub contract_version: &'static str,
    pub status: BasicQueryStatus,
    pub code: &'static str,
    pub message: String,
    pub object_type: Option<String>,
    pub records: Vec<StoredCatalogRecord>,
}

/// Executes the basic storage-backed query and returns records ordered by canonical ID.
pub fn execute_basic_query(
    object_type: Option<&str>,
    backend: &impl StorageBackend,
) -> BasicQueryResult {
    let normalized_type = match object_type {
        Some(value) if value.trim().is_empty() => {
            return BasicQueryResult {
                contract_version: BASIC_QUERY_CONTRACT_VERSION,
                status: BasicQueryStatus::InvalidRequest,
                code: "LB_QUERY_TYPE_REQUIRED",
                message: "query type must not be empty".to_string(),
                object_type: None,
                records: Vec::new(),
            };
        }
        Some(value) => Some(value.trim().to_string()),
        None => None,
    };

    match backend.subscribe(normalized_type.as_deref()) {
        Ok(mut records) => {
            records.sort_by(|left, right| left.canonical_id.cmp(&right.canonical_id));
            let (status, code, message) = if records.is_empty() {
                (
                    BasicQueryStatus::Empty,
                    "LB_QUERY_EMPTY",
                    "query completed with no matching objects".to_string(),
                )
            } else {
                (
                    BasicQueryStatus::Success,
                    "LB_QUERY_SUCCESS",
                    "query completed".to_string(),
                )
            };
            BasicQueryResult {
                contract_version: BASIC_QUERY_CONTRACT_VERSION,
                status,
                code,
                message,
                object_type: normalized_type,
                records,
            }
        }
        Err(error) => BasicQueryResult {
            contract_version: BASIC_QUERY_CONTRACT_VERSION,
            status: BasicQueryStatus::Failed,
            code: error.code,
            message: error.message,
            object_type: normalized_type,
            records: Vec::new(),
        },
    }
}

pub fn basic_query_result_json(result: &BasicQueryResult) -> JsonValue {
    let objects = result
        .records
        .iter()
        .map(|record| {
            json_object(vec![
                (
                    "canonicalId",
                    JsonValue::String(record.canonical_id.clone()),
                ),
                (
                    "carrierIdentity",
                    JsonValue::String(record.carrier_identity.clone()),
                ),
                ("storedAt", JsonValue::String(record.stored_at.clone())),
                ("canonical", record.object.clone()),
            ])
        })
        .collect();

    json_object(vec![
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
        (
            "filter",
            match &result.object_type {
                Some(value) => json_object(vec![("type", JsonValue::String(value.clone()))]),
                None => json_object(vec![]),
            },
        ),
        ("count", JsonValue::Number(result.records.len().to_string())),
        ("objects", JsonValue::Array(objects)),
        (
            "ordering",
            JsonValue::String("canonicalId-ascending".to_string()),
        ),
    ])
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}
