use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, JsonValue};

use crate::{
    build_quarantine_replacement_cleanup_plan, read_quarantine_replacement_transaction_journal,
    store_error, verify_quarantine_replacement_completion_evidence_artifact,
    verify_quarantine_replacement_generation, QuarantineReplacementCleanupPlan,
    QuarantineReplacementCleanupSubject, QuarantineReplacementRetentionDecisionReport,
    QuarantineReplacementTransactionState, StoreError, QUARANTINE_CURRENT_GENERATION_POINTER_FILE,
    QUARANTINE_CURRENT_GENERATION_POINTER_VERSION,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupSubjectInput {
    pub transaction_dir: PathBuf,
    pub generation_id: String,
    pub classification: String,
    pub expected_generation_digest: String,
    pub expected_managed_paths: Vec<String>,
}

pub fn build_quarantine_replacement_cleanup_preview_from_state(
    state_dir: impl AsRef<Path>,
    decisions: &QuarantineReplacementRetentionDecisionReport,
    state_identity: &str,
    runtime_fingerprint: &str,
    now_unix_seconds: u64,
    inputs: &[QuarantineReplacementCleanupSubjectInput],
) -> Result<QuarantineReplacementCleanupPlan, StoreError> {
    let state_dir = state_dir.as_ref();
    require_real_directory(state_dir, "state directory")?;

    let pointer_path = state_dir.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE);
    require_regular_file(&pointer_path, "current generation pointer")?;
    let pointer_text = fs::read_to_string(&pointer_path).map_err(io_error)?;
    let (active_transaction_id, active_generation_digest) = parse_active_pointer(&pointer_text)?;
    let active_pointer_digest = integrity_digest(pointer_text.as_bytes());

    let mut subjects = Vec::new();
    let mut seen_dirs = BTreeSet::new();
    for input in inputs {
        require_real_directory(&input.transaction_dir, "transaction directory")?;
        let canonical_dir = fs::canonicalize(&input.transaction_dir).map_err(io_error)?;
        if !seen_dirs.insert(canonical_dir) {
            return Err(builder_error("duplicate transaction directory"));
        }

        let journal = read_quarantine_replacement_transaction_journal(&input.transaction_dir)?;
        if journal.transaction_id != input.generation_id {
            return Err(builder_error("transaction and generation ID mismatch"));
        }
        let expected_terminal_state = match input.classification.as_str() {
            "previous-committed-generation" => QuarantineReplacementTransactionState::Committed,
            "rolled-back-generation" => QuarantineReplacementTransactionState::RolledBack,
            _ => return Err(builder_error("unsupported cleanup subject classification")),
        };
        if journal.state != expected_terminal_state {
            return Err(builder_error(
                "cleanup subject journal is not in the expected terminal state",
            ));
        }
        if journal.transaction_id == active_transaction_id
            || input.expected_generation_digest == active_generation_digest
        {
            return Err(builder_error(
                "active generation cannot be a cleanup subject",
            ));
        }

        let journal_digest = read_bound_digest(
            &input
                .transaction_dir
                .join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_DIGEST_FILE),
            "transaction journal digest",
        )?;
        let completion_digest = read_bound_digest(
            &input
                .transaction_dir
                .join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE),
            "completion evidence digest",
        )?;

        if expected_terminal_state == QuarantineReplacementTransactionState::Committed {
            let generation = verify_quarantine_replacement_generation(&input.transaction_dir)?;
            if generation.generation_digest != input.expected_generation_digest {
                return Err(builder_error("verified generation digest changed"));
            }
        }

        verify_quarantine_replacement_completion_evidence_artifact(
            &input.transaction_dir,
            &journal.transaction_id,
            expected_terminal_state.as_str(),
            journal.sequence,
            &journal_digest,
            Some(&input.expected_generation_digest),
            now_unix_seconds,
        )?;

        let managed_paths =
            verify_exact_inventory(&input.transaction_dir, &input.expected_managed_paths)?;
        subjects.push(QuarantineReplacementCleanupSubject {
            generation_id: input.generation_id.clone(),
            classification: input.classification.clone(),
            transaction_journal_digest: journal_digest,
            generation_digest: input.expected_generation_digest.clone(),
            completion_evidence_digest: completion_digest,
            managed_paths,
        });
    }

    build_quarantine_replacement_cleanup_plan(
        decisions,
        state_identity,
        &active_pointer_digest,
        runtime_fingerprint,
        subjects,
    )
}

fn parse_active_pointer(text: &str) -> Result<(String, String), StoreError> {
    let value = parse_json(text)
        .map_err(|error| builder_error(format!("invalid current generation pointer: {error}")))?;
    let map = object_map(&value, "current generation pointer")?;
    if object_string(map, "version")? != QUARANTINE_CURRENT_GENERATION_POINTER_VERSION {
        return Err(builder_error(
            "unsupported current generation pointer version",
        ));
    }
    Ok((
        object_string(map, "transactionId")?,
        object_string(map, "generationDigest")?,
    ))
}

