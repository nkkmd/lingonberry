use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    advance_quarantine_replacement_transaction_journal, acquire_quarantine_lock,
    read_quarantine_replacement_transaction_journal, resolve_quarantine_active_path, store_error,
    verify_quarantine_replacement_proof, QuarantineReplacementStagedLedger,
    QuarantineReplacementTransactionState, StoreError, QUARANTINE_BACKUP_FILES,
    QUARANTINE_REPLACEMENT_PLAN_FILE, QUARANTINE_REPLACEMENT_STAGING_DIR,
    QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE, QUARANTINE_REPLACEMENT_STAGING_VERSION,
};

const IMMUTABLE_LEDGERS: [&str; 3] = [
    "quarantine.jsonl",
    "quarantine-annotations.jsonl",
    "admin-auth-audit.jsonl",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementStagingVerificationReport {
    pub staging_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub ledgers: Vec<QuarantineReplacementStagedLedger>,
    pub replacement_lines: u64,
}

pub fn verify_quarantine_replacement_staging(
    state_dir: impl AsRef<Path>,
    verified_proof_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementStagingVerificationReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let proof_dir = verified_proof_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-replacement-verify-staging-v1")?;

    let journal = read_quarantine_replacement_transaction_journal(transaction_dir)?;
    if journal.state != QuarantineReplacementTransactionState::Staged {
        return Err(verification_error(
            "transaction must be staged before verification",
        ));
    }

    let result = verify_staging_set(
        state_dir,
        proof_dir,
        transaction_dir,
        &journal.transaction_id,
    );
    match result {
        Ok(report) => {
            advance_quarantine_replacement_transaction_journal(
                transaction_dir,
                QuarantineReplacementTransactionState::Verified,
            )?;
            Ok(report)
        }
        Err(error) => {
            let _ = advance_quarantine_replacement_transaction_journal(
                transaction_dir,
                QuarantineReplacementTransactionState::RecoveryRequired,
            );
            Err(error)
        }
    }
}

fn verify_staging_set(
    state_dir: &Path,
    proof_dir: &Path,
    transaction_dir: &Path,
    transaction_id: &str,
) -> Result<QuarantineReplacementStagingVerificationReport, StoreError> {
    verify_quarantine_replacement_proof(proof_dir)?;
    let plan_text = fs::read_to_string(proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_FILE))
        .map_err(io_error)?;
    let plan = parse_json(&plan_text)
        .map_err(|error| verification_error(&format!("invalid replacement plan JSON: {error}")))?;

    let staging_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_STAGING_DIR);
    let manifest_path = staging_dir.join(QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE);
    let manifest_text = fs::read_to_string(&manifest_path).map_err(io_error)?;
    let manifest = parse_json(&manifest_text)
        .map_err(|error| verification_error(&format!("invalid staging manifest JSON: {error}")))?;
    require_string(
        &manifest,
        "version",
        QUARANTINE_REPLACEMENT_STAGING_VERSION,
    )?;
    if object_string(&manifest, "transactionId")? != transaction_id {
        return Err(verification_error("staging manifest transaction mismatch"));
    }

    let plan_ledgers = object_array(&plan, "ledgers")?;
    let manifest_ledgers = object_array(&manifest, "ledgers")?;
    validate_exact_ledger_set(plan_ledgers, manifest_ledgers)?;
    validate_staging_directory_membership(&staging_dir, manifest_ledgers)?;

    let mut reports = Vec::new();
    for ledger in QUARANTINE_BACKUP_FILES {
        let ledger_plan = find_ledger(plan_ledgers, ledger)?;
        let ledger_manifest = find_ledger(manifest_ledgers, ledger)?;
        reports.push(verify_staged_ledger(
            state_dir,
            &staging_dir,
            ledger,
            ledger_plan,
            ledger_manifest,
        )?);
    }

    let replacement_lines = reports.iter().map(|ledger| ledger.replacements).sum();
    if object_number(&manifest, "replacementLines")? != replacement_lines {
        return Err(verification_error(
            "staging manifest replacement count mismatch",
        ));
    }

    Ok(QuarantineReplacementStagingVerificationReport {
        staging_dir,
        manifest_path,
        ledgers: reports,
        replacement_lines,
    })
}

