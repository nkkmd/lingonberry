use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    acquire_quarantine_lock, store_error, verify_quarantine_ledger_index, StoreError,
    QUARANTINE_BACKUP_FILES,
};

pub const QUARANTINE_SEGMENT_MANIFEST_VERSION: &str =
    "lingonberry-quarantine-segments/v1";
pub const QUARANTINE_SEGMENT_MANIFEST_FILE: &str = "quarantine-segments.json";
pub const QUARANTINE_SEGMENT_ARCHIVE_DIR: &str = "quarantine-segments";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineLedgerSegment {
    pub ledger: String,
    pub sequence: u64,
    pub file: String,
    pub created_at: String,
    pub bytes: u64,
    pub lines: u64,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineSegmentManifest {
    pub version: String,
    pub updated_at: String,
    pub segments: Vec<QuarantineLedgerSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineSegmentReport {
    pub manifest_path: PathBuf,
    pub segments: usize,
    pub archived_bytes: u64,
    pub archived_lines: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineRotationReport {
    pub ledger: String,
    pub segment_file: String,
    pub sequence: u64,
    pub bytes: u64,
    pub lines: u64,
    pub logical_lines_before: u64,
    pub logical_lines_after: u64,
    pub semantic_digest: String,
}

pub fn read_managed_ledger_lines(
    state_dir: impl AsRef<Path>,
    ledger: &str,
) -> Result<Vec<String>, StoreError> {
    validate_ledger_name(ledger)?;
    let state_dir = state_dir.as_ref();
    let manifest = load_manifest(state_dir)?;
    verify_manifest_and_segments(state_dir, &manifest)?;
    let mut lines = Vec::new();
    for segment in manifest.segments.iter().filter(|segment| segment.ledger == ledger) {
        let path = state_dir
            .join(QUARANTINE_SEGMENT_ARCHIVE_DIR)
            .join(&segment.file);
        lines.extend(read_valid_jsonl_lines(&path, &segment.file)?);
    }
    lines.extend(read_valid_jsonl_lines(&state_dir.join(ledger), ledger)?);
    Ok(lines)
}

pub fn verify_quarantine_segments(
    state_dir: impl AsRef<Path>,
) -> Result<QuarantineSegmentReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let manifest = load_manifest(state_dir)?;
    verify_manifest_and_segments(state_dir, &manifest)?;
    Ok(QuarantineSegmentReport {
        manifest_path: state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE),
        segments: manifest.segments.len(),
        archived_bytes: manifest.segments.iter().map(|segment| segment.bytes).sum(),
        archived_lines: manifest.segments.iter().map(|segment| segment.lines).sum(),
    })
}

pub fn rotate_quarantine_ledger(
    state_dir: impl AsRef<Path>,
    ledger: &str,
) -> Result<QuarantineRotationReport, StoreError> {
    validate_ledger_name(ledger)?;
    let state_dir = state_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "quarantine-ledger-rotate")?;
    verify_quarantine_ledger_index(state_dir)?;

    let active_path = state_dir.join(ledger);
    if !active_path.exists() {
        return Err(store_error(
            "LB_QUARANTINE_ROTATION_EMPTY",
            format!("active ledger does not exist: {ledger}"),
        ));
    }
    let active_bytes = fs::read(&active_path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    if active_bytes.is_empty() {
        return Err(store_error(
            "LB_QUARANTINE_ROTATION_EMPTY",
            format!("active ledger is empty: {ledger}"),
        ));
    }
    let active_lines = validate_jsonl_bytes(&active_bytes, ledger)?;
    let before_lines = read_managed_ledger_lines(state_dir, ledger)?;
    let before_digest = logical_stream_digest(&before_lines);

    let mut manifest = load_manifest(state_dir)?;
    verify_manifest_and_segments(state_dir, &manifest)?;
    let sequence = manifest
        .segments
        .iter()
        .filter(|segment| segment.ledger == ledger)
        .map(|segment| segment.sequence)
        .max()
        .unwrap_or(0)
        + 1;
    let stem = ledger.strip_suffix(".jsonl").unwrap_or(ledger);
    let segment_file = format!("{stem}.{sequence:020}.jsonl");
    let archive_dir = state_dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR);
    fs::create_dir_all(&archive_dir)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    let segment_path = archive_dir.join(&segment_file);
    if segment_path.exists() {
        return Err(store_error(
            "LB_QUARANTINE_ROTATION_CONFLICT",
            format!("archive segment already exists: {segment_file}"),
        ));
    }

    let old_manifest_text = manifest_text_if_present(state_dir)?;
    let temporary_segment = archive_dir.join(format!(".{segment_file}.tmp"));
    fs::write(&temporary_segment, &active_bytes)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    fs::rename(&temporary_segment, &segment_path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;

    let segment = QuarantineLedgerSegment {
        ledger: ledger.to_string(),
        sequence,
        file: segment_file.clone(),
        created_at: timestamp()?,
        bytes: active_bytes.len() as u64,
        lines: active_lines,
        digest: integrity_digest(&active_bytes),
    };
    manifest.updated_at = timestamp()?;
    manifest.segments.push(segment.clone());
    manifest
        .segments
        .sort_by(|left, right| (&left.ledger, left.sequence).cmp(&(&right.ledger, right.sequence)));

    let temporary_active = state_dir.join(format!(".{ledger}.rotate-empty"));
    fs::write(&temporary_active, b"")
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    fs::rename(&temporary_active, &active_path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;

    if let Err(error) = write_manifest(state_dir, &manifest) {
        rollback_rotation(
            state_dir,
            ledger,
            &segment_path,
            &active_bytes,
            old_manifest_text.as_deref(),
        );
        return Err(error);
    }

    let after_lines = match read_managed_ledger_lines(state_dir, ledger) {
        Ok(lines) => lines,
        Err(error) => {
            rollback_rotation(
                state_dir,
                ledger,
                &segment_path,
                &active_bytes,
                old_manifest_text.as_deref(),
            );
            return Err(error);
        }
    };
    let after_digest = logical_stream_digest(&after_lines);
    if before_lines.len() != after_lines.len() || before_digest != after_digest {
        rollback_rotation(
            state_dir,
            ledger,
            &segment_path,
            &active_bytes,
            old_manifest_text.as_deref(),
        );
        return Err(store_error(
            "LB_QUARANTINE_ROTATION_EQUIVALENCE",
            format!("logical ledger stream changed during rotation: {ledger}"),
        ));
    }

    Ok(QuarantineRotationReport {
        ledger: ledger.to_string(),
        segment_file,
        sequence,
        bytes: segment.bytes,
        lines: segment.lines,
        logical_lines_before: before_lines.len() as u64,
        logical_lines_after: after_lines.len() as u64,
        semantic_digest: before_digest,
    })
}

pub fn quarantine_segment_report_json(report: &QuarantineSegmentReport) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "archivedBytes".to_string(),
            JsonValue::Number(report.archived_bytes.to_string()),
        ),
        (
            "archivedLines".to_string(),
            JsonValue::Number(report.archived_lines.to_string()),
        ),
        (
            "manifestPath".to_string(),
            JsonValue::String(report.manifest_path.to_string_lossy().to_string()),
        ),
        (
            "segments".to_string(),
            JsonValue::Number(report.segments.to_string()),
        ),
    ]))
}

