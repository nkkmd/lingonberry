use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{acquire_quarantine_lock, store_error, StoreError, QUARANTINE_BACKUP_FILES};

pub const QUARANTINE_LEDGER_INDEX_VERSION: &str = "lingonberry-quarantine-ledger-index/v1";
pub const QUARANTINE_LEDGER_INDEX_FILE: &str = "quarantine-ledger-index.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineLedgerIndexEntry {
    pub name: String,
    pub present: bool,
    pub bytes: u64,
    pub lines: u64,
    pub first_offset: Option<u64>,
    pub last_offset: Option<u64>,
    pub digest: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineLedgerIndex {
    pub version: String,
    pub generated_at: String,
    pub state_dir: String,
    pub files: Vec<QuarantineLedgerIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineLedgerIndexReport {
    pub index_path: PathBuf,
    pub present_files: usize,
    pub total_bytes: u64,
    pub total_lines: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineLedgerMaintenanceEntry {
    pub name: String,
    pub bytes: u64,
    pub lines: u64,
    pub exceeds_bytes: bool,
    pub exceeds_lines: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineLedgerMaintenancePlan {
    pub byte_threshold: u64,
    pub line_threshold: u64,
    pub requires_maintenance: bool,
    pub destructive_actions_blocked: bool,
    pub entries: Vec<QuarantineLedgerMaintenanceEntry>,
}

pub fn build_quarantine_ledger_index(
    state_dir: impl AsRef<Path>,
) -> Result<QuarantineLedgerIndexReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-ledger-index-build")?;
    let mut files = Vec::new();
    for name in QUARANTINE_BACKUP_FILES {
        let path = state_dir.join(name);
        files.push(scan_ledger(&path, name)?);
    }
    let index = QuarantineLedgerIndex {
        version: QUARANTINE_LEDGER_INDEX_VERSION.to_string(),
        generated_at: timestamp()?,
        state_dir: state_dir.to_string_lossy().to_string(),
        files,
    };
    let index_path = state_dir.join(QUARANTINE_LEDGER_INDEX_FILE);
    let temporary = state_dir.join(format!(".{QUARANTINE_LEDGER_INDEX_FILE}.tmp"));
    fs::write(
        &temporary,
        to_canonical_json(&quarantine_ledger_index_json(&index)),
    )
    .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    fs::rename(&temporary, &index_path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    report(&index, index_path)
}

pub fn verify_quarantine_ledger_index(
    state_dir: impl AsRef<Path>,
) -> Result<QuarantineLedgerIndexReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let index_path = state_dir.join(QUARANTINE_LEDGER_INDEX_FILE);
    let text = fs::read_to_string(&index_path).map_err(|error| {
        store_error(
            "LB_QUARANTINE_INDEX_INVALID",
            format!("failed to read ledger index: {error}"),
        )
    })?;
    let index = parse_index(&text)?;
    validate_index_shape(&index)?;
    for expected in &index.files {
        let actual = scan_ledger(&state_dir.join(&expected.name), &expected.name)?;
        if &actual != expected {
            return Err(store_error(
                "LB_QUARANTINE_INDEX_STALE",
                format!("ledger index does not match source: {}", expected.name),
            ));
        }
    }
    report(&index, index_path)
}

pub fn plan_quarantine_ledger_maintenance(
    state_dir: impl AsRef<Path>,
    byte_threshold: u64,
    line_threshold: u64,
) -> Result<QuarantineLedgerMaintenancePlan, StoreError> {
    if byte_threshold == 0 || line_threshold == 0 {
        return Err(store_error(
            "LB_QUARANTINE_MAINTENANCE",
            "byte and line thresholds must be greater than zero",
        ));
    }
    let state_dir = state_dir.as_ref();
    let report = verify_quarantine_ledger_index(state_dir)?;
    let text = fs::read_to_string(&report.index_path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    let index = parse_index(&text)?;
    let entries = index
        .files
        .into_iter()
        .filter(|entry| entry.present)
        .map(|entry| QuarantineLedgerMaintenanceEntry {
            name: entry.name,
            bytes: entry.bytes,
            lines: entry.lines,
            exceeds_bytes: entry.bytes >= byte_threshold,
            exceeds_lines: entry.lines >= line_threshold,
        })
        .collect::<Vec<_>>();
    let requires_maintenance = entries
        .iter()
        .any(|entry| entry.exceeds_bytes || entry.exceeds_lines);
    Ok(QuarantineLedgerMaintenancePlan {
        byte_threshold,
        line_threshold,
        requires_maintenance,
        destructive_actions_blocked: true,
        entries,
    })
}

pub fn quarantine_ledger_index_report_json(report: &QuarantineLedgerIndexReport) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "indexPath".to_string(),
            JsonValue::String(report.index_path.to_string_lossy().to_string()),
        ),
        (
            "presentFiles".to_string(),
            JsonValue::Number(report.present_files.to_string()),
        ),
        (
            "totalBytes".to_string(),
            JsonValue::Number(report.total_bytes.to_string()),
        ),
        (
            "totalLines".to_string(),
            JsonValue::Number(report.total_lines.to_string()),
        ),
    ]))
}

