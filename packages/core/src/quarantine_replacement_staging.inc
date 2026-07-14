use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    advance_quarantine_replacement_transaction_journal, acquire_quarantine_lock,
    read_quarantine_replacement_transaction_journal, resolve_quarantine_active_path, store_error,
    verify_quarantine_replacement_proof, QuarantineReplacementTransactionState, StoreError,
    QUARANTINE_BACKUP_FILES, QUARANTINE_REPLACEMENT_PLAN_FILE,
};

pub const QUARANTINE_REPLACEMENT_STAGING_DIR: &str = "staged-active-ledgers";
pub const QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE: &str =
    "quarantine-replacement-staging.json";
pub const QUARANTINE_REPLACEMENT_STAGING_VERSION: &str =
    "lingonberry-quarantine-replacement-staging/v1";

const IMMUTABLE_LEDGERS: [&str; 3] = [
    "quarantine.jsonl",
    "quarantine-annotations.jsonl",
    "admin-auth-audit.jsonl",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementStagedLedger {
    pub ledger: String,
    pub present: bool,
    pub bytes: u64,
    pub lines: u64,
    pub digest: Option<String>,
    pub replacements: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementStagingReport {
    pub staging_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub ledgers: Vec<QuarantineReplacementStagedLedger>,
    pub replacement_lines: u64,
}

pub fn stage_quarantine_replacement_ledgers(
    state_dir: impl AsRef<Path>,
    verified_proof_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementStagingReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let proof_dir = verified_proof_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-replacement-stage-v1")?;

    let journal = read_quarantine_replacement_transaction_journal(transaction_dir)?;
    match journal.state {
        QuarantineReplacementTransactionState::Prepared => {
            advance_quarantine_replacement_transaction_journal(
                transaction_dir,
                QuarantineReplacementTransactionState::Writing,
            )?;
        }
        QuarantineReplacementTransactionState::Writing => {}
        QuarantineReplacementTransactionState::RecoveryRequired => {
            advance_quarantine_replacement_transaction_journal(
                transaction_dir,
                QuarantineReplacementTransactionState::Writing,
            )?;
        }
        _ => {
            return Err(staging_error(
                "transaction must be prepared, writing, or recovery-required before staging",
            ))
        }
    }
    verify_quarantine_replacement_proof(proof_dir)?;
    let plan_text = fs::read_to_string(proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_FILE))
        .map_err(io_error)?;
    let plan = parse_json(&plan_text)
        .map_err(|error| staging_error(&format!("invalid replacement plan JSON: {error}")))?;

    let staging_dir = transaction_dir.join(QUARANTINE_REPLACEMENT_STAGING_DIR);
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).map_err(io_error)?;
        sync_directory(transaction_dir)?;
    }

    let result = build_staging_set(state_dir, &staging_dir, &plan);
    let ledgers = match result {
        Ok(ledgers) => ledgers,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            let _ = advance_quarantine_replacement_transaction_journal(
                transaction_dir,
                QuarantineReplacementTransactionState::RecoveryRequired,
            );
            return Err(error);
        }
    };

    let replacement_lines = ledgers.iter().map(|ledger| ledger.replacements).sum();
    let manifest = staging_manifest_json(&journal.transaction_id, &ledgers, replacement_lines);
    let manifest_path = staging_dir.join(QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE);
    write_new_synced(&manifest_path, to_canonical_json(&manifest).as_bytes())?;
    sync_directory(&staging_dir)?;

    advance_quarantine_replacement_transaction_journal(
        transaction_dir,
        QuarantineReplacementTransactionState::Staged,
    )?;

    Ok(QuarantineReplacementStagingReport {
        staging_dir,
        manifest_path,
        ledgers,
        replacement_lines,
    })
}

