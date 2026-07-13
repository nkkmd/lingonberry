use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    read_quarantine_replacement_transaction_journal, store_error,
    QuarantineReplacementTransactionState, StoreError, QUARANTINE_BACKUP_FILES,
    QUARANTINE_REPLACEMENT_STAGING_DIR, QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE,
    QUARANTINE_REPLACEMENT_STAGING_VERSION,
};

pub const QUARANTINE_REPLACEMENT_PUBLICATION_DIR: &str = "publication";
pub const QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE: &str =
    "quarantine-replacement-generation.json";
pub const QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE: &str =
    "quarantine-replacement-generation.digest";
pub const QUARANTINE_REPLACEMENT_GENERATION_VERSION: &str =
    "lingonberry-quarantine-replacement-generation/v1";
pub const QUARANTINE_CURRENT_GENERATION_POINTER_FILE: &str =
    "quarantine-current-generation.json";
pub const QUARANTINE_CURRENT_GENERATION_POINTER_VERSION: &str =
    "lingonberry-quarantine-current-generation/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementGenerationReport {
    pub publication_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub digest_path: PathBuf,
    pub generation_digest: String,
    pub ledgers: usize,
}

pub fn seal_quarantine_replacement_generation(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementGenerationReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let journal = read_quarantine_replacement_transaction_journal(transaction_dir)?;
    if journal.state != QuarantineReplacementTransactionState::Verified {
        return Err(generation_error(
            "transaction must be verified before sealing a generation",
        ));
    }

    let staging_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_STAGING_DIR);
    let staging_manifest_path = staging_dir.join(QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE);
    let staging_manifest_text = fs::read_to_string(&staging_manifest_path).map_err(io_error)?;
    let staging_manifest = parse_json(&staging_manifest_text)
        .map_err(|error| generation_error(&format!("invalid staging manifest JSON: {error}")))?;
    require_string(
        &staging_manifest,
        "version",
        QUARANTINE_REPLACEMENT_STAGING_VERSION,
    )?;
    if object_string(&staging_manifest, "transactionId")? != journal.transaction_id {
        return Err(generation_error("staging manifest transaction mismatch"));
    }

    let ledgers = object_array(&staging_manifest, "ledgers")?;
    validate_exact_ledger_set(ledgers)?;
    validate_staging_files(&staging_dir, ledgers)?;

    let generation_ledgers = ledgers
        .iter()
        .map(|ledger| {
            let name = object_string(ledger, "ledger")?;
            let present = object_bool(ledger, "present")?;
            let bytes = object_number(ledger, "bytes")?;
            let lines = object_number(ledger, "lines")?;
            let replacements = object_number(ledger, "replacements")?;
            let digest = object_optional_string(ledger, "digest")?;
            if present {
                let path = staging_dir.join(&name);
                let contents = fs::read(&path).map_err(io_error)?;
                if bytes != contents.len() as u64
                    || digest.as_deref() != Some(integrity_digest(&contents).as_str())
                {
                    return Err(generation_error("staged ledger changed after verification"));
                }
            } else if bytes != 0 || lines != 0 || replacements != 0 || digest.is_some() {
                return Err(generation_error("invalid absent ledger metadata"));
            }
            Ok(JsonValue::Object(BTreeMap::from([
                ("bytes".to_string(), JsonValue::Number(bytes.to_string())),
                (
                    "digest".to_string(),
                    digest.map(JsonValue::String).unwrap_or(JsonValue::Null),
                ),
                ("ledger".to_string(), JsonValue::String(name)),
                ("lines".to_string(), JsonValue::Number(lines.to_string())),
                ("present".to_string(), JsonValue::Bool(present)),
                (
                    "replacements".to_string(),
                    JsonValue::Number(replacements.to_string()),
                ),
            ])))
        })
        .collect::<Result<Vec<_>, StoreError>>()?;

    let manifest = JsonValue::Object(BTreeMap::from([
        (
            "ledgers".to_string(),
            JsonValue::Array(generation_ledgers),
        ),
        (
            "sourceDirectory".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_STAGING_DIR.to_string()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(journal.transaction_id),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_GENERATION_VERSION.to_string()),
        ),
    ]));
    let manifest_text = to_canonical_json(&manifest);
    let generation_digest = integrity_digest(manifest_text.as_bytes());

    let publication_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR);
    if publication_dir.exists() {
        return Err(generation_error("publication directory already exists"));
    }
    fs::create_dir(&publication_dir).map_err(io_error)?;
    sync_directory(transaction_dir)?;
    let manifest_path = publication_dir.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE);
    let digest_path = publication_dir.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE);
    write_new_synced(&manifest_path, manifest_text.as_bytes())?;
    write_new_synced(&digest_path, format!("{generation_digest}\n").as_bytes())?;
    sync_directory(&publication_dir)?;

    verify_quarantine_replacement_generation(transaction_dir)
}

