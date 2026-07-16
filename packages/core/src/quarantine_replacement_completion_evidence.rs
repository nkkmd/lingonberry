use std::collections::BTreeMap;

use lingonberry_protocol::JsonValue;

use crate::{store_error, StoreError};

pub const QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_VERSION: &str =
    "lingonberry-quarantine-replacement-completion-evidence/v1";
pub const QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE: &str =
    "quarantine-replacement-completion-evidence.json";
pub const QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE: &str =
    "quarantine-replacement-completion-evidence.digest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCompletionEvidence {
    pub transaction_id: String,
    pub terminal_state: String,
    pub terminal_sequence: u64,
    pub completed_at_unix_seconds: u64,
    pub journal_digest: String,
    pub generation_digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCompletionEvidenceReport {
    pub transaction_id: String,
    pub terminal_state: String,
    pub terminal_sequence: u64,
    pub completed_at_unix_seconds: u64,
    pub durable_age_seconds: u64,
}

pub fn verify_quarantine_replacement_completion_evidence(
    evidence: &QuarantineReplacementCompletionEvidence,
    expected_transaction_id: &str,
    expected_terminal_state: &str,
    expected_terminal_sequence: u64,
    expected_journal_digest: &str,
    expected_generation_digest: Option<&str>,
    now_unix_seconds: u64,
) -> Result<QuarantineReplacementCompletionEvidenceReport, StoreError> {
    validate_transaction_id(&evidence.transaction_id)?;
    validate_digest(&evidence.journal_digest, "journal digest")?;
    if let Some(digest) = &evidence.generation_digest {
        validate_digest(digest, "generation digest")?;
    }

    if evidence.transaction_id != expected_transaction_id {
        return Err(completion_error("transaction ID mismatch"));
    }
    if !matches!(evidence.terminal_state.as_str(), "committed" | "rolled-back") {
        return Err(completion_error("completion evidence state is not terminal"));
    }
    if evidence.terminal_state != expected_terminal_state {
        return Err(completion_error("terminal state mismatch"));
    }
    if evidence.terminal_sequence != expected_terminal_sequence {
        return Err(completion_error("terminal sequence mismatch"));
    }
    if evidence.journal_digest != expected_journal_digest {
        return Err(completion_error("journal digest mismatch"));
    }
    if evidence.generation_digest.as_deref() != expected_generation_digest {
        return Err(completion_error("generation digest mismatch"));
    }
    if evidence.completed_at_unix_seconds > now_unix_seconds {
        return Err(completion_error("completion timestamp is in the future"));
    }

    Ok(QuarantineReplacementCompletionEvidenceReport {
        transaction_id: evidence.transaction_id.clone(),
        terminal_state: evidence.terminal_state.clone(),
        terminal_sequence: evidence.terminal_sequence,
        completed_at_unix_seconds: evidence.completed_at_unix_seconds,
        durable_age_seconds: now_unix_seconds - evidence.completed_at_unix_seconds,
    })
}

pub fn quarantine_replacement_completion_evidence_json(
    evidence: &QuarantineReplacementCompletionEvidence,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "completedAtUnixSeconds".to_string(),
            JsonValue::Number(evidence.completed_at_unix_seconds.to_string()),
        ),
        (
            "generationDigest".to_string(),
            evidence
                .generation_digest
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "journalDigest".to_string(),
            JsonValue::String(evidence.journal_digest.clone()),
        ),
        (
            "terminalSequence".to_string(),
            JsonValue::Number(evidence.terminal_sequence.to_string()),
        ),
        (
            "terminalState".to_string(),
            JsonValue::String(evidence.terminal_state.clone()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(evidence.transaction_id.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_VERSION.to_string()),
        ),
    ]))
}

pub fn quarantine_replacement_completion_evidence_report_json(
    report: &QuarantineReplacementCompletionEvidenceReport,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "completedAtUnixSeconds".to_string(),
            JsonValue::Number(report.completed_at_unix_seconds.to_string()),
        ),
        (
            "durableAgeSeconds".to_string(),
            JsonValue::Number(report.durable_age_seconds.to_string()),
        ),
        (
            "terminalSequence".to_string(),
            JsonValue::Number(report.terminal_sequence.to_string()),
        ),
        (
            "terminalState".to_string(),
            JsonValue::String(report.terminal_state.clone()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(report.transaction_id.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_VERSION.to_string()),
        ),
    ]))
}

fn validate_transaction_id(value: &str) -> Result<(), StoreError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains(['/', '\\'])
        || !value.is_ascii()
    {
        return Err(completion_error("invalid transaction ID"));
    }
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<(), StoreError> {
    let Some(hex) = value.strip_prefix("fnv1a64:") else {
        return Err(completion_error(&format!("invalid {label}")));
    };
    if hex.len() != 16 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(completion_error(&format!("invalid {label}")));
    }
    Ok(())
}

fn completion_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> QuarantineReplacementCompletionEvidence {
        QuarantineReplacementCompletionEvidence {
            transaction_id: "tx-1".to_string(),
            terminal_state: "committed".to_string(),
            terminal_sequence: 5,
            completed_at_unix_seconds: 1_000,
            journal_digest: "fnv1a64:1111111111111111".to_string(),
            generation_digest: Some("fnv1a64:2222222222222222".to_string()),
        }
    }

    fn verify(
        value: &QuarantineReplacementCompletionEvidence,
        now: u64,
    ) -> Result<QuarantineReplacementCompletionEvidenceReport, StoreError> {
        verify_quarantine_replacement_completion_evidence(
            value,
            "tx-1",
            "committed",
            5,
            "fnv1a64:1111111111111111",
            Some("fnv1a64:2222222222222222"),
            now,
        )
    }

    #[test]
    fn verifies_bound_completion_evidence_and_computes_age() {
        let report = verify(&evidence(), 1_600).unwrap();
        assert_eq!(report.durable_age_seconds, 600);
    }

    #[test]
    fn rejects_future_completion_timestamp() {
        let error = verify(&evidence(), 999).unwrap_err();
        assert!(error.to_string().contains("timestamp is in the future"));
    }

    #[test]
    fn rejects_non_terminal_and_mismatched_state() {
        let mut value = evidence();
        value.terminal_state = "publishing".to_string();
        assert!(verify(&value, 2_000).is_err());

        value.terminal_state = "rolled-back".to_string();
        assert!(verify(&value, 2_000).is_err());
    }

    #[test]
    fn rejects_sequence_and_digest_mismatches() {
        let mut value = evidence();
        value.terminal_sequence = 4;
        assert!(verify(&value, 2_000).is_err());

        let mut value = evidence();
        value.journal_digest = "fnv1a64:3333333333333333".to_string();
        assert!(verify(&value, 2_000).is_err());

        let mut value = evidence();
        value.generation_digest = None;
        assert!(verify(&value, 2_000).is_err());
    }

    #[test]
    fn rejects_invalid_transaction_id_and_digest_shape() {
        let mut value = evidence();
        value.transaction_id = "../tx".to_string();
        assert!(verify(&value, 2_000).is_err());

        let mut value = evidence();
        value.journal_digest = "sha256:not-supported".to_string();
        assert!(verify(&value, 2_000).is_err());
    }
}
