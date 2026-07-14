use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    acquire_quarantine_lock, create_quarantine_replacement_transaction_journal, store_error,
    resolve_quarantine_active_path, verify_any_quarantine_backup,
    verify_quarantine_replacement_proof, verify_quarantine_segments,
    write_quarantine_replacement_inputs,
    QuarantineReplacementTransactionJournal, QuarantineReplacementTransactionReport,
    QuarantineReplacementTransactionState, StoreError, QUARANTINE_BACKUP_FILES,
    QUARANTINE_BACKUP_MANIFEST, QUARANTINE_COMPLETE_BACKUP_VERSION,
    QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE, QUARANTINE_REPLACEMENT_PLAN_FILE,
    QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE, QUARANTINE_REPLACEMENT_PROOF_FILE,
    QUARANTINE_SEGMENT_ARCHIVE_DIR, QUARANTINE_SEGMENT_MANIFEST_FILE,
};

pub fn prepare_quarantine_replacement_transaction(
    state_dir: impl AsRef<Path>,
    verified_backup_dir: impl AsRef<Path>,
    verified_proof_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
    transaction_id: &str,
) -> Result<QuarantineReplacementTransactionReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let backup_dir = verified_backup_dir.as_ref();
    let proof_dir = verified_proof_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();

    let _lock = acquire_quarantine_lock(state_dir, "quarantine-replacement-prepare-v1")?;
    verify_quarantine_segments(state_dir)?;
    verify_any_quarantine_backup(backup_dir)?;
    require_v2_backup(backup_dir)?;
    let proof_report = verify_quarantine_replacement_proof(proof_dir)?;
    if proof_report.mutation_allowed {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            "replacement proof unexpectedly permits mutation",
        ));
    }

    let plan_text = fs::read_to_string(proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_FILE))
        .map_err(io_error)?;
    let plan = parse_json(&plan_text).map_err(|error| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            &format!("invalid replacement plan JSON: {error}"),
        )
    })?;
    let plan_digest = read_digest(
        &proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE),
        "replacement plan digest",
    )?;
    let proof_digest = read_digest(
        &proof_dir.join(QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE),
        "replacement proof digest",
    )?;

    let backup_manifest_digest = file_digest(&backup_dir.join(QUARANTINE_BACKUP_MANIFEST))?;
    if object_string(&plan, "sourceBackupManifestDigest")? != backup_manifest_digest {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_BACKUP",
            "verified backup does not match replacement plan",
        ));
    }

    let segment_manifest_digest =
        optional_file_digest(&state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE))?;
    if object_optional_string(&plan, "sourceSegmentManifestDigest")? != segment_manifest_digest {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_CHANGED",
            "segment manifest changed since replacement preview",
        ));
    }

    let runtime_fingerprint = runtime_fingerprint_json(state_dir)?;
    let planned_fingerprint = object_field(&plan, "runtimeFingerprint")?;
    if to_canonical_json(planned_fingerprint) != to_canonical_json(&runtime_fingerprint) {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_CHANGED",
            "runtime fingerprint changed since replacement preview",
        ));
    }
    let runtime_fingerprint_digest =
        integrity_digest(to_canonical_json(&runtime_fingerprint).as_bytes());

    let report = create_quarantine_replacement_transaction_journal(
        transaction_dir,
        &QuarantineReplacementTransactionJournal {
            transaction_id: transaction_id.to_string(),
            state: QuarantineReplacementTransactionState::Prepared,
            sequence: 0,
            backup_manifest_digest,
            segment_manifest_digest,
            plan_digest,
            proof_digest,
            runtime_fingerprint_digest,
        },
    )?;
    if let Err(error) = write_quarantine_replacement_inputs(transaction_dir, backup_dir, proof_dir)
    {
        let _ = crate::advance_quarantine_replacement_transaction_journal(
            transaction_dir,
            QuarantineReplacementTransactionState::RecoveryRequired,
        );
        return Err(error);
    }
    Ok(report)
}

fn require_v2_backup(backup_dir: &Path) -> Result<(), StoreError> {
    let manifest_text = fs::read_to_string(backup_dir.join(QUARANTINE_BACKUP_MANIFEST))
        .map_err(io_error)?;
    let manifest = parse_json(&manifest_text).map_err(|error| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_BACKUP",
            &format!("invalid backup manifest JSON: {error}"),
        )
    })?;
    if object_string(&manifest, "version")? != QUARANTINE_COMPLETE_BACKUP_VERSION {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_BACKUP",
            "replacement transaction requires a verified backup v2",
        ));
    }
    Ok(())
}