fn build_staging_set(
    state_dir: &Path,
    staging_dir: &Path,
    plan: &JsonValue,
) -> Result<Vec<QuarantineReplacementStagedLedger>, StoreError> {
    fs::create_dir(staging_dir).map_err(io_error)?;
    sync_directory(staging_dir.parent().ok_or_else(|| staging_error("missing transaction parent"))?)?;

    let plan_ledgers = object_array(plan, "ledgers")?;
    let names = plan_ledgers
        .iter()
        .map(|ledger| object_string(ledger, "ledger"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<BTreeSet<_>>();
    if names != expected || plan_ledgers.len() != QUARANTINE_BACKUP_FILES.len() {
        return Err(staging_error("plan managed-ledger set mismatch"));
    }

    let mut reports = Vec::new();
    for ledger in QUARANTINE_BACKUP_FILES {
        let ledger_plan = plan_ledgers
            .iter()
            .find(|candidate| object_string(candidate, "ledger").ok().as_deref() == Some(ledger))
            .ok_or_else(|| staging_error("missing managed ledger plan"))?;
        reports.push(stage_active_ledger(state_dir, staging_dir, ledger, ledger_plan)?);
    }
    sync_directory(staging_dir)?;
    Ok(reports)
}

fn stage_active_ledger(
    state_dir: &Path,
    staging_dir: &Path,
    ledger: &str,
    ledger_plan: &JsonValue,
) -> Result<QuarantineReplacementStagedLedger, StoreError> {
    let source_path = resolve_quarantine_active_path(state_dir, ledger)?;
    if !source_path.exists() {
        reject_active_entries_for_absent_ledger(ledger_plan)?;
        return Ok(QuarantineReplacementStagedLedger {
            ledger: ledger.to_string(),
            present: false,
            bytes: 0,
            lines: 0,
            digest: None,
            replacements: 0,
        });
    }
    if !source_path.is_file() {
        return Err(staging_error(&format!("active ledger is not a file: {ledger}")));
    }

    let source_bytes = fs::read(&source_path).map_err(io_error)?;
    let (output, lines, replacements) = if IMMUTABLE_LEDGERS.contains(&ledger) {
        require_immutable_plan(ledger_plan)?;
        (source_bytes.clone(), count_jsonl_lines(&source_bytes, ledger)?, 0)
    } else {
        apply_terminal_plan(&source_bytes, ledger, ledger_plan)?
    };

    let target = staging_dir.join(ledger);
    write_new_synced(&target, &output)?;
    Ok(QuarantineReplacementStagedLedger {
        ledger: ledger.to_string(),
        present: true,
        bytes: output.len() as u64,
        lines,
        digest: Some(integrity_digest(&output)),
        replacements,
    })
}

fn reject_active_entries_for_absent_ledger(ledger_plan: &JsonValue) -> Result<(), StoreError> {
    for entry in object_array(ledger_plan, "entries")? {
        let source = object_field(entry, "source")?;
        if object_string(source, "location")? == "active-ledger" {
            return Err(staging_error(
                "plan references active lines for an absent ledger",
            ));
        }
        reject_archived_replacement(entry)?;
    }
    Ok(())
}

fn require_immutable_plan(ledger_plan: &JsonValue) -> Result<(), StoreError> {
    for entry in object_array(ledger_plan, "entries")? {
        if object_string(entry, "decision")? != "retain-byte-for-byte" {
            return Err(staging_error("immutable ledger contains a replacement"));
        }
    }
    Ok(())
}

fn apply_terminal_plan(
    source_bytes: &[u8],
    ledger: &str,
    ledger_plan: &JsonValue,
) -> Result<(Vec<u8>, u64, u64), StoreError> {
    let source_text = std::str::from_utf8(source_bytes)
        .map_err(|error| staging_error(&format!("invalid UTF-8 in {ledger}: {error}")))?;
    let mut active_entries = BTreeMap::<u64, &JsonValue>::new();
    for entry in object_array(ledger_plan, "entries")? {
        let source = object_field(entry, "source")?;
        let location = object_string(source, "location")?;
        if location != "active-ledger" {
            reject_archived_replacement(entry)?;
            continue;
        }
        let line_number = object_number(source, "lineNumber")?;
        if line_number == 0 || active_entries.insert(line_number, entry).is_some() {
            return Err(staging_error("duplicate or invalid active line mapping"));
        }
    }

    let mut output = Vec::new();
    let mut line_count = 0u64;
    let mut replacements = 0u64;
    for (index, line) in source_text.lines().enumerate() {
        let line_number = index as u64 + 1;
        line_count += 1;
        let entry = active_entries
            .remove(&line_number)
            .ok_or_else(|| staging_error("active ledger line is missing from replacement plan"))?;
        verify_source_line(entry, ledger, line_number, line)?;
        let decision = object_string(entry, "decision")?;
        let rendered = match decision.as_str() {
            "retain-byte-for-byte" => line.to_string(),
            "canonical-json-representation" => {
                replacements += 1;
                let value = parse_json(line).map_err(|error| {
                    staging_error(&format!("invalid JSON in {ledger} line {line_number}: {error}"))
                })?;
                let canonical = to_canonical_json(&value);
                let replacement = object_field(entry, "replacement")?;
                if object_string(replacement, "lineDigest")?
                    != integrity_digest(canonical.as_bytes())
                {
                    return Err(staging_error("canonical replacement digest mismatch"));
                }
                canonical
            }
            _ => return Err(staging_error("unsupported replacement decision")),
        };
        output.extend_from_slice(rendered.as_bytes());
        output.push(b'\n');
    }
    if !active_entries.is_empty() {
        return Err(staging_error("plan references missing active ledger lines"));
    }
    Ok((output, line_count, replacements))
}

fn reject_archived_replacement(entry: &JsonValue) -> Result<(), StoreError> {
    if object_string(entry, "decision")? != "retain-byte-for-byte" {
        return Err(staging_error(
            "replacement of archived records is forbidden by transaction policy",
        ));
    }
    Ok(())
}

fn verify_source_line(
    entry: &JsonValue,
    ledger: &str,
    line_number: u64,
    line: &str,
) -> Result<(), StoreError> {
    if object_string(entry, "ledger")? != ledger
        || object_number(entry, "logicalOrdinal")? > u64::MAX - 1
    {
        return Err(staging_error("replacement entry ledger mismatch"));
    }
    let source = object_field(entry, "source")?;
    if object_number(source, "lineNumber")? != line_number
        || object_string(source, "lineDigest")? != integrity_digest(line.as_bytes())
    {
        return Err(staging_error("active source line no longer matches plan"));
    }
    let value = parse_json(line).map_err(|error| {
        staging_error(&format!("invalid JSON in {ledger} line {line_number}: {error}"))
    })?;
    let value_digest = integrity_digest(to_canonical_json(&value).as_bytes());
    if object_string(source, "valueDigest")? != value_digest {
        return Err(staging_error("active source value no longer matches plan"));
    }
    Ok(())
}

fn count_jsonl_lines(bytes: &[u8], ledger: &str) -> Result<u64, StoreError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| staging_error(&format!("invalid UTF-8 in {ledger}: {error}")))?;
    let mut count = 0u64;
    for (index, line) in text.lines().enumerate() {
        parse_json(line).map_err(|error| {
            staging_error(&format!("invalid JSON in {ledger} line {}: {error}", index + 1))
        })?;
        count += 1;
    }
    Ok(count)
}

