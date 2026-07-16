use std::fs;
use std::path::Path;

use crate::{
    store_error, verify_quarantine_replacement_completion_evidence_artifact,
    verify_quarantine_replacement_generation, QuarantineReplacementGenerationInspection,
    QuarantineReplacementRetentionCandidate, StoreError,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE,
    QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE,
};

pub fn build_quarantine_replacement_retention_candidate(
    inspection: &QuarantineReplacementGenerationInspection,
    transaction_dir: impl AsRef<Path>,
    now_unix_seconds: u64,
) -> Result<QuarantineReplacementRetentionCandidate, StoreError> {
    let generation_id = inspection
        .generation
        .as_ref()
        .ok_or_else(|| candidate_error("retention candidate requires a generation ID"))?;
    let transaction_dir = transaction_dir.as_ref();

    let generation = if inspection.verification_status == "metadata-present" {
        Some(verify_quarantine_replacement_generation(transaction_dir)?)
    } else {
        None
    };
    let verification_status = if generation.is_some() {
        "verified"
    } else {
        inspection.verification_status.as_str()
    };

    let evidence_path = transaction_dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE);
    let evidence_digest_path =
        transaction_dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE);
    let durable_age_seconds = match (evidence_path.exists(), evidence_digest_path.exists()) {
        (false, false) => None,
        (true, false) | (false, true) => {
            return Err(candidate_error(
                "completion evidence artifact pair is incomplete",
            ));
        }
        (true, true) => {
            let terminal_state = inspection
                .terminal_transaction_state
                .as_deref()
                .ok_or_else(|| candidate_error("completion evidence requires terminal state"))?;
            let generation_digest = generation
                .as_ref()
                .map(|report| report.generation_digest.as_str());
            let journal_digest = fs::read_to_string(
                transaction_dir.join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE),
            )
            .map_err(|error| {
                candidate_error(format!("failed to read transaction journal digest: {error}"))
            })?;
            let report = verify_quarantine_replacement_completion_evidence_artifact(
                transaction_dir,
                generation_id,
                terminal_state,
                terminal_sequence(transaction_dir)?,
                journal_digest.trim(),
                generation_digest,
                now_unix_seconds,
            )?;
            Some(report.durable_age_seconds)
        }
    };

    Ok(QuarantineReplacementRetentionCandidate {
        generation_id: generation_id.clone(),
        classification: inspection.classification.clone(),
        terminal_transaction_state: inspection.terminal_transaction_state.clone(),
        verification_status: verification_status.to_string(),
        durable_age_seconds,
    })
}

fn terminal_sequence(transaction_dir: &Path) -> Result<u64, StoreError> {
    Ok(crate::read_quarantine_replacement_transaction_journal(transaction_dir)?.sequence)
}

fn candidate_error(message: impl Into<String>) -> StoreError {
    store_error(
        "LB_QUARANTINE_REPLACEMENT_RETENTION_CANDIDATE",
        message,
    )
}
