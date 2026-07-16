use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    quarantine_replacement_cleanup_transaction_journal_json, store_error,
    validate_quarantine_replacement_cleanup_transaction_transition,
    QuarantineReplacementCleanupTransactionJournal,
    QuarantineReplacementCleanupTransactionState, StoreError,
    QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_VERSION,
};

pub const QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL_FILE: &str =
    "quarantine-replacement-cleanup-transaction.json";
pub const QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL_DIGEST_FILE: &str =
    "quarantine-replacement-cleanup-transaction.digest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupTransactionReport {
    pub journal_path: PathBuf,
    pub state: QuarantineReplacementCleanupTransactionState,
    pub sequence: u64,
    pub transaction_id: String,
    pub deleted_subjects: Vec<String>,
}

pub fn create_quarantine_replacement_cleanup_transaction_journal(
    transaction_dir: impl AsRef<Path>,
    journal: &QuarantineReplacementCleanupTransactionJournal,
) -> Result<QuarantineReplacementCleanupTransactionReport, StoreError> {
    validate_journal(journal)?;
    if journal.state != QuarantineReplacementCleanupTransactionState::Prepared
        || journal.sequence != 0
        || !journal.deleted_subjects.is_empty()
        || journal.tomb_inventory_digest.is_some()
    {
        return Err(journal_error(
            "new cleanup transaction must begin prepared with no tomb or deletion progress",
        ));
    }
    let transaction_dir = transaction_dir.as_ref();
    fs::create_dir_all(transaction_dir).map_err(io_error)?;
    if fs::read_dir(transaction_dir).map_err(io_error)?.next().is_some() {
        return Err(journal_error("cleanup transaction directory is not empty"));
    }
    publish_journal(transaction_dir, journal)?;
    read_quarantine_replacement_cleanup_transaction_journal(transaction_dir)
}

pub fn advance_quarantine_replacement_cleanup_transaction_journal(
    transaction_dir: impl AsRef<Path>,
    next_state: QuarantineReplacementCleanupTransactionState,
    tomb_inventory_digest: Option<String>,
) -> Result<QuarantineReplacementCleanupTransactionReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let mut journal = read_journal(transaction_dir)?;
    validate_quarantine_replacement_cleanup_transaction_transition(journal.state, next_state)?;
    journal.state = next_state;
    journal.sequence = journal
        .sequence
        .checked_add(1)
        .ok_or_else(|| journal_error("cleanup transaction sequence overflow"))?;
    if tomb_inventory_digest.is_some() {
        journal.tomb_inventory_digest = tomb_inventory_digest;
    }
    validate_journal(&journal)?;
    publish_journal(transaction_dir, &journal)?;
    read_quarantine_replacement_cleanup_transaction_journal(transaction_dir)
}

pub fn record_quarantine_replacement_cleanup_subject_deleted(
    transaction_dir: impl AsRef<Path>,
    generation_id: &str,
) -> Result<QuarantineReplacementCleanupTransactionReport, StoreError> {
    validate_generation_id(generation_id)?;
    let transaction_dir = transaction_dir.as_ref();
    let mut journal = read_journal(transaction_dir)?;
    if journal.state != QuarantineReplacementCleanupTransactionState::Deleting {
        return Err(journal_error(
            "deletion progress may only be recorded in deleting state",
        ));
    }
    if journal.deleted_subjects.iter().any(|value| value == generation_id) {
        return read_quarantine_replacement_cleanup_transaction_journal(transaction_dir);
    }
    if journal
        .deleted_subjects
        .last()
        .is_some_and(|previous| previous.as_str() >= generation_id)
    {
        return Err(journal_error(
            "deleted subjects must be recorded in strict deterministic order",
        ));
    }
    journal.deleted_subjects.push(generation_id.to_string());
    journal.sequence = journal
        .sequence
        .checked_add(1)
        .ok_or_else(|| journal_error("cleanup transaction sequence overflow"))?;
    validate_journal(&journal)?;
    publish_journal(transaction_dir, &journal)?;
    read_quarantine_replacement_cleanup_transaction_journal(transaction_dir)
}

