use lingonberry_protocol::{
    build_capability_manifest, finalize_knowledge_object, parse_json, to_canonical_json,
    FinalizedKnowledgeObject, JsonValue, ARCHIVE_VERSION, CAPABILITY_VERSION, CARRIER_KIND_ARCHIVE,
    DEFAULT_ACCESS_SCOPE, DEFAULT_RETENTION_HINT, HTTP_PUBLISH_REQUEST_SCHEMA_VERSION,
    KNOWLEDGE_OBJECT_SCHEMA_VERSION, PROTOCOL_VERSION,
};
use lingonberry_validation::{
    evaluate_acceptance, finalize_knowledge_object_full, validate_knowledge_object_full,
    AcceptanceDecision, AcceptancePolicy,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod quarantine;
mod sqlite;
pub use quarantine::{quarantine_record_json, QuarantineRecord, QuarantineStore};
pub use sqlite::SqliteStorageBackend;

#[derive(Debug, Clone)]
pub struct StorePaths {
    pub state_dir: PathBuf,
    pub raw_log_path: PathBuf,
    pub catalog_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct StoredCatalogRecord {
    pub stored_at: String,
    pub canonical_id: String,
    pub carrier_identity: String,
    pub object: JsonValue,
}

#[derive(Debug, Clone)]
pub struct StoredReplayRecord {
    pub stored_at: String,
    pub canonical_id: String,
    pub carrier_identity: String,
    pub object: JsonValue,
}

#[derive(Debug, Clone)]
pub struct RawRequestRecord {
    pub stored_at: String,
    pub canonical_id: String,
    pub carrier_identity: String,
    pub request_json: String,
}

#[derive(Debug, Clone)]
pub struct AppendOutcome {
    pub stored_at: Option<String>,
    pub canonical_id: String,
    pub carrier_identity: String,
    pub object: JsonValue,
    pub duplicate: bool,
}

#[derive(Debug)]
pub struct StoreError {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for StoreError {}

pub trait StorageBackend {
    fn append_publish_request(
        &self,
        request_json: &str,
        finalized: &FinalizedKnowledgeObject,
    ) -> Result<AppendOutcome, StoreError>;
    fn get(&self, canonical_id: &str) -> Result<Option<StoredCatalogRecord>, StoreError>;
    fn get_raw_request(&self, canonical_id: &str) -> Result<Option<RawRequestRecord>, StoreError>;
    fn list_ids(&self) -> Result<Vec<String>, StoreError>;
    fn subscribe(&self, object_type: Option<&str>) -> Result<Vec<StoredCatalogRecord>, StoreError>;
    fn replay(&self) -> Result<Vec<StoredReplayRecord>, StoreError>;
}

#[derive(Debug, Clone)]
pub struct ArchiveExportReport {
    pub archive_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub wire_log_path: PathBuf,
    pub catalog_path: PathBuf,
    pub record_count: usize,
}

#[derive(Debug, Clone)]
pub struct ArchiveImportReport {
    pub archive_dir: PathBuf,
    pub record_count: usize,
    pub duplicate_count: usize,
}

#[derive(Debug, Clone)]
pub struct FileStorageBackend {
    paths: StorePaths,
}

impl FileStorageBackend {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let state_dir = base_dir.as_ref().to_path_buf();
        let raw_log_path = state_dir.join("relay-wire-log.jsonl");
        let catalog_path = state_dir.join("canonical-catalog.jsonl");
        Self {
            paths: StorePaths {
                state_dir,
                raw_log_path,
                catalog_path,
            },
        }
    }

    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }
}

impl StorageBackend for FileStorageBackend {
    fn append_publish_request(
        &self,
        request_json: &str,
        finalized: &FinalizedKnowledgeObject,
    ) -> Result<AppendOutcome, StoreError> {
        append_publish_request(&self.paths, request_json, finalized)
    }

    fn get(&self, canonical_id: &str) -> Result<Option<StoredCatalogRecord>, StoreError> {
        get_record(&self.paths, canonical_id)
    }

    fn get_raw_request(&self, canonical_id: &str) -> Result<Option<RawRequestRecord>, StoreError> {
        get_raw_request(&self.paths, canonical_id)
    }

    fn list_ids(&self) -> Result<Vec<String>, StoreError> {
        list_ids(&self.paths)
    }

    fn subscribe(&self, object_type: Option<&str>) -> Result<Vec<StoredCatalogRecord>, StoreError> {
        subscribe(&self.paths, object_type)
    }

    fn replay(&self) -> Result<Vec<StoredReplayRecord>, StoreError> {
        replay(&self.paths)
    }
}

pub fn default_state_dir() -> PathBuf {
    PathBuf::from(".lingonberry")
}

pub fn runtime_state_dir() -> PathBuf {
    env::var_os("LINGONBERRY_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(default_state_dir)
}

pub fn build_runtime_storage_backend() -> SqliteStorageBackend {
    SqliteStorageBackend::new(runtime_state_dir())
}

pub fn build_runtime_capability_manifest() -> JsonValue {
    build_capability_manifest(
        lingonberry_protocol::CARRIER_KIND_RELAY,
        DEFAULT_ACCESS_SCOPE,
        DEFAULT_RETENTION_HINT,
    )
}

pub fn export_archive(
    backend: &impl StorageBackend,
    archive_dir: impl AsRef<Path>,
) -> Result<ArchiveExportReport, StoreError> {
    let archive_dir = archive_dir.as_ref().to_path_buf();
    fs::create_dir_all(&archive_dir)
        .map_err(|error| store_error("LB_IO_ERROR", error.to_string()))?;
    let manifest_path = archive_dir.join("manifest.json");
    let wire_log_path = archive_dir.join("wire-log.jsonl");
    let catalog_path = archive_dir.join("canonical-catalog.jsonl");

    let ids = backend.list_ids()?;
    let mut wire_log_lines = Vec::new();
    let mut catalog_lines = Vec::new();

    for canonical_id in &ids {
        let raw_request = backend.get_raw_request(canonical_id)?.ok_or_else(|| {
            store_error(
                "LB_ARCHIVE_EXPORT",
                format!("raw request not found for {}", canonical_id),
            )
        })?;
        wire_log_lines.push(json_object(vec![
            ("storedAt", JsonValue::String(raw_request.stored_at)),
            ("canonicalId", JsonValue::String(raw_request.canonical_id)),
            (
                "carrierIdentity",
                JsonValue::String(raw_request.carrier_identity),
            ),
            ("requestJson", JsonValue::String(raw_request.request_json)),
        ]));

        let record = backend.get(canonical_id)?.ok_or_else(|| {
            store_error(
                "LB_ARCHIVE_EXPORT",
                format!("catalog record not found for {}", canonical_id),
            )
        })?;
        catalog_lines.push(json_object(vec![
            ("storedAt", JsonValue::String(record.stored_at)),
            ("canonicalId", JsonValue::String(record.canonical_id)),
            (
                "carrierIdentity",
                JsonValue::String(record.carrier_identity),
            ),
            ("object", record.object),
        ]));
    }

    write_jsonl(&wire_log_path, &wire_log_lines)?;
    write_jsonl(&catalog_path, &catalog_lines)?;

    let manifest = archive_manifest(ids.len());
    write_text_file(
        &manifest_path,
        &format!("{}\n", to_canonical_json(&manifest)),
    )?;

    Ok(ArchiveExportReport {
        archive_dir,
        manifest_path,
        wire_log_path,
        catalog_path,
        record_count: ids.len(),
    })
}

pub fn import_archive(
    backend: &impl StorageBackend,
    archive_dir: impl AsRef<Path>,
) -> Result<ArchiveImportReport, StoreError> {
    let archive_dir = archive_dir.as_ref().to_path_buf();
    let manifest_path = archive_dir.join("manifest.json");
    let wire_log_path = archive_dir.join("wire-log.jsonl");
    let manifest_raw = fs::read_to_string(&manifest_path)
        .map_err(|error| store_error("LB_IO_ERROR", error.to_string()))?;
    let manifest_value = parse_json(&manifest_raw)
        .map_err(|error| store_error("LB_ARCHIVE_IMPORT", error.to_string()))?;
    validate_archive_manifest(&manifest_value)?;

    let lines = read_lines(&wire_log_path)?;
    let mut imported = 0usize;
    let mut duplicates = 0usize;
    for line in lines {
        let value = parse_json(&line)
            .map_err(|error| store_error("LB_ARCHIVE_IMPORT", error.to_string()))?;
        let Some(map) = as_object(&value) else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "wire-log record must be an object",
            ));
        };
        let Some(request_json) = map.get("requestJson").and_then(as_string) else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "wire-log record missing requestJson",
            ));
        };
        let request_value = parse_json(request_json)
            .map_err(|error| store_error("LB_ARCHIVE_IMPORT", error.to_string()))?;
        let Some(request_map) = as_object(&request_value) else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "requestJson is not a publish request",
            ));
        };
        let Some(object_value) = request_map.get("object") else {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "publish request missing object",
            ));
        };
        let report = validate_knowledge_object_full(object_value);
        let policy = AcceptancePolicy::from_env()
            .map_err(|error| store_error("LB_ACCEPTANCE_POLICY", error))?;
        match evaluate_acceptance(&report, &policy) {
            AcceptanceDecision::Accept => {}
            AcceptanceDecision::Reject { code, errors } => {
                return Err(store_error(code, errors.join("; ")))
            }
            AcceptanceDecision::Defer { code, errors } => {
                let record = QuarantineStore::new(runtime_state_dir()).append(
                    request_json,
                    code,
                    &errors,
                )?;
                return Err(store_error(
                    code,
                    format!("{}; quarantineId={}", errors.join("; "), record.id),
                ));
            }
        }
        let finalized = finalize_knowledge_object_full(object_value).map_err(|report| {
            store_error("LB_ARCHIVE_IMPORT", report.combined_errors().join("; "))
        })?;
        let outcome = backend.append_publish_request(request_json, &finalized)?;
        if outcome.duplicate {
            duplicates += 1;
        } else {
            imported += 1;
        }
    }

    Ok(ArchiveImportReport {
        archive_dir,
        record_count: imported,
        duplicate_count: duplicates,
    })
}

