use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::quarantine_ledger_index::build_quarantine_ledger_index_locked;
use crate::{
    acquire_quarantine_lock, advance_quarantine_replacement_transaction_journal,
    prepare_quarantine_replacement_publication,
    prepare_quarantine_replacement_transaction, read_quarantine_replacement_inputs,
    read_quarantine_replacement_transaction_details,
    read_quarantine_replacement_transaction_journal, resolve_quarantine_active_generation,
    resolve_quarantine_active_path, seal_quarantine_replacement_generation,
    stage_quarantine_replacement_ledgers, store_error, verify_any_quarantine_backup,
    verify_quarantine_ledger_index, verify_quarantine_replacement_generation,
    verify_quarantine_replacement_proof, verify_quarantine_replacement_staging,
    verify_quarantine_segments, QuarantineReplacementTransactionState, StoreError,
    QUARANTINE_BACKUP_FILES, QUARANTINE_BACKUP_MANIFEST,
    QUARANTINE_CURRENT_GENERATION_POINTER_FILE, QUARANTINE_GENERATIONS_DIR,
    QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE,
    QUARANTINE_REPLACEMENT_PENDING_POINTER_FILE, QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE, QUARANTINE_REPLACEMENT_PUBLICATION_DIR,
    QUARANTINE_REPLACEMENT_STAGING_DIR, QUARANTINE_SEGMENT_ARCHIVE_DIR,
    QUARANTINE_SEGMENT_MANIFEST_FILE,
};

pub const QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_FILE: &str =
    "quarantine-replacement-publication-intent.json";
pub const QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_DIGEST_FILE: &str =
    "quarantine-replacement-publication-intent.digest";
pub const QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_VERSION: &str =
    "lingonberry-quarantine-replacement-publication-intent/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementStatusReport {
    pub transaction_id: String,
    pub state: QuarantineReplacementTransactionState,
    pub classification: String,
    pub generation_digest: Option<String>,
    pub active_generation: Option<String>,
}

#[derive(Debug, Clone)]
struct PublicationIntent {
    transaction_id: String,
    generation_digest: String,
    target_pointer: String,
    previous_pointer: Option<String>,
}

pub fn apply_quarantine_replacement_transaction(
    state_dir: impl AsRef<Path>,
    backup_dir: impl AsRef<Path>,
    proof_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
    transaction_id: &str,
) -> Result<QuarantineReplacementStatusReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    if !transaction_dir.exists()
        || fs::read_dir(transaction_dir)
            .map_err(io_error)?
            .next()
            .is_none()
    {
        prepare_quarantine_replacement_transaction(
            state_dir.as_ref(),
            backup_dir.as_ref(),
            proof_dir.as_ref(),
            transaction_dir,
            transaction_id,
        )?;
    }
    resume_quarantine_replacement_transaction(state_dir, transaction_dir)
}

pub fn resume_quarantine_replacement_transaction(
    state_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementStatusReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();
    let inputs = read_quarantine_replacement_inputs(transaction_dir)?;

    for _ in 0..10 {
        let report = read_quarantine_replacement_transaction_journal(transaction_dir)?;
        match report.state {
            QuarantineReplacementTransactionState::Prepared
            | QuarantineReplacementTransactionState::Writing => {
                stage_quarantine_replacement_ledgers(
                    state_dir,
                    &inputs.proof_dir,
                    transaction_dir,
                )?;
            }
            QuarantineReplacementTransactionState::Staged => {
                verify_quarantine_replacement_staging(
                    state_dir,
                    &inputs.proof_dir,
                    transaction_dir,
                )?;
            }
            QuarantineReplacementTransactionState::Verified => {
                let publication_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR);
                let generation_manifest =
                    publication_dir.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE);
                if generation_manifest.exists() {
                    verify_quarantine_replacement_generation(transaction_dir)?;
                } else {
                    seal_quarantine_replacement_generation(transaction_dir)?;
                }
                let pending = publication_dir.join(QUARANTINE_REPLACEMENT_PENDING_POINTER_FILE);
                if !pending.exists() {
                    prepare_quarantine_replacement_publication(
                        state_dir,
                        &inputs.proof_dir,
                        transaction_dir,
                    )?;
                }
                return publish_quarantine_replacement_generation(state_dir, transaction_dir);
            }
            QuarantineReplacementTransactionState::Publishing => {
                return publish_quarantine_replacement_generation(state_dir, transaction_dir);
            }
            QuarantineReplacementTransactionState::RecoveryRequired => {
                if publication_intent_path(transaction_dir).exists() {
                    return publish_quarantine_replacement_generation(state_dir, transaction_dir);
                }
                stage_quarantine_replacement_ledgers(
                    state_dir,
                    &inputs.proof_dir,
                    transaction_dir,
                )?;
            }
            QuarantineReplacementTransactionState::Committed
            | QuarantineReplacementTransactionState::RolledBack => {
                return quarantine_replacement_status(state_dir, transaction_dir);
            }
        }
    }
    Err(recovery_error("replacement resume exceeded its bounded step count"))
}

