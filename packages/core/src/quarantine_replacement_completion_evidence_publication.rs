#![rustfmt::skip]

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use lingonberry_protocol::to_canonical_json;

use crate::{
    quarantine_replacement_completion_evidence_json, store_error,
    QuarantineReplacementCompletionEvidence, StoreError,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE,
};

pub fn publish_quarantine_replacement_completion_evidence(
    transaction_dir: impl AsRef<Path>,
    evidence: &QuarantineReplacementCompletionEvidence,
) -> Result<(), StoreError> {
    let transaction_dir = transaction_dir.as_ref();
    let text = to_canonical_json(&quarantine_replacement_completion_evidence_json(evidence));
    let digest = integrity_digest(text.as_bytes());

    let evidence_path = transaction_dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE);
    let digest_path = transaction_dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE);
    let evidence_tmp = transaction_dir.join(".quarantine-replacement-completion-evidence.json.tmp");
    let digest_tmp = transaction_dir.join(".quarantine-replacement-completion-evidence.digest.tmp");

    if evidence_path.exists() || digest_path.exists() {
        return verify_existing_pair(&evidence_path, &digest_path, &text, &digest);
    }

    if evidence_tmp.exists() || digest_tmp.exists() {
        return Err(publication_error(
            "stale completion evidence temporary artifact requires manual review",
        ));
    }

    let result = (|| {
        write_new_synced(&evidence_tmp, text.as_bytes())?;
        write_new_synced(&digest_tmp, format!("{digest}\n").as_bytes())?;
        fs::rename(&evidence_tmp, &evidence_path).map_err(io_error)?;
        fs::rename(&digest_tmp, &digest_path).map_err(io_error)?;
        sync_directory(transaction_dir)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&evidence_tmp);
        let _ = fs::remove_file(&digest_tmp);
    }

    result
}

fn verify_existing_pair(
    evidence_path: &Path,
    digest_path: &Path,
    expected_text: &str,
    expected_digest: &str,
) -> Result<(), StoreError> {
    if !evidence_path.is_file() || !digest_path.is_file() {
        return Err(publication_error(
            "partial completion evidence artifact pair requires manual review",
        ));
    }

    let actual_text = fs::read_to_string(evidence_path).map_err(io_error)?;
    let actual_digest = fs::read_to_string(digest_path).map_err(io_error)?;
    if actual_text != expected_text || actual_digest.trim() != expected_digest {
        return Err(publication_error(
            "conflicting completion evidence artifact already exists",
        ));
    }

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

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn io_error(error: std::io::Error) -> StoreError {
    publication_error(&error.to_string())
}

fn publication_error(message: &str) -> StoreError {
    store_error(
        "LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_PUBLICATION",
        message,
    )
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

    #[test]
    fn publishes_synced_artifact_pair_and_is_idempotent() {
        let dir = temp_dir("completion-evidence-publication");
        fs::create_dir_all(&dir).unwrap();

        publish_quarantine_replacement_completion_evidence(&dir, &evidence()).unwrap();
        publish_quarantine_replacement_completion_evidence(&dir, &evidence()).unwrap();

        assert!(dir
            .join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE)
            .is_file());
        assert!(dir
            .join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE)
            .is_file());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_partial_or_conflicting_existing_artifacts() {
        let dir = temp_dir("completion-evidence-conflict");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE),
            "conflict",
        )
        .unwrap();
        assert!(publish_quarantine_replacement_completion_evidence(&dir, &evidence()).is_err());

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        publish_quarantine_replacement_completion_evidence(&dir, &evidence()).unwrap();
        fs::write(
            dir.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE),
            "fnv1a64:0000000000000000\n",
        )
        .unwrap();
        assert!(publish_quarantine_replacement_completion_evidence(&dir, &evidence()).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_stale_temporary_artifacts() {
        let dir = temp_dir("completion-evidence-stale-temp");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(".quarantine-replacement-completion-evidence.json.tmp"),
            "stale",
        )
        .unwrap();

        let error =
            publish_quarantine_replacement_completion_evidence(&dir, &evidence()).unwrap_err();
        assert!(error.to_string().contains("temporary artifact"));
        let _ = fs::remove_dir_all(dir);
    }
}