fn archive_manifest(record_count: usize) -> JsonValue {
    json_object(vec![
        (
            "archiveVersion",
            JsonValue::String(ARCHIVE_VERSION.to_string()),
        ),
        (
            "capabilityVersion",
            JsonValue::String(CAPABILITY_VERSION.to_string()),
        ),
        (
            "protocolVersion",
            JsonValue::String(PROTOCOL_VERSION.to_string()),
        ),
        (
            "carrierKind",
            JsonValue::String(CARRIER_KIND_ARCHIVE.to_string()),
        ),
        ("createdAt", JsonValue::String(now_utc_rfc3339())),
        ("itemCount", JsonValue::Number(record_count.to_string())),
        (
            "schemaVersions",
            json_object(vec![
                (
                    "knowledgeObject",
                    JsonValue::String(KNOWLEDGE_OBJECT_SCHEMA_VERSION.to_string()),
                ),
                (
                    "httpPublishRequest",
                    JsonValue::String(HTTP_PUBLISH_REQUEST_SCHEMA_VERSION.to_string()),
                ),
            ]),
        ),
        (
            "policy",
            json_object(vec![
                (
                    "defaultAccess",
                    JsonValue::String(DEFAULT_ACCESS_SCOPE.to_string()),
                ),
                (
                    "defaultRetention",
                    JsonValue::String(DEFAULT_RETENTION_HINT.to_string()),
                ),
                ("privateEnabled", JsonValue::Bool(false)),
                (
                    "scrubMode",
                    JsonValue::String("operator-controlled".to_string()),
                ),
            ]),
        ),
        (
            "paths",
            json_object(vec![
                ("manifest", JsonValue::String("manifest.json".to_string())),
                ("wireLog", JsonValue::String("wire-log.jsonl".to_string())),
                (
                    "catalog",
                    JsonValue::String("canonical-catalog.jsonl".to_string()),
                ),
            ]),
        ),
    ])
}

