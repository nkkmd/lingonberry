use lingonberry_core::{
    export_archive, import_archive, FileStorageBackend, SqliteStorageBackend, StorageBackend,
};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::finalize_knowledge_object_full;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn live_retry_is_idempotent_for_file_and_sqlite_backends() {
    let workspace = workspace_root();
    let request =
        fs::read_to_string(workspace.join("fixtures/http-publish-request/minimal-request.json"))
            .expect("read minimal request");
    let finalized = finalized_request(&request);

    let file_dir = unique_temp_dir("retry-file");
    assert_retry_contract(&FileStorageBackend::new(&file_dir), &request, &finalized);

    let sqlite_dir = unique_temp_dir("retry-sqlite");
    assert_retry_contract(
        &SqliteStorageBackend::new(&sqlite_dir),
        &request,
        &finalized,
    );

    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

#[test]
fn archive_import_is_idempotent_for_file_and_sqlite_backends() {
    let workspace = workspace_root();
    let request =
        fs::read_to_string(workspace.join("fixtures/http-publish-request/minimal-request.json"))
            .expect("read minimal request");
    let finalized = finalized_request(&request);

    let source_dir = unique_temp_dir("archive-source");
    let source = FileStorageBackend::new(&source_dir);
    source
        .append_publish_request(&request, &finalized)
        .expect("store archive source object");
    let archive_dir = unique_temp_dir("archive");
    export_archive(&source, &archive_dir).expect("export archive");

    let file_dir = unique_temp_dir("archive-file");
    assert_archive_retry_contract(&FileStorageBackend::new(&file_dir), &archive_dir);

    let sqlite_dir = unique_temp_dir("archive-sqlite");
    assert_archive_retry_contract(&SqliteStorageBackend::new(&sqlite_dir), &archive_dir);

    fs::remove_dir_all(source_dir).ok();
    fs::remove_dir_all(archive_dir).ok();
    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

#[test]
fn archive_import_conflict_preserves_existing_object_for_both_backends() {
    let workspace = workspace_root();
    let minimal_request =
        fs::read_to_string(workspace.join("fixtures/http-publish-request/minimal-request.json"))
            .expect("read minimal request");
    let minimal = finalized_request(&minimal_request);

    let source_dir = unique_temp_dir("conflict-source");
    let source = FileStorageBackend::new(&source_dir);
    source
        .append_publish_request(&minimal_request, &minimal)
        .expect("store archive source object");
    let archive_dir = unique_temp_dir("conflict-archive");
    export_archive(&source, &archive_dir).expect("export archive");

    let conflict_request = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/with-identity-claim.json"),
    )
    .expect("read conflicting request");
    let conflict = finalized_request(&conflict_request);

    let file_dir = unique_temp_dir("conflict-file");
    assert_archive_conflict_contract(
        &FileStorageBackend::new(&file_dir),
        &archive_dir,
        &conflict_request,
        &conflict,
    );

    let sqlite_dir = unique_temp_dir("conflict-sqlite");
    assert_archive_conflict_contract(
        &SqliteStorageBackend::new(&sqlite_dir),
        &archive_dir,
        &conflict_request,
        &conflict,
    );

    fs::remove_dir_all(source_dir).ok();
    fs::remove_dir_all(archive_dir).ok();
    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

fn assert_retry_contract(
    backend: &impl StorageBackend,
    request: &str,
    finalized: &lingonberry_protocol::FinalizedKnowledgeObject,
) {
    let stored = backend
        .append_publish_request(request, finalized)
        .expect("store initial request");
    assert!(!stored.duplicate);

    let retried = backend
        .append_publish_request(request, finalized)
        .expect("retry exact request");
    assert!(retried.duplicate);
    assert_eq!(retried.canonical_id, stored.canonical_id);
    assert_eq!(retried.object, stored.object);
}

fn assert_archive_retry_contract(backend: &impl StorageBackend, archive_dir: &Path) {
    let first = import_archive(backend, archive_dir).expect("first archive import");
    assert_eq!(first.record_count, 1);
    assert_eq!(first.duplicate_count, 0);

    let second = import_archive(backend, archive_dir).expect("retry archive import");
    assert_eq!(second.record_count, 0);
    assert_eq!(second.duplicate_count, 1);
    assert_eq!(backend.list_ids().expect("list imported ids").len(), 1);
}

fn assert_archive_conflict_contract(
    backend: &impl StorageBackend,
    archive_dir: &Path,
    conflict_request: &str,
    conflict: &lingonberry_protocol::FinalizedKnowledgeObject,
) {
    backend
        .append_publish_request(conflict_request, conflict)
        .expect("store conflicting target object");
    let before = backend
        .get(&conflict.canonical_id)
        .expect("read target before import")
        .expect("target object exists");

    let error = import_archive(backend, archive_dir).expect_err("reject archive conflict");
    assert_eq!(error.code, "LB_OBJECT_CONFLICT");

    let after = backend
        .get(&conflict.canonical_id)
        .expect("read target after import")
        .expect("target object still exists");
    assert_eq!(after.object, before.object);
    assert_eq!(after.carrier_identity, before.carrier_identity);
}

fn finalized_request(request_json: &str) -> lingonberry_protocol::FinalizedKnowledgeObject {
    let request = parse_json(request_json).expect("request parses");
    let JsonValue::Object(request) = request else {
        panic!("request must be an object");
    };
    let object = request.get("object").expect("request contains object");
    finalize_knowledge_object_full(object).expect("object finalizes")
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
        "lingonberry-archive-retry-{label}-{}-{nonce}",
        std::process::id()
    ))
}