fn validate_exact_ledger_set(
    plan_ledgers: &[JsonValue],
    manifest_ledgers: &[JsonValue],
) -> Result<(), StoreError> {
    let expected = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    let plan_names = ledger_names(plan_ledgers)?;
    let manifest_names = ledger_names(manifest_ledgers)?;
    if plan_names != expected
        || manifest_names != expected
        || plan_ledgers.len() != QUARANTINE_BACKUP_FILES.len()
        || manifest_ledgers.len() != QUARANTINE_BACKUP_FILES.len()
    {
        return Err(verification_error("staging managed-ledger set mismatch"));
    }
    Ok(())
}

fn ledger_names(ledgers: &[JsonValue]) -> Result<BTreeSet<String>, StoreError> {
    ledgers
        .iter()
        .map(|ledger| object_string(ledger, "ledger"))
        .collect()
}

fn validate_staging_directory_membership(
    staging_dir: &Path,
    manifest_ledgers: &[JsonValue],
) -> Result<(), StoreError> {
    let mut expected = BTreeSet::from([QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE.to_string()]);
    for ledger in manifest_ledgers {
        if object_bool(ledger, "present")? {
            expected.insert(object_string(ledger, "ledger")?);
        }
    }
    let actual = fs::read_dir(staging_dir)
        .map_err(io_error)?
        .map(|entry| {
            entry
                .map_err(io_error)
                .map(|entry| entry.file_name().to_string_lossy().to_string())
        })
        .collect::<Result<BTreeSet<_>, StoreError>>()?;
    if actual != expected {
        return Err(verification_error(
            "staging directory contains missing or unexpected files",
        ));
    }
    Ok(())
}

fn find_ledger<'a>(ledgers: &'a [JsonValue], name: &str) -> Result<&'a JsonValue, StoreError> {
    ledgers
        .iter()
        .find(|ledger| object_string(ledger, "ledger").ok().as_deref() == Some(name))
        .ok_or_else(|| verification_error(&format!("missing ledger metadata: {name}")))
}

fn verify_staged_ledger(
    state_dir: &Path,
    staging_dir: &Path,
    ledger: &str,
    ledger_plan: &JsonValue,
    ledger_manifest: &JsonValue,
) -> Result<QuarantineReplacementStagedLedger, StoreError> {
    let present = object_bool(ledger_manifest, "present")?;
    let bytes = object_number(ledger_manifest, "bytes")?;
    let lines = object_number(ledger_manifest, "lines")?;
    let replacements = object_number(ledger_manifest, "replacements")?;
    let digest = object_optional_string(ledger_manifest, "digest")?;
    let active_path = resolve_quarantine_active_path(state_dir, ledger)?;
    let staged_path = staging_dir.join(ledger);

    if !present {
        if active_path.exists()
            || staged_path.exists()
            || bytes != 0
            || lines != 0
            || replacements != 0
            || digest.is_some()
        {
            return Err(verification_error("invalid absent staged ledger metadata"));
        }
        ensure_no_active_plan_entries(ledger_plan)?;
        return Ok(QuarantineReplacementStagedLedger {
            ledger: ledger.to_string(),
            present,
            bytes,
            lines,
            digest,
            replacements,
        });
    }

    if !active_path.is_file() || !staged_path.is_file() {
        return Err(verification_error("active or staged ledger is missing"));
    }
    let active = fs::read(&active_path).map_err(io_error)?;
    let staged = fs::read(&staged_path).map_err(io_error)?;
    if bytes != staged.len() as u64 || digest.as_deref() != Some(integrity_digest(&staged).as_str()) {
        return Err(verification_error("staged ledger metadata mismatch"));
    }

    let (actual_lines, actual_replacements) = if IMMUTABLE_LEDGERS.contains(&ledger) {
        if active != staged {
            return Err(verification_error("immutable staged ledger changed bytes"));
        }
        require_immutable_plan(ledger_plan)?;
        (count_jsonl_lines(&staged, ledger)?, 0)
    } else {
        verify_terminal_staging(&active, &staged, ledger, ledger_plan)?
    };
    if lines != actual_lines || replacements != actual_replacements {
        return Err(verification_error("staged ledger count mismatch"));
    }

    Ok(QuarantineReplacementStagedLedger {
        ledger: ledger.to_string(),
        present,
        bytes,
        lines,
        digest,
        replacements,
    })
}