fn validate_archive_manifest(value: &JsonValue) -> Result<(), StoreError> {
    let Some(map) = as_object(value) else {
        return Err(store_error(
            "LB_ARCHIVE_IMPORT",
            "archive manifest must be an object",
        ));
    };
    match map.get("archiveVersion") {
        Some(JsonValue::String(value)) if value == ARCHIVE_VERSION => {}
        Some(JsonValue::String(_)) => {
            return Err(store_error("LB_ARCHIVE_IMPORT", "archiveVersion mismatch"))
        }
        _ => {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "archive manifest missing archiveVersion",
            ))
        }
    }
    match map.get("protocolVersion") {
        Some(JsonValue::String(value)) if value == PROTOCOL_VERSION => {}
        Some(JsonValue::String(_)) => {
            return Err(store_error("LB_ARCHIVE_IMPORT", "protocolVersion mismatch"))
        }
        _ => {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "archive manifest missing protocolVersion",
            ))
        }
    }
    match map.get("carrierKind") {
        Some(JsonValue::String(value)) if value == CARRIER_KIND_ARCHIVE => {}
        Some(JsonValue::String(_)) => {
            return Err(store_error("LB_ARCHIVE_IMPORT", "carrierKind mismatch"))
        }
        _ => {
            return Err(store_error(
                "LB_ARCHIVE_IMPORT",
                "archive manifest missing carrierKind",
            ))
        }
    }
    Ok(())
}

