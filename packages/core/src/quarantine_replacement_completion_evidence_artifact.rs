use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use lingonberry_protocol::{parse_json, JsonValue};

use crate::{
    store_error, verify_quarantine_replacement_completion_evidence,
    QuarantineReplacementCompletionEvidence, QuarantineReplacementCompletionEvidenceReport,
    StoreError, QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_VERSION,
};

pub fn verify_quarantine_replacement_completion_evidence_artifact(
    transaction_dir: impl AsRef<Path>,
    expected_transaction_id: &str,
    expected_terminal_state: &str,
    expected_terminal_sequence: u64,
    expected_journal_digest: &str,
    expected_generation_digest: Option<&str>,
    now_unix_seconds: u64,
) -> Result<QuarantineReplacementCompletionEvidenceReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let evidence_path = transaction_dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE);
    let digest_path = transaction_dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE);

    let text = fs::read_to_string(&evidence_path).map_err(|error| {
        artifact_error(&format!("failed to read completion evidence: {error}"))
    })?;
    let expected_digest = fs::read_to_string(&digest_path).map_err(|error| {
        artifact_error(&format!("failed to read completion evidence digest: {error}"))
    })?;
    if expected_digest.trim() != integrity_digest(text.as_bytes()) {
        return Err(artifact_error("completion evidence digest mismatch"));
    }

    let value = parse_json(&text).map_err(|error| {
        artifact_error(&format!("invalid completion evidence JSON: {error}"))
    })?;
    let evidence = parse_evidence(&value)?;
    verify_quarantine_replacement_completion_evidence(
        &evidence,
        expected_transaction_id,
        expected_terminal_state,
        expected_terminal_sequence,
        expected_journal_digest,
        expected_generation_digest,
        now_unix_seconds,
    )
}

fn parse_evidence(value: &JsonValue) -> Result<QuarantineReplacementCompletionEvidence, StoreError> {
    require_string(value, "version", QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_VERSION)?;
    Ok(QuarantineReplacementCompletionEvidence {
        transaction_id: object_string(value, "transactionId")?,
        terminal_state: object_string(value, "terminalState")?,
        terminal_sequence: object_number(value, "terminalSequence")?,
        completed_at_unix_seconds: object_number(value, "completedAtUnixSeconds")?,
        journal_digest: object_string(value, "journalDigest")?,
        generation_digest: object_optional_string(value, "generationDigest")?,
    })
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(artifact_error("completion evidence must be an object")),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(artifact_error(&format!("missing or invalid string field: {name}"))),
    }
}

fn object_optional_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(artifact_error(&format!("missing or invalid optional string field: {name}"))),
    }
}

fn object_number(value: &JsonValue, name: &str) -> Result<u64, StoreError> {
    match object_map(value)?.get(name) {
        Some(JsonValue::Number(value)) => value.parse().map_err(|_| {
            artifact_error(&format!("invalid number field: {name}"))
        }),
        _ => Err(artifact_error(&format!("missing or invalid number field: {name}"))),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(artifact_error(&format!("unsupported {name}")));
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

fn artifact_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_ARTIFACT", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn evidence_text() -> String {
        format!(
            "{{\"completedAtUnixSeconds\":1000,\"generationDigest\":\"fnv1a64:2222222222222222\",\"journalDigest\":\"fnv1a64:1111111111111111\",\"terminalSequence\":5,\"terminalState\":\"committed\",\"transactionId\":\"tx-1\",\"version\":\"{}\"}}",
            QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_VERSION
        )
    }

    fn write_artifacts(dir: &Path, text: &str, digest: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE),
            text,
        )
        .unwrap();
        fs::write(
            dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE),
            format!("{digest}\n"),
        )
        .unwrap();
    }

    fn verify(dir: &Path) -> Result<QuarantineReplacementCompletionEvidenceReport, StoreError> {
        verify_quarantine_replacement_completion_evidence_artifact(
            dir,
            "tx-1",
            "committed",
            5,
            "fnv1a64:1111111111111111",
            Some("fnv1a64:2222222222222222"),
            1_600,
        )
    }

    #[test]
    fn verifies_bound_artifact_pair() {
        let dir = temp_dir("completion-evidence-artifact");
        let text = evidence_text();
        write_artifacts(&dir, &text, &integrity_digest(text.as_bytes()));
        let report = verify(&dir).unwrap();
        assert_eq!(report.durable_age_seconds, 600);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_missing_or_partial_artifact_pair() {
        let dir = temp_dir("completion-evidence-partial");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE),
            evidence_text(),
        )
        .unwrap();
        assert!(verify(&dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_tampered_artifact_and_unsupported_version() {
        let dir = temp_dir("completion-evidence-tampered");
        let text = evidence_text();
        write_artifacts(&dir, &text, "fnv1a64:0000000000000000");
        assert!(verify(&dir).is_err());

        let unsupported = text.replace(
            QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_VERSION,
            "lingonberry-quarantine-replacement-completion-evidence/v2",
        );
        write_artifacts(&dir, &unsupported, &integrity_digest(unsupported.as_bytes()));
        assert!(verify(&dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