pub fn publish_quarantine_replacement_generation(
    state_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementStatusReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-replacement-publish-v1")?;

    let result = publish_locked(state_dir, transaction_dir);
    if result.is_err() {
        if let Ok(report) = read_quarantine_replacement_transaction_journal(transaction_dir) {
            if matches!(
                report.state,
                QuarantineReplacementTransactionState::Verified
                    | QuarantineReplacementTransactionState::Publishing
            ) {
                let _ = advance_quarantine_replacement_transaction_journal(
                    transaction_dir,
                    QuarantineReplacementTransactionState::RecoveryRequired,
                );
            }
        }
    }
    result
}

fn publish_locked(
    state_dir: &Path,
    transaction_dir: &Path,
) -> Result<QuarantineReplacementStatusReport, StoreError> {
    let mut journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    if journal.state == QuarantineReplacementTransactionState::Committed {
        return quarantine_replacement_status(state_dir, transaction_dir);
    }
    if !matches!(
        journal.state,
        QuarantineReplacementTransactionState::Verified
            | QuarantineReplacementTransactionState::Publishing
            | QuarantineReplacementTransactionState::RecoveryRequired
    ) {
        return Err(publication_error(
            "transaction is not ready for generation publication",
        ));
    }

    let inputs = read_quarantine_replacement_inputs(transaction_dir)?;
    verify_bound_inputs(&journal, &inputs.backup_dir, &inputs.proof_dir)?;
    let generation = verify_quarantine_replacement_generation(transaction_dir)?;
    let intent = create_or_read_intent(state_dir, transaction_dir, &generation.generation_digest)?;
    materialize_generation(state_dir, transaction_dir, &intent)?;

    let current_pointer = read_optional_pointer(state_dir)?;
    let target_is_active = current_pointer.as_deref() == Some(intent.target_pointer.as_str());
    let previous_is_active = current_pointer == intent.previous_pointer;
    if !target_is_active && !previous_is_active {
        return Err(publication_error(
            "current-generation pointer does not match the publication intent",
        ));
    }

    if journal.state == QuarantineReplacementTransactionState::Verified {
        if runtime_fingerprint_digest(state_dir)? != journal.runtime_fingerprint_digest {
            return Err(publication_error(
                "runtime fingerprint changed before atomic publication",
            ));
        }
        advance_quarantine_replacement_transaction_journal(
            transaction_dir,
            QuarantineReplacementTransactionState::Publishing,
        )?;
        journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    } else if journal.state == QuarantineReplacementTransactionState::RecoveryRequired {
        advance_quarantine_replacement_transaction_journal(
            transaction_dir,
            QuarantineReplacementTransactionState::Publishing,
        )?;
        journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    }

    if !target_is_active {
        atomic_write_pointer(state_dir, &intent.target_pointer)?;
    }
    verify_active_target(state_dir, &intent)?;
    verify_materialized_generation(state_dir, transaction_dir, &intent)?;

    build_quarantine_ledger_index_locked(state_dir)?;
    verify_quarantine_ledger_index(state_dir)?;
    verify_quarantine_segments(state_dir)?;
    verify_active_target(state_dir, &intent)?;

    if journal.state != QuarantineReplacementTransactionState::Publishing {
        return Err(publication_error("publication journal state changed unexpectedly"));
    }
    advance_quarantine_replacement_transaction_journal(
        transaction_dir,
        QuarantineReplacementTransactionState::Committed,
    )?;
    quarantine_replacement_status(state_dir, transaction_dir)
}