fn ensure_no_active_plan_entries(ledger_plan: &JsonValue) -> Result<(), StoreError> {
    for entry in object_array(ledger_plan, "entries")? {
        let source = object_field(entry, "source")?;
        if object_string(source, "location")? == "active-ledger" {
            return Err(verification_error(
                "absent ledger has active replacement-plan entries",
            ));
        }
        require_archived_retention(entry)?;
    }
    Ok(())
}

fn require_immutable_plan(ledger_plan: &JsonValue) -> Result<(), StoreError> {
    for entry in object_array(ledger_plan, "entries")? {
        if object_string(entry, "decision")? != "retain-byte-for-byte" {
            return Err(verification_error("immutable ledger contains replacement"));
        }
    }
    Ok(())
}

fn verify_terminal_staging(
    active_bytes: &[u8],
    staged_bytes: &[u8],
    ledger: &str,
    ledger_plan: &JsonValue,
) -> Result<(u64, u64), StoreError> {
    let active_lines = jsonl_lines(active_bytes, ledger)?;
    let staged_lines = jsonl_lines(staged_bytes, ledger)?;
    if active_lines.len() != staged_lines.len() {
        return Err(verification_error("staged terminal ledger line count changed"));
    }

    let mut active_entries = BTreeMap::<u64, &JsonValue>::new();
    for entry in object_array(ledger_plan, "entries")? {
        let source = object_field(entry, "source")?;
        if object_string(source, "location")? != "active-ledger" {
            require_archived_retention(entry)?;
            continue;
        }
        let line_number = object_number(source, "lineNumber")?;
        if line_number == 0 || active_entries.insert(line_number, entry).is_some() {
            return Err(verification_error("invalid active replacement mapping"));
        }
    }

    let mut replacements = 0u64;
    for (index, (active_line, staged_line)) in active_lines.iter().zip(&staged_lines).enumerate() {
        let line_number = index as u64 + 1;
        let entry = active_entries
            .remove(&line_number)
            .ok_or_else(|| verification_error("active line missing from replacement plan"))?;
        verify_active_source(entry, ledger, line_number, active_line)?;
        let active_value = parse_json(active_line)
            .map_err(|error| verification_error(&format!("invalid active JSON: {error}")))?;
        let staged_value = parse_json(staged_line)
            .map_err(|error| verification_error(&format!("invalid staged JSON: {error}")))?;
        if to_canonical_json(&active_value) != to_canonical_json(&staged_value) {
            return Err(verification_error("staged value is not semantically equivalent"));
        }

        match object_string(entry, "decision")?.as_str() {
            "retain-byte-for-byte" => {
                if active_line != staged_line {
                    return Err(verification_error("retained line changed bytes"));
                }
            }
            "canonical-json-representation" => {
                replacements += 1;
                let canonical = to_canonical_json(&active_value);
                if staged_line != &canonical {
                    return Err(verification_error("staged canonical line mismatch"));
                }
                let replacement = object_field(entry, "replacement")?;
                if object_string(replacement, "lineDigest")?
                    != integrity_digest(staged_line.as_bytes())
                {
                    return Err(verification_error("staged replacement digest mismatch"));
                }
            }
            _ => return Err(verification_error("unsupported replacement decision")),
        }
    }
    if !active_entries.is_empty() {
        return Err(verification_error("replacement plan references missing active lines"));
    }
    Ok((active_lines.len() as u64, replacements))
}

fn verify_active_source(
    entry: &JsonValue,
    ledger: &str,
    line_number: u64,
    line: &str,
) -> Result<(), StoreError> {
    if object_string(entry, "ledger")? != ledger {
        return Err(verification_error("replacement entry ledger mismatch"));
    }
    let source = object_field(entry, "source")?;
    if object_number(source, "lineNumber")? != line_number
        || object_string(source, "lineDigest")? != integrity_digest(line.as_bytes())
    {
        return Err(verification_error("active source no longer matches plan"));
    }
    let value = parse_json(line)
        .map_err(|error| verification_error(&format!("invalid active JSON: {error}")))?;
    if object_string(source, "valueDigest")?
        != integrity_digest(to_canonical_json(&value).as_bytes())
    {
        return Err(verification_error("active source value no longer matches plan"));
    }
    Ok(())
}