fn object_map<'a>(
    value: &'a JsonValue,
    label: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(builder_error(format!("{label} must be an object"))),
    }
}

fn object_string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(builder_error(format!(
            "missing or invalid current generation pointer field: {name}"
        ))),
    }
}

fn verify_exact_inventory(root: &Path, expected: &[String]) -> Result<Vec<String>, StoreError> {
    if expected.is_empty() {
        return Err(builder_error("managed path inventory must not be empty"));
    }
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    if expected_set.len() != expected.len() {
        return Err(builder_error("duplicate expected managed path"));
    }
    let mut actual = BTreeSet::new();
    collect_inventory(root, root, &mut actual)?;
    if actual != expected_set {
        return Err(builder_error(
            "managed path inventory changed or contains unexpected entries",
        ));
    }
    Ok(actual.into_iter().collect())
}

fn collect_inventory(
    root: &Path,
    current: &Path,
    paths: &mut BTreeSet<String>,
) -> Result<(), StoreError> {
    let mut entries = fs::read_dir(current)
        .map_err(io_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(io_error)?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
        if metadata.file_type().is_symlink() {
            return Err(builder_error("managed inventory contains a symlink"));
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| builder_error("managed inventory escaped its root"))?
            .to_string_lossy()
            .replace('\\', "/");
        if metadata.is_dir() {
            collect_inventory(root, &path, paths)?;
        } else if metadata.is_file() {
            if !paths.insert(relative) {
                return Err(builder_error("duplicate managed inventory path"));
            }
        } else {
            return Err(builder_error(
                "managed inventory contains an unsupported file type",
            ));
        }
    }
    Ok(())
}

fn require_real_directory(path: &Path, label: &str) -> Result<(), StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(builder_error(format!("{label} must be a real directory")));
    }
    Ok(())
}

fn require_regular_file(path: &Path, label: &str) -> Result<(), StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(builder_error(format!("{label} must be a regular file")));
    }
    Ok(())
}

fn read_bound_digest(path: &Path, label: &str) -> Result<String, StoreError> {
    require_regular_file(path, label)?;
    let digest = fs::read_to_string(path).map_err(io_error)?;
    let digest = digest.trim().to_string();
    let Some(hex) = digest.strip_prefix("fnv1a64:") else {
        return Err(builder_error(format!("invalid {label}")));
    };
    if hex.len() != 16 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(builder_error(format!("invalid {label}")));
    }
    Ok(digest)
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
    builder_error(error.to_string())
}

fn builder_error(message: impl Into<String>) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_CLEANUP_PREVIEW_BUILDER", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-cleanup-builder-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn parses_only_supported_active_pointer() {
        let valid = format!(
            "{{\"generationDigest\":\"fnv1a64:1111111111111111\",\"transactionId\":\"tx-active\",\"version\":\"{}\"}}",
            QUARANTINE_CURRENT_GENERATION_POINTER_VERSION
        );
        assert_eq!(
            parse_active_pointer(&valid).unwrap(),
            (
                "tx-active".to_string(),
                "fnv1a64:1111111111111111".to_string()
            )
        );
        assert!(parse_active_pointer("{}").is_err());
        assert!(parse_active_pointer(&valid.replace("/v1", "/v2")).is_err());
    }

    #[test]
    fn exact_inventory_rejects_missing_unexpected_and_duplicates() {
        let dir = temp_dir("inventory");
        fs::create_dir_all(dir.join("nested")).unwrap();
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("nested/b.txt"), "b").unwrap();

        assert_eq!(
            verify_exact_inventory(&dir, &["nested/b.txt".to_string(), "a.txt".to_string()])
                .unwrap(),
            ["a.txt", "nested/b.txt"]
        );
        assert!(verify_exact_inventory(&dir, &["a.txt".to_string()]).is_err());
        assert!(verify_exact_inventory(&dir, &["a.txt".to_string(), "a.txt".to_string()]).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn exact_inventory_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir("symlink");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("target.txt"), "target").unwrap();
        symlink(dir.join("target.txt"), dir.join("link.txt")).unwrap();
        assert!(
            verify_exact_inventory(&dir, &["target.txt".to_string(), "link.txt".to_string()])
                .is_err()
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn digest_reader_rejects_malformed_and_symlinked_sidecars() {
        let dir = temp_dir("digest");
        fs::create_dir_all(&dir).unwrap();
        let digest = dir.join("record.digest");
        fs::write(&digest, "invalid\n").unwrap();
        assert!(read_bound_digest(&digest, "record digest").is_err());
        fs::write(&digest, "fnv1a64:1111111111111111\n").unwrap();
        assert_eq!(
            read_bound_digest(&digest, "record digest").unwrap(),
            "fnv1a64:1111111111111111"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
