use lingonberry_core::{FileStorageBackend, SqliteStorageBackend, StorageBackend};
use lingonberry_indexer::{
    index_rebuild_result_json, rebuild_index, verify_index, IndexConsistencyStatus, IndexSnapshot,
    INDEX_LIFECYCLE_CONTRACT_VERSION,
};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::finalize_knowledge_object_full;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn rebuild_and_consistency_contract_has_file_sqlite_parity() {
    let request_json = fs::read_to_string(
        workspace_root().join("fixtures/http-publish-request/minimal-request.json"),
    )
    .expect("read fixture");

    let file_dir = unique_temp_dir("file");
    assert_lifecycle_contract(&FileStorageBackend::new(&file_dir), &request_json);

    let sqlite_dir = unique_temp_dir("sqlite");
    assert_lifecycle_contract(&SqliteStorageBackend::new(&sqlite_dir), &request_json);

    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

fn assert_lifecycle_contract(backend: &impl StorageBackend, request_json: &str) {
    let empty = rebuild_index(backend);
    assert_eq!(empty.contract_version, INDEX_LIFECYCLE_CONTRACT_VERSION);
    assert_eq!(empty.status, IndexConsistencyStatus::Consistent);
    assert_eq!(empty.code, "LB_INDEX_CONSISTENT");
    assert_eq!(empty.storage.as_ref().expect("storage generation").record_count, 0);
    assert_eq!(empty.index.as_ref().expect("index generation").record_count, 0);

    let finalized = finalized_request(request_json);
    backend
        .append_publish_request(request_json, &finalized)
        .expect("store object");

    let rebuilt = rebuild_index(backend);
    assert_eq!(rebuilt.status, IndexConsistencyStatus::Consistent);
    assert_eq!(rebuilt.code, "LB_INDEX_CONSISTENT");
    let storage = rebuilt.storage.as_ref().expect("storage generation");
    let index = rebuilt.index.as_ref().expect("index generation");
    assert_eq!(storage.record_count, 1);
    assert_eq!(storage, index);
    assert!(storage.generation.starts_with("idx:fnv1a64:"));
    assert!(rebuilt.missing_from_index.is_empty());
    assert!(rebuilt.unexpected_in_index.is_empty());

    let JsonValue::Object(json) = index_rebuild_result_json(&rebuilt) else {
        panic!("result must be object");
    };
    assert_eq!(
        json.get("contractVersion"),
        Some(&JsonValue::String("1".to_string()))
    );
    assert_eq!(
        json.get("status"),
        Some(&JsonValue::String("consistent".to_string()))
    );

    let stale = verify_index(backend, IndexSnapshot::default());
    assert_eq!(stale.status, IndexConsistencyStatus::Inconsistent);
    assert_eq!(stale.code, "LB_INDEX_INCONSISTENT");
    assert_eq!(stale.missing_from_index, vec![finalized.canonical_id]);
    assert!(stale.unexpected_in_index.is_empty());
}

fn finalized_request(request_json: &str) -> lingonberry_protocol::FinalizedKnowledgeObject {
    let request = parse_json(request_json).expect("request parses");
    let JsonValue::Object(request) = request else {
        panic!("request must be an object");
    };
    finalize_knowledge_object_full(request.get("object").expect("object"))
        .expect("object finalizes")
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
        "lingonberry-index-lifecycle-{label}-{}-{nonce}",
        std::process::id()
    ))
}
