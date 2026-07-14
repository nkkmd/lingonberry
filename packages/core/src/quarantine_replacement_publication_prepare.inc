use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    acquire_quarantine_lock, advance_quarantine_replacement_transaction_journal,
    read_quarantine_replacement_transaction_journal, resolve_quarantine_active_path, store_error,
    validate_quarantine_current_generation_pointer, verify_quarantine_replacement_generation,
    verify_quarantine_replacement_proof, QuarantineReplacementTransactionState, StoreError,
    QUARANTINE_BACKUP_FILES, QUARANTINE_CURRENT_GENERATION_POINTER_VERSION,
    QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE, QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_PUBLICATION_DIR, QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE, QUARANTINE_REPLACEMENT_TRANSACTION_VERSION,
    QUARANTINE_SEGMENT_ARCHIVE_DIR, QUARANTINE_SEGMENT_MANIFEST_FILE,
};

pub const QUARANTINE_REPLACEMENT_PENDING_POINTER_FILE: &str =
    "quarantine-current-generation.next.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementPublicationPreparationReport {
    pub pointer_path: PathBuf,
    pub transaction_id: String,
    pub generation_digest: String,
    pub runtime_fingerprint_digest: String,
}

struct BoundJournalDigests {
    plan_digest: String,
    proof_digest: String,
    runtime_fingerprint_digest: String,
}

pub fn prepare_quarantine_replacement_publication(
    state_dir: impl AsRef<Path>,
    verified_proof_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementPublicationPreparationReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let proof_dir = verified_proof_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-replacement-publication-prepare-v1")?;

    let result = prepare_publication_locked(state_dir, proof_dir, transaction_dir);
    match result {
        Ok(report) => Ok(report),
        Err(error) => {
            let pointer_path = transaction_dir
                .join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR)
                .join(QUARANTINE_REPLACEMENT_PENDING_POINTER_FILE);
            let _ = fs::remove_file(pointer_path);
            let journal = read_quarantine_replacement_transaction_journal(transaction_dir);
            if matches!(
                journal.as_ref().map(|journal| journal.state),
                Ok(QuarantineReplacementTransactionState::Verified)
            ) {
                let _ = advance_quarantine_replacement_transaction_journal(
                    transaction_dir,
                    QuarantineReplacementTransactionState::RecoveryRequired,
                );
            }
            Err(error)
        }
    }
}

fn prepare_publication_locked(
    state_dir: &Path,
    proof_dir: &Path,
    transaction_dir: &Path,
) -> Result<QuarantineReplacementPublicationPreparationReport, StoreError> {
    let journal = read_quarantine_replacement_transaction_journal(transaction_dir)?;
    if journal.state != QuarantineReplacementTransactionState::Verified {
        return Err(publication_error(
            "transaction must be verified before publication preparation",
        ));
    }
    let bound = read_bound_journal_digests(
        transaction_dir,
        &journal.transaction_id,
        journal.state,
        journal.sequence,
    )?;

    verify_quarantine_replacement_proof(proof_dir)?;
    let plan_digest = read_bound_digest(
        &proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE),
        "replacement plan digest",
    )?;
    let proof_digest = read_bound_digest(
        &proof_dir.join(QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE),
        "replacement proof digest",
    )?;
    if plan_digest != bound.plan_digest || proof_digest != bound.proof_digest {
        return Err(publication_error(
            "replacement proof inputs do not match the transaction journal",
        ));
    }

    let generation = verify_quarantine_replacement_generation(transaction_dir)?;
    let initial_runtime_digest = runtime_fingerprint_digest(state_dir)?;
    if initial_runtime_digest != bound.runtime_fingerprint_digest {
        return Err(publication_error(
            "runtime fingerprint changed before publication",
        ));
    }

    let pointer = JsonValue::Object(BTreeMap::from([
        (
            "generationDigest".to_string(),
            JsonValue::String(generation.generation_digest.clone()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(journal.transaction_id.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_CURRENT_GENERATION_POINTER_VERSION.to_string()),
        ),
    ]));
    let pointer_text = to_canonical_json(&pointer);
    validate_quarantine_current_generation_pointer(
        &pointer_text,
        &journal.transaction_id,
        &generation.generation_digest,
    )?;

    let pointer_path = generation
        .publication_dir
        .join(QUARANTINE_REPLACEMENT_PENDING_POINTER_FILE);
    if pointer_path.exists() {
        let existing = fs::read_to_string(&pointer_path).map_err(io_error)?;
        if existing != pointer_text {
            return Err(publication_error(
                "pending current-generation pointer already exists with different contents",
            ));
        }
    } else {
        write_new_synced(&pointer_path, pointer_text.as_bytes())?;
        sync_directory(&generation.publication_dir)?;
    }

    let persisted = fs::read_to_string(&pointer_path).map_err(io_error)?;
    validate_quarantine_current_generation_pointer(
        &persisted,
        &journal.transaction_id,
        &generation.generation_digest,
    )?;
    verify_quarantine_replacement_generation(transaction_dir)?;
    let final_runtime_fingerprint_digest = runtime_fingerprint_digest(state_dir)?;
    if final_runtime_fingerprint_digest != bound.runtime_fingerprint_digest {
        return Err(publication_error(
            "runtime fingerprint changed while preparing publication",
        ));
    }

    Ok(QuarantineReplacementPublicationPreparationReport {
        pointer_path,
        transaction_id: journal.transaction_id,
        generation_digest: generation.generation_digest,
        runtime_fingerprint_digest: final_runtime_fingerprint_digest,
    })
}