fn write_text_file(path: &Path, contents: &str) -> Result<(), StoreError> {
    ensure_parent(path)?;
    fs::write(path, contents).map_err(|error| store_error("LB_IO_ERROR", error.to_string()))
}

fn write_jsonl(path: &Path, lines: &[JsonValue]) -> Result<(), StoreError> {
    ensure_parent(path)?;
    let mut contents = String::new();
    for line in lines {
        contents.push_str(&to_canonical_json(line));
        contents.push('\n');
    }
    fs::write(path, contents).map_err(|error| store_error("LB_IO_ERROR", error.to_string()))
}

pub fn get_store_paths(base_dir: impl AsRef<Path>) -> StorePaths {
    let state_dir = base_dir.as_ref().to_path_buf();
    StorePaths {
        raw_log_path: state_dir.join("relay-wire-log.jsonl"),
        catalog_path: state_dir.join("canonical-catalog.jsonl"),
        state_dir,
    }
}

pub fn append_publish_request(
    paths: &StorePaths,
    request_json: &str,
    finalized: &FinalizedKnowledgeObject,
) -> Result<AppendOutcome, StoreError> {
    let carrier_identity = carrier_identity_for_request(request_json)?;
    ensure_parent(&paths.raw_log_path)?;
    ensure_parent(&paths.catalog_path)?;

    if let Some(existing) = get_record_by_carrier_identity(paths, &carrier_identity)? {
        let existing_json = to_canonical_json(&existing.object);
        if existing_json != finalized.canonical_json {
            return Err(StoreError {
                code: "LB_OBJECT_CONFLICT",
                message: format!(
                    "carrier identity already exists with different content: {}",
                    carrier_identity
                ),
            });
        }
        return Ok(AppendOutcome {
            stored_at: Some(existing.stored_at),
            canonical_id: existing.canonical_id,
            carrier_identity,
            object: existing.object,
            duplicate: true,
        });
    }

    if let Some(existing) = get_record(paths, &finalized.canonical_id)? {
        let existing_json = to_canonical_json(&existing.object);
        if existing_json != finalized.canonical_json {
            return Err(StoreError {
                code: "LB_OBJECT_CONFLICT",
                message: format!(
                    "object already exists with different content: {}",
                    finalized.canonical_id
                ),
            });
        }
        return Ok(AppendOutcome {
            stored_at: Some(existing.stored_at),
            canonical_id: finalized.canonical_id.clone(),
            carrier_identity,
            object: existing.object,
            duplicate: true,
        });
    }

    let stored_at = now_utc_rfc3339();
    let raw_record = json_object(vec![
        ("storedAt", JsonValue::String(stored_at.clone())),
        (
            "canonicalId",
            JsonValue::String(finalized.canonical_id.clone()),
        ),
        (
            "carrierIdentity",
            JsonValue::String(carrier_identity.clone()),
        ),
        ("requestJson", JsonValue::String(request_json.to_string())),
    ]);
    let catalog_record = json_object(vec![
        ("storedAt", JsonValue::String(stored_at.clone())),
        (
            "canonicalId",
            JsonValue::String(finalized.canonical_id.clone()),
        ),
        (
            "carrierIdentity",
            JsonValue::String(carrier_identity.clone()),
        ),
        ("object", finalized.object.clone()),
    ]);

    append_line(&paths.raw_log_path, &to_canonical_json(&raw_record))?;
    append_line(&paths.catalog_path, &to_canonical_json(&catalog_record))?;

    Ok(AppendOutcome {
        stored_at: Some(stored_at),
        canonical_id: finalized.canonical_id.clone(),
        carrier_identity,
        object: finalized.object.clone(),
        duplicate: false,
    })
}

