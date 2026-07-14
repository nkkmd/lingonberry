use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    read_managed_ledger_lines, resolve_quarantine_active_path, store_error,
    verify_any_quarantine_backup, verify_quarantine_segments, StoreError,
    QUARANTINE_BACKUP_FILES, QUARANTINE_BACKUP_MANIFEST,
    QUARANTINE_COMPLETE_BACKUP_VERSION, QUARANTINE_SEGMENT_MANIFEST_FILE,
};

pub const QUARANTINE_COMPACTION_POLICY_VERSION: &str =
    "lingonberry-quarantine-compaction-policy/v1";
pub const QUARANTINE_COMPACTION_PROOF_VERSION: &str = "lingonberry-quarantine-compaction-proof/v1";
pub const QUARANTINE_COMPACTION_PROOF_FILE: &str = "quarantine-compaction-proof.json";
pub const QUARANTINE_COMPACTION_PROOF_DIGEST_FILE: &str = "quarantine-compaction-proof.digest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineCompactionLedgerPreview {
    pub ledger: String,
    pub classification: String,
    pub lines: u64,
    pub bytes: u64,
    pub ordered_digest: String,
    pub retained_lines: u64,
    pub removable_lines: u64,
    pub unique_keys: u64,
    pub blocked_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineCompactionProof {
    pub version: String,
    pub policy_version: String,
    pub generated_at: String,
    pub source_backup_manifest_digest: String,
    pub source_segment_manifest_digest: Option<String>,
    pub promoted: u64,
    pub dismissed: u64,
    pub permanently_rejected: u64,
    pub mutation_allowed: bool,
    pub rewrite_performed: bool,
    pub ledgers: Vec<QuarantineCompactionLedgerPreview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineCompactionProofReport {
    pub proof_path: PathBuf,
    pub ledgers: usize,
    pub total_lines: u64,
    pub removable_lines: u64,
    pub mutation_allowed: bool,
}

pub fn create_quarantine_compaction_preview(
    state_dir: impl AsRef<Path>,
    verified_backup_dir: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<QuarantineCompactionProofReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let backup_dir = verified_backup_dir.as_ref();
    let output_dir = output_dir.as_ref();

    verify_quarantine_segments(state_dir)?;
    verify_any_quarantine_backup(backup_dir)?;
    require_v2_backup(backup_dir)?;
    prepare_empty_output(output_dir)?;

    let before = runtime_fingerprint(state_dir)?;
    let mut ledgers = Vec::new();
    for ledger in QUARANTINE_BACKUP_FILES {
        let lines = read_managed_ledger_lines(state_dir, ledger)?;
        ledgers.push(preview_ledger(ledger, &lines)?);
    }
    let after = runtime_fingerprint(state_dir)?;
    if before != after {
        return Err(store_error(
            "LB_QUARANTINE_COMPACTION_CHANGED",
            "runtime state changed during compaction preview",
        ));
    }

    let proof = QuarantineCompactionProof {
        version: QUARANTINE_COMPACTION_PROOF_VERSION.to_string(),
        policy_version: QUARANTINE_COMPACTION_POLICY_VERSION.to_string(),
        generated_at: timestamp()?,
        source_backup_manifest_digest: file_digest(&backup_dir.join(QUARANTINE_BACKUP_MANIFEST))?,
        source_segment_manifest_digest: optional_file_digest(
            &state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE),
        )?,
        promoted: ledger_lines(&ledgers, "quarantine-resolutions.jsonl"),
        dismissed: ledger_lines(&ledgers, "quarantine-dismissals.jsonl"),
        permanently_rejected: ledger_lines(&ledgers, "quarantine-rejections.jsonl"),
        mutation_allowed: false,
        rewrite_performed: false,
        ledgers,
    };
    let proof_text = to_canonical_json(&proof_json(&proof));
    let proof_path = output_dir.join(QUARANTINE_COMPACTION_PROOF_FILE);
    let proof_tmp = output_dir.join(format!(".{QUARANTINE_COMPACTION_PROOF_FILE}.tmp"));
    fs::write(&proof_tmp, &proof_text).map_err(io_error)?;
    fs::rename(&proof_tmp, &proof_path).map_err(io_error)?;
    let digest = integrity_digest(proof_text.as_bytes());
    let digest_path = output_dir.join(QUARANTINE_COMPACTION_PROOF_DIGEST_FILE);
    let digest_tmp = output_dir.join(format!(".{QUARANTINE_COMPACTION_PROOF_DIGEST_FILE}.tmp"));
    fs::write(&digest_tmp, format!("{digest}\n")).map_err(io_error)?;
    fs::rename(&digest_tmp, &digest_path).map_err(io_error)?;
    verify_quarantine_compaction_proof(output_dir)
}