fn read_bound_journal_digests(
    transaction_dir: &Path,
    expected_transaction_id: &str,
    expected_state: QuarantineReplacementTransactionState,
    expected_sequence: u64,
) -> Result<BoundJournalDigests, StoreError> {
    let journal_path = transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE);
    let digest_path = transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE);
    let text = fs::read_to_string(journal_path).map_err(io_error)?;
    let expected_digest = fs::read_to_string(digest_path).map_err(io_error)?;
    let expected_digest = expected_digest.trim();
    validate_digest(expected_digest, "transaction journal digest")?;
    if expected_digest != integrity_digest(text.as_bytes()) {
        return Err(publication_error("transaction journal digest mismatch"));
    }

    let journal = parse_json(&text)
        .map_err(|error| publication_error(&format!("invalid transaction journal JSON: {error}")))?;
    if object_string(&journal, "version")? != QUARANTINE_REPLACEMENT_TRANSACTION_VERSION
        || object_string(&journal, "transactionId")? != expected_transaction_id
        || object_string(&journal, "state")? != expected_state.as_str()
        || object_number(&journal, "sequence")? != expected_sequence
    {
        return Err(publication_error(
            "transaction journal changed during publication preparation",
        ));
    }

    let plan_digest = object_string(&journal, "planDigest")?;
    let proof_digest = object_string(&journal, "proofDigest")?;
    let runtime_fingerprint_digest = object_string(&journal, "runtimeFingerprintDigest")?;
    validate_digest(&plan_digest, "journal plan digest")?;
    validate_digest(&proof_digest, "journal proof digest")?;
    validate_digest(
        &runtime_fingerprint_digest,
        "journal runtime fingerprint digest",
    )?;
    Ok(BoundJournalDigests {
        plan_digest,
        proof_digest,
        runtime_fingerprint_digest,
    })
}

fn runtime_fingerprint_digest(state_dir: &Path) -> Result<String, StoreError> {
    let mut paths = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    paths.push(QUARANTINE_SEGMENT_MANIFEST_FILE.to_string());

    let archive_dir = state_dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR);
    if archive_dir.exists() {
        let mut archive = fs::read_dir(&archive_dir)
            .map_err(io_error)?
            .map(|entry| {
                entry.map_err(io_error).map(|entry| {
                    format!(
                        "{QUARANTINE_SEGMENT_ARCHIVE_DIR}/{}",
                        entry.file_name().to_string_lossy()
                    )
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        archive.sort();
        paths.extend(archive);
    }

    let values = paths
        .into_iter()
        .map(|relative| {
            let path = if QUARANTINE_BACKUP_FILES.contains(&relative.as_str()) {
                resolve_quarantine_active_path(state_dir, &relative)?
            } else {
                state_dir.join(&relative)
            };
            Ok(JsonValue::Object(BTreeMap::from([
                (
                    "digest".to_string(),
                    if path.exists() {
                        JsonValue::String(file_digest(&path)?)
                    } else {
                        JsonValue::Null
                    },
                ),
                ("path".to_string(), JsonValue::String(relative)),
            ])))
        })
        .collect::<Result<Vec<_>, StoreError>>()?;
    Ok(integrity_digest(
        to_canonical_json(&JsonValue::Array(values)).as_bytes(),
    ))
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(publication_error("expected JSON object")),
    }
}

fn object_field<'a>(value: &'a JsonValue, name: &str) -> Result<&'a JsonValue, StoreError> {
    object_map(value)?
        .get(name)
        .ok_or_else(|| publication_error(&format!("missing field: {name}")))
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(publication_error(&format!("invalid string field: {name}"))),
    }
}

