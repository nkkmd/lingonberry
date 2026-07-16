use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    publish_quarantine_replacement_completion_evidence,
    read_quarantine_replacement_transaction_details,
    verify_quarantine_replacement_completion_evidence_artifact,
    QuarantineReplacementCompletionEvidence, StoreError,
    QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE,
};

pub(crate) fn finalize_quarantine_replacement_completion_evidence(
    transaction_dir: &Path,
    generation_digest: Option<&str>,
) -> Result<(), StoreError> {
    let journal = read_quarantine_replacement_transaction_details(transaction_dir)?;
    if !journal.state.is_terminal() {
        return Err(crate::store_error(
            "LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_TERMINAL",
            "completion evidence requires a terminal transaction journal",
        ));
    }

    let journal_digest = fs::read_to_string(
        transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE),
    )
    .map_err(|error| {
        crate::store_error(
            "LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_TERMINAL",
            &format!("failed to read terminal journal digest: {error}"),
        )
    })?;
    let journal_digest = journal_digest.trim();
    let completed_at_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            crate::store_error(
                "LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_TERMINAL",
                &format!("system clock is before Unix epoch: {error}"),
            )
        })?
        .as_secs();

    let evidence = QuarantineReplacementCompletionEvidence {
        transaction_id: journal.transaction_id.clone(),
        terminal_state: journal.state.as_str().to_string(),
        terminal_sequence: journal.sequence,
        completed_at_unix_seconds,
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
        completed_at_unix_seconds,
    )?;
    Ok(())
}
