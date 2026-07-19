use lingonberry_core::{FileStorageBackend, SqliteStorageBackend, StorageBackend};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::finalize_knowledge_object_full;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn file_and_sqlite_backends_share_duplicate_conflict_semantics() {
    let workspace = workspace_root();
    let minimal =
        fs::read_to_string(workspace.join("fixtures/http-publish-request/minimal-request.json"))
            .expect("read minimal request");
    let conflict = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/with-identity-claim.json"),
    )
    .expect("read conflict request");

    let file_dir = unique_temp_dir("file");
    let file_backend = FileStorageBackend::new(&file_dir);
    assert_backend_contract(
        &file_backend,
        &file_backend.paths().raw_log_path,
        &minimal,
        &conflict,
    );

    let sqlite_dir = unique_temp_dir("sqlite");
    let sqlite_backend = SqliteStorageBackend::new(&sqlite_dir);
    assert_backend_contract(
        &sqlite_backend,
        &sqlite_backend.paths().raw_log_path,
        &minimal,
        &conflict,
    );

    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

fn assert_backend_contract(
    backend: &impl StorageBackend,
    raw_log_path: &Path,
    minimal_request: &str,
    conflict_request: &str,
) {
    let minimal = finalized_request(minimal_request);
    let stored = backend
        .append_publish_request(minimal_request, &minimal)
        .expect("store new object");
    assert!(!stored.duplicate);
    let raw_after_store = raw_log_line_count(raw_log_path);
    assert_eq!(raw_after_store, 1);

    let duplicate = backend
        .append_publish_request(minimal_request, &minimal)
        .expect("accept exact duplicate");
    assert!(duplicate.duplicate);
    assert_eq!(raw_log_line_count(raw_log_path), raw_after_store);

    let conflict = finalized_request(conflict_request);
    let error = backend
        .append_publish_request(conflict_request, &conflict)
        .expect_err("reject conflicting object");
    assert_eq!(error.code, "LB_OBJECT_CONFLICT");
    assert_eq!(raw_log_line_count(raw_log_path), raw_after_store);

    let stored_record = backend
        .get(&minimal.canonical_id)
        .expect("read stored object")
        .expect("stored object exists");
    assert_eq!(stored_record.object, minimal.object);
}

fn finalized_request(request_json: &str) -> lingonberry_protocol::FinalizedKnowledgeObject {
    let request = parse_json(request_json).expect("request parses");
    let JsonValue::Object(request) = request else {
        panic!("request must be an object");
    };
    let object = request.get("object").expect("request contains object");
    finalize_knowledge_object_full(object).expect("object finalizes")
}

fn raw_log_line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .map(|contents| contents.lines().count())
        .unwrap_or_default()
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
        "lingonberry-duplicate-conflict-{label}-{}-{nonce}",
        std::process::id()
    ))
}