fn staging_manifest_json(
    transaction_id: &str,
    ledgers: &[QuarantineReplacementStagedLedger],
    replacement_lines: u64,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "ledgers".to_string(),
            JsonValue::Array(
                ledgers
                    .iter()
                    .map(|ledger| {
                        JsonValue::Object(BTreeMap::from([
                            ("bytes".to_string(), JsonValue::Number(ledger.bytes.to_string())),
                            (
                                "digest".to_string(),
                                ledger
                                    .digest
                                    .as_ref()
                                    .map(|digest| JsonValue::String(digest.clone()))
                                    .unwrap_or(JsonValue::Null),
                            ),
                            ("ledger".to_string(), JsonValue::String(ledger.ledger.clone())),
                            ("lines".to_string(), JsonValue::Number(ledger.lines.to_string())),
                            ("present".to_string(), JsonValue::Bool(ledger.present)),
                            (
                                "replacements".to_string(),
                                JsonValue::Number(ledger.replacements.to_string()),
                            ),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "replacementLines".to_string(),
            JsonValue::Number(replacement_lines.to_string()),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(transaction_id.to_string()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_STAGING_VERSION.to_string()),
        ),
    ]))
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

fn object_map(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(staging_error("expected JSON object")),
    }
}

fn object_field<'a>(value: &'a JsonValue, name: &str) -> Result<&'a JsonValue, StoreError> {
    object_map(value)?
        .get(name)
        .ok_or_else(|| staging_error(&format!("missing field: {name}")))
}