fn require_archived_retention(entry: &JsonValue) -> Result<(), StoreError> {
    if object_string(entry, "decision")? != "retain-byte-for-byte" {
        return Err(verification_error(
            "archived replacement is forbidden by transaction policy",
        ));
    }
    Ok(())
}

fn jsonl_lines(bytes: &[u8], ledger: &str) -> Result<Vec<String>, StoreError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| verification_error(&format!("invalid UTF-8 in {ledger}: {error}")))?;
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            parse_json(line).map_err(|error| {
                verification_error(&format!(
                    "invalid JSON in {ledger} line {}: {error}",
                    index + 1
                ))
            })?;
            Ok(line.to_string())
        })
        .collect()
}

fn count_jsonl_lines(bytes: &[u8], ledger: &str) -> Result<u64, StoreError> {
    Ok(jsonl_lines(bytes, ledger)?.len() as u64)
}

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(verification_error("expected JSON object")),
    }
}

fn object_field<'a>(value: &'a JsonValue, name: &str) -> Result<&'a JsonValue, StoreError> {
    object_map(value)?
        .get(name)
        .ok_or_else(|| verification_error(&format!("missing field: {name}")))
}

fn object_array<'a>(value: &'a JsonValue, name: &str) -> Result<&'a [JsonValue], StoreError> {
    match object_field(value, name)? {
        JsonValue::Array(values) => Ok(values),
        _ => Err(verification_error(&format!("invalid array field: {name}"))),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(verification_error(&format!("invalid string field: {name}"))),
    }
}

fn object_optional_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(Some(value.clone())),
        JsonValue::Null => Ok(None),
        _ => Err(verification_error(&format!(
            "invalid optional string field: {name}"
        ))),
    }
}

fn object_number(value: &JsonValue, name: &str) -> Result<u64, StoreError> {
    match object_field(value, name)? {
        JsonValue::Number(value) => value
            .parse()
            .map_err(|_| verification_error(&format!("invalid number field: {name}"))),
        _ => Err(verification_error(&format!("invalid number field: {name}"))),
    }
}

fn object_bool(value: &JsonValue, name: &str) -> Result<bool, StoreError> {
    match object_field(value, name)? {
        JsonValue::Bool(value) => Ok(*value),
        _ => Err(verification_error(&format!("invalid boolean field: {name}"))),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(verification_error(&format!("unsupported {name}")));
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
    verification_error(&error.to_string())
}

fn verification_error(message: &str) -> StoreError {
    store_error(
        "LB_QUARANTINE_REPLACEMENT_STAGING",
        message.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_quarantine_replacement_preview, export_complete_quarantine_backup,
        prepare_quarantine_replacement_transaction, stage_quarantine_replacement_ledgers,
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
        let state = temp_dir("replacement-verify-stage-state");
        let backup = temp_dir("replacement-verify-stage-backup");
        let proof = temp_dir("replacement-verify-stage-proof");
        let transaction = temp_dir("replacement-verify-stage-transaction");
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), b"{\"id\":\"q0\"}\n").unwrap();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"c1\", \"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        create_quarantine_replacement_preview(&state, &backup, &proof).unwrap();
        prepare_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-verify-stage-001",
        )
        .unwrap();
        stage_quarantine_replacement_ledgers(&state, &proof, &transaction).unwrap();
        (state, backup, proof, transaction)
    }

    #[test]
    fn verifies_staged_ledgers_and_advances_transaction() {
        let (state, backup, proof, transaction) = fixture();
        let report = verify_quarantine_replacement_staging(&state, &proof, &transaction).unwrap();
        assert_eq!(report.replacement_lines, 1);
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            QuarantineReplacementTransactionState::Verified
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn detects_staged_tampering_and_requires_recovery() {
        let (state, backup, proof, transaction) = fixture();
        fs::write(
            transaction
                .join(QUARANTINE_REPLACEMENT_STAGING_DIR)
                .join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"tampered\",\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        assert_eq!(
            verify_quarantine_replacement_staging(&state, &proof, &transaction)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_STAGING"
        );
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            QuarantineReplacementTransactionState::RecoveryRequired
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn detects_active_change_after_staging() {
        let (state, backup, proof, transaction) = fixture();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"changed\",\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        assert_eq!(
            verify_quarantine_replacement_staging(&state, &proof, &transaction)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_STAGING"
        );
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            QuarantineReplacementTransactionState::RecoveryRequired
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }
}
