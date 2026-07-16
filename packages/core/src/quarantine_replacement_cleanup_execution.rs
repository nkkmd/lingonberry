use std::path::Path;

use crate::{
    acquire_quarantine_lock, advance_quarantine_replacement_cleanup_transaction_journal,
    move_quarantine_replacement_cleanup_to_tomb,
    read_quarantine_replacement_cleanup_transaction_details,
    resume_quarantine_replacement_cleanup_deletion, rollback_quarantine_replacement_cleanup_tomb,
    store_error, verify_quarantine_replacement_cleanup_preview_against_state,
    verify_quarantine_replacement_cleanup_preview_artifacts, QuarantineReplacementCleanupProof,
    QuarantineReplacementCleanupSubjectInput, QuarantineReplacementCleanupTombReport,
    QuarantineReplacementCleanupTransactionState, QuarantineReplacementRetentionDecisionReport,
    StoreError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupAuthorization {
    pub operator_requested: bool,
    pub irreversible_delete_confirmed: bool,
}

pub fn prepare_verified_quarantine_replacement_cleanup_tomb(
    state_dir: impl AsRef<Path>,
    preview_artifact_dir: impl AsRef<Path>,
    cleanup_transaction_dir: impl AsRef<Path>,
    decisions: &QuarantineReplacementRetentionDecisionReport,
    state_identity: &str,
    runtime_fingerprint: &str,
    now_unix_seconds: u64,
    inputs: &[QuarantineReplacementCleanupSubjectInput],
    proof: &QuarantineReplacementCleanupProof,
    authorization: QuarantineReplacementCleanupAuthorization,
) -> Result<QuarantineReplacementCleanupTombReport, StoreError> {
    if !authorization.operator_requested || authorization.irreversible_delete_confirmed {
        return Err(execution_error(
            "tomb preparation requires operator request without irreversible delete confirmation",
        ));
    }
    let state_dir = state_dir.as_ref();
    let transaction_dir = cleanup_transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "replacement-cleanup-prepare")?;

    verify_quarantine_replacement_cleanup_preview_artifacts(&preview_artifact_dir, proof)?;
    verify_quarantine_replacement_cleanup_preview_against_state(
        &preview_artifact_dir,
        state_dir,
        decisions,
        state_identity,
        runtime_fingerprint,
        now_unix_seconds,
        inputs,
    )?;

    let journal = read_quarantine_replacement_cleanup_transaction_details(transaction_dir)?;
    if journal.state != QuarantineReplacementCleanupTransactionState::Prepared
        || journal.cleanup_proof_digest != proof.plan_digest
        || journal.runtime_fingerprint != runtime_fingerprint
    {
        return Err(execution_error(
            "cleanup transaction journal does not match the revalidated proof and runtime",
        ));
    }
    advance_quarantine_replacement_cleanup_transaction_journal(
        transaction_dir,
        QuarantineReplacementCleanupTransactionState::Revalidated,
        None,
    )?;
    move_quarantine_replacement_cleanup_to_tomb(state_dir, transaction_dir, proof)
}

pub fn commit_verified_quarantine_replacement_cleanup_deletion(
    state_dir: impl AsRef<Path>,
    cleanup_transaction_dir: impl AsRef<Path>,
    authorization: QuarantineReplacementCleanupAuthorization,
) -> Result<QuarantineReplacementCleanupTombReport, StoreError> {
    if !authorization.operator_requested || !authorization.irreversible_delete_confirmed {
        return Err(execution_error(
            "irreversible cleanup deletion requires both operator request and explicit confirmation",
        ));
    }
    let state_dir = state_dir.as_ref();
    let transaction_dir = cleanup_transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "replacement-cleanup-delete")?;

    match resume_quarantine_replacement_cleanup_deletion(transaction_dir) {
        Ok(report) => Ok(report),
        Err(error) => {
            if let Ok(journal) =
                read_quarantine_replacement_cleanup_transaction_details(transaction_dir)
            {
                if journal.state == QuarantineReplacementCleanupTransactionState::Deleting {
                    let _ = advance_quarantine_replacement_cleanup_transaction_journal(
                        transaction_dir,
                        QuarantineReplacementCleanupTransactionState::RecoveryRequired,
                        None,
                    );
                    let _ = advance_quarantine_replacement_cleanup_transaction_journal(
                        transaction_dir,
                        QuarantineReplacementCleanupTransactionState::PartiallyDeleted,
                        None,
                    );
                }
            }
            Err(error)
        }
    }
}

pub fn rollback_verified_quarantine_replacement_cleanup(
    state_dir: impl AsRef<Path>,
    cleanup_transaction_dir: impl AsRef<Path>,
    authorization: QuarantineReplacementCleanupAuthorization,
) -> Result<(), StoreError> {
    if !authorization.operator_requested || authorization.irreversible_delete_confirmed {
        return Err(execution_error(
            "cleanup rollback requires operator request without delete confirmation",
        ));
    }
    let state_dir = state_dir.as_ref();
    let transaction_dir = cleanup_transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "replacement-cleanup-rollback")?;
    rollback_quarantine_replacement_cleanup_tomb(state_dir, transaction_dir)
}

fn execution_error(message: impl Into<String>) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_CLEANUP_EXECUTION", message)
}
