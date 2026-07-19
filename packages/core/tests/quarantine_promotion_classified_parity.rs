use lingonberry_core::{
    promote_quarantine_record_classified, FileStorageBackend, QuarantinePromotionOutcome,
    QuarantineStore, SqliteStorageBackend, StorageBackend,
};
use lingonberry_protocol::{parse_json, JsonValue};
use lingonberry_validation::finalize_knowledge_object_full;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn classified_quarantine_promotion_has_file_sqlite_parity() {
    let _guard = ENV_LOCK.lock().expect("lock environment");
    let workspace = workspace_root();
    let minimal_request =
        fs::read_to_string(workspace.join("fixtures/http-publish-request/minimal-request.json"))
            .expect("read minimal request");
    let conflicting_request = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/with-identity-claim.json"),
    )
    .expect("read conflicting request");

    let file_dir = unique_temp_dir("file");
    assert_promotion_contract(
        &FileStorageBackend::new(file_dir.join("storage")),
        &file_dir,
        &minimal_request,
        &conflicting_request,
    );

    let sqlite_dir = unique_temp_dir("sqlite");
    assert_promotion_contract(
        &SqliteStorageBackend::new(sqlite_dir.join("storage")),
        &sqlite_dir,
        &minimal_request,
        &conflicting_request,
    );

    fs::remove_dir_all(file_dir).ok();
    fs::remove_dir_all(sqlite_dir).ok();
}

fn assert_promotion_contract(
    backend: &impl StorageBackend,
    state_dir: &Path,
    minimal_request: &str,
    conflicting_request: &str,
) {
    std::env::set_var("LINGONBERRY_STATE_DIR", state_dir);
    let quarantine = QuarantineStore::new(state_dir);
    let minimal = finalized_request(minimal_request);

    backend
        .append_publish_request(minimal_request, &minimal)
        .expect("pre-store duplicate target");
    let duplicate_record = quarantine
        .append(
            minimal_request,
            "LB_TEST_DEFERRED",
            &["test duplicate promotion".to_string()],
        )
        .expect("append duplicate quarantine record");
    let duplicate = promote_quarantine_record_classified(&duplicate_record.id, backend)
        .expect("promote duplicate record");
    match duplicate {
        QuarantinePromotionOutcome::Promoted {
            canonical_id,
            duplicate,
            ..
        } => {
            assert!(duplicate);
            assert_eq!(canonical_id, minimal.canonical_id);
        }
        other => panic!("unexpected duplicate promotion outcome: {other:?}"),
    }
    let resolution = quarantine
        .get_resolution(&duplicate_record.id)
        .expect("read duplicate resolution")
        .expect("duplicate resolution exists");
    assert!(resolution.duplicate);

    let conflict = finalized_request(conflicting_request);
    let conflict_record = quarantine
        .append(
            conflicting_request,
            "LB_TEST_DEFERRED",
            &["test conflict promotion".to_string()],
        )
        .expect("append conflict quarantine record");
    let before = backend
        .get(&conflict.canonical_id)
        .expect("read conflict target")
        .expect("conflict target exists");
    let error = promote_quarantine_record_classified(&conflict_record.id, backend)
        .expect_err("reject conflicting promotion");
    assert_eq!(error.code, "LB_OBJECT_CONFLICT");
    assert!(quarantine
        .get_resolution(&conflict_record.id)
        .expect("read conflict resolution")
        .is_none());
    let after = backend
        .get(&conflict.canonical_id)
        .expect("read target after conflict")
        .expect("target remains");
    assert_eq!(after.object, before.object);
    assert_eq!(after.carrier_identity, before.carrier_identity);

    std::env::remove_var("LINGONBERRY_STATE_DIR");
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
        "lingonberry-quarantine-promotion-{label}-{}-{nonce}",
        std::process::id()
    ))
}
