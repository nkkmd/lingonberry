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

#[derive(Debug)]
pub struct QuarantineReplacementCleanupPreparation<'a> {
    pub state_dir: &'a Path,
    pub preview_artifact_dir: &'a Path,
    pub cleanup_transaction_dir: &'a Path,
    pub decisions: &'a QuarantineReplacementRetentionDecisionReport,
    pub state_identity: &'a str,
    pub runtime_fingerprint: &'a str,
    pub now_unix_seconds: u64,
    pub inputs: &'a [QuarantineReplacementCleanupSubjectInput],
    pub proof: &'a QuarantineReplacementCleanupProof,
}

pub fn prepare_verified_quarantine_replacement_cleanup_tomb(
    request: &QuarantineReplacementCleanupPreparation<'_>,
    authorization: QuarantineReplacementCleanupAuthorization,
) -> Result<QuarantineReplacementCleanupTombReport, StoreError> {
    if !authorization.operator_requested || authorization.irreversible_delete_confirmed {
        return Err(execution_error(
            "tomb preparation requires operator request without irreversible delete confirmation",
        ));
    }
    let _lock = acquire_quarantine_lock(request.state_dir, "replacement-cleanup-prepare")?;

    verify_quarantine_replacement_cleanup_preview_artifacts(
        request.preview_artifact_dir,
        request.proof,
    )?;
    verify_quarantine_replacement_cleanup_preview_against_state(
        request.preview_artifact_dir,
        request.state_dir,
        request.decisions,
        request.state_identity,
        request.runtime_fingerprint,
        request.now_unix_seconds,
        request.inputs,
    )?;

    let journal =
        read_quarantine_replacement_cleanup_transaction_details(request.cleanup_transaction_dir)?;
    if journal.state != QuarantineReplacementCleanupTransactionState::Prepared
        || journal.cleanup_proof_digest != request.proof.plan_digest
        || journal.runtime_fingerprint != request.runtime_fingerprint
    {
        return Err(execution_error(
            "cleanup transaction journal does not match the revalidated proof and runtime",
        ));
    }
    advance_quarantine_replacement_cleanup_transaction_journal(
        request.cleanup_transaction_dir,
        QuarantineReplacementCleanupTransactionState::Revalidated,
        None,
    )?;
    move_quarantine_replacement_cleanup_to_tomb(
        request.state_dir,
        request.cleanup_transaction_dir,
        request.proof,
    )
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
            if matches!(
                read_quarantine_replacement_cleanup_transaction_details(transaction_dir),
                Ok(journal)
                    if journal.state == QuarantineReplacementCleanupTransactionState::Deleting
            ) {
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