pub fn rollback_quarantine_replacement_transaction(
    state_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementStatusReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-replacement-rollback-v1")?;
    let mut journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    if journal.state == QuarantineReplacementTransactionState::RolledBack {
        return quarantine_replacement_status(state_dir, transaction_dir);
    }
    if journal.state == QuarantineReplacementTransactionState::Committed {
        return Err(rollback_error("committed transactions are terminal"));
    }

    let inputs = read_quarantine_replacement_inputs(transaction_dir)?;
    verify_bound_backup(&journal, &inputs.backup_dir)?;
    let intent = if publication_intent_path(transaction_dir).exists() {
        Some(read_intent(transaction_dir)?)
    } else {
        None
    };

    if let Some(intent) = &intent {
        let current = read_optional_pointer(state_dir)?;
        if current.as_deref() == Some(intent.target_pointer.as_str()) {
            restore_previous_pointer(state_dir, intent.previous_pointer.as_deref())?;
        } else if current != intent.previous_pointer {
            return Err(rollback_error(
                "current-generation pointer is unrelated to this transaction",
            ));
        }
    }

    if journal.state != QuarantineReplacementTransactionState::RecoveryRequired {
        advance_quarantine_replacement_transaction_journal(
            transaction_dir,
            QuarantineReplacementTransactionState::RecoveryRequired,
        )?;
        journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    }
    build_quarantine_ledger_index_locked(state_dir)?;
    verify_quarantine_ledger_index(state_dir)?;
    verify_quarantine_segments(state_dir)?;
    if journal.state != QuarantineReplacementTransactionState::RecoveryRequired {
        return Err(rollback_error("rollback journal state changed unexpectedly"));
    }
    advance_quarantine_replacement_transaction_journal(
        transaction_dir,
        QuarantineReplacementTransactionState::RolledBack,
    )?;
    quarantine_replacement_status(state_dir, transaction_dir)
}

pub fn quarantine_replacement_status(
    state_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementStatusReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();
    let journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    let active = resolve_quarantine_active_generation(state_dir)?;
    let intent = if publication_intent_path(transaction_dir).exists() {
        Some(read_intent(transaction_dir)?)
    } else {
        None
    };
    let pointer = read_optional_pointer(state_dir)?;

    let classification = match journal.state {
        QuarantineReplacementTransactionState::Committed => {
            let intent = intent.as_ref().ok_or_else(|| recovery_error("committed transaction has no publication intent"))?;
            if pointer.as_deref() != Some(intent.target_pointer.as_str()) {
                return Err(recovery_error(
                    "committed transaction is not the active generation",
                ));
            }
            "committed"
        }
        QuarantineReplacementTransactionState::RolledBack => {
            if let Some(intent) = &intent {
                if pointer != intent.previous_pointer {
                    return Err(recovery_error(
                        "rolled-back transaction does not match the previous pointer",
                    ));
                }
            }
            "rolled-back"
        }
        QuarantineReplacementTransactionState::Publishing => {
            let intent = intent.as_ref().ok_or_else(|| recovery_error("publishing transaction has no publication intent"))?;
            if pointer.as_deref() == Some(intent.target_pointer.as_str()) {
                "resumable-after-switch"
            } else if pointer == intent.previous_pointer {
                "resumable-before-switch"
            } else {
                return Err(recovery_error("publishing pointer state is contradictory"));
            }
        }
        QuarantineReplacementTransactionState::RecoveryRequired => {
            if let Some(intent) = &intent {
                if pointer.as_deref() == Some(intent.target_pointer.as_str()) {
                    "resumable-or-rollback-after-switch"
                } else if pointer == intent.previous_pointer {
                    "resumable-or-rollback-before-switch"
                } else {
                    return Err(recovery_error("recovery pointer state is contradictory"));
                }
            } else {
                "resumable-or-rollback-before-publication"
            }
        }
        QuarantineReplacementTransactionState::Prepared
        | QuarantineReplacementTransactionState::Writing
        | QuarantineReplacementTransactionState::Staged
        | QuarantineReplacementTransactionState::Verified => "resumable-before-publication",
    };

    Ok(QuarantineReplacementStatusReport {
        transaction_id: journal.transaction_id,
        state: journal.state,
        classification: classification.to_string(),
        generation_digest: intent.as_ref().map(|intent| intent.generation_digest.clone()),
        active_generation: active.transaction_id,
    })
}

