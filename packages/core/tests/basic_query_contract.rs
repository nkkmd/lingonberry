use lingonberry_core::{
    basic_query_result_json, execute_basic_query, BasicQueryStatus, FileStorageBackend,
    SqliteStorageBackend, StorageBackend, BASIC_QUERY_CONTRACT_VERSION,
};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::finalize_knowledge_object_full;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn file_and_sqlite_share_basic_query_contract() {
    let request_json = fs::read_to_string(
        workspace_root().join("fixtures/http-publish-request/minimal-request.json"),
    )
    .expect("read fixture");

    let file_dir = unique_temp_dir("file");
    let file_backend = FileStorageBackend::new(&file_dir);
    assert_query_contract(&file_backend, &request_json);

    let sqlite_dir = unique_temp_dir("sqlite");
    let sqlite_backend = SqliteStorageBackend::new(&sqlite_dir);
    assert_query_contract(&sqlite_backend, &request_json);

    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

fn assert_query_contract(backend: &impl StorageBackend, request_json: &str) {
    let empty = execute_basic_query(None, backend);
    assert_eq!(empty.contract_version, BASIC_QUERY_CONTRACT_VERSION);
    assert_eq!(empty.status, BasicQueryStatus::Empty);
    assert_eq!(empty.code, "LB_QUERY_EMPTY");
    assert!(empty.records.is_empty());

    let invalid = execute_basic_query(Some("  "), backend);
    assert_eq!(invalid.status, BasicQueryStatus::InvalidRequest);
    assert_eq!(invalid.code, "LB_QUERY_TYPE_REQUIRED");

    let request = parse_json(request_json).expect("parse fixture");
    let JsonValue::Object(request) = request else {
        panic!("request must be an object");
    };
    let finalized = finalize_knowledge_object_full(request.get("object").expect("object"))
        .expect("finalize object");
    backend
        .append_publish_request(request_json, &finalized)
        .expect("store object");

    let all = execute_basic_query(None, backend);
    assert_eq!(all.status, BasicQueryStatus::Success);
    assert_eq!(all.code, "LB_QUERY_SUCCESS");
    assert_eq!(all.records.len(), 1);
    assert_eq!(all.records[0].canonical_id, finalized.canonical_id);

    let JsonValue::Object(rendered) = basic_query_result_json(&all) else {
        panic!("query result must be object");
    };
    assert_eq!(
        rendered.get("contractVersion"),
        Some(&JsonValue::String("1".to_string()))
    );
    assert_eq!(
        rendered.get("status"),
        Some(&JsonValue::String("success".to_string()))
    );
    assert_eq!(
        rendered.get("ordering"),
        Some(&JsonValue::String("canonicalId-ascending".to_string()))
    );
    assert_eq!(
        rendered.get("count"),
        Some(&JsonValue::Number("1".to_string()))
    );
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
        "lingonberry-basic-query-{label}-{}-{nonce}",
        std::process::id()
    ))
}
