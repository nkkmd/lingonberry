use crate::IndexSnapshot;
use lingonberry_core::{StorageBackend, StoreError};
use lingonberry_protocol::JsonValue;
use std::collections::{BTreeMap, BTreeSet};

pub const INDEX_LIFECYCLE_CONTRACT_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexConsistencyStatus {
    Consistent,
    Inconsistent,
    Failed,
}

impl IndexConsistencyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Consistent => "consistent",
            Self::Inconsistent => "inconsistent",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGeneration {
    pub generation: String,
    pub record_count: usize,
    pub id_digest: String,
}

#[derive(Debug, Clone)]
pub struct IndexRebuildResult {
    pub contract_version: &'static str,
    pub status: IndexConsistencyStatus,
    pub code: &'static str,
    pub message: String,
    pub storage: Option<IndexGeneration>,
    pub index: Option<IndexGeneration>,
    pub missing_from_index: Vec<String>,
    pub unexpected_in_index: Vec<String>,
    pub snapshot: Option<IndexSnapshot>,
}

pub fn rebuild_index(backend: &impl StorageBackend) -> IndexRebuildResult {
    let storage_records = match backend.subscribe(None) {
        Ok(records) => records,
        Err(error) => return failed_result(error),
    };
    let storage_ids = storage_records
        .iter()
        .map(|record| record.canonical_id.clone())
        .collect::<BTreeSet<_>>();
    let snapshot = IndexSnapshot::from_records(storage_records);
    verify_snapshot(storage_ids, snapshot)
}

pub fn verify_index(
    backend: &impl StorageBackend,
    snapshot: IndexSnapshot,
) -> IndexRebuildResult {
    let storage_ids = match backend.list_ids() {
        Ok(ids) => ids.into_iter().collect::<BTreeSet<_>>(),
        Err(error) => return failed_result(error),
    };
    verify_snapshot(storage_ids, snapshot)
}

pub fn index_rebuild_result_json(result: &IndexRebuildResult) -> JsonValue {
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
        ("storage", generation_json(result.storage.as_ref())),
        ("index", generation_json(result.index.as_ref())),
        (
            "missingFromIndex",
            JsonValue::Array(
                result
                    .missing_from_index
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect(),
            ),
        ),
        (
            "unexpectedInIndex",
            JsonValue::Array(
                result
                    .unexpected_in_index
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect(),
            ),
        ),
    ])
}

fn verify_snapshot(
    storage_ids: BTreeSet<String>,
    snapshot: IndexSnapshot,
) -> IndexRebuildResult {
    let index_ids = snapshot.canonical_ids().into_iter().collect::<BTreeSet<_>>();
    let missing_from_index = storage_ids.difference(&index_ids).cloned().collect::<Vec<_>>();
    let unexpected_in_index = index_ids.difference(&storage_ids).cloned().collect::<Vec<_>>();
    let storage = generation(&storage_ids);
    let index = generation(&index_ids);
    let consistent = missing_from_index.is_empty()
        && unexpected_in_index.is_empty()
        && storage.record_count == index.record_count
        && storage.id_digest == index.id_digest;
    IndexRebuildResult {
        contract_version: INDEX_LIFECYCLE_CONTRACT_VERSION,
        status: if consistent {
            IndexConsistencyStatus::Consistent
        } else {
            IndexConsistencyStatus::Inconsistent
        },
        code: if consistent {
            "LB_INDEX_CONSISTENT"
        } else {
            "LB_INDEX_INCONSISTENT"
        },
        message: if consistent {
            "index matches canonical storage".to_string()
        } else {
            "index does not match canonical storage".to_string()
        },
        storage: Some(storage),
        index: Some(index),
        missing_from_index,
        unexpected_in_index,
        snapshot: Some(snapshot),
    }
}

fn generation(ids: &BTreeSet<String>) -> IndexGeneration {
    let id_digest = digest_ids(ids);
    IndexGeneration {
        generation: format!("idx:{id_digest}"),
        record_count: ids.len(),
        id_digest,
    }
}

fn digest_ids(ids: &BTreeSet<String>) -> String {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut digest = OFFSET_BASIS;
    for id in ids {
        for byte in id.as_bytes().iter().copied().chain(std::iter::once(b'\n')) {
            digest ^= u64::from(byte);
            digest = digest.wrapping_mul(PRIME);
        }
    }
    format!("fnv1a64:{digest:016x}")
}

fn failed_result(error: StoreError) -> IndexRebuildResult {
    IndexRebuildResult {
        contract_version: INDEX_LIFECYCLE_CONTRACT_VERSION,
        status: IndexConsistencyStatus::Failed,
        code: error.code,
        message: error.message,
        storage: None,
        index: None,
        missing_from_index: Vec::new(),
        unexpected_in_index: Vec::new(),
        snapshot: None,
    }
}

fn generation_json(generation: Option<&IndexGeneration>) -> JsonValue {
    match generation {
        Some(value) => json_object(vec![
            (
                "generation",
                JsonValue::String(value.generation.clone()),
            ),
            (
                "recordCount",
                JsonValue::Number(value.record_count.to_string()),
            ),
            ("idDigest", JsonValue::String(value.id_digest.clone())),
        ]),
        None => JsonValue::Null,
    }
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}
