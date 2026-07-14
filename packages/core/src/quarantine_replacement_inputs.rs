use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{store_error, StoreError};

pub const QUARANTINE_REPLACEMENT_INPUTS_FILE: &str = "quarantine-replacement-inputs.json";
pub const QUARANTINE_REPLACEMENT_INPUTS_VERSION: &str =
    "lingonberry-quarantine-replacement-inputs/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementInputs {
    pub backup_dir: PathBuf,
    pub proof_dir: PathBuf,
}

pub fn write_quarantine_replacement_inputs(
    transaction_dir: impl AsRef<Path>,
    backup_dir: impl AsRef<Path>,
    proof_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementInputs, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let backup_dir = fs::canonicalize(backup_dir).map_err(io_error)?;
    let proof_dir = fs::canonicalize(proof_dir).map_err(io_error)?;
    let inputs = QuarantineReplacementInputs {
        backup_dir,
        proof_dir,
    };
    let value = JsonValue::Object(BTreeMap::from([
        (
            "backupDir".to_string(),
            JsonValue::String(inputs.backup_dir.to_string_lossy().to_string()),
        ),
        (
            "proofDir".to_string(),
            JsonValue::String(inputs.proof_dir.to_string_lossy().to_string()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_INPUTS_VERSION.to_string()),
        ),
    ]));
    let path = transaction_dir.join(QUARANTINE_REPLACEMENT_INPUTS_FILE);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(io_error)?;
    file.write_all(to_canonical_json(&value).as_bytes())
        .map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    File::open(transaction_dir)
        .map_err(io_error)?
        .sync_all()
        .map_err(io_error)?;
    read_quarantine_replacement_inputs(transaction_dir)
}

pub fn read_quarantine_replacement_inputs(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementInputs, StoreError> {
    let path = transaction_dir
        .as_ref()
        .join(QUARANTINE_REPLACEMENT_INPUTS_FILE);
    let text = fs::read_to_string(path).map_err(io_error)?;
    let value = parse_json(&text)
        .map_err(|error| inputs_error(&format!("invalid replacement input JSON: {error}")))?;
    require_string(&value, "version", QUARANTINE_REPLACEMENT_INPUTS_VERSION)?;
    let backup_dir = PathBuf::from(object_string(&value, "backupDir")?);
    let proof_dir = PathBuf::from(object_string(&value, "proofDir")?);
    if !backup_dir.is_dir() || !proof_dir.is_dir() {
        return Err(inputs_error(
            "replacement backup or proof directory is unavailable",
        ));
    }
    Ok(QuarantineReplacementInputs {
        backup_dir,
        proof_dir,
    })
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(inputs_error("expected replacement input object")),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(_) => Err(inputs_error(&format!("invalid string field: {name}"))),
        None => Err(inputs_error(&format!("missing field: {name}"))),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(inputs_error(&format!("unexpected {name}")));
    }
    Ok(())
}

fn io_error(error: std::io::Error) -> StoreError {
    inputs_error(&error.to_string())
}

fn inputs_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_TRANSACTION", message.to_string())
}
