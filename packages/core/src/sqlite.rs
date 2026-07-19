use crate::{
    append_line, as_object, as_string, carrier_identity_for_request, ensure_parent,
    filter_records_by_type, finalize_knowledge_object, get_raw_request, json_object,
    now_utc_rfc3339, read_lines, store_error, to_canonical_json, AppendOutcome,
    FinalizedKnowledgeObject, JsonValue, RawRequestRecord, StorageBackend, StoreError, StorePaths,
    StoredCatalogRecord, StoredReplayRecord,
};
use lingonberry_protocol::{parse_json, to_canonical_json as protocol_to_canonical_json};
use std::collections::BTreeSet;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr;

#[allow(non_camel_case_types)]
type sqlite3 = c_void;
#[allow(non_camel_case_types)]
type sqlite3_stmt = c_void;
#[allow(non_camel_case_types)]
type sqlite3_destructor_type = Option<unsafe extern "C" fn(*mut c_void)>;

const SQLITE_OK: c_int = 0;
const SQLITE_ROW: c_int = 100;
const SQLITE_DONE: c_int = 101;
const SQLITE_OPEN_READWRITE: c_int = 0x0000_0002;
const SQLITE_OPEN_CREATE: c_int = 0x0000_0004;
const SQLITE_OPEN_FULLMUTEX: c_int = 0x0001_0000;

