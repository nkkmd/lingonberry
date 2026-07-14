use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, JsonValue};

use crate::{
    store_error, StoreError, QUARANTINE_BACKUP_FILES,
    QUARANTINE_CURRENT_GENERATION_POINTER_FILE, QUARANTINE_CURRENT_GENERATION_POINTER_VERSION,
    QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE,
    QUARANTINE_REPLACEMENT_GENERATION_VERSION,
};

pub const QUARANTINE_GENERATIONS_DIR: &str = "quarantine-generations";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineActiveGeneration {
    pub active_dir: PathBuf,
    pub transaction_id: Option<String>,
    pub generation_digest: Option<String>,
}

pub fn resolve_quarantine_active_generation(
    state_dir: impl AsRef<Path>,
) -> Result<QuarantineActiveGeneration, StoreError> {
    let state_dir = state_dir.as_ref();
    let pointer_path = state_dir.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE);
    if !pointer_path.exists() {
        return Ok(QuarantineActiveGeneration {
            active_dir: state_dir.to_path_buf(),
            transaction_id: None,
            generation_digest: None,
        });
    }
    if !pointer_path.is_file() {
        return Err(generation_error("current-generation pointer is not a regular file"));
    }

    let pointer_text = fs::read_to_string(&pointer_path).map_err(io_error)?;
    let pointer = parse_json(&pointer_text)
        .map_err(|error| generation_error(&format!("invalid current-generation pointer JSON: {error}")))?;
    require_string(
        &pointer,
        "version",
        QUARANTINE_CURRENT_GENERATION_POINTER_VERSION,
    )?;
    let transaction_id = object_string(&pointer, "transactionId")?;
    validate_transaction_id(&transaction_id)?;
    let generation_digest = object_string(&pointer, "generationDigest")?;
    validate_digest(&generation_digest)?;

    let active_dir = state_dir
        .join(QUARANTINE_GENERATIONS_DIR)
        .join(&transaction_id);
    let metadata = fs::symlink_metadata(&active_dir).map_err(|error| {
        generation_error(&format!(
            "current generation directory is unavailable: {error}"
        ))
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(generation_error(
            "current generation path is not a regular directory",
        ));
    }

    let manifest_path = active_dir.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE);
    let digest_path = active_dir.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE);
    let manifest_text = fs::read_to_string(&manifest_path).map_err(io_error)?;
    let persisted_digest = fs::read_to_string(&digest_path).map_err(io_error)?;
    let persisted_digest = persisted_digest.trim();
    validate_digest(persisted_digest)?;
    if persisted_digest != generation_digest || integrity_digest(manifest_text.as_bytes()) != generation_digest {
        return Err(generation_error(
            "current generation metadata does not match the pointer",
        ));
    }

    let manifest = parse_json(&manifest_text)
        .map_err(|error| generation_error(&format!("invalid current generation manifest: {error}")))?;
    require_string(
        &manifest,
        "version",
        QUARANTINE_REPLACEMENT_GENERATION_VERSION,
    )?;
    if object_string(&manifest, "transactionId")? != transaction_id {
        return Err(generation_error(
            "current generation manifest transaction mismatch",
        ));
    }

    Ok(QuarantineActiveGeneration {
        active_dir,
        transaction_id: Some(transaction_id),
        generation_digest: Some(generation_digest),
    })
}

pub fn resolve_quarantine_active_dir(
    state_dir: impl AsRef<Path>,
) -> Result<PathBuf, StoreError> {
    resolve_quarantine_active_generation(state_dir).map(|generation| generation.active_dir)
}

pub fn resolve_quarantine_active_path(
    state_dir: impl AsRef<Path>,
    ledger: &str,
) -> Result<PathBuf, StoreError> {
    if !QUARANTINE_BACKUP_FILES.contains(&ledger) {
        return Err(generation_error("unsupported managed ledger name"));
    }
    Ok(resolve_quarantine_active_dir(state_dir)?.join(ledger))
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(generation_error("expected JSON object")),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(_) => Err(generation_error(&format!("invalid string field: {name}"))),
        None => Err(generation_error(&format!("missing field: {name}"))),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(generation_error(&format!("unexpected {name}")));
    }
    Ok(())
}

fn validate_transaction_id(value: &str) -> Result<(), StoreError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(generation_error("invalid generation transaction identifier"));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), StoreError> {
    if value.len() != "fnv1a64:".len() + 16
        || !value.starts_with("fnv1a64:")
        || !value["fnv1a64:".len()..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(generation_error("invalid generation digest"));
    }
    Ok(())
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn io_error(error: std::io::Error) -> StoreError {
    generation_error(&error.to_string())
}

fn generation_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_GENERATION", message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::to_canonical_json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-generation-resolver-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn uses_legacy_root_without_pointer() {
        let state = temp_dir();
        fs::create_dir_all(&state).unwrap();
        assert_eq!(resolve_quarantine_active_dir(&state).unwrap(), state);
        let _ = fs::remove_dir_all(state);
    }

    #[test]
    fn resolves_verified_generation_directory() {
        let state = temp_dir();
        let transaction_id = "tx-generation-resolver";
        let generation_dir = state.join(QUARANTINE_GENERATIONS_DIR).join(transaction_id);
        fs::create_dir_all(&generation_dir).unwrap();
        let manifest = JsonValue::Object(BTreeMap::from([
            ("ledgers".to_string(), JsonValue::Array(Vec::new())),
            (
                "sourceDirectory".to_string(),
                JsonValue::String("staged-active-ledgers".to_string()),
            ),
            (
                "transactionId".to_string(),
                JsonValue::String(transaction_id.to_string()),
            ),
            (
                "version".to_string(),
                JsonValue::String(QUARANTINE_REPLACEMENT_GENERATION_VERSION.to_string()),
            ),
        ]));
        let manifest_text = to_canonical_json(&manifest);
        let digest = integrity_digest(manifest_text.as_bytes());
        fs::write(
            generation_dir.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE),
            &manifest_text,
        )
        .unwrap();
        fs::write(
            generation_dir.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE),
            format!("{digest}\n"),
        )
        .unwrap();
        let pointer = JsonValue::Object(BTreeMap::from([
            (
                "generationDigest".to_string(),
                JsonValue::String(digest),
            ),
            (
                "transactionId".to_string(),
                JsonValue::String(transaction_id.to_string()),
            ),
            (
                "version".to_string(),
                JsonValue::String(QUARANTINE_CURRENT_GENERATION_POINTER_VERSION.to_string()),
            ),
        ]));
        fs::write(
            state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE),
            to_canonical_json(&pointer),
        )
        .unwrap();
        assert_eq!(resolve_quarantine_active_dir(&state).unwrap(), generation_dir);
        let _ = fs::remove_dir_all(state);
    }

    #[test]
    fn rejects_pointer_without_generation() {
        let state = temp_dir();
        fs::create_dir_all(&state).unwrap();
        fs::write(
            state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE),
            r#"{"generationDigest":"fnv1a64:0000000000000000","transactionId":"tx-missing","version":"lingonberry-quarantine-current-generation/v1"}"#,
        )
        .unwrap();
        assert_eq!(
            resolve_quarantine_active_dir(&state).unwrap_err().code,
            "LB_QUARANTINE_GENERATION"
        );
        let _ = fs::remove_dir_all(state);
    }
}
