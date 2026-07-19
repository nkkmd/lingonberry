use lingonberry_core::{
    object_retrieval_result_json, retrieve_object, FileStorageBackend, ObjectRetrievalStatus,
    StorageBackend, OBJECT_RETRIEVAL_CONTRACT_VERSION,
};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::finalize_knowledge_object_full;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn retrieval_contract_covers_found_not_found_and_invalid_id() {
    let dir = unique_temp_dir();
    let backend = FileStorageBackend::new(&dir);
    let request_json = fs::read_to_string(
        workspace_root().join("fixtures/http-publish-request/minimal-request.json"),
    )
    .expect("read fixture");
    let request = parse_json(&request_json).expect("parse fixture");
    let JsonValue::Object(request) = request else {
        panic!("request must be object");
    };
    let finalized = finalize_knowledge_object_full(request.get("object").expect("object"))
        .expect("finalize object");
    backend
        .append_publish_request(&request_json, &finalized)
        .expect("store object");

    let found = retrieve_object(&finalized.canonical_id, &backend);
    assert_eq!(found.contract_version, OBJECT_RETRIEVAL_CONTRACT_VERSION);
    assert_eq!(found.status, ObjectRetrievalStatus::Found);
    assert_eq!(found.code, "LB_OBJECT_FOUND");
    let JsonValue::Object(found_json) = object_retrieval_result_json(&found) else {
        panic!("result must be object");
    };
    assert_eq!(
        found_json.get("contractVersion"),
        Some(&JsonValue::String("1".to_string()))
    );
    assert_eq!(
        found_json.get("canonicalId"),
        Some(&JsonValue::String(finalized.canonical_id.clone()))
    );
    assert!(found_json.contains_key("canonical"));

    let missing = retrieve_object("sha256:missing", &backend);
    assert_eq!(missing.status, ObjectRetrievalStatus::NotFound);
    assert_eq!(missing.code, "LB_OBJECT_NOT_FOUND");
    assert!(missing.record.is_none());

    let invalid = retrieve_object("  ", &backend);
    assert_eq!(invalid.status, ObjectRetrievalStatus::InvalidRequest);
    assert_eq!(invalid.code, "LB_CANONICAL_ID_REQUIRED");
    assert!(invalid.record.is_none());

    fs::remove_dir_all(dir).ok();
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lingonberry-object-retrieval-{}-{nonce}",
        std::process::id()
    ))
}
