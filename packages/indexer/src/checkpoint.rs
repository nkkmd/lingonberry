use crate::{IndexConsistencyStatus, IndexRebuildResult, INDEX_LIFECYCLE_CONTRACT_VERSION};
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const INDEX_CHECKPOINT_VERSION: &str = "1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexCheckpoint {
    pub checkpoint_version: String,
    pub lifecycle_contract_version: String,
    pub generation: String,
    pub record_count: usize,
    pub id_digest: String,
}

pub fn persist_index_checkpoint(
    path: impl AsRef<Path>,
    result: &IndexRebuildResult,
) -> Result<IndexCheckpoint, String> {
    if result.status != IndexConsistencyStatus::Consistent {
        return Err("LB_INDEX_CHECKPOINT_REFUSED: inconsistent index result".to_string());
    }
    let generation = result
        .index
        .as_ref()
        .ok_or_else(|| "LB_INDEX_CHECKPOINT_REFUSED: missing index generation".to_string())?;
    let checkpoint = IndexCheckpoint {
        checkpoint_version: INDEX_CHECKPOINT_VERSION.to_string(),
        lifecycle_contract_version: INDEX_LIFECYCLE_CONTRACT_VERSION.to_string(),
        generation: generation.generation.clone(),
        record_count: generation.record_count,
        id_digest: generation.id_digest.clone(),
    };
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("LB_INDEX_CHECKPOINT_IO: failed to create checkpoint directory: {error}")
        })?;
    }
    let temporary = temporary_path(path);
    fs::write(&temporary, to_canonical_json(&checkpoint_json(&checkpoint)))
        .map_err(|error| format!("LB_INDEX_CHECKPOINT_IO: failed to write checkpoint: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| {
        fs::remove_file(&temporary).ok();
        format!("LB_INDEX_CHECKPOINT_IO: failed to commit checkpoint: {error}")
    })?;
    Ok(checkpoint)
}

pub fn load_index_checkpoint(path: impl AsRef<Path>) -> Result<Option<IndexCheckpoint>, String> {
    let path = path.as_ref();
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "LB_INDEX_CHECKPOINT_IO: failed to read checkpoint: {error}"
            ))
        }
    };
    let value =
        parse_json(&raw).map_err(|error| format!("LB_INDEX_CHECKPOINT_CORRUPT: {error}"))?;
    parse_checkpoint(&value).map(Some)
}

pub fn index_checkpoint_json(checkpoint: &IndexCheckpoint) -> JsonValue {
    checkpoint_json(checkpoint)
}

fn checkpoint_json(checkpoint: &IndexCheckpoint) -> JsonValue {
    json_object(vec![
        (
            "checkpointVersion",
            JsonValue::String(checkpoint.checkpoint_version.clone()),
        ),
        (
            "lifecycleContractVersion",
            JsonValue::String(checkpoint.lifecycle_contract_version.clone()),
        ),
        (
            "generation",
            JsonValue::String(checkpoint.generation.clone()),
        ),
        (
            "recordCount",
            JsonValue::Number(checkpoint.record_count.to_string()),
        ),
        ("idDigest", JsonValue::String(checkpoint.id_digest.clone())),
    ])
}

fn parse_checkpoint(value: &JsonValue) -> Result<IndexCheckpoint, String> {
    let JsonValue::Object(map) = value else {
        return Err("LB_INDEX_CHECKPOINT_CORRUPT: checkpoint must be an object".to_string());
    };
    let checkpoint_version = required_string(map, "checkpointVersion")?;
    if checkpoint_version != INDEX_CHECKPOINT_VERSION {
        return Err(format!(
            "LB_INDEX_CHECKPOINT_UNSUPPORTED: checkpoint version {checkpoint_version}"
        ));
    }
    let lifecycle_contract_version = required_string(map, "lifecycleContractVersion")?;
    if lifecycle_contract_version != INDEX_LIFECYCLE_CONTRACT_VERSION {
        return Err(format!(
            "LB_INDEX_CHECKPOINT_UNSUPPORTED: lifecycle contract version {lifecycle_contract_version}"
        ));
    }
    let generation = required_string(map, "generation")?;
    let id_digest = required_string(map, "idDigest")?;
    if generation != format!("idx:{id_digest}") {
        return Err("LB_INDEX_CHECKPOINT_CORRUPT: generation and digest disagree".to_string());
    }
    let record_count = required_usize(map, "recordCount")?;
    Ok(IndexCheckpoint {
        checkpoint_version,
        lifecycle_contract_version,
        generation,
        record_count,
        id_digest,
    })
}

fn required_string(map: &BTreeMap<String, JsonValue>, key: &str) -> Result<String, String> {
    match map.get(key) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(format!(
            "LB_INDEX_CHECKPOINT_CORRUPT: missing or invalid {key}"
        )),
    }
}

fn required_usize(map: &BTreeMap<String, JsonValue>, key: &str) -> Result<usize, String> {
    match map.get(key) {
        Some(JsonValue::Number(value)) => value
            .parse::<usize>()
            .map_err(|_| format!("LB_INDEX_CHECKPOINT_CORRUPT: invalid {key}")),
        _ => Err(format!(
            "LB_INDEX_CHECKPOINT_CORRUPT: missing or invalid {key}"
        )),
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("checkpoint.json");
    path.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()))
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}
