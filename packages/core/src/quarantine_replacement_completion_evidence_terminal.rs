use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    publish_quarantine_replacement_completion_evidence,
    read_quarantine_replacement_transaction_details,
    verify_quarantine_replacement_completion_evidence_artifact,
    QuarantineReplacementCompletionEvidence, StoreError,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE,
    QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE,
};

pub(crate) fn finalize_quarantine_replacement_completion_evidence(
    transaction_dir: &Path,
    generation_digest: Option<&str>,
) -> Result<(), StoreError> {
    let journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    if !journal.state.is_terminal() {
        return Err(terminal_error(
            "completion evidence requires a terminal transaction journal",
        ));
    }

    let journal_digest = fs::read_to_string(
        transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE),
    )
    .map_err(|error| terminal_error(format!("failed to read terminal journal digest: {error}")))?;
    let journal_digest = journal_digest.trim();
    let now_unix_seconds = current_unix_seconds()?;

    let evidence_exists = transaction_dir
        .join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE)
        .exists();
    let digest_exists = transaction_dir
        .join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE)
        .exists();
    match (evidence_exists, digest_exists) {
        (true, true) => {
            verify_quarantine_replacement_completion_evidence_artifact(
                transaction_dir,
                &journal.transaction_id,
                journal.state.as_str(),
                journal.sequence,
                journal_digest,
                generation_digest,
                now_unix_seconds,
            )?;
            return Ok(());
        }
        (true, false) | (false, true) => {
            return Err(terminal_error(
                "completion evidence artifact pair is incomplete",
            ));
        }
        (false, false) => {}
    }

    let evidence = QuarantineReplacementCompletionEvidence {
        transaction_id: journal.transaction_id.clone(),
        terminal_state: journal.state.as_str().to_string(),
        terminal_sequence: journal.sequence,
        completed_at_unix_seconds: now_unix_seconds,
        journal_digest: journal_digest.to_string(),
        generation_digest: generation_digest.map(str::to_string),
    };
    publish_quarantine_replacement_completion_evidence(transaction_dir, &evidence)?;
    verify_quarantine_replacement_completion_evidence_artifact(
        transaction_dir,
        &journal.transaction_id,
        journal.state.as_str(),
        journal.sequence,
        journal_digest,
        generation_digest,
        now_unix_seconds,
    )?;
    Ok(())
}

fn current_unix_seconds() -> Result<u64, StoreError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| terminal_error(format!("system clock is before Unix epoch: {error}")))
        .map(|duration| duration.as_secs())
}

fn terminal_error(message: impl Into<String>) -> StoreError {
    crate::store_error(
        "LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_TERMINAL",
        message,
    )
}