pub fn quarantine_rotation_report_json(report: &QuarantineRotationReport) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "bytes".to_string(),
            JsonValue::Number(report.bytes.to_string()),
        ),
        ("ledger".to_string(), JsonValue::String(report.ledger.clone())),
        (
            "lines".to_string(),
            JsonValue::Number(report.lines.to_string()),
        ),
        (
            "logicalLinesAfter".to_string(),
            JsonValue::Number(report.logical_lines_after.to_string()),
        ),
        (
            "logicalLinesBefore".to_string(),
            JsonValue::Number(report.logical_lines_before.to_string()),
        ),
        (
            "segmentFile".to_string(),
            JsonValue::String(report.segment_file.clone()),
        ),
        (
            "semanticDigest".to_string(),
            JsonValue::String(report.semantic_digest.clone()),
        ),
        (
            "sequence".to_string(),
            JsonValue::Number(report.sequence.to_string()),
        ),
    ]))
}

fn load_manifest(state_dir: &Path) -> Result<QuarantineSegmentManifest, StoreError> {
    let path = state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE);
    if !path.exists() {
        return Ok(QuarantineSegmentManifest {
            version: QUARANTINE_SEGMENT_MANIFEST_VERSION.to_string(),
            updated_at: timestamp()?,
            segments: Vec::new(),
        });
    }
    let text = fs::read_to_string(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    parse_manifest(&text)
}

fn write_manifest(
    state_dir: &Path,
    manifest: &QuarantineSegmentManifest,
) -> Result<(), StoreError> {
    let path = state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE);
    let temporary = state_dir.join(format!(".{QUARANTINE_SEGMENT_MANIFEST_FILE}.tmp"));
    fs::write(&temporary, to_canonical_json(&manifest_json(manifest)))
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    fs::rename(&temporary, path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
}