fn object_array<'a>(value: &'a JsonValue, name: &str) -> Result<&'a [JsonValue], StoreError> {
    match object_field(value, name)? {
        JsonValue::Array(values) => Ok(values),
        _ => Err(staging_error(&format!("invalid array field: {name}"))),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(staging_error(&format!("invalid string field: {name}"))),
    }
}

fn object_number(value: &JsonValue, name: &str) -> Result<u64, StoreError> {
    match object_field(value, name)? {
        JsonValue::Number(value) => value
            .parse()
            .map_err(|_| staging_error(&format!("invalid number field: {name}"))),
        _ => Err(staging_error(&format!("invalid number field: {name}"))),
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

fn io_error(error: std::io::Error) -> StoreError {
    staging_error(&error.to_string())
}

fn staging_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_STAGING", message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_quarantine_replacement_preview, export_complete_quarantine_backup,
        prepare_quarantine_replacement_transaction,
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
        let state = temp_dir("replacement-stage-state");
        let backup = temp_dir("replacement-stage-backup");
        let proof = temp_dir("replacement-stage-proof");
        let transaction = temp_dir("replacement-stage-transaction");
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
            "tx-stage-001",
        )
        .unwrap();
        (state, backup, proof, transaction)
    }

    #[test]
    fn stages_only_active_ledgers_and_preserves_immutable_bytes() {
        let (state, backup, proof, transaction) = fixture();
        let immutable_before = fs::read(state.join("quarantine.jsonl")).unwrap();
        let active_before = fs::read(state.join("quarantine-resolutions.jsonl")).unwrap();
        let report = stage_quarantine_replacement_ledgers(&state, &proof, &transaction).unwrap();
        assert_eq!(report.replacement_lines, 1);
        assert_eq!(
            fs::read(report.staging_dir.join("quarantine.jsonl")).unwrap(),
            immutable_before
        );
        assert_eq!(
            fs::read_to_string(report.staging_dir.join("quarantine-resolutions.jsonl")).unwrap(),
            "{\"canonicalId\":\"c1\",\"quarantineId\":\"q1\"}\n"
        );
        assert_eq!(
            fs::read(state.join("quarantine-resolutions.jsonl")).unwrap(),
            active_before
        );
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            QuarantineReplacementTransactionState::Staged
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
        let _ = fs::remove_dir_all(transaction);
    }

    #[test]
    fn fails_closed_when_active_source_changes_after_prepare() {
        let (state, backup, proof, transaction) = fixture();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            b"{\"canonicalId\":\"changed\",\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        assert_eq!(
            stage_quarantine_replacement_ledgers(&state, &proof, &transaction)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_STAGING"
        );
        assert!(!transaction
            .join(QUARANTINE_REPLACEMENT_STAGING_DIR)
            .exists());
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