#[link(name = "sqlite3")]
extern "C" {
    fn sqlite3_open_v2(
        filename: *const c_char,
        pp_db: *mut *mut sqlite3,
        flags: c_int,
        z_vfs: *const c_char,
    ) -> c_int;
    fn sqlite3_close(db: *mut sqlite3) -> c_int;
    fn sqlite3_errmsg(db: *mut sqlite3) -> *const c_char;
    fn sqlite3_exec(
        db: *mut sqlite3,
        sql: *const c_char,
        callback: Option<
            unsafe extern "C" fn(*mut c_void, c_int, *mut *mut c_char, *mut *mut c_char) -> c_int,
        >,
        arg: *mut c_void,
        errmsg: *mut *mut c_char,
    ) -> c_int;
    fn sqlite3_prepare_v2(
        db: *mut sqlite3,
        z_sql: *const c_char,
        n_byte: c_int,
        pp_stmt: *mut *mut sqlite3_stmt,
        pz_tail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_step(stmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_finalize(stmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_bind_text(
        stmt: *mut sqlite3_stmt,
        idx: c_int,
        value: *const c_char,
        n: c_int,
        destructor: sqlite3_destructor_type,
    ) -> c_int;
    fn sqlite3_column_text(stmt: *mut sqlite3_stmt, i_col: c_int) -> *const u8;
}

#[derive(Debug, Clone)]
pub struct SqliteStorageBackend {
    paths: StorePaths,
}

impl SqliteStorageBackend {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let state_dir = base_dir.as_ref().to_path_buf();
        Self {
            paths: StorePaths {
                raw_log_path: state_dir.join("relay-wire-log.jsonl"),
                catalog_path: state_dir.join("canonical-catalog.sqlite3"),
                state_dir,
            },
        }
    }

    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }

    fn open_db(&self) -> Result<SqliteDb, StoreError> {
        ensure_parent(&self.paths.catalog_path)?;
        let db = SqliteDb::open(&self.paths.catalog_path)?;
        db.exec_batch(
            "CREATE TABLE IF NOT EXISTS canonical_objects (
                canonical_id TEXT PRIMARY KEY,
                carrier_identity TEXT UNIQUE NOT NULL,
                stored_at TEXT NOT NULL,
                object_json TEXT NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_canonical_objects_carrier_identity ON canonical_objects(carrier_identity);
            CREATE INDEX IF NOT EXISTS idx_canonical_objects_stored_at ON canonical_objects(stored_at);",
        )?;
        Ok(db)
    }
}

impl StorageBackend for SqliteStorageBackend {
    fn append_publish_request(
    &self,
    request_json: &str,
    finalized: &FinalizedKnowledgeObject,
) -> Result<AppendOutcome, StoreError> {
    ensure_parent(&self.paths.raw_log_path)?;
    let carrier_identity = carrier_identity_for_request(request_json)?;
    let db = self.open_db()?;
    let existing_by_carrier_identity =
        db.get_object_by_carrier_identity(&carrier_identity)?;
    let existing_by_canonical_id = db.get_object(&finalized.canonical_id)?;
    let classification = crate::classify_duplicate_or_conflict(
        existing_by_canonical_id
            .as_ref()
            .map(|existing| crate::ExistingObjectIdentity {
                canonical_id: &existing.canonical_id,
                carrier_identity: &existing.carrier_identity,
                object: &existing.object,
            }),
        existing_by_carrier_identity
            .as_ref()
            .map(|existing| crate::ExistingObjectIdentity {
                canonical_id: &existing.canonical_id,
                carrier_identity: &existing.carrier_identity,
                object: &existing.object,
            }),
        crate::IncomingObjectIdentity {
            canonical_id: &finalized.canonical_id,
            carrier_identity: &carrier_identity,
            canonical_json: &finalized.canonical_json,
        },
    );

    match classification {
        crate::DuplicateConflictClassification::New => {}
        crate::DuplicateConflictClassification::ExactDuplicate => {
            let existing = existing_by_carrier_identity
                .or(existing_by_canonical_id)
                .ok_or_else(|| {
                    store_error(
                        "LB_OBJECT_CONFLICT",
                        "duplicate classification missing existing record",
                    )
                })?;
            return Ok(AppendOutcome {
                stored_at: Some(existing.stored_at),
                canonical_id: existing.canonical_id,
                carrier_identity: existing.carrier_identity,
                object: existing.object,
                duplicate: true,
            });
        }
        conflict => {
            return Err(store_error(
                conflict.code(),
                format!(
                    "duplicate/conflict contract {} classified write as {:?}",
                    crate::DUPLICATE_CONFLICT_CONTRACT_VERSION,
                    conflict
                ),
            ));
        }
    }

    let stored_at = now_utc_rfc3339();
    append_line(
        &self.paths.raw_log_path,
        &to_canonical_json(&json_object(vec![
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
        ])),
    )?;
    db.insert_object(
        &finalized.canonical_id,
        &carrier_identity,
        &stored_at,
        &finalized.object,
    )?;
    Ok(AppendOutcome {
        stored_at: Some(stored_at),
        canonical_id: finalized.canonical_id.clone(),
        carrier_identity,
        object: finalized.object.clone(),
        duplicate: false,
    })
}

    fn get(&self, canonical_id: &str) -> Result<Option<StoredCatalogRecord>, StoreError> {
        self.open_db()?.get_object(canonical_id)
    }

    fn get_raw_request(&self, canonical_id: &str) -> Result<Option<RawRequestRecord>, StoreError> {
        get_raw_request(&self.paths, canonical_id)
    }

    fn list_ids(&self) -> Result<Vec<String>, StoreError> {
        self.open_db()?.list_ids()
    }

    fn subscribe(&self, object_type: Option<&str>) -> Result<Vec<StoredCatalogRecord>, StoreError> {
        let records = self.open_db()?.list_records()?;
        Ok(filter_records_by_type(records, object_type))
    }

    fn replay(&self) -> Result<Vec<StoredReplayRecord>, StoreError> {
        replay(&self.paths)
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

struct SqliteDb {
    db: *mut sqlite3,
}

impl SqliteDb {
    fn open(path: &Path) -> Result<Self, StoreError> {
        let c_path =
            CString::new(path.as_os_str().to_string_lossy().as_bytes().to_vec()).map_err(|_| {
                store_error(
                    "LB_SQLITE_OPEN",
                    "database path contains interior null byte",
                )
            })?;
        let mut db = ptr::null_mut();
        let rc = unsafe {
            sqlite3_open_v2(
                c_path.as_ptr(),
                &mut db,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_FULLMUTEX,
                ptr::null(),
            )
        };
        if rc != SQLITE_OK {
            let message = unsafe { sqlite_error_message(db) };
            if !db.is_null() {
                unsafe {
                    sqlite3_close(db);
                }
            }
            return Err(store_error("LB_SQLITE_OPEN", message));
        }
        Ok(Self { db })
    }

    fn exec_batch(&self, sql: &str) -> Result<(), StoreError> {
        let c_sql = CString::new(sql)
            .map_err(|_| store_error("LB_SQLITE_EXEC", "SQL contains interior null byte"))?;
        let rc = unsafe {
            sqlite3_exec(
                self.db,
                c_sql.as_ptr(),
                None,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if rc != SQLITE_OK {
            return Err(store_error("LB_SQLITE_EXEC", unsafe {
                sqlite_error_message(self.db)
            }));
        }
        Ok(())
    }

    fn get_object(&self, canonical_id: &str) -> Result<Option<StoredCatalogRecord>, StoreError> {
        let stmt = self.prepare(
            "SELECT canonical_id, carrier_identity, stored_at, object_json FROM canonical_objects WHERE canonical_id = ?1 LIMIT 1",
        )?;
        bind_text(stmt, 1, canonical_id)?;
        let result = unsafe {
            match sqlite3_step(stmt) {
                SQLITE_ROW => {
                    let carrier_identity = column_text(stmt, 1);
                    let stored_at = column_text(stmt, 2);
                    let object_json = column_text(stmt, 3);
                    let object = parse_json(&object_json)
                        .map_err(|error| store_error("LB_SQLITE_SELECT", error.to_string()))?;
                    Some(StoredCatalogRecord {
                        stored_at,
                        canonical_id: canonical_id.to_string(),
                        carrier_identity,
                        object,
                    })
                }
                SQLITE_DONE => None,
                _ => {
                    return Err(store_error(
                        "LB_SQLITE_SELECT",
                        sqlite_error_message(self.db),
                    ))
                }
            }
        };
        finalize(stmt)?;
        Ok(result)
    }

    fn get_object_by_carrier_identity(
        &self,
        carrier_identity: &str,
    ) -> Result<Option<StoredCatalogRecord>, StoreError> {
        let stmt = self.prepare(
            "SELECT canonical_id, carrier_identity, stored_at, object_json FROM canonical_objects WHERE carrier_identity = ?1 LIMIT 1",
        )?;
        bind_text(stmt, 1, carrier_identity)?;
        let result = unsafe {
            match sqlite3_step(stmt) {
                SQLITE_ROW => {
                    let canonical_id = column_text(stmt, 0);
                    let carrier_identity = column_text(stmt, 1);
                    let stored_at = column_text(stmt, 2);
                    let object_json = column_text(stmt, 3);
                    let object = parse_json(&object_json)
                        .map_err(|error| store_error("LB_SQLITE_SELECT", error.to_string()))?;
                    Some(StoredCatalogRecord {
                        stored_at,
                        canonical_id,
                        carrier_identity,
                        object,
                    })
                }
                SQLITE_DONE => None,
                _ => {
                    return Err(store_error(
                        "LB_SQLITE_SELECT",
                        sqlite_error_message(self.db),
                    ))
                }
            }
        };
        finalize(stmt)?;
        Ok(result)
    }

    fn list_ids(&self) -> Result<Vec<String>, StoreError> {
        let stmt = self.prepare("SELECT canonical_id FROM canonical_objects ORDER BY rowid ASC")?;
        let mut ids = Vec::new();
        let mut seen = BTreeSet::new();
        loop {
            let rc = unsafe { sqlite3_step(stmt) };
            match rc {
                SQLITE_ROW => {
                    let id = column_text(stmt, 0);
                    if seen.insert(id.clone()) {
                        ids.push(id);
                    }
                }
                SQLITE_DONE => break,
                _ => {
                    finalize(stmt)?;
                    return Err(store_error("LB_SQLITE_SELECT", unsafe {
                        sqlite_error_message(self.db)
                    }));
                }
            }
        }
        finalize(stmt)?;
        Ok(ids)
    }

    fn list_records(&self) -> Result<Vec<StoredCatalogRecord>, StoreError> {
        let stmt = self.prepare(
            "SELECT canonical_id, carrier_identity, stored_at, object_json FROM canonical_objects ORDER BY rowid ASC",
        )?;
        let mut records = Vec::new();
        loop {
            let rc = unsafe { sqlite3_step(stmt) };
            match rc {
                SQLITE_ROW => {
                    let canonical_id = column_text(stmt, 0);
                    let carrier_identity = column_text(stmt, 1);
                    let stored_at = column_text(stmt, 2);
                    let object_json = column_text(stmt, 3);
                    let object = parse_json(&object_json)
                        .map_err(|error| store_error("LB_SQLITE_SELECT", error.to_string()))?;
                    records.push(StoredCatalogRecord {
                        stored_at,
                        canonical_id,
                        carrier_identity,
                        object,
                    });
                }
                SQLITE_DONE => break,
                _ => {
                    finalize(stmt)?;
                    return Err(store_error("LB_SQLITE_SELECT", unsafe {
                        sqlite_error_message(self.db)
                    }));
                }
            }
        }
        finalize(stmt)?;
        Ok(records)
    }

    fn insert_object(
        &self,
        canonical_id: &str,
        carrier_identity: &str,
        stored_at: &str,
        object: &JsonValue,
    ) -> Result<(), StoreError> {
        let object_json = protocol_to_canonical_json(object);
        let stmt = self.prepare(
            "INSERT INTO canonical_objects (canonical_id, carrier_identity, stored_at, object_json) VALUES (?1, ?2, ?3, ?4)",
        )?;
        bind_text(stmt, 1, canonical_id)?;
        bind_text(stmt, 2, carrier_identity)?;
        bind_text(stmt, 3, stored_at)?;
        bind_text(stmt, 4, &object_json)?;
        let rc = unsafe { sqlite3_step(stmt) };
        if rc != SQLITE_DONE {
            finalize(stmt)?;
            return Err(store_error("LB_SQLITE_INSERT", unsafe {
                sqlite_error_message(self.db)
            }));
        }
        finalize(stmt)?;
        Ok(())
    }

    fn prepare(&self, sql: &str) -> Result<*mut sqlite3_stmt, StoreError> {
        let c_sql = CString::new(sql)
            .map_err(|_| store_error("LB_SQLITE_PREPARE", "SQL contains interior null byte"))?;
        let mut stmt = ptr::null_mut();
        let rc =
            unsafe { sqlite3_prepare_v2(self.db, c_sql.as_ptr(), -1, &mut stmt, ptr::null_mut()) };
        if rc != SQLITE_OK {
            return Err(store_error("LB_SQLITE_PREPARE", unsafe {
                sqlite_error_message(self.db)
            }));
        }
        Ok(stmt)
    }
}

impl Drop for SqliteDb {
    fn drop(&mut self) {
        if !self.db.is_null() {
            unsafe {
                sqlite3_close(self.db);
            }
        }
    }
}

fn finalize(stmt: *mut sqlite3_stmt) -> Result<(), StoreError> {
    let rc = unsafe { sqlite3_finalize(stmt) };
    if rc != SQLITE_OK {
        return Err(store_error(
            "LB_SQLITE_FINALIZE",
            "failed to finalize sqlite statement",
        ));
    }
    Ok(())
}

fn bind_text(stmt: *mut sqlite3_stmt, idx: c_int, value: &str) -> Result<(), StoreError> {
    let c_value = CString::new(value)
        .map_err(|_| store_error("LB_SQLITE_BIND", "value contains interior null byte"))?;
    let rc = unsafe { sqlite3_bind_text(stmt, idx, c_value.as_ptr(), -1, sqlite_transient()) };
    if rc != SQLITE_OK {
        return Err(store_error(
            "LB_SQLITE_BIND",
            "failed to bind sqlite parameter",
        ));
    }
    Ok(())
}

fn column_text(stmt: *mut sqlite3_stmt, index: c_int) -> String {
    unsafe {
        let ptr = sqlite3_column_text(stmt, index);
        if ptr.is_null() {
            return String::new();
        }
        CStr::from_ptr(ptr as *const c_char)
            .to_string_lossy()
            .into_owned()
    }
}

unsafe fn sqlite_error_message(db: *mut sqlite3) -> String {
    if db.is_null() {
        return "sqlite error".to_string();
    }
    let ptr = sqlite3_errmsg(db);
    if ptr.is_null() {
        return "sqlite error".to_string();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

fn sqlite_transient() -> sqlite3_destructor_type {
    unsafe { std::mem::transmute::<isize, sqlite3_destructor_type>(-1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::{parse_json, validate_knowledge_object, validate_publish_request};
    use std::collections::BTreeMap;

    #[test]
    fn sqlite_backend_round_trip() {
        let backend = SqliteStorageBackend::new(crate::temp_store_dir("sqlite-round-trip"));
        let request = parse_json(include_str!(
            "../../../fixtures/http-publish-request/minimal-request.json"
        ))
        .unwrap();
        assert!(validate_publish_request(&request).is_empty());
        let object = as_object(&request).unwrap().get("object").unwrap().clone();
        assert!(validate_knowledge_object(&object).is_empty());
        let finalized = finalize_knowledge_object(&object).unwrap();
        let raw = include_str!("../../../fixtures/http-publish-request/minimal-request.json");

        let first = backend.append_publish_request(raw, &finalized).unwrap();
        assert!(!first.duplicate);
        let second = backend.append_publish_request(raw, &finalized).unwrap();
        assert!(second.duplicate);

        let record = backend.get("lb:obj:example-0001").unwrap().unwrap();
        assert_eq!(record.stored_at, first.stored_at.unwrap());
        assert_eq!(
            backend.list_ids().unwrap(),
            vec!["lb:obj:example-0001".to_string()]
        );
        assert_eq!(backend.replay().unwrap().len(), 1);
    }

    #[test]
    fn sqlite_backend_subscription_filters_by_type() {
        let backend = SqliteStorageBackend::new(crate::temp_store_dir("sqlite-subscribe"));
        let request = parse_json(include_str!(
            "../../../fixtures/http-publish-request/minimal-request.json"
        ))
        .unwrap();
        let inquiry = as_object(&request).unwrap().get("object").unwrap().clone();
        let inquiry_finalized = finalize_knowledge_object(&inquiry).unwrap();
        backend
            .append_publish_request(
                include_str!("../../../fixtures/http-publish-request/minimal-request.json"),
                &inquiry_finalized,
            )
            .unwrap();

        let claim = if let JsonValue::Object(mut map) = inquiry.clone() {
            map.insert(
                "id".to_string(),
                JsonValue::String("lb:obj:example-0002".to_string()),
            );
            map.insert("type".to_string(), JsonValue::String("claim".to_string()));
            map.insert(
                "body".to_string(),
                JsonValue::Object({
                    let mut body = BTreeMap::new();
                    body.insert(
                        "text".to_string(),
                        JsonValue::String("This is a claim".to_string()),
                    );
                    body.insert("language".to_string(), JsonValue::String("en".to_string()));
                    body
                }),
            );
            if let Some(JsonValue::Object(provenance)) = map.get_mut("provenance") {
                if let Some(JsonValue::Array(sources)) = provenance.get_mut("sources") {
                    if let Some(JsonValue::Object(source)) = sources.first_mut() {
                        source.insert(
                            "sourceId".to_string(),
                            JsonValue::String("draft:example-0002".to_string()),
                        );
                        source.insert(
                            "observedAt".to_string(),
                            JsonValue::String("2026-06-17T00:00:00Z".to_string()),
                        );
                    }
                }
            }
            if let Some(JsonValue::Object(raw_ref)) = map.get_mut("rawRef") {
                raw_ref.insert(
                    "sourceId".to_string(),
                    JsonValue::String("draft:example-0002".to_string()),
                );
            }
            JsonValue::Object(map)
        } else {
            panic!("expected object");
        };
        let claim_request = JsonValue::Object({
            let mut publisher = BTreeMap::new();
            publisher.insert(
                "publicKey".to_string(),
                JsonValue::String(
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
                ),
            );
            publisher.insert(
                "signature".to_string(),
                JsonValue::String("sig:example-0002".to_string()),
            );

            let mut request = BTreeMap::new();
            request.insert("object".to_string(), claim.clone());
            request.insert("publisher".to_string(), JsonValue::Object(publisher));
            request
        });
        let claim_finalized = finalize_knowledge_object(&claim).unwrap();
        backend
            .append_publish_request(
                &protocol_to_canonical_json(&claim_request),
                &claim_finalized,
            )
            .unwrap();

        let filtered = backend.subscribe(Some("claim")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].canonical_id, "lb:obj:example-0002");
    }
}