fn verify_manifest_and_segments(
    state_dir: &Path,
    manifest: &QuarantineSegmentManifest,
) -> Result<(), StoreError> {
    if manifest.version != QUARANTINE_SEGMENT_MANIFEST_VERSION {
        return Err(segment_corrupt(&format!(
            "unsupported segment manifest version: {}",
            manifest.version
        )));
    }
    let mut identities = BTreeSet::new();
    let mut listed_files = BTreeSet::new();
    let mut last_sequence = BTreeMap::<String, u64>::new();
    for segment in &manifest.segments {
        validate_ledger_name(&segment.ledger)?;
        validate_segment_file(&segment.file)?;
        if !identities.insert((segment.ledger.clone(), segment.sequence)) {
            return Err(segment_corrupt("duplicate ledger segment sequence"));
        }
        if !listed_files.insert(segment.file.clone()) {
            return Err(segment_corrupt("duplicate archive segment file"));
        }
        if let Some(previous) = last_sequence.insert(segment.ledger.clone(), segment.sequence) {
            if segment.sequence <= previous {
                return Err(segment_corrupt("segment sequences are not strictly ordered"));
            }
        }
        let path = state_dir
            .join(QUARANTINE_SEGMENT_ARCHIVE_DIR)
            .join(&segment.file);
        let bytes = fs::read(&path).map_err(|error| {
            segment_corrupt(&format!("failed to read archive segment {}: {error}", segment.file))
        })?;
        let lines = validate_jsonl_bytes(&bytes, &segment.file)?;
        if bytes.len() as u64 != segment.bytes
            || lines != segment.lines
            || integrity_digest(&bytes) != segment.digest
        {
            return Err(segment_corrupt(&format!(
                "archive segment metadata mismatch: {}",
                segment.file
            )));
        }
    }
    let archive_dir = state_dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR);
    if archive_dir.exists() {
        for entry in fs::read_dir(&archive_dir)
            .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?
        {
            let entry = entry
                .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if !listed_files.contains(&name) {
                return Err(segment_corrupt(&format!(
                    "unlisted archive segment: {name}"
                )));
            }
        }
    }
    Ok(())
}

fn read_valid_jsonl_lines(path: &Path, label: &str) -> Result<Vec<String>, StoreError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    validate_jsonl_bytes(&bytes, label)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
    Ok(text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(ToString::to_string)
        .collect())
}

fn validate_jsonl_bytes(bytes: &[u8], label: &str) -> Result<u64, StoreError> {
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        return Err(store_error(
            "LB_QUARANTINE_CORRUPT",
            format!("partial trailing JSONL line: {label}"),
        ));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
    let mut lines = 0u64;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        parse_json(line)
            .map_err(|error| store_error("LB_QUARANTINE_CORRUPT", error.to_string()))?;
        lines += 1;
    }
    Ok(lines)
}

fn rollback_rotation(
    state_dir: &Path,
    ledger: &str,
    segment_path: &Path,
    active_bytes: &[u8],
    old_manifest_text: Option<&str>,
) {
    let _ = fs::write(state_dir.join(ledger), active_bytes);
    let _ = fs::remove_file(segment_path);
    let manifest_path = state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE);
    match old_manifest_text {
        Some(text) => {
            let _ = fs::write(manifest_path, text);
        }
        None => {
            let _ = fs::remove_file(manifest_path);
        }
    }
}

fn manifest_text_if_present(state_dir: &Path) -> Result<Option<String>, StoreError> {
    let path = state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE);
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(path)
        .map(Some)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
}

fn validate_ledger_name(name: &str) -> Result<(), StoreError> {
    if !QUARANTINE_BACKUP_FILES.contains(&name) {
        return Err(store_error(
            "LB_QUARANTINE_SEGMENT_INVALID",
            format!("unsupported managed ledger: {name}"),
        ));
    }
    Ok(())
}

fn validate_segment_file(name: &str) -> Result<(), StoreError> {
    let path = Path::new(name);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
        || !name.ends_with(".jsonl")
    {
        return Err(segment_corrupt(&format!("invalid segment file name: {name}")));
    }
    Ok(())
}