pub fn verify_quarantine_compaction_proof(
    proof_dir: impl AsRef<Path>,
) -> Result<QuarantineCompactionProofReport, StoreError> {
    let proof_dir = proof_dir.as_ref();
    let proof_path = proof_dir.join(QUARANTINE_COMPACTION_PROOF_FILE);
    let proof_text = fs::read_to_string(&proof_path)
        .map_err(|error| invalid(&format!("failed to read proof: {error}")))?;
    let expected_digest =
        fs::read_to_string(proof_dir.join(QUARANTINE_COMPACTION_PROOF_DIGEST_FILE))
            .map_err(|error| invalid(&format!("failed to read proof digest: {error}")))?;
    if expected_digest.trim() != integrity_digest(proof_text.as_bytes()) {
        return Err(invalid("compaction proof digest mismatch"));
    }
    let proof = parse_proof(&proof_text)?;
    validate_proof(&proof)?;
    Ok(report(proof_path, &proof))
}

pub fn quarantine_compaction_proof_report_json(
    report: &QuarantineCompactionProofReport,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "ledgers".to_string(),
            JsonValue::Number(report.ledgers.to_string()),
        ),
        (
            "mutationAllowed".to_string(),
            JsonValue::Bool(report.mutation_allowed),
        ),
        (
            "proofPath".to_string(),
            JsonValue::String(report.proof_path.to_string_lossy().to_string()),
        ),
        (
            "removableLines".to_string(),
            JsonValue::Number(report.removable_lines.to_string()),
        ),
        (
            "totalLines".to_string(),
            JsonValue::Number(report.total_lines.to_string()),
        ),
    ]))
}

fn preview_ledger(
    ledger: &str,
    lines: &[String],
) -> Result<QuarantineCompactionLedgerPreview, StoreError> {
    let classification = match ledger {
        "quarantine.jsonl" | "quarantine-annotations.jsonl" | "admin-auth-audit.jsonl" => {
            "immutable-evidence"
        }
        _ => "terminal-single-event",
    };
    let mut keys = BTreeSet::new();
    for line in lines {
        let value = parse_json(line)
            .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
        if classification == "terminal-single-event" {
            let key = object_string(&value, "quarantineId")?;
            if !keys.insert(key.clone()) {
                return Err(store_error(
                    "LB_QUARANTINE_CORRUPT",
                    format!("duplicate terminal event in {ledger}: {key}"),
                ));
            }
        } else if let Some(key) = optional_object_string(&value, "id")? {
            keys.insert(key);
        }
    }
    let bytes = logical_bytes(lines);
    Ok(QuarantineCompactionLedgerPreview {
        ledger: ledger.to_string(),
        classification: classification.to_string(),
        lines: lines.len() as u64,
        bytes: bytes.len() as u64,
        ordered_digest: integrity_digest(&bytes),
        retained_lines: lines.len() as u64,
        removable_lines: 0,
        unique_keys: keys.len() as u64,
        blocked_reason: if classification == "immutable-evidence" {
            "audit evidence is immutable under policy v1"
        } else {
            "duplicate terminal events are corruption, not removable history"
        }
        .to_string(),
    })
}

