use crate::{
    index_checkpoint_json, load_index_checkpoint, persist_index_checkpoint, rebuild_index,
    IndexCheckpoint, IndexConsistencyStatus, StorageBackend,
};
use lingonberry_protocol::JsonValue;
use std::collections::BTreeMap;
use std::path::Path;

pub const INDEX_CATCH_UP_CONTRACT_VERSION: &str = "1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexCatchUpStatus {
    UpToDate,
    Rebuilt,
    Failed,
}

impl IndexCatchUpStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UpToDate => "up-to-date",
            Self::Rebuilt => "rebuilt",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexCatchUpResult {
    pub contract_version: &'static str,
    pub status: IndexCatchUpStatus,
    pub code: &'static str,
    pub message: String,
    pub previous_checkpoint: Option<IndexCheckpoint>,
    pub current_checkpoint: Option<IndexCheckpoint>,
}

pub fn catch_up_index(
    backend: &impl StorageBackend,
    checkpoint_path: impl AsRef<Path>,
) -> IndexCatchUpResult {
    let checkpoint_path = checkpoint_path.as_ref();
    let previous_checkpoint = match load_index_checkpoint(checkpoint_path) {
        Ok(checkpoint) => checkpoint,
        Err(error) => return failed_result(error),
    };

    let rebuild = rebuild_index(backend);
    if rebuild.status != IndexConsistencyStatus::Consistent {
        return IndexCatchUpResult {
            contract_version: INDEX_CATCH_UP_CONTRACT_VERSION,
            status: IndexCatchUpStatus::Failed,
            code: rebuild.code,
            message: rebuild.message,
            previous_checkpoint,
            current_checkpoint: None,
        };
    }

    let Some(generation) = rebuild.index.as_ref() else {
        return failed_result("LB_INDEX_CATCH_UP_FAILED: missing rebuilt generation".to_string());
    };

    if previous_checkpoint.as_ref().is_some_and(|checkpoint| {
        checkpoint.generation == generation.generation
            && checkpoint.record_count == generation.record_count
            && checkpoint.id_digest == generation.id_digest
    }) {
        return IndexCatchUpResult {
            contract_version: INDEX_CATCH_UP_CONTRACT_VERSION,
            status: IndexCatchUpStatus::UpToDate,
            code: "LB_INDEX_UP_TO_DATE",
            message: "index checkpoint matches canonical storage".to_string(),
            current_checkpoint: previous_checkpoint.clone(),
            previous_checkpoint,
        };
    }

    match persist_index_checkpoint(checkpoint_path, &rebuild) {
        Ok(current_checkpoint) => IndexCatchUpResult {
            contract_version: INDEX_CATCH_UP_CONTRACT_VERSION,
            status: IndexCatchUpStatus::Rebuilt,
            code: "LB_INDEX_REBUILT",
            message: "index checkpoint rebuilt from canonical storage".to_string(),
            previous_checkpoint,
            current_checkpoint: Some(current_checkpoint),
        },
        Err(error) => failed_result(error),
    }
}

pub fn index_catch_up_result_json(result: &IndexCatchUpResult) -> JsonValue {
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
            "previousCheckpoint",
            checkpoint_json(result.previous_checkpoint.as_ref()),
        ),
        (
            "currentCheckpoint",
            checkpoint_json(result.current_checkpoint.as_ref()),
        ),
    ])
}

fn failed_result(message: String) -> IndexCatchUpResult {
    IndexCatchUpResult {
        contract_version: INDEX_CATCH_UP_CONTRACT_VERSION,
        status: IndexCatchUpStatus::Failed,
        code: catch_up_error_code(&message),
        message,
        previous_checkpoint: None,
        current_checkpoint: None,
    }
}

fn catch_up_error_code(message: &str) -> &'static str {
    if message.starts_with("LB_INDEX_CHECKPOINT_CORRUPT") {
        "LB_INDEX_CHECKPOINT_CORRUPT"
    } else if message.starts_with("LB_INDEX_CHECKPOINT_UNSUPPORTED") {
        "LB_INDEX_CHECKPOINT_UNSUPPORTED"
    } else if message.starts_with("LB_INDEX_CHECKPOINT_IO") {
        "LB_INDEX_CHECKPOINT_IO"
    } else {
        "LB_INDEX_CATCH_UP_FAILED"
    }
}

fn checkpoint_json(checkpoint: Option<&IndexCheckpoint>) -> JsonValue {
    checkpoint
        .map(index_checkpoint_json)
        .unwrap_or(JsonValue::Null)
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<BTreeMap<_, _>>(),
    )
}