fn logical_stream_digest(lines: &[String]) -> String {
    let mut bytes = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    integrity_digest(&bytes)
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

fn manifest_json(manifest: &QuarantineSegmentManifest) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "segments".to_string(),
            JsonValue::Array(
                manifest
                    .segments
                    .iter()
                    .map(|segment| {
                        JsonValue::Object(BTreeMap::from([
                            (
                                "bytes".to_string(),
                                JsonValue::Number(segment.bytes.to_string()),
                            ),
                            (
                                "createdAt".to_string(),
                                JsonValue::String(segment.created_at.clone()),
                            ),
                            (
                                "digest".to_string(),
                                JsonValue::String(segment.digest.clone()),
                            ),
                            ("file".to_string(), JsonValue::String(segment.file.clone())),
                            (
                                "ledger".to_string(),
                                JsonValue::String(segment.ledger.clone()),
                            ),
                            (
                                "lines".to_string(),
                                JsonValue::Number(segment.lines.to_string()),
                            ),
                            (
                                "sequence".to_string(),
                                JsonValue::Number(segment.sequence.to_string()),
                            ),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "updatedAt".to_string(),
            JsonValue::String(manifest.updated_at.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(manifest.version.clone()),
        ),
    ]))
}

fn parse_manifest(text: &str) -> Result<QuarantineSegmentManifest, StoreError> {
    let map = object(parse_json(text).map_err(|error| segment_corrupt(&error.to_string()))?)?;
    let segments = match map.get("segments") {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let map = object(value.clone())?;
                Ok(QuarantineLedgerSegment {
                    ledger: string(&map, "ledger")?,
                    sequence: number(&map, "sequence")?,
                    file: string(&map, "file")?,
                    created_at: string(&map, "createdAt")?,
                    bytes: number(&map, "bytes")?,
                    lines: number(&map, "lines")?,
                    digest: string(&map, "digest")?,
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?,
        _ => return Err(segment_corrupt("segment manifest missing segments")),
    };
    Ok(QuarantineSegmentManifest {
        version: string(&map, "version")?,
        updated_at: string(&map, "updatedAt")?,
        segments,
    })
}

fn object(value: JsonValue) -> Result<BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(segment_corrupt("expected JSON object")),
    }
}

fn string(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, StoreError> {
    match map.get(name) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        _ => Err(segment_corrupt(&format!("missing string field: {name}"))),
    }
}

fn number(map: &BTreeMap<String, JsonValue>, name: &str) -> Result<u64, StoreError> {
    match map.get(name) {
        Some(JsonValue::Number(value)) => value
            .parse()
            .map_err(|_| segment_corrupt(&format!("invalid number field: {name}"))),
        _ => Err(segment_corrupt(&format!("missing number field: {name}"))),
    }
}

fn segment_corrupt(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_SEGMENT_CORRUPT", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_quarantine_ledger_index;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-segments-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn rotates_and_preserves_ordered_logical_stream() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let ledger = dir.join("quarantine.jsonl");
        fs::write(&ledger, "{\"n\":1}\n{\"n\":2}\n").unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        let report = rotate_quarantine_ledger(&dir, "quarantine.jsonl").unwrap();
        assert_eq!(report.logical_lines_before, 2);
        assert_eq!(report.logical_lines_after, 2);
        assert!(fs::read(&ledger).unwrap().is_empty());
        fs::write(&ledger, "{\"n\":3}\n").unwrap();
        assert_eq!(
            read_managed_ledger_lines(&dir, "quarantine.jsonl").unwrap(),
            vec!["{\"n\":1}", "{\"n\":2}", "{\"n\":3}"]
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn supports_repeated_rotation_and_detects_tampering() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let ledger = dir.join("quarantine.jsonl");
        fs::write(&ledger, "{\"n\":1}\n").unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        rotate_quarantine_ledger(&dir, "quarantine.jsonl").unwrap();
        fs::write(&ledger, "{\"n\":2}\n").unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        rotate_quarantine_ledger(&dir, "quarantine.jsonl").unwrap();
        assert_eq!(verify_quarantine_segments(&dir).unwrap().segments, 2);
        let archive = dir
            .join(QUARANTINE_SEGMENT_ARCHIVE_DIR)
            .join("quarantine.00000000000000000001.jsonl");
        fs::write(archive, "{\"tampered\":true}\n").unwrap();
        assert_eq!(
            verify_quarantine_segments(&dir).unwrap_err().code,
            "LB_QUARANTINE_SEGMENT_CORRUPT"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn refuses_stale_index_and_unlisted_segment() {
        let dir = temp_dir();
        fs::create_dir_all(dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR)).unwrap();
        fs::write(dir.join("quarantine.jsonl"), "{\"n\":1}\n").unwrap();
        build_quarantine_ledger_index(&dir).unwrap();
        fs::write(dir.join("quarantine.jsonl"), "{\"n\":2}\n").unwrap();
        assert_eq!(
            rotate_quarantine_ledger(&dir, "quarantine.jsonl")
                .unwrap_err()
                .code,
            "LB_QUARANTINE_INDEX_STALE"
        );
        fs::write(
            dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR)
                .join("orphan.jsonl"),
            "{\"n\":0}\n",
        )
        .unwrap();
        assert_eq!(
            verify_quarantine_segments(&dir).unwrap_err().code,
            "LB_QUARANTINE_SEGMENT_CORRUPT"
        );
        let _ = fs::remove_dir_all(dir);
    }
}