fn validate_proof(proof: &QuarantineCompactionProof) -> Result<(), StoreError> {
    if proof.version != QUARANTINE_COMPACTION_PROOF_VERSION
        || proof.policy_version != QUARANTINE_COMPACTION_POLICY_VERSION
    {
        return Err(invalid("unsupported proof or policy version"));
    }
    if proof.mutation_allowed || proof.rewrite_performed {
        return Err(invalid("proof must document a non-mutating preview"));
    }
    let names = proof
        .ledgers
        .iter()
        .map(|ledger| ledger.ledger.as_str())
        .collect::<BTreeSet<_>>();
    let expected = QUARANTINE_BACKUP_FILES
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if names != expected || proof.ledgers.len() != QUARANTINE_BACKUP_FILES.len() {
        return Err(invalid("proof must contain the exact managed ledger set"));
    }
    for ledger in &proof.ledgers {
        if ledger.retained_lines != ledger.lines || ledger.removable_lines != 0 {
            return Err(invalid(&format!(
                "policy v1 cannot remove lines from {}",
                ledger.ledger
            )));
        }
    }
    Ok(())
}

fn require_v2_backup(backup_dir: &Path) -> Result<(), StoreError> {
    let text = fs::read_to_string(backup_dir.join(QUARANTINE_BACKUP_MANIFEST)).map_err(io_error)?;
    let value = parse_json(&text).map_err(|error| invalid(&error.to_string()))?;
    if object_string(&value, "version")? != QUARANTINE_COMPLETE_BACKUP_VERSION {
        return Err(store_error(
            "LB_QUARANTINE_COMPACTION_BACKUP",
            "compaction preview requires a verified backup v2",
        ));
    }
    Ok(())
}

fn runtime_fingerprint(state_dir: &Path) -> Result<Vec<(String, Option<String>)>, StoreError> {
    let mut paths = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    paths.push(QUARANTINE_SEGMENT_MANIFEST_FILE.to_string());
    let archive_dir = state_dir.join("quarantine-segments");
    if archive_dir.exists() {
        let mut archive = fs::read_dir(&archive_dir)
            .map_err(io_error)?
            .map(|entry| {
                entry.map_err(io_error).map(|entry| {
                    format!(
                        "quarantine-segments/{}",
                        entry.file_name().to_string_lossy()
                    )
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        archive.sort();
        paths.extend(archive);
    }
    paths
        .into_iter()
        .map(|relative| {
            let path = if QUARANTINE_BACKUP_FILES.contains(&relative.as_str()) {
                resolve_quarantine_active_path(state_dir, &relative)?
            } else {
                state_dir.join(&relative)
            };
            let digest = if path.exists() {
                Some(file_digest(&path)?)
            } else {
                None
            };
            Ok((relative, digest))
        })
        .collect()
}

fn prepare_empty_output(path: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(path).map_err(io_error)?;
    if fs::read_dir(path).map_err(io_error)?.next().is_some() {
        return Err(store_error(
            "LB_QUARANTINE_COMPACTION_CONFLICT",
            "compaction proof output directory must be empty",
        ));
    }
    Ok(())
}

fn proof_json(proof: &QuarantineCompactionProof) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "dismissed".to_string(),
            JsonValue::Number(proof.dismissed.to_string()),
        ),
        (
            "generatedAt".to_string(),
            JsonValue::String(proof.generated_at.clone()),
        ),
        (
            "ledgers".to_string(),
            JsonValue::Array(proof.ledgers.iter().map(ledger_json).collect()),
        ),
        (
            "mutationAllowed".to_string(),
            JsonValue::Bool(proof.mutation_allowed),
        ),
        (
            "permanentlyRejected".to_string(),
            JsonValue::Number(proof.permanently_rejected.to_string()),
        ),
        (
            "policyVersion".to_string(),
            JsonValue::String(proof.policy_version.clone()),
        ),
        (
            "promoted".to_string(),
            JsonValue::Number(proof.promoted.to_string()),
        ),
        (
            "rewritePerformed".to_string(),
            JsonValue::Bool(proof.rewrite_performed),
        ),
        (
            "sourceBackupManifestDigest".to_string(),
            JsonValue::String(proof.source_backup_manifest_digest.clone()),
        ),
        (
            "sourceSegmentManifestDigest".to_string(),
            proof
                .source_segment_manifest_digest
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "version".to_string(),
            JsonValue::String(proof.version.clone()),
        ),
    ]))
}