pub fn verify_quarantine_replacement_generation(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementGenerationReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let journal = read_quarantine_replacement_transaction_journal(transaction_dir)?;
    let publication_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR);
    let manifest_path = publication_dir.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE);
    let digest_path = publication_dir.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE);
    let manifest_text = fs::read_to_string(&manifest_path).map_err(io_error)?;
    let expected_digest = fs::read_to_string(&digest_path).map_err(io_error)?;
    let expected_digest = expected_digest.trim();
    validate_digest(expected_digest)?;
    let actual_digest = integrity_digest(manifest_text.as_bytes());
    if expected_digest != actual_digest {
        return Err(generation_error("generation manifest digest mismatch"));
    }

    let manifest = parse_json(&manifest_text)
        .map_err(|error| generation_error(&format!("invalid generation manifest JSON: {error}")))?;
    require_string(
        &manifest,
        "version",
        QUARANTINE_REPLACEMENT_GENERATION_VERSION,
    )?;
    if object_string(&manifest, "transactionId")? != journal.transaction_id {
        return Err(generation_error("generation transaction mismatch"));
    }
    require_string(
        &manifest,
        "sourceDirectory",
        QUARANTINE_REPLACEMENT_STAGING_DIR,
    )?;

    let ledgers = object_array(&manifest, "ledgers")?;
    validate_exact_ledger_set(ledgers)?;
    let staging_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_STAGING_DIR);
    validate_staging_files(&staging_dir, ledgers)?;
    for ledger in ledgers {
        let name = object_string(ledger, "ledger")?;
        let present = object_bool(ledger, "present")?;
        let bytes = object_number(ledger, "bytes")?;
        let digest = object_optional_string(ledger, "digest")?;
        if present {
            let contents = fs::read(staging_dir.join(&name)).map_err(io_error)?;
            if contents.len() as u64 != bytes
                || digest.as_deref() != Some(integrity_digest(&contents).as_str())
            {
                return Err(generation_error("sealed generation contains mixed ledger data"));
            }
        } else if bytes != 0 || digest.is_some() {
            return Err(generation_error("sealed generation absent ledger mismatch"));
        }
    }

    Ok(QuarantineReplacementGenerationReport {
        publication_dir,
        manifest_path,
        digest_path,
        generation_digest: actual_digest,
        ledgers: ledgers.len(),
    })
}

pub fn validate_quarantine_current_generation_pointer(
    pointer_text: &str,
    expected_transaction_id: &str,
    expected_generation_digest: &str,
) -> Result<(), StoreError> {
    let pointer = parse_json(pointer_text)
        .map_err(|error| generation_error(&format!("invalid generation pointer JSON: {error}")))?;
    require_string(
        &pointer,
        "version",
        QUARANTINE_CURRENT_GENERATION_POINTER_VERSION,
    )?;
    if object_string(&pointer, "transactionId")? != expected_transaction_id
        || object_string(&pointer, "generationDigest")? != expected_generation_digest
    {
        return Err(generation_error("current-generation pointer mismatch"));
    }
    Ok(())
}

fn validate_exact_ledger_set(ledgers: &[JsonValue]) -> Result<(), StoreError> {
    let names = ledgers
        .iter()
        .map(|ledger| object_string(ledger, "ledger"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    if names != expected || ledgers.len() != QUARANTINE_BACKUP_FILES.len() {
        return Err(generation_error("generation managed-ledger set mismatch"));
    }
    Ok(())
}

fn validate_staging_files(staging_dir: &Path, ledgers: &[JsonValue]) -> Result<(), StoreError> {
    let mut expected = BTreeSet::from([QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE.to_string()]);
    for ledger in ledgers {
        if object_bool(ledger, "present")? {
            expected.insert(object_string(ledger, "ledger")?);
        }
    }
    let actual = fs::read_dir(staging_dir)
        .map_err(io_error)?
        .map(|entry| {
            entry
                .map_err(io_error)
                .map(|entry| entry.file_name().to_string_lossy().to_string())
        })
        .collect::<Result<BTreeSet<_>, StoreError>>()?;
    if actual != expected {
        return Err(generation_error(
            "generation source contains missing or unexpected files",
        ));
    }
    Ok(())
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}

fn sync_directory(path: &Path) -> Result<(), StoreError> {
    File::open(path).map_err(io_error)?.sync_all().map_err(io_error)
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(generation_error("expected JSON object")),
    }
}

fn object_field<'a>(value: &'a JsonValue, name: &str) -> Result<&'a JsonValue, StoreError> {
    object_map(value)?
        .get(name)
        .ok_or_else(|| generation_error(&format!("missing field: {name}")))
}