pub fn get_record(
    paths: &StorePaths,
    canonical_id: &str,
) -> Result<Option<StoredCatalogRecord>, StoreError> {
    let lines = read_lines(&paths.catalog_path)?;
    for line in lines.into_iter().rev() {
        let value = parse_json(&line)
            .map_err(|error| store_error("LB_INVALID_CATALOG", error.to_string()))?;
        let Some(map) = as_object(&value) else {
            continue;
        };
        if map.get("canonicalId").and_then(as_string) == Some(canonical_id) {
            let stored_at = map
                .get("storedAt")
                .and_then(as_string)
                .unwrap_or_default()
                .to_string();
            let carrier_identity = map
                .get("carrierIdentity")
                .and_then(as_string)
                .unwrap_or_default()
                .to_string();
            let object = map.get("object").cloned().ok_or_else(|| {
                store_error("LB_INVALID_CATALOG", "catalog record missing object")
            })?;
            return Ok(Some(StoredCatalogRecord {
                stored_at,
                canonical_id: canonical_id.to_string(),
                carrier_identity,
                object,
            }));
        }
    }
    Ok(None)
}

pub fn get_record_by_carrier_identity(
    paths: &StorePaths,
    carrier_identity: &str,
) -> Result<Option<StoredCatalogRecord>, StoreError> {
    let records = list_records(paths)?;
    Ok(records
        .into_iter()
        .rev()
        .find(|record| record.carrier_identity == carrier_identity))
}

pub fn get_raw_request(
    paths: &StorePaths,
    canonical_id: &str,
) -> Result<Option<RawRequestRecord>, StoreError> {
    let lines = read_lines(&paths.raw_log_path)?;
    for line in lines.into_iter().rev() {
        let value =
            parse_json(&line).map_err(|error| store_error("LB_INVALID_LOG", error.to_string()))?;
        let Some(map) = as_object(&value) else {
            continue;
        };
        if map.get("canonicalId").and_then(as_string) == Some(canonical_id) {
            let stored_at = map
                .get("storedAt")
                .and_then(as_string)
                .unwrap_or_default()
                .to_string();
            let carrier_identity = map
                .get("carrierIdentity")
                .and_then(as_string)
                .unwrap_or_default()
                .to_string();
            let request_json = map
                .get("requestJson")
                .and_then(as_string)
                .unwrap_or_default()
                .to_string();
            return Ok(Some(RawRequestRecord {
                stored_at,
                canonical_id: canonical_id.to_string(),
                carrier_identity,
                request_json,
            }));
        }
    }
    Ok(None)
}

pub fn list_ids(paths: &StorePaths) -> Result<Vec<String>, StoreError> {
    let records = list_records(paths)?;
    let mut ids = Vec::new();
    let mut seen = BTreeSet::new();
    for record in records {
        if seen.insert(record.canonical_id.clone()) {
            ids.push(record.canonical_id);
        }
    }
    Ok(ids)
}