fn ledger_json(ledger: &QuarantineCompactionLedgerPreview) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "blockedReason".to_string(),
            JsonValue::String(ledger.blocked_reason.clone()),
        ),
        (
            "bytes".to_string(),
            JsonValue::Number(ledger.bytes.to_string()),
        ),
        (
            "classification".to_string(),
            JsonValue::String(ledger.classification.clone()),
        ),
        (
            "ledger".to_string(),
            JsonValue::String(ledger.ledger.clone()),
        ),
        (
            "lines".to_string(),
            JsonValue::Number(ledger.lines.to_string()),
        ),
        (
            "orderedDigest".to_string(),
            JsonValue::String(ledger.ordered_digest.clone()),
        ),
        (
            "removableLines".to_string(),
            JsonValue::Number(ledger.removable_lines.to_string()),
        ),
        (
            "retainedLines".to_string(),
            JsonValue::Number(ledger.retained_lines.to_string()),
        ),
        (
            "uniqueKeys".to_string(),
            JsonValue::Number(ledger.unique_keys.to_string()),
        ),
    ]))
}

fn parse_proof(text: &str) -> Result<QuarantineCompactionProof, StoreError> {
    let map = object(parse_json(text).map_err(|error| invalid(&error.to_string()))?)?;
    let ledgers = match map.get("ledgers") {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(parse_ledger)
            .collect::<Result<Vec<_>, StoreError>>()?,
        _ => return Err(invalid("proof missing ledgers")),
    };
    Ok(QuarantineCompactionProof {
        version: string(&map, "version")?,
        policy_version: string(&map, "policyVersion")?,
        generated_at: string(&map, "generatedAt")?,
        source_backup_manifest_digest: string(&map, "sourceBackupManifestDigest")?,
        source_segment_manifest_digest: optional_string(&map, "sourceSegmentManifestDigest")?,
        promoted: number(&map, "promoted")?,
        dismissed: number(&map, "dismissed")?,
        permanently_rejected: number(&map, "permanentlyRejected")?,
        mutation_allowed: boolean(&map, "mutationAllowed")?,
        rewrite_performed: boolean(&map, "rewritePerformed")?,
        ledgers,
    })
}

fn parse_ledger(value: &JsonValue) -> Result<QuarantineCompactionLedgerPreview, StoreError> {
    let map = object(value.clone())?;
    Ok(QuarantineCompactionLedgerPreview {
        ledger: string(&map, "ledger")?,
        classification: string(&map, "classification")?,
        lines: number(&map, "lines")?,
        bytes: number(&map, "bytes")?,
        ordered_digest: string(&map, "orderedDigest")?,
        retained_lines: number(&map, "retainedLines")?,
        removable_lines: number(&map, "removableLines")?,
        unique_keys: number(&map, "uniqueKeys")?,
        blocked_reason: string(&map, "blockedReason")?,
    })
}

fn report(path: PathBuf, proof: &QuarantineCompactionProof) -> QuarantineCompactionProofReport {
    QuarantineCompactionProofReport {
        proof_path: path,
        ledgers: proof.ledgers.len(),
        total_lines: proof.ledgers.iter().map(|ledger| ledger.lines).sum(),
        removable_lines: proof
            .ledgers
            .iter()
            .map(|ledger| ledger.removable_lines)
            .sum(),
        mutation_allowed: proof.mutation_allowed,
    }
}