pub fn quarantine_ledger_maintenance_plan_json(
    plan: &QuarantineLedgerMaintenancePlan,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "byteThreshold".to_string(),
            JsonValue::Number(plan.byte_threshold.to_string()),
        ),
        (
            "destructiveActionsBlocked".to_string(),
            JsonValue::Bool(plan.destructive_actions_blocked),
        ),
        (
            "entries".to_string(),
            JsonValue::Array(
                plan.entries
                    .iter()
                    .map(|entry| {
                        JsonValue::Object(BTreeMap::from([
                            (
                                "bytes".to_string(),
                                JsonValue::Number(entry.bytes.to_string()),
                            ),
                            (
                                "exceedsBytes".to_string(),
                                JsonValue::Bool(entry.exceeds_bytes),
                            ),
                            (
                                "exceedsLines".to_string(),
                                JsonValue::Bool(entry.exceeds_lines),
                            ),
                            (
                                "lines".to_string(),
                                JsonValue::Number(entry.lines.to_string()),
                            ),
                            ("name".to_string(), JsonValue::String(entry.name.clone())),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "lineThreshold".to_string(),
            JsonValue::Number(plan.line_threshold.to_string()),
        ),
        (
            "requiresMaintenance".to_string(),
            JsonValue::Bool(plan.requires_maintenance),
        ),
    ]))
}

fn scan_ledger(path: &Path, name: &str) -> Result<QuarantineLedgerIndexEntry, StoreError> {
    if !path.exists() {
        return Ok(QuarantineLedgerIndexEntry {
            name: name.to_string(),
            present: false,
            bytes: 0,
            lines: 0,
            first_offset: None,
            last_offset: None,
            digest: None,
        });
    }
    if !path.is_file() {
        return Err(store_error(
            "LB_QUARANTINE_INDEX_INVALID",
            format!("ledger path is not a regular file: {}", path.display()),
        ));
    }
    let before =
        fs::read(path).map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    if !before.is_empty() && !before.ends_with(b"\n") {
        return Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            format!("ledger has a partial trailing line: {name}"),
        ));
    }
    let mut offsets = Vec::new();
    let mut start = 0usize;
    for (index, byte) in before.iter().enumerate() {
        if *byte == b'\n' {
            let line = &before[start..index];
            if !line.iter().all(u8::is_ascii_whitespace) {
                let text = std::str::from_utf8(line)
                    .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
                parse_json(text)
                    .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
                offsets.push(start as u64);
            }
            start = index + 1;
        }
    }
    let digest = integrity_digest(&before);
    let after =
        fs::read(path).map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    if before.len() != after.len() || digest != integrity_digest(&after) {
        return Err(store_error(
            "LB_QUARANTINE_INDEX_CHANGED",
            format!("ledger changed during indexing: {name}"),
        ));
    }
    Ok(QuarantineLedgerIndexEntry {
        name: name.to_string(),
        present: true,
        bytes: before.len() as u64,
        lines: offsets.len() as u64,
        first_offset: offsets.first().copied(),
        last_offset: offsets.last().copied(),
        digest: Some(digest),
    })
}