pub fn quarantine_replacement_status_json(report: &QuarantineReplacementStatusReport) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "activeGeneration".to_string(),
            report
                .active_generation
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "classification".to_string(),
            JsonValue::String(report.classification.clone()),
        ),
        (
            "generationDigest".to_string(),
            report
                .generation_digest
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "state".to_string(),
            JsonValue::String(report.state.as_str().to_string()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(report.transaction_id.clone()),
        ),
    ]))
}

fn create_or_read_intent(
    state_dir: &Path,
    transaction_dir: &Path,
    generation_digest: &str,
) -> Result<PublicationIntent, StoreError> {
    if publication_intent_path(transaction_dir).exists() {
        let intent = read_intent(transaction_dir)?;
        if intent.generation_digest != generation_digest {
            return Err(publication_error("publication intent generation mismatch"));
        }
        return Ok(intent);
    }

    let journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    let target_pointer_path = transaction_dir
        .join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR)
        .join(QUARANTINE_REPLACEMENT_PENDING_POINTER_FILE);
    let target_pointer = fs::read_to_string(target_pointer_path).map_err(io_error)?;
    let previous_pointer = read_optional_pointer(state_dir)?;
    if previous_pointer.is_some() {
        resolve_quarantine_active_generation(state_dir)?;
    }
    let intent = PublicationIntent {
        transaction_id: journal.transaction_id,
        generation_digest: generation_digest.to_string(),
        target_pointer,
        previous_pointer,
    };
    write_intent(transaction_dir, &intent)?;
    read_intent(transaction_dir)
}

fn materialize_generation(
    state_dir: &Path,
    transaction_dir: &Path,
    intent: &PublicationIntent,
) -> Result<(), StoreError> {
    let generations_dir = state_dir.join(QUARANTINE_GENERATIONS_DIR);
    fs::create_dir_all(&generations_dir).map_err(io_error)?;
    sync_directory(state_dir)?;
    let final_dir = generations_dir.join(&intent.transaction_id);
    if final_dir.exists() {
        return verify_materialized_generation(state_dir, transaction_dir, intent);
    }
    let temporary_dir = generations_dir.join(format!(".{}.tmp", intent.transaction_id));
    if temporary_dir.exists() {
        fs::remove_dir_all(&temporary_dir).map_err(io_error)?;
    }
    fs::create_dir(&temporary_dir).map_err(io_error)?;
    sync_directory(&generations_dir)?;

    let staging_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_STAGING_DIR);
    for ledger in QUARANTINE_BACKUP_FILES {
        let source = staging_dir.join(ledger);
        if source.exists() {
            copy_new_synced(&source, &temporary_dir.join(ledger))?;
        }
    }
    let publication_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR);
    copy_new_synced(
        &publication_dir.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE),
        &temporary_dir.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE),
    )?;
    copy_new_synced(
        &publication_dir.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE),
        &temporary_dir.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE),
    )?;
    sync_directory(&temporary_dir)?;
    fs::rename(&temporary_dir, &final_dir).map_err(io_error)?;
    sync_directory(&generations_dir)?;
    verify_materialized_generation(state_dir, transaction_dir, intent)
}

fn verify_materialized_generation(
    state_dir: &Path,
    transaction_dir: &Path,
    intent: &PublicationIntent,
) -> Result<(), StoreError> {
    let target = state_dir
        .join(QUARANTINE_GENERATIONS_DIR)
        .join(&intent.transaction_id);
    if !target.is_dir() {
        return Err(publication_error("materialized generation is missing"));
    }
    let staging = transaction_dir.join(QUARANTINE_REPLACEMENT_STAGING_DIR);
    for ledger in QUARANTINE_BACKUP_FILES {
        let source = staging.join(ledger);
        let destination = target.join(ledger);
        if source.exists() != destination.exists() {
            return Err(publication_error("materialized generation membership mismatch"));
        }
        if source.exists() && fs::read(&source).map_err(io_error)? != fs::read(&destination).map_err(io_error)? {
            return Err(publication_error("materialized generation ledger mismatch"));
        }
    }
    let publication = transaction_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR);
    for name in [
        QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE,
        QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE,
    ] {
        if fs::read(publication.join(name)).map_err(io_error)?
            != fs::read(target.join(name)).map_err(io_error)?
        {
            return Err(publication_error("materialized generation metadata mismatch"));
        }
    }
    let digest = fs::read_to_string(target.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE))
        .map_err(io_error)?;
    if digest.trim() != intent.generation_digest {
        return Err(publication_error("materialized generation digest mismatch"));
    }
    Ok(())
}

fn verify_active_target(state_dir: &Path, intent: &PublicationIntent) -> Result<(), StoreError> {
    let pointer = read_optional_pointer(state_dir)?;
    if pointer.as_deref() != Some(intent.target_pointer.as_str()) {
        return Err(publication_error("atomic generation pointer switch was not durable"));
    }
    let active = resolve_quarantine_active_generation(state_dir)?;
    if active.transaction_id.as_deref() != Some(intent.transaction_id.as_str())
        || active.generation_digest.as_deref() != Some(intent.generation_digest.as_str())
    {
        return Err(publication_error("active generation does not match publication intent"));
    }
    Ok(())
}

fn verify_bound_inputs(
    journal: &crate::QuarantineReplacementTransactionJournal,
    backup_dir: &Path,
    proof_dir: &Path,
) -> Result<(), StoreError> {
    verify_bound_backup(journal, backup_dir)?;
    verify_quarantine_replacement_proof(proof_dir)?;
    let plan_digest = read_digest(&proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE))?;
    let proof_digest = read_digest(&proof_dir.join(QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE))?;
    if plan_digest != journal.plan_digest || proof_digest != journal.proof_digest {
        return Err(publication_error("replacement proof does not match the journal"));
    }
    Ok(())
}

fn verify_bound_backup(
    journal: &crate::QuarantineReplacementTransactionJournal,
    backup_dir: &Path,
) -> Result<(), StoreError> {
    verify_any_quarantine_backup(backup_dir)?;
    let digest = file_digest(&backup_dir.join(QUARANTINE_BACKUP_MANIFEST))?;
    if digest != journal.backup_manifest_digest {
        return Err(rollback_error("backup manifest does not match the journal"));
    }
    Ok(())
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

fn write_intent(transaction_dir: &Path, intent: &PublicationIntent) -> Result<(), StoreError> {
    let value = JsonValue::Object(BTreeMap::from([
        (
            "generationDigest".to_string(),
            JsonValue::String(intent.generation_digest.clone()),
        ),
        (
            "previousPointer".to_string(),
            intent
                .previous_pointer
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "targetPointer".to_string(),
            JsonValue::String(intent.target_pointer.clone()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(intent.transaction_id.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_VERSION.to_string()),
        ),
    ]));
    let text = to_canonical_json(&value);
    let publication_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR);
    write_new_synced(&publication_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_FILE), text.as_bytes())?;
    write_new_synced(
        &publication_dir.join(QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_DIGEST_FILE),
        format!("{}\n", integrity_digest(text.as_bytes())).as_bytes(),
    )?;
    sync_directory(&publication_dir)
}

fn read_intent(transaction_dir: &Path) -> Result<PublicationIntent, StoreError> {
    let path = publication_intent_path(transaction_dir);
    let text = fs::read_to_string(&path).map_err(io_error)?;
    let expected = fs::read_to_string(
        transaction_dir
            .join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR)
            .join(QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_DIGEST_FILE),
    )
    .map_err(io_error)?;
    if expected.trim() != integrity_digest(text.as_bytes()) {
        return Err(recovery_error("publication intent digest mismatch"));
    }
    let value = parse_json(&text)
        .map_err(|error| recovery_error(&format!("invalid publication intent JSON: {error}")))?;
    require_string(
        &value,
        "version",
        QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_VERSION,
    )?;
    Ok(PublicationIntent {
        transaction_id: object_string(&value, "transactionId")?,
        generation_digest: object_string(&value, "generationDigest")?,
        target_pointer: object_string(&value, "targetPointer")?,
        previous_pointer: object_optional_string(&value, "previousPointer")?,
    })
}

fn publication_intent_path(transaction_dir: &Path) -> PathBuf {
    transaction_dir
        .join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR)
        .join(QUARANTINE_REPLACEMENT_PUBLICATION_INTENT_FILE)
}

fn read_optional_pointer(state_dir: &Path) -> Result<Option<String>, StoreError> {
    let path = state_dir.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE);
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path).map(Some).map_err(io_error)
}