fn ledger_lines(ledgers: &[QuarantineCompactionLedgerPreview], name: &str) -> u64 {
    ledgers
        .iter()
        .find(|ledger| ledger.ledger == name)
        .map(|ledger| ledger.lines)
        .unwrap_or(0)
}

fn logical_bytes(lines: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    bytes
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

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match value {
        JsonValue::Object(map) => string(map, name),
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            "ledger line is not an object",
        )),
    }
}

fn optional_object_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match value {
        JsonValue::Object(map) => match map.get(name) {
            Some(JsonValue::String(value)) => Ok(Some(value.clone())),
            None => Ok(None),
            _ => Err(store_error(
                "LB_QUARANTINE_CORRUPT",
                format!("invalid optional string field: {name}"),
            )),
        },
        _ => Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            "ledger line is not an object",
        )),
    }
}

fn object(value: JsonValue) -> Result<BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(invalid("expected JSON object")),
    }
}

fn string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(invalid(&format!("missing string field: {name}"))),
    }
}

fn number(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<u64, StoreError> {
    match map.get(name) {
        Some(JsonValue::Number(value)) => value
            .parse()
            .map_err(|_| invalid(&format!("invalid number field: {name}"))),
        _ => Err(invalid(&format!("missing number field: {name}"))),
    }
}

fn boolean(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<bool, StoreError> {
    match map.get(name) {
        Some(JsonValue::Bool(value)) => Ok(*value),
        _ => Err(invalid(&format!("missing boolean field: {name}"))),
    }
}

fn optional_string(
    map: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<Option<String>, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(invalid(&format!("invalid optional string field: {name}"))),
    }
}

fn timestamp() -> Result<String, StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    Ok(format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()))
}

fn io_error(error: std::io::Error) -> StoreError {
    store_error("LB_QUARANTINE_IO", error.to_string())
}

fn invalid(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_COMPACTION_PROOF_INVALID", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build_quarantine_ledger_index, export_complete_quarantine_backup, rotate_quarantine_ledger,
    };

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn creates_non_mutating_preview_for_rotated_state() {
        let state = temp_dir("compaction-state");
        let backup = temp_dir("compaction-backup");
        let proof = temp_dir("compaction-proof");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), "{\"id\":\"q1\"}\n").unwrap();
        build_quarantine_ledger_index(&state).unwrap();
        rotate_quarantine_ledger(&state, "quarantine.jsonl").unwrap();
        fs::write(state.join("quarantine.jsonl"), "{\"id\":\"q2\"}\n").unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        let before = runtime_fingerprint(&state).unwrap();
        let report = create_quarantine_compaction_preview(&state, &backup, &proof).unwrap();
        assert_eq!(report.removable_lines, 0);
        assert!(!report.mutation_allowed);
        assert_eq!(before, runtime_fingerprint(&state).unwrap());
        assert_eq!(verify_quarantine_compaction_proof(&proof).unwrap(), report);
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
    }

    #[test]
    fn rejects_duplicate_terminal_events_and_tampered_proof() {
        let state = temp_dir("compaction-duplicate");
        let backup = temp_dir("compaction-duplicate-backup");
        let proof = temp_dir("compaction-duplicate-proof");
        fs::create_dir_all(&state).unwrap();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            "{\"quarantineId\":\"q1\"}\n{\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        assert_eq!(
            create_quarantine_compaction_preview(&state, &backup, &proof)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_CORRUPT"
        );
        let _ = fs::remove_dir_all(&proof);
        fs::create_dir_all(&proof).unwrap();
        fs::write(
            proof.join(QUARANTINE_COMPACTION_PROOF_FILE),
            "{\"tampered\":true}",
        )
        .unwrap();
        fs::write(
            proof.join(QUARANTINE_COMPACTION_PROOF_DIGEST_FILE),
            "fnv1a64:0000000000000000\n",
        )
        .unwrap();
        assert_eq!(
            verify_quarantine_compaction_proof(&proof).unwrap_err().code,
            "LB_QUARANTINE_COMPACTION_PROOF_INVALID"
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
    }
}