fn quarantine_ledger_index_json(index: &QuarantineLedgerIndex) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "files".to_string(),
            JsonValue::Array(
                index
                    .files
                    .iter()
                    .map(|entry| {
                        JsonValue::Object(BTreeMap::from([
                            (
                                "bytes".to_string(),
                                JsonValue::Number(entry.bytes.to_string()),
                            ),
                            ("digest".to_string(), optional_string(&entry.digest)),
                            (
                                "firstOffset".to_string(),
                                optional_number(entry.first_offset),
                            ),
                            ("lastOffset".to_string(), optional_number(entry.last_offset)),
                            (
                                "lines".to_string(),
                                JsonValue::Number(entry.lines.to_string()),
                            ),
                            ("name".to_string(), JsonValue::String(entry.name.clone())),
                            ("present".to_string(), JsonValue::Bool(entry.present)),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "generatedAt".to_string(),
            JsonValue::String(index.generated_at.clone()),
        ),
        (
            "stateDir".to_string(),
            JsonValue::String(index.state_dir.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(index.version.clone()),
        ),
    ]))
}

fn parse_index(text: &str) -> Result<QuarantineLedgerIndex, StoreError> {
    let map = object(
        parse_json(text)
            .map_err(|error| store_error("LB_QUARANTINE_INDEX_INVALID", error.to_string()))?,
    )?;
    let files = match map.get("files") {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let map = object(value.clone())?;
                Ok(QuarantineLedgerIndexEntry {
                    name: string(&map, "name")?,
                    present: boolean(&map, "present")?,
                    bytes: number(&map, "bytes")?,
                    lines: number(&map, "lines")?,
                    first_offset: optional_u64(&map, "firstOffset")?,
                    last_offset: optional_u64(&map, "lastOffset")?,
                    digest: optional_text(&map, "digest")?,
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?,
        _ => return Err(index_invalid("index missing files")),
    };
    Ok(QuarantineLedgerIndex {
        version: string(&map, "version")?,
        generated_at: string(&map, "generatedAt")?,
        state_dir: string(&map, "stateDir")?,
        files,
    })
}

fn validate_index_shape(index: &QuarantineLedgerIndex) -> Result<(), StoreError> {
    if index.version != QUARANTINE_LEDGER_INDEX_VERSION {
        return Err(index_invalid(&format!(
            "unsupported ledger index version: {}",
            index.version
        )));
    }
    let names = index
        .files
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<BTreeSet<_>>();
    let expected = QUARANTINE_BACKUP_FILES.into_iter().collect::<BTreeSet<_>>();
    if names != expected || index.files.len() != QUARANTINE_BACKUP_FILES.len() {
        return Err(index_invalid(
            "ledger index must contain the exact managed file set",
        ));
    }
    for entry in &index.files {
        if !QUARANTINE_BACKUP_FILES.contains(&entry.name.as_str()) {
            return Err(index_invalid(&format!(
                "unsupported ledger name: {}",
                entry.name
            )));
        }
        if !entry.present
            && (entry.bytes != 0
                || entry.lines != 0
                || entry.first_offset.is_some()
                || entry.last_offset.is_some()
                || entry.digest.is_some())
        {
            return Err(index_invalid(&format!(
                "absent ledger has indexed content: {}",
                entry.name
            )));
        }
        if entry.present != entry.digest.is_some() {
            return Err(index_invalid(&format!(
                "invalid digest presence for {}",
                entry.name
            )));
        }
        if entry.lines == 0 && (entry.first_offset.is_some() || entry.last_offset.is_some()) {
            return Err(index_invalid(&format!(
                "empty ledger has offsets: {}",
                entry.name
            )));
        }
        if entry.lines > 0 && (entry.first_offset.is_none() || entry.last_offset.is_none()) {
            return Err(index_invalid(&format!(
                "non-empty ledger missing offsets: {}",
                entry.name
            )));
        }
    }
    Ok(())
}

fn report(
    index: &QuarantineLedgerIndex,
    index_path: PathBuf,
) -> Result<QuarantineLedgerIndexReport, StoreError> {
    validate_index_shape(index)?;
    Ok(QuarantineLedgerIndexReport {
        index_path,
        present_files: index.files.iter().filter(|entry| entry.present).count(),
        total_bytes: index.files.iter().map(|entry| entry.bytes).sum(),
        total_lines: index.files.iter().map(|entry| entry.lines).sum(),
    })
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn timestamp() -> Result<String, StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    Ok(format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()))
}

fn optional_string(value: &Option<String>) -> JsonValue {
    value
        .as_ref()
        .map(|value| JsonValue::String(value.clone()))
        .unwrap_or(JsonValue::Null)
}

fn optional_number(value: Option<u64>) -> JsonValue {
    value
        .map(|value| JsonValue::Number(value.to_string()))
        .unwrap_or(JsonValue::Null)
}

fn object(value: JsonValue) -> Result<BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(index_invalid("expected JSON object")),
    }
}

fn string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(index_invalid(&format!("missing string field: {name}"))),
    }
}