pub fn subscribe(
    paths: &StorePaths,
    object_type: Option<&str>,
) -> Result<Vec<StoredCatalogRecord>, StoreError> {
    let records = list_records(paths)?;
    Ok(filter_records_by_type(records, object_type))
}

pub fn list_records(paths: &StorePaths) -> Result<Vec<StoredCatalogRecord>, StoreError> {
    let lines = read_lines(&paths.catalog_path)?;
    let mut records = Vec::new();
    for line in lines {
        let value = parse_json(&line)
            .map_err(|error| store_error("LB_INVALID_CATALOG", error.to_string()))?;
        let Some(map) = as_object(&value) else {
            continue;
        };
        let canonical_id = map
            .get("canonicalId")
            .and_then(as_string)
            .ok_or_else(|| store_error("LB_INVALID_CATALOG", "catalog record missing canonicalId"))?
            .to_string();
        let carrier_identity = map
            .get("carrierIdentity")
            .and_then(as_string)
            .unwrap_or_default()
            .to_string();
        let stored_at = map
            .get("storedAt")
            .and_then(as_string)
            .unwrap_or_default()
            .to_string();
        let object = map
            .get("object")
            .cloned()
            .ok_or_else(|| store_error("LB_INVALID_CATALOG", "catalog record missing object"))?;
        records.push(StoredCatalogRecord {
            stored_at,
            canonical_id,
            carrier_identity,
            object,
        });
    }
    Ok(records)
}

pub fn filter_records_by_type(
    records: Vec<StoredCatalogRecord>,
    object_type: Option<&str>,
) -> Vec<StoredCatalogRecord> {
    match object_type {
        Some(expected) => records
            .into_iter()
            .filter(|record| object_type_of(&record.object).as_deref() == Some(expected))
            .collect(),
        None => records,
    }
}

pub fn replay(paths: &StorePaths) -> Result<Vec<StoredReplayRecord>, StoreError> {
    let lines = read_lines(&paths.raw_log_path)?;
    let mut replayed = Vec::new();
    for line in lines {
        let value =
            parse_json(&line).map_err(|error| store_error("LB_INVALID_LOG", error.to_string()))?;
        let Some(map) = as_object(&value) else {
            continue;
        };
        let stored_at = map
            .get("storedAt")
            .and_then(as_string)
            .unwrap_or_default()
            .to_string();
        let canonical_id = map
            .get("canonicalId")
            .and_then(as_string)
            .unwrap_or_default()
            .to_string();
        let carrier_identity = map
            .get("carrierIdentity")
            .and_then(as_string)
            .unwrap_or_default()
            .to_string();
        let Some(request_json) = map.get("requestJson").and_then(as_string) else {
            return Err(store_error(
                "LB_INVALID_LOG",
                "log record missing requestJson",
            ));
        };
        let request_value = parse_json(request_json)
            .map_err(|error| store_error("LB_INVALID_LOG", error.to_string()))?;
        let Some(request_map) = as_object(&request_value) else {
            return Err(store_error(
                "LB_INVALID_LOG",
                "requestJson is not a publish request",
            ));
        };
        let Some(object_value) = request_map.get("object") else {
            return Err(store_error(
                "LB_INVALID_LOG",
                "publish request missing object",
            ));
        };
        let finalized = finalize_knowledge_object(object_value)
            .map_err(|errors| store_error("LB_INVALID_LOG", errors.join("; ")))?;
        if !canonical_id.is_empty() && canonical_id != finalized.canonical_id {
            return Err(store_error(
                "LB_INVALID_LOG",
                "log canonicalId does not match restored object",
            ));
        }
        replayed.push(StoredReplayRecord {
            stored_at,
            canonical_id: finalized.canonical_id,
            carrier_identity,
            object: finalized.object,
        });
    }
    Ok(replayed)
}