fn atomic_write_pointer(state_dir: &Path, text: &str) -> Result<(), StoreError> {
    let temporary = state_dir.join(".quarantine-current-generation.json.tmp");
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(io_error)?;
    }
    write_new_synced(&temporary, text.as_bytes())?;
    fs::rename(
        &temporary,
        state_dir.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE),
    )
    .map_err(io_error)?;
    sync_directory(state_dir)
}

fn restore_previous_pointer(state_dir: &Path, previous: Option<&str>) -> Result<(), StoreError> {
    match previous {
        Some(text) => atomic_write_pointer(state_dir, text),
        None => {
            let pointer = state_dir.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE);
            if pointer.exists() {
                fs::remove_file(pointer).map_err(io_error)?;
                sync_directory(state_dir)?;
            }
            Ok(())
        }
    }
}

fn copy_new_synced(source: &Path, target: &Path) -> Result<(), StoreError> {
    let bytes = fs::read(source).map_err(io_error)?;
    write_new_synced(target, &bytes)
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

fn file_digest(path: &Path) -> Result<String, StoreError> {
    fs::read(path)
        .map(|bytes| integrity_digest(&bytes))
        .map_err(io_error)
}

fn read_digest(path: &Path) -> Result<String, StoreError> {
    let value = fs::read_to_string(path).map_err(io_error)?;
    let value = value.trim().to_string();
    if value.len() != "fnv1a64:".len() + 16 || !value.starts_with("fnv1a64:") {
        return Err(publication_error("invalid bound digest"));
    }
    Ok(value)
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(recovery_error("expected JSON object")),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(_) => Err(recovery_error(&format!("invalid string field: {name}"))),
        None => Err(recovery_error(&format!("missing field: {name}"))),
    }
}

fn object_optional_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        Some(_) => Err(recovery_error(&format!("invalid optional string field: {name}"))),
        None => Err(recovery_error(&format!("missing field: {name}"))),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(recovery_error(&format!("unexpected {name}")));
    }
    Ok(())
}

fn io_error(error: std::io::Error) -> StoreError {
    publication_error(&error.to_string())
}

fn publication_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_PUBLICATION", message.to_string())
}

fn recovery_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_RECOVERY", message.to_string())
}

fn rollback_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_ROLLBACK", message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_quarantine_replacement_preview, export_complete_quarantine_backup};
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
        let state = temp_dir("publication-state");
        let backup = temp_dir("publication-backup");
        let proof = temp_dir("publication-proof");
        let transaction = temp_dir("publication-transaction");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), b"{\"id\":\"q0\"}\n").unwrap();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"c1\", \"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        create_quarantine_replacement_preview(&state, &backup, &proof).unwrap();
        (state, backup, proof, transaction)
    }

    #[test]
    fn applies_generation_with_one_atomic_pointer() {
        let (state, backup, proof, transaction) = fixture();
        let report = apply_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-publication-001",
        )
        .unwrap();
        assert_eq!(report.state, QuarantineReplacementTransactionState::Committed);
        assert_eq!(report.active_generation.as_deref(), Some("tx-publication-001"));
        assert!(state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE).is_file());
        assert!(
            state
                .join(QUARANTINE_GENERATIONS_DIR)
                .join("tx-publication-001")
                .is_dir()
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn rollback_before_commit_restores_legacy_root() {
        let (state, backup, proof, transaction) = fixture();
        prepare_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-rollback-001",
        )
        .unwrap();
        stage_quarantine_replacement_ledgers(&state, &proof, &transaction).unwrap();
        verify_quarantine_replacement_staging(&state, &proof, &transaction).unwrap();
        seal_quarantine_replacement_generation(&transaction).unwrap();
        prepare_quarantine_replacement_publication(&state, &proof, &transaction).unwrap();
        let intent = create_or_read_intent(
            &state,
            &transaction,
            &verify_quarantine_replacement_generation(&transaction)
                .unwrap()
                .generation_digest,
        )
        .unwrap();
        materialize_generation(&state, &transaction, &intent).unwrap();
        advance_quarantine_replacement_transaction_journal(
            &transaction,
            QuarantineReplacementTransactionState::RecoveryRequired,
        )
        .unwrap();
        let report = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap();
        assert_eq!(report.state, QuarantineReplacementTransactionState::RolledBack);
        assert!(!state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE).exists());
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }
}