pub fn read_quarantine_replacement_cleanup_transaction_journal(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementCleanupTransactionReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let journal = read_journal(transaction_dir)?;
    Ok(QuarantineReplacementCleanupTransactionReport {
        journal_path: transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL_FILE),
        state: journal.state,
        sequence: journal.sequence,
        transaction_id: journal.transaction_id,
        deleted_subjects: journal.deleted_subjects,
    })
}

pub fn read_quarantine_replacement_cleanup_transaction_details(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementCleanupTransactionJournal, StoreError> {
    read_journal(transaction_dir.as_ref())
}

fn publish_journal(
    transaction_dir: &Path,
    journal: &QuarantineReplacementCleanupTransactionJournal,
) -> Result<(), StoreError> {
    let text = to_canonical_json(&quarantine_replacement_cleanup_transaction_journal_json(journal));
    let digest = integrity_digest(text.as_bytes());
    let journal_path = transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL_FILE);
    let digest_path =
        transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL_DIGEST_FILE);
    let journal_tmp = transaction_dir.join(".quarantine-replacement-cleanup-transaction.json.tmp");
    let digest_tmp = transaction_dir.join(".quarantine-replacement-cleanup-transaction.digest.tmp");
    if journal_tmp.exists() || digest_tmp.exists() {
        return Err(journal_error(
            "stale cleanup transaction temporary artifact requires manual review",
        ));
    }
    write_new_synced(&journal_tmp, text.as_bytes())?;
    write_new_synced(&digest_tmp, format!("{digest}\n").as_bytes())?;
    fs::rename(&journal_tmp, &journal_path).map_err(io_error)?;
    fs::rename(&digest_tmp, &digest_path).map_err(io_error)?;
    sync_directory(transaction_dir)
}

fn read_journal(
    transaction_dir: &Path,
) -> Result<QuarantineReplacementCleanupTransactionJournal, StoreError> {
    let journal_path = transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL_FILE);
    let digest_path =
        transaction_dir.join(QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL_DIGEST_FILE);
    if !journal_path.is_file() || !digest_path.is_file() {
        return Err(journal_error("cleanup transaction journal pair is incomplete"));
    }
    let text = fs::read_to_string(journal_path).map_err(io_error)?;
    let expected_digest = fs::read_to_string(digest_path).map_err(io_error)?;
    if integrity_digest(text.as_bytes()) != expected_digest.trim() {
        return Err(journal_error("cleanup transaction journal digest mismatch"));
    }
    let value = parse_json(&text)
        .map_err(|error| journal_error(format!("invalid cleanup transaction journal JSON: {error}")))?;
    let journal = parse_journal(&value)?;
    validate_journal(&journal)?;
    Ok(journal)
}

fn parse_journal(value: &JsonValue) -> Result<QuarantineReplacementCleanupTransactionJournal, StoreError> {
    require_string(value, "version", QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_VERSION)?;
    let state = match object_string(value, "state")?.as_str() {
        "prepared" => QuarantineReplacementCleanupTransactionState::Prepared,
        "revalidated" => QuarantineReplacementCleanupTransactionState::Revalidated,
        "renaming-to-tomb" => QuarantineReplacementCleanupTransactionState::RenamingToTomb,
        "tomb-sealed" => QuarantineReplacementCleanupTransactionState::TombSealed,
        "deleting" => QuarantineReplacementCleanupTransactionState::Deleting,
        "committed" => QuarantineReplacementCleanupTransactionState::Committed,
        "recovery-required" => QuarantineReplacementCleanupTransactionState::RecoveryRequired,
        "rolled-back" => QuarantineReplacementCleanupTransactionState::RolledBack,
        "partially-deleted" => QuarantineReplacementCleanupTransactionState::PartiallyDeleted,
        _ => return Err(journal_error("unsupported cleanup transaction state")),
    };
    Ok(QuarantineReplacementCleanupTransactionJournal {
        transaction_id: object_string(value, "transactionId")?,
        state,
        sequence: object_u64(value, "sequence")?,
        cleanup_proof_digest: object_string(value, "cleanupProofDigest")?,
        runtime_fingerprint: object_string(value, "runtimeFingerprint")?,
        tomb_inventory_digest: object_optional_string(value, "tombInventoryDigest")?,
        deleted_subjects: object_string_array(value, "deletedSubjects")?,
    })
}