fn runtime_fingerprint_json(state_dir: &Path) -> Result<JsonValue, StoreError> {
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
    Ok(JsonValue::Array(values))
}

fn read_digest(path: &Path, name: &str) -> Result<String, StoreError> {
    let digest = fs::read_to_string(path).map_err(|error| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            &format!("failed to read {name}: {error}"),
        )
    })?;
    let digest = digest.trim().to_string();
    validate_digest(&digest, name)?;
    Ok(digest)
}

fn file_digest(path: &Path) -> Result<String, StoreError> {
    fs::read(path)
        .map(|bytes| integrity_digest(&bytes))
        .map_err(io_error)
}

fn optional_file_digest(path: &Path) -> Result<Option<String>, StoreError> {
    if path.exists() {
        Ok(Some(file_digest(path)?))
    } else {
        Ok(None)
    }
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            "expected JSON object",
        )),
    }
}

fn object_field<'a>(value: &'a JsonValue, name: &str) -> Result<&'a JsonValue, StoreError> {
    object_map(value)?.get(name).ok_or_else(|| {
        transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            &format!("missing field: {name}"),
        )
    })
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            &format!("invalid string field: {name}"),
        )),
    }
}

fn object_optional_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(Some(value.clone())),
        JsonValue::Null => Ok(None),
        _ => Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            &format!("invalid optional string field: {name}"),
        )),
    }
}

fn validate_digest(value: &str, name: &str) -> Result<(), StoreError> {
    if value.len() != "fnv1a64:".len() + 16
        || !value.starts_with("fnv1a64:")
        || !value["fnv1a64:".len()..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(transaction_error(
            "LB_QUARANTINE_REPLACEMENT_PROOF",
            &format!("invalid {name}"),
        ));
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

fn io_error(error: std::io::Error) -> StoreError {
    transaction_error("LB_QUARANTINE_REPLACEMENT_TRANSACTION", &error.to_string())
}

fn transaction_error(code: &'static str, message: &str) -> StoreError {
    store_error(code, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_quarantine_replacement_preview, export_complete_quarantine_backup,
        QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE,
    };
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
        let state = temp_dir("replacement-prepare-state");
        let backup = temp_dir("replacement-prepare-backup");
        let proof = temp_dir("replacement-prepare-proof");
        let transaction = temp_dir("replacement-prepare-transaction");
        fs::create_dir_all(&state).unwrap();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            "{\"canonicalId\":\"c1\", \"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        create_quarantine_replacement_preview(&state, &backup, &proof).unwrap();
        (state, backup, proof, transaction)
    }

    #[test]
    fn prepares_journal_only_after_all_verified_gates_pass() {
        let (state, backup, proof, transaction) = fixture();
        let report = prepare_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-verified-001",
        )
        .unwrap();
        assert_eq!(report.state, QuarantineReplacementTransactionState::Prepared);
        assert_eq!(report.sequence, 0);
        assert!(transaction
            .join(QUARANTINE_REPLACEMENT_TRANSACTION_JOURNAL_FILE)
            .is_file());
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn rejects_changed_runtime_before_creating_transaction_directory() {
        let (state, backup, proof, transaction) = fixture();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            "{\"canonicalId\":\"c2\",\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        assert_eq!(
            prepare_quarantine_replacement_transaction(
                &state,
                &backup,
                &proof,
                &transaction,
                "tx-stale-001"
            )
            .unwrap_err()
            .code,
            "LB_QUARANTINE_REPLACEMENT_CHANGED"
        );
        assert!(!transaction.exists());
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
    }

    #[test]
    fn rejects_tampered_proof_before_creating_transaction_directory() {
        let (state, backup, proof, transaction) = fixture();
        fs::write(
            proof.join(QUARANTINE_REPLACEMENT_PROOF_FILE),
            "{\"tampered\":true}",
        )
        .unwrap();
        assert_eq!(
            prepare_quarantine_replacement_transaction(
                &state,
                &backup,
                &proof,
                &transaction,
                "tx-tampered-001"
            )
            .unwrap_err()
            .code,
            "LB_QUARANTINE_REPLACEMENT_PROOF"
        );
        assert!(!transaction.exists());
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
    }
}
