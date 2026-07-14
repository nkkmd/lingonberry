use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{store_error, StoreError};

pub const QUARANTINE_REPLACEMENT_TRANSACTION_VERSION: &str =
    "lingonberry-quarantine-replacement-transaction/v1";
pub const QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE: &str =
    "quarantine-replacement-transaction.json";
pub const QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE: &str =
    "quarantine-replacement-transaction.digest";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarantineReplacementTransactionState {
    Prepared,
    Writing,
    Staged,
    Verified,
    Publishing,
    Committed,
    RecoveryRequired,
    RolledBack,
}

impl QuarantineReplacementTransactionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Writing => "writing",
            Self::Staged => "staged",
            Self::Verified => "verified",
            Self::Publishing => "publishing",
            Self::Committed => "committed",
            Self::RecoveryRequired => "recovery-required",
            Self::RolledBack => "rolled-back",
        }
    }

    fn parse(value: &str) -> Result<Self, StoreError> {
        match value {
            "prepared" => Ok(Self::Prepared),
            "writing" => Ok(Self::Writing),
            "staged" => Ok(Self::Staged),
            "verified" => Ok(Self::Verified),
            "publishing" => Ok(Self::Publishing),
            "committed" => Ok(Self::Committed),
            "recovery-required" => Ok(Self::RecoveryRequired),
            "rolled-back" => Ok(Self::RolledBack),
            _ => Err(transaction_error(
                "LB_QUARANTINE_REPLACEMENT_JOURNAL",
                "unsupported transaction state",
            )),
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Committed | Self::RolledBack)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementTransactionJournal {
    pub transaction_id: String,
    pub state: QuarantineReplacementTransactionState,
    pub sequence: u64,
    pub backup_manifest_digest: String,
    pub segment_manifest_digest: Option<String>,
    pub plan_digest: String,
    pub proof_digest: String,
    pub runtime_fingerprint_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementTransactionReport {
    pub journal_path: PathBuf,
    pub state: QuarantineReplacementTransactionState,
    pub sequence: u64,
    pub transaction_id: String,
}

pub fn create_quarantine_replacement_transaction_journal(
    transaction_dir: impl AsRef<Path>,
    journal: &QuarantineReplacementTransactionJournal,
) -> Result<QuarantineReplacementTransactionReport, StoreError> {
    validate_transaction_id(&journal.transaction_id)?;
    if journal.state != QuarantineReplacementTransactionState::Prepared || journal.sequence != 0 {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            "new transaction journal must begin at prepared sequence 0",
        ));
    }
    validate_bound_digest(&journal.backup_manifest_digest, "backup manifest digest")?;
    if let Some(digest) = &journal.segment_manifest_digest {
        validate_bound_digest(digest, "segment manifest digest")?;
    }
    validate_bound_digest(&journal.plan_digest, "plan digest")?;
    validate_bound_digest(&journal.proof_digest, "proof digest")?;
    validate_bound_digest(
        &journal.runtime_fingerprint_digest,
        "runtime fingerprint digest",
    )?;

    let transaction_dir = transaction_dir.as_ref();
    prepare_empty_transaction_dir(transaction_dir)?;
    publish_journal(transaction_dir, journal)?;
    read_quarantine_replacement_transaction_journal(transaction_dir)
}

pub fn advance_quarantine_replacement_transaction_journal(
    transaction_dir: impl AsRef<Path>,
    next_state: QuarantineReplacementTransactionState,
) -> Result<QuarantineReplacementTransactionReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let mut journal = read_journal(transaction_dir)?;
    validate_transition(journal.state, next_state)?;
    journal.state = next_state;
    journal.sequence = journal.sequence.checked_add(1).ok_or_else(|| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            "transaction sequence overflow",
        )
    })?;
    publish_journal(transaction_dir, &journal)?;
    read_quarantine_replacement_transaction_journal(transaction_dir)
}

pub fn read_quarantine_replacement_transaction_journal(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementTransactionReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let journal = read_journal(transaction_dir)?;
    Ok(QuarantineReplacementTransactionReport {
        journal_path: transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE),
        state: journal.state,
        sequence: journal.sequence,
        transaction_id: journal.transaction_id,
    })
}