fn validate_journal(journal: &QuarantineReplacementCleanupTransactionJournal) -> Result<(), StoreError> {
    validate_generation_id(&journal.transaction_id)?;
    validate_digest(&journal.cleanup_proof_digest, "cleanup proof digest")?;
    if journal.runtime_fingerprint.is_empty() || journal.runtime_fingerprint.contains(['\n', '\r']) {
        return Err(journal_error("invalid runtime fingerprint"));
    }
    if let Some(digest) = &journal.tomb_inventory_digest {
        validate_digest(digest, "tomb inventory digest")?;
    }
    let unique = journal.deleted_subjects.iter().collect::<BTreeSet<_>>();
    if unique.len() != journal.deleted_subjects.len()
        || !journal.deleted_subjects.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(journal_error(
            "deleted subjects must be unique and strictly sorted",
        ));
    }
    for generation_id in &journal.deleted_subjects {
        validate_generation_id(generation_id)?;
    }
    if !journal.deleted_subjects.is_empty() && !journal.state.deletion_has_started() {
        return Err(journal_error(
            "deletion progress exists before irreversible deletion state",
        ));
    }
    if matches!(
        journal.state,
        QuarantineReplacementCleanupTransactionState::TombSealed
            | QuarantineReplacementCleanupTransactionState::Deleting
            | QuarantineReplacementCleanupTransactionState::Committed
            | QuarantineReplacementCleanupTransactionState::PartiallyDeleted
    ) && journal.tomb_inventory_digest.is_none()
    {
        return Err(journal_error(
            "sealed or deleting cleanup transaction requires tomb inventory digest",
        ));
    }
    Ok(())
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(journal_error("cleanup transaction journal must be an object")),
    }
}

fn object_string(value: &JsonValue, key: &str) -> Result<String, StoreError> {
    match object_map(value)?.get(key) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(journal_error(format!("missing or invalid {key}"))),
    }
}

fn object_optional_string(value: &JsonValue, key: &str) -> Result<Option<String>, StoreError> {
    match object_map(value)?.get(key) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(journal_error(format!("missing or invalid {key}"))),
    }
}

fn object_u64(value: &JsonValue, key: &str) -> Result<u64, StoreError> {
    match object_map(value)?.get(key) {
        Some(JsonValue::Number(value)) => value
            .parse()
            .map_err(|_| journal_error(format!("invalid {key}"))),
        _ => Err(journal_error(format!("missing or invalid {key}"))),
    }
}

fn object_string_array(value: &JsonValue, key: &str) -> Result<Vec<String>, StoreError> {
    match object_map(value)?.get(key) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                JsonValue::String(value) => Ok(value.clone()),
                _ => Err(journal_error(format!("invalid {key}"))),
            })
            .collect(),
        _ => Err(journal_error(format!("missing or invalid {key}"))),
    }
}

fn require_string(value: &JsonValue, key: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, key)? != expected {
        return Err(journal_error(format!("unsupported {key}")));
    }
    Ok(())
}

fn validate_generation_id(value: &str) -> Result<(), StoreError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains(['/', '\\', '*', '?', '[', ']'])
        || !value.is_ascii()
    {
        return Err(journal_error("invalid cleanup transaction or generation ID"));
    }
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<(), StoreError> {
    let Some(hex) = value.strip_prefix("fnv1a64:") else {
        return Err(journal_error(format!("invalid {label}")));
    };
    if hex.len() != 16 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(journal_error(format!("invalid {label}")));
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

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn io_error(error: std::io::Error) -> StoreError {
    journal_error(error.to_string())
}

fn journal_error(message: impl Into<String>) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_JOURNAL", message)
}