fn object_array<'a>(value: &'a JsonValue, name: &str) -> Result<&'a [JsonValue], StoreError> {
    match object_field(value, name)? {
        JsonValue::Array(values) => Ok(values),
        _ => Err(generation_error(&format!("invalid array field: {name}"))),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(generation_error(&format!("invalid string field: {name}"))),
    }
}

fn object_optional_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(Some(value.clone())),
        JsonValue::Null => Ok(None),
        _ => Err(generation_error(&format!(
            "invalid optional string field: {name}"
        ))),
    }
}

fn object_bool(value: &JsonValue, name: &str) -> Result<bool, StoreError> {
    match object_field(value, name)? {
        JsonValue::Bool(value) => Ok(*value),
        _ => Err(generation_error(&format!("invalid boolean field: {name}"))),
    }
}

fn object_number(value: &JsonValue, name: &str) -> Result<u64, StoreError> {
    match object_field(value, name)? {
        JsonValue::Number(value) => value
            .parse()
            .map_err(|_| generation_error(&format!("invalid number field: {name}"))),
        _ => Err(generation_error(&format!("invalid number field: {name}"))),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(generation_error(&format!("unexpected {name}")));
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
    store_error("LB_QUARANTINE_REPLACEMENT_PUBLICATION", message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_quarantine_replacement_preview, export_complete_quarantine_backup,
        prepare_quarantine_replacement_transaction, stage_quarantine_replacement_ledgers,
        verify_quarantine_replacement_staging,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn fixture() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
        let state = temp_dir("replacement-generation-state");
        let backup = temp_dir("replacement-generation-backup");
        let proof = temp_dir("replacement-generation-proof");
        let transaction = temp_dir("replacement-generation-transaction");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), b"{\"id\":\"q0\"}\n").unwrap();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"c1\", \"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        create_quarantine_replacement_preview(&state, &backup, &proof).unwrap();
        prepare_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-generation-001",
        )
        .unwrap();
        stage_quarantine_replacement_ledgers(&state, &proof, &transaction).unwrap();
        verify_quarantine_replacement_staging(&state, &proof, &transaction).unwrap();
        (state, backup, proof, transaction)
    }

    #[test]
    fn seals_and_reverifies_complete_generation() {
        let (state, backup, proof, transaction) = fixture();
        let report = seal_quarantine_replacement_generation(&transaction).unwrap();
        assert_eq!(report.ledgers, QUARANTINE_BACKUP_FILES.len());
        assert!(report.manifest_path.is_file());
        assert_eq!(
            verify_quarantine_replacement_generation(&transaction)
                .unwrap()
                .generation_digest,
            report.generation_digest
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn rejects_mixed_generation_after_staged_file_tampering() {
        let (state, backup, proof, transaction) = fixture();
        seal_quarantine_replacement_generation(&transaction).unwrap();
        fs::write(
            transaction
                .join(QUARANTINE_REPLACEMENT_STAGING_DIR)
                .join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"tampered\",\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        assert_eq!(
            verify_quarantine_replacement_generation(&transaction)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_PUBLICATION"
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn rejects_pointer_to_different_generation() {
        let pointer = to_canonical_json(&JsonValue::Object(BTreeMap::from([
            (
                "generationDigest".to_string(),
                JsonValue::String("fnv1a64:0000000000000000".to_string()),
            ),
            (
                "transactionId".to_string(),
                JsonValue::String("tx-other".to_string()),
            ),
            (
                "version".to_string(),
                JsonValue::String(QUARANTINE_CURRENT_GENERATION_POINTER_VERSION.to_string()),
            ),
        ])));
        assert_eq!(
            validate_quarantine_current_generation_pointer(
                &pointer,
                "tx-generation-001",
                "fnv1a64:1111111111111111"
            )
            .unwrap_err()
            .code,
            "LB_QUARANTINE_REPLACEMENT_PUBLICATION"
        );
    }
}