fn boolean(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<bool, StoreError> {
    match map.get(name) {
        Some(JsonValue::Bool(value)) => Ok(*value),
        _ => Err(index_invalid(&format!("missing boolean field: {name}"))),
    }
}

fn number(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<u64, StoreError> {
    match map.get(name) {
        Some(JsonValue::Number(value)) => value
            .parse()
            .map_err(|_| index_invalid(&format!("invalid number field: {name}"))),
        _ => Err(index_invalid(&format!("missing number field: {name}"))),
    }
}

fn optional_u64(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<Option<u64>, StoreError> {
    match map.get(name) {
        Some(JsonValue::Number(value)) => value
            .parse()
            .map(Some)
            .map_err(|_| index_invalid(&format!("invalid optional number field: {name}"))),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(index_invalid(&format!(
            "invalid optional number field: {name}"
        ))),
    }
}

fn optional_text(
    map: &BTreeMap<String, JsonValue>,
    name: &str,
) -> Result<Option<String>, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        _ => Err(index_invalid(&format!(
            "invalid optional string field: {name}"
        ))),
    }
}

fn index_invalid(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_INDEX_INVALID", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-ledger-index-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn builds_verifies_and_plans_without_mutating_ledgers() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let ledger = dir.join("quarantine.jsonl");
        fs::write(&ledger, "{\"a\":1}\n{\"b\":2}\n").unwrap();
        let original = fs::read(&ledger).unwrap();
        let report = build_quarantine_ledger_index(&dir).unwrap();
        assert_eq!(report.present_files, 1);
        assert_eq!(report.total_lines, 2);
        assert_eq!(verify_quarantine_ledger_index(&dir).unwrap(), report);
        let plan = plan_quarantine_ledger_maintenance(&dir, 1, 2).unwrap();
        assert!(plan.requires_maintenance);
        assert!(plan.destructive_actions_blocked);
        assert_eq!(fs::read(&ledger).unwrap(), original);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_partial_and_malformed_lines() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("quarantine.jsonl"), "{\"a\":1}").unwrap();
        assert_eq!(
            build_quarantine_ledger_index(&dir).unwrap_err().code,
            "LB_QUARANTINE_CORRUPT"
        );
        fs::write(dir.join("quarantine.jsonl"), "not-json\n").unwrap();
        assert_eq!(
            build_quarantine_ledger_index(&dir).unwrap_err().code,
            "LB_QUARANTINE_CORRUPT"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_stale_index_and_respects_operation_lock() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("quarantine.jsonl"), "{\"a\":1}\n").unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        fs::write(dir.join("quarantine.jsonl"), "{\"a\":1}\n{\"b\":2}\n").unwrap();
        assert_eq!(
            verify_quarantine_ledger_index(&dir).unwrap_err().code,
            "LB_QUARANTINE_INDEX_STALE"
        );
        let _guard = acquire_quarantine_lock(&dir, "test-holder").unwrap();
        assert_eq!(
            build_quarantine_ledger_index(&dir).unwrap_err().code,
            "LB_QUARANTINE_BUSY"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
