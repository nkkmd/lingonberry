use crate::IndexSnapshot;
use lingonberry_core::{StorageBackend, StoreError, StoredCatalogRecord};
use lingonberry_protocol::{to_canonical_json, JsonValue};
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
    pub content_digest: String,
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
    pub ambiguous_ids: Vec<String>,
    pub snapshot: Option<IndexSnapshot>,
}

pub fn rebuild_index(backend: &impl StorageBackend) -> IndexRebuildResult {
    let storage_records = match backend.subscribe(None) {
        Ok(records) => records,
        Err(error) => return failed_result(error),
    };
    let snapshot = IndexSnapshot::from_records(storage_records.clone());
    verify_snapshot(storage_records, snapshot)
}

pub fn verify_index(backend: &impl StorageBackend, snapshot: IndexSnapshot) -> IndexRebuildResult {
    let storage_records = match backend.subscribe(None) {
        Ok(records) => records,
        Err(error) => return failed_result(error),
    };
    verify_snapshot(storage_records, snapshot)
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
            string_array(&result.missing_from_index),
        ),
        (
            "unexpectedInIndex",
            string_array(&result.unexpected_in_index),
        ),
        ("ambiguousIds", string_array(&result.ambiguous_ids)),
    ])
}

fn verify_snapshot(
    storage_records: Vec<StoredCatalogRecord>,
    snapshot: IndexSnapshot,
) -> IndexRebuildResult {
    let storage_fingerprints = storage_records
        .iter()
        .map(|record| {
            (
                record.canonical_id.clone(),
                record_fingerprint(
                    &record.carrier_identity,
                    &record.stored_at,
                    &record.object,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let index_fingerprints = snapshot
        .records
        .iter()
        .map(|(canonical_id, record)| {
            (
                canonical_id.clone(),
                record_fingerprint(
                    &record.carrier_identity,
                    &record.stored_at,
                    &record.object,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let storage_ids = storage_fingerprints.keys().cloned().collect::<BTreeSet<_>>();
    let index_ids = index_fingerprints.keys().cloned().collect::<BTreeSet<_>>();
    let missing_from_index = storage_ids
        .difference(&index_ids)
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_in_index = index_ids
        .difference(&storage_ids)
        .cloned()
        .collect::<Vec<_>>();
    let ambiguous_ids = storage_ids
        .intersection(&index_ids)
        .filter(|canonical_id| {
            storage_fingerprints.get(*canonical_id) != index_fingerprints.get(*canonical_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    let storage = generation(&storage_fingerprints);
    let index = generation(&index_fingerprints);
    let consistent = missing_from_index.is_empty()
        && unexpected_in_index.is_empty()
        && ambiguous_ids.is_empty()
        && storage.record_count == index.record_count
        && storage.id_digest == index.id_digest
        && storage.content_digest == index.content_digest;
    let ambiguous = missing_from_index.is_empty()
        && unexpected_in_index.is_empty()
        && !ambiguous_ids.is_empty();
    IndexRebuildResult {
        contract_version: INDEX_LIFECYCLE_CONTRACT_VERSION,
        status: if consistent {
            IndexConsistencyStatus::Consistent
        } else {
            IndexConsistencyStatus::Inconsistent
        },
        code: if consistent {
            "LB_INDEX_CONSISTENT"
        } else if ambiguous {
            "LB_INDEX_AMBIGUOUS"
        } else {
            "LB_INDEX_INCONSISTENT"
        },
        message: if consistent {
            "index matches canonical storage".to_string()
        } else if ambiguous {
            "index IDs match canonical storage but record content is ambiguous".to_string()
        } else {
            "index does not match canonical storage".to_string()
        },
        storage: Some(storage),
        index: Some(index),
        missing_from_index,
        unexpected_in_index,
        ambiguous_ids,
        snapshot: Some(snapshot),
    }
}

fn generation(fingerprints: &BTreeMap<String, String>) -> IndexGeneration {
    let ids = fingerprints.keys().cloned().collect::<BTreeSet<_>>();
    let id_digest = digest_lines(ids.iter().map(String::as_str));
    let content_digest = digest_lines(
        fingerprints
            .iter()
            .map(|(canonical_id, fingerprint)| format!("{canonical_id}\0{fingerprint}"))
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str),
    );
    IndexGeneration {
        generation: format!("idx:{id_digest}"),
        record_count: fingerprints.len(),
        id_digest,
        content_digest,
    }
}

fn record_fingerprint(carrier_identity: &str, stored_at: &str, object: &JsonValue) -> String {
    digest_lines([
        carrier_identity,
        stored_at,
        to_canonical_json(object).as_str(),
    ])
}

fn digest_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> String {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut digest = OFFSET_BASIS;
    for line in lines {
        for byte in line
            .as_bytes()
            .iter()
            .copied()
            .chain(std::iter::once(b'\n'))
        {
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
        ambiguous_ids: Vec::new(),
        snapshot: None,
    }
}

fn generation_json(generation: Option<&IndexGeneration>) -> JsonValue {
    match generation {
        Some(value) => json_object(vec![
            ("generation", JsonValue::String(value.generation.clone())),
            (
                "recordCount",
                JsonValue::Number(value.record_count.to_string()),
            ),
            ("idDigest", JsonValue::String(value.id_digest.clone())),
            (
                "contentDigest",
                JsonValue::String(value.content_digest.clone()),
            ),
        ]),
        None => JsonValue::Null,
    }
}

fn string_array(values: &[String]) -> JsonValue {
    JsonValue::Array(values.iter().cloned().map(JsonValue::String).collect())
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}