fn object_type_of(value: &JsonValue) -> Option<String> {
    let map = as_object(value)?;
    match map.get("type") {
        Some(JsonValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn carrier_identity_for_request(request_json: &str) -> Result<String, StoreError> {
    let value = parse_json(request_json)
        .map_err(|error| store_error("LB_INVALID_LOG", error.to_string()))?;
    let normalized = normalize_carrier_request(value)?;
    let fingerprint = fnv1a64_hex(&to_canonical_json(&normalized));
    Ok(format!("lb:carrier:{}", fingerprint))
}

fn normalize_carrier_request(value: JsonValue) -> Result<JsonValue, StoreError> {
    let Some(map) = as_object(&value) else {
        return Err(store_error(
            "LB_INVALID_LOG",
            "publish request is not an object",
        ));
    };
    let mut normalized = map.clone();
    if let Some(JsonValue::Object(publisher)) = normalized.get_mut("publisher") {
        publisher.remove("signature");
    }
    Ok(JsonValue::Object(normalized))
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

fn read_lines(path: &Path) -> Result<Vec<String>, StoreError> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(store_error("LB_IO_ERROR", error.to_string())),
    };

    let mut lines = Vec::new();
    for line in BufReader::new(file).lines() {
        lines.push(line.map_err(|error| store_error("LB_IO_ERROR", error.to_string()))?);
    }
    Ok(lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect())
}

fn append_line(path: &Path, line: &str) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| store_error("LB_IO_ERROR", error.to_string()))?;
    writeln!(file, "{}", line).map_err(|error| store_error("LB_IO_ERROR", error.to_string()))
}

fn ensure_parent(path: &Path) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| store_error("LB_IO_ERROR", error.to_string()))?;
    }
    Ok(())
}

fn store_error(code: &'static str, message: impl Into<String>) -> StoreError {
    StoreError {
        code,
        message: message.into(),
    }
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    let mut map = BTreeMap::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    JsonValue::Object(map)
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn as_string(value: &JsonValue) -> Option<&str> {
    match value {
        JsonValue::String(value) => Some(value.as_str()),
        _ => None,
    }
}

fn now_utc_rfc3339() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = duration.as_secs() as i64;
    let (year, month, day, hour, minute, second) = unix_seconds_to_utc(seconds);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, minute, second
    )
}