pub fn read_quarantine_replacement_transaction_details(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementTransactionJournal, StoreError> {
    read_journal(transaction_dir.as_ref())
}

pub fn validate_quarantine_replacement_transaction_transition(
    current: QuarantineReplacementTransactionState,
    next: QuarantineReplacementTransactionState,
) -> Result<(), StoreError> {
    validate_transition(current, next)
}

fn validate_transition(
    current: QuarantineReplacementTransactionState,
    next: QuarantineReplacementTransactionState,
) -> Result<(), StoreError> {
    let allowed = matches!(
        (current, next),
        (
            QuarantineReplacementTransactionState::Prepared,
            QuarantineReplacementTransactionState::Writing
        ) | (
            QuarantineReplacementTransactionState::Writing,
            QuarantineReplacementTransactionState::Staged
        ) | (
            QuarantineReplacementTransactionState::Staged,
            QuarantineReplacementTransactionState::Verified
        ) | (
            QuarantineReplacementTransactionState::Verified,
            QuarantineReplacementTransactionState::Publishing
        ) | (
            QuarantineReplacementTransactionState::Publishing,
            QuarantineReplacementTransactionState::Committed
        ) | (
            QuarantineReplacementTransactionState::Prepared,
            QuarantineReplacementTransactionState::RecoveryRequired
        ) | (
            QuarantineReplacementTransactionState::Writing,
            QuarantineReplacementTransactionState::RecoveryRequired
        ) | (
            QuarantineReplacementTransactionState::Staged,
            QuarantineReplacementTransactionState::RecoveryRequired
        ) | (
            QuarantineReplacementTransactionState::Verified,
            QuarantineReplacementTransactionState::RecoveryRequired
        ) | (
            QuarantineReplacementTransactionState::Publishing,
            QuarantineReplacementTransactionState::RecoveryRequired
        ) | (
            QuarantineReplacementTransactionState::RecoveryRequired,
            QuarantineReplacementTransactionState::Writing
        ) | (
            QuarantineReplacementTransactionState::RecoveryRequired,
            QuarantineReplacementTransactionState::Publishing
        ) | (
            QuarantineReplacementTransactionState::RecoveryRequired,
            QuarantineReplacementTransactionState::RolledBack
        )
    );
    if !allowed {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_TRANSACTION",
            &format!(
                "invalid transaction transition: {} -> {}",
                current.as_str(),
                next.as_str()
            ),
        ));
    }
    Ok(())
}

fn prepare_empty_transaction_dir(path: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(path).map_err(io_error)?;
    if fs::read_dir(path).map_err(io_error)?.next().is_some() {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_TRANSACTION",
            "transaction directory must be empty",
        ));
    }
    sync_directory(path)?;
    Ok(())
}

fn publish_journal(
    transaction_dir: &Path,
    journal: &QuarantineReplacementTransactionJournal,
) -> Result<(), StoreError> {
    let text = to_canonical_json(&journal_json(journal));
    let digest = integrity_digest(text.as_bytes());
    let journal_tmp = transaction_dir.join(".quarantine-replacement-transaction.json.tmp");
    let digest_tmp = transaction_dir.join(".quarantine-replacement-transaction.digest.tmp");
    let journal_path = transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE);
    let digest_path = transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE);

    write_new_synced(&journal_tmp, text.as_bytes())?;
    write_new_synced(&digest_tmp, format!("{digest}\n").as_bytes())?;
    fs::rename(&journal_tmp, &journal_path).map_err(io_error)?;
    fs::rename(&digest_tmp, &digest_path).map_err(io_error)?;
    sync_directory(transaction_dir)?;
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

fn read_journal(
    transaction_dir: &Path,
) -> Result<QuarantineReplacementTransactionJournal, StoreError> {
    let journal_path = transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE);
    let digest_path = transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE);
    let text = fs::read_to_string(&journal_path).map_err(|error| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("failed to read transaction journal: {error}"),
        )
    })?;
    let expected = fs::read_to_string(&digest_path).map_err(|error| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("failed to read transaction journal digest: {error}"),
        )
    })?;
    if expected.trim() != integrity_digest(text.as_bytes()) {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            "transaction journal digest mismatch",
        ));
    }
    let value = parse_json(&text).map_err(|error| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("invalid transaction journal JSON: {error}"),
        )
    })?;
    parse_journal(&value)
}

fn journal_json(journal: &QuarantineReplacementTransactionJournal) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "backupManifestDigest".to_string(),
            JsonValue::String(journal.backup_manifest_digest.clone()),
        ),
        (
            "planDigest".to_string(),
            JsonValue::String(journal.plan_digest.clone()),
        ),
        (
            "proofDigest".to_string(),
            JsonValue::String(journal.proof_digest.clone()),
        ),
        (
            "runtimeFingerprintDigest".to_string(),
            JsonValue::String(journal.runtime_fingerprint_digest.clone()),
        ),
        (
            "segmentManifestDigest".to_string(),
            journal
                .segment_manifest_digest
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "sequence".to_string(),
            JsonValue::Number(journal.sequence.to_string()),
        ),
        (
            "state".to_string(),
            JsonValue::String(journal.state.as_str().to_string()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(journal.transaction_id.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_TRANSACTION_VERSION.to_string()),
        ),
    ]))
}