fn object_number(value: &JsonValue, name: &str) -> Result<u64, StoreError> {
    match object_field(value, name)? {
        JsonValue::Number(value) => value
            .parse()
            .map_err(|_| publication_error(&format!("invalid number field: {name}"))),
        _ => Err(publication_error(&format!("invalid number field: {name}"))),
    }
}

fn read_bound_digest(path: &Path, name: &str) -> Result<String, StoreError> {
    let digest = fs::read_to_string(path).map_err(io_error)?;
    let digest = digest.trim().to_string();
    validate_digest(&digest, name)?;
    Ok(digest)
}

fn validate_digest(value: &str, name: &str) -> Result<(), StoreError> {
    if value.len() != "fnv1a64:".len() + 16
        || !value.starts_with("fnv1a64:")
        || !value["fnv1a64:".len()..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(publication_error(&format!("invalid {name}")));
    }
    Ok(())
}

fn file_digest(path: &Path) -> Result<String, StoreError> {
    fs::read(path)
        .map(|bytes| integrity_digest(&bytes))
        .map_err(io_error)
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
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

fn io_error(error: std::io::Error) -> StoreError {
    publication_error(&error.to_string())
}

fn publication_error(message: &str) -> StoreError {
    store_error(
        "LB_QUARANTINE_REPLACEMENT_PUBLICATION",
        message.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_quarantine_replacement_preview, export_complete_quarantine_backup,
        prepare_quarantine_replacement_transaction, seal_quarantine_replacement_generation,
        stage_quarantine_replacement_ledgers, verify_quarantine_replacement_staging,
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
        let state = temp_dir("replacement-publish-prepare-state");
        let backup = temp_dir("replacement-publish-prepare-backup");
        let proof = temp_dir("replacement-publish-prepare-proof");
        let transaction = temp_dir("replacement-publish-prepare-transaction");
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
            "tx-publish-prepare-001",
        )
        .unwrap();
        stage_quarantine_replacement_ledgers(&state, &proof, &transaction).unwrap();
        verify_quarantine_replacement_staging(&state, &proof, &transaction).unwrap();
        seal_quarantine_replacement_generation(&transaction).unwrap();
        (state, backup, proof, transaction)
    }

    #[test]
    fn prepares_durable_pointer_candidate_without_touching_active_state() {
        let (state, backup, proof, transaction) = fixture();
        let active_before = fs::read(state.join("quarantine-resolutions.jsonl")).unwrap();
        let report =
            prepare_quarantine_replacement_publication(&state, &proof, &transaction).unwrap();
        assert!(report.pointer_path.is_file());
        assert_eq!(
            fs::read(state.join("quarantine-resolutions.jsonl")).unwrap(),
            active_before
        );
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            QuarantineReplacementTransactionState::Verified
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn rejects_runtime_change_and_marks_recovery_required() {
        let (state, backup, proof, transaction) = fixture();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"changed\",\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        assert_eq!(
            prepare_quarantine_replacement_publication(&state, &proof, &transaction)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_PUBLICATION"
        );
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            QuarantineReplacementTransactionState::RecoveryRequired
        );
        assert!(!transaction
            .join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR)
            .join(QUARANTINE_REPLACEMENT_PENDING_POINTER_FILE)
            .exists());
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn is_idempotent_for_the_same_verified_generation() {
        let (state, backup, proof, transaction) = fixture();
        let first = prepare_quarantine_replacement_publication(&state, &proof, &transaction).unwrap();
        let second = prepare_quarantine_replacement_publication(&state, &proof, &transaction).unwrap();
        assert_eq!(first, second);
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }
}