#[cfg(test)]
pub(crate) fn temp_store_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "lingonberry-{}-{}",
        name,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn unix_seconds_to_utc(seconds: i64) -> (i32, u32, u32, u32, u32, u32) {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let hour = (seconds_of_day / 3_600) as u32;
    let minute = ((seconds_of_day % 3_600) / 60) as u32;
    let second = (seconds_of_day % 60) as u32;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day_of_month = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (
        year as i32,
        month as u32,
        day_of_month as u32,
        hour,
        minute,
        second,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::{parse_json, validate_knowledge_object, validate_publish_request};

    #[test]
    fn append_duplicate_is_idempotent() {
        let paths_dir = temp_store_dir("append-duplicate");
        let backend = FileStorageBackend::new(&paths_dir);
        let request = parse_json(include_str!(
            "../../../fixtures/http-publish-request/minimal-request.json"
        ))
        .unwrap();
        assert!(validate_publish_request(&request).is_empty());
        let object = as_object(&request).unwrap().get("object").unwrap().clone();
        assert!(validate_knowledge_object(&object).is_empty());
        let finalized = lingonberry_protocol::finalize_knowledge_object(&object).unwrap();

        let raw = include_str!("../../../fixtures/http-publish-request/minimal-request.json");
        let first = backend.append_publish_request(raw, &finalized).unwrap();
        assert!(!first.duplicate);
        let second = backend.append_publish_request(raw, &finalized).unwrap();
        assert!(second.duplicate);
        assert_eq!(
            backend.list_ids().unwrap(),
            vec!["lb:obj:example-0001".to_string()]
        );
        assert!(backend.get("lb:obj:example-0001").unwrap().is_some());
        let raw_request = backend
            .get_raw_request("lb:obj:example-0001")
            .unwrap()
            .unwrap();
        assert!(raw_request.request_json.contains("\"publisher\""));
    }

    #[test]
    fn append_conflict_is_rejected() {
        let paths_dir = temp_store_dir("append-conflict");
        let backend = FileStorageBackend::new(&paths_dir);
        let request = parse_json(include_str!(
            "../../../fixtures/http-publish-request/minimal-request.json"
        ))
        .unwrap();
        let object = as_object(&request).unwrap().get("object").unwrap().clone();
        let finalized = lingonberry_protocol::finalize_knowledge_object(&object).unwrap();
        backend
            .append_publish_request(
                include_str!("../../../fixtures/http-publish-request/minimal-request.json"),
                &finalized,
            )
            .unwrap();

        let altered = if let JsonValue::Object(mut map) = object.clone() {
            map.insert(
                "body".to_string(),
                JsonValue::Object({
                    let mut body = BTreeMap::new();
                    body.insert(
                        "text".to_string(),
                        JsonValue::String("Different content".to_string()),
                    );
                    body.insert("language".to_string(), JsonValue::String("en".to_string()));
                    body
                }),
            );
            JsonValue::Object(map)
        } else {
            object.clone()
        };
        let altered_finalized = lingonberry_protocol::finalize_knowledge_object(&altered).unwrap();
        let error = backend
            .append_publish_request(
                include_str!("../../../fixtures/http-publish-request/minimal-request.json"),
                &altered_finalized,
            )
            .expect_err("must conflict");
        assert_eq!(error.code, "LB_OBJECT_CONFLICT");
    }

    #[test]
    fn export_and_import_archive_round_trip() {
        let source_dir = temp_store_dir("archive-source");
        let backend = FileStorageBackend::new(&source_dir);
        let request = parse_json(include_str!(
            "../../../fixtures/http-publish-request/minimal-request.json"
        ))
        .unwrap();
        let object = as_object(&request).unwrap().get("object").unwrap().clone();
        let finalized = lingonberry_protocol::finalize_knowledge_object(&object).unwrap();
        backend
            .append_publish_request(
                include_str!("../../../fixtures/http-publish-request/minimal-request.json"),
                &finalized,
            )
            .unwrap();

        let archive_dir = temp_store_dir("archive-export");
        let export_report = export_archive(&backend, &archive_dir).expect("must export archive");
        assert_eq!(export_report.record_count, 1);
        assert!(archive_dir.join("manifest.json").exists());
        assert!(archive_dir.join("wire-log.jsonl").exists());
        assert!(archive_dir.join("canonical-catalog.jsonl").exists());

        let import_dir = temp_store_dir("archive-import");
        let import_backend = FileStorageBackend::new(&import_dir);
        let import_report =
            import_archive(&import_backend, &archive_dir).expect("must import archive");
        assert_eq!(import_report.record_count, 1);
        assert_eq!(import_report.duplicate_count, 0);
        assert_eq!(
            import_backend.list_ids().unwrap(),
            vec!["lb:obj:example-0001".to_string()]
        );
    }

    #[test]
    fn runtime_capability_manifest_mentions_carrier_defaults() {
        let manifest = build_runtime_capability_manifest();
        let map = as_object(&manifest).expect("manifest must be an object");
        assert_eq!(map.get("carrierKind").and_then(as_string), Some("relay"));
        assert_eq!(
            map.get("protocolVersion").and_then(as_string),
            Some(PROTOCOL_VERSION)
        );
        let schema_versions = match map.get("supportedSchemaVersions") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected supportedSchemaVersions: {:?}", other),
        };
        assert_eq!(schema_versions.len(), 2);
        let validation_constraints = match map.get("validationConstraints") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected validationConstraints: {:?}", other),
        };
        assert!(validation_constraints
            .iter()
            .any(|value| as_string(value) == Some("schema-version-match")));
        let finalize_constraints = match map.get("finalizeConstraints") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected finalizeConstraints: {:?}", other),
        };
        assert!(finalize_constraints
            .iter()
            .any(|value| as_string(value) == Some("rawref-preservation")));
        let defaults =
            as_object(map.get("defaults").expect("manifest defaults")).expect("defaults object");
        assert_eq!(
            defaults.get("accessScope").and_then(as_string),
            Some(DEFAULT_ACCESS_SCOPE)
        );
        assert_eq!(
            defaults.get("retentionHint").and_then(as_string),
            Some(DEFAULT_RETENTION_HINT)
        );
    }
}
