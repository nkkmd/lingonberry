use lingonberry_core::{FileStorageBackend, SqliteStorageBackend, StorageBackend};
use lingonberry_indexer::{verify_index, IndexConsistencyStatus, IndexSnapshot};
use lingonberry_protocol::{derive_identity_key, parse_json, to_canonical_json, JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn ambiguous_record_content_is_rejected_for_file_and_sqlite() {
    let request_json = fs::read_to_string(
        workspace_root().join("fixtures/http-publish-request/minimal-request.json"),
    )
    .expect("read fixture");

    let file_dir = unique_temp_dir("file");
    assert_ambiguous(&FileStorageBackend::new(&file_dir), &request_json);

    let sqlite_dir = unique_temp_dir("sqlite");
    assert_ambiguous(&SqliteStorageBackend::new(&sqlite_dir), &request_json);

    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

fn assert_ambiguous(backend: &impl StorageBackend, request_json: &str) {
    let finalized = finalized_request(request_json);
    backend
        .append_publish_request(request_json, &finalized)
        .expect("store object");

    let mut records = backend.subscribe(None).expect("read records");
    let record = records.first_mut().expect("stored record");
    let JsonValue::Object(object) = &mut record.object else {
        panic!("stored object must be an object");
    };
    object.insert(
        "type".to_string(),
        JsonValue::String("ambiguous-test-type".to_string()),
    );

    let result = verify_index(backend, IndexSnapshot::from_records(records));
    assert_eq!(result.status, IndexConsistencyStatus::Inconsistent);
    assert_eq!(result.code, "LB_INDEX_AMBIGUOUS");
    assert!(result.missing_from_index.is_empty());
    assert!(result.unexpected_in_index.is_empty());
    assert_eq!(result.ambiguous_ids, vec![finalized.canonical_id]);

    let storage = result.storage.as_ref().expect("storage generation");
    let index = result.index.as_ref().expect("index generation");
    assert_eq!(storage.id_digest, index.id_digest);
    assert_ne!(storage.content_digest, index.content_digest);
}

fn finalized_request(request_json: &str) -> lingonberry_protocol::FinalizedKnowledgeObject {
    let request = parse_json(request_json).expect("request parses");
    let JsonValue::Object(request) = request else {
        panic!("request must be an object");
    };
    let object = request.get("object").expect("object").clone();
    let JsonValue::Object(object_map) = &object else {
        panic!("object must be an object");
    };
    let canonical_id = match object_map.get("id") {
        Some(JsonValue::String(value)) => value.clone(),
        other => panic!("object missing id: {other:?}"),
    };
    lingonberry_protocol::FinalizedKnowledgeObject {
        canonical_id,
        identity_key: derive_identity_key(&object),
        canonical_json: to_canonical_json(&object),
        object,
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lingonberry-index-ambiguity-{label}-{}-{nonce}",
        std::process::id()
    ))
}