fn parse_journal(value: &JsonValue) -> Result<QuarantineReplacementTransactionJournal, StoreError> {
    require_string(value, "version", QUARANTINE_REPLACEMENT_TRANSACTION_VERSION)?;
    let transaction_id = object_string(value, "transactionId")?;
    validate_transaction_id(&transaction_id)?;
    let state = QuarantineReplacementTransactionState::parse(&object_string(value, "state")?)?;
    let sequence = object_number(value, "sequence")?;
    let backup_manifest_digest = object_string(value, "backupManifestDigest")?;
    let segment_manifest_digest = object_optional_string(value, "segmentManifestDigest")?;
    let plan_digest = object_string(value, "planDigest")?;
    let proof_digest = object_string(value, "proofDigest")?;
    let runtime_fingerprint_digest = object_string(value, "runtimeFingerprintDigest")?;
    validate_bound_digest(&backup_manifest_digest, "backup manifest digest")?;
    if let Some(digest) = &segment_manifest_digest {
        validate_bound_digest(digest, "segment manifest digest")?;
    }
    validate_bound_digest(&plan_digest, "plan digest")?;
    validate_bound_digest(&proof_digest, "proof digest")?;
    validate_bound_digest(&runtime_fingerprint_digest, "runtime fingerprint digest")?;
    Ok(QuarantineReplacementTransactionJournal {
        transaction_id,
        state,
        sequence,
        backup_manifest_digest,
        segment_manifest_digest,
        plan_digest,
        proof_digest,
        runtime_fingerprint_digest,
    })
}

fn validate_transaction_id(value: &str) -> Result<(), StoreError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            "transactionId must be a bounded ASCII identifier",
        ));
    }
    Ok(())
}

fn validate_bound_digest(value: &str, name: &str) -> Result<(), StoreError> {
    if value.len() > 160
        || !value.starts_with("fnv1a64:")
        || value.len() != "fnv1a64:".len() + 16
        || !value["fnv1a64:".len()..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("invalid {name}"),
        ));
    }
    Ok(())
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            "transaction journal must be an object",
        )),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("missing or invalid string field: {name}"),
        )),
    }
}

fn object_optional_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("missing or invalid optional string field: {name}"),
        )),
    }
}

fn object_number(value: &JsonValue, name: &str) -> Result<u64, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::Number(value)) => value.parse().map_err(|_| {
            transaction_error(
                "LB_QUARANTINE_REPLACEMENT_JOURNAL",
                &format!("invalid number field: {name}"),
            )
        }),
        _ => Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("missing or invalid number field: {name}"),
        )),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_JOURNAL",
            &format!("unsupported {name}"),
        ));
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
    transaction_error("LB_QUARANTINE_REPLACEMENT_JOURNAL", &error.to_string())
}

fn transaction_error(code: &'static str, message: &str) -> StoreError {
    store_error(code, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn journal() -> QuarantineReplacementTransactionJournal {
        QuarantineReplacementTransactionJournal {
            transaction_id: "tx-test-001".to_string(),
            state: QuarantineReplacementTransactionState::Prepared,
            sequence: 0,
            backup_manifest_digest: "fnv1a64:0000000000000001".to_string(),
            segment_manifest_digest: Some("fnv1a64:0000000000000002".to_string()),
            plan_digest: "fnv1a64:0000000000000003".to_string(),
            proof_digest: "fnv1a64:0000000000000004".to_string(),
            runtime_fingerprint_digest: "fnv1a64:0000000000000005".to_string(),
        }
    }

    #[test]
    fn creates_and_advances_a_durable_journal() {
        let dir = temp_dir("replacement-transaction");
        let report = create_quarantine_replacement_transaction_journal(&dir, &journal()).unwrap();
        assert_eq!(report.state, QuarantineReplacementTransactionState::Prepared);
        assert_eq!(report.sequence, 0);

        let report = advance_quarantine_replacement_transaction_journal(
            &dir,
            QuarantineReplacementTransactionState::Writing,
        )
        .unwrap();
        assert_eq!(report.state, QuarantineReplacementTransactionState::Writing);
        assert_eq!(report.sequence, 1);
        assert!(dir
            .join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE)
            .is_file());
        assert!(dir
            .join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE)
            .is_file());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_skipped_and_terminal_transitions() {
        assert_eq!(
            validate_transition(
                QuarantineReplacementTransactionState::Prepared,
                QuarantineReplacementTransactionState::Verified
            )
            .unwrap_err()
            .code,
            "LB_QUARANTINE_REPLACEMENT_TRANSACTION"
        );
        assert_eq!(
            validate_transition(
                QuarantineReplacementTransactionState::Committed,
                QuarantineReplacementTransactionState::RecoveryRequired
            )
            .unwrap_err()
            .code,
            "LB_QUARANTINE_REPLACEMENT_TRANSACTION"
        );
    }

    #[test]
    fn detects_journal_tampering() {
        let dir = temp_dir("replacement-transaction-tamper");
        create_quarantine_replacement_transaction_journal(&dir, &journal()).unwrap();
        fs::write(
            dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE),
            "{\"tampered\":true}",
        )
        .unwrap();
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&dir)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_JOURNAL"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn refuses_non_empty_transaction_directory() {
        let dir = temp_dir("replacement-transaction-conflict");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("unexpected"), b"data").unwrap();
        assert_eq!(
            create_quarantine_replacement_transaction_journal(&dir, &journal())
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_TRANSACTION"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
