use lingonberry_core::{FileStorageBackend, StorageBackend};
use lingonberry_indexer::{
    persist_index_checkpoint, rebuild_index, verify_index, IndexConsistencyStatus, IndexSnapshot,
};
use lingonberry_protocol::{derive_identity_key, parse_json, to_canonical_json, JsonValue};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn ambiguous_index_does_not_replace_checkpoint() {
    let dir = temp_dir();
    let backend = FileStorageBackend::new(&dir);
    let raw = fs::read_to_string(root().join("fixtures/http-publish-request/minimal-request.json"))
        .unwrap();
    let finalized = finalize(&raw);
    backend.append_publish_request(&raw, &finalized).unwrap();

    let checkpoint = dir.join("index/checkpoint.json");
    persist_index_checkpoint(&checkpoint, &rebuild_index(&backend)).unwrap();
    let before = fs::read(&checkpoint).unwrap();

    let mut records = backend.subscribe(None).unwrap();
    let JsonValue::Object(object) = &mut records[0].object else {
        panic!()
    };
    object.insert("type".into(), JsonValue::String("ambiguous-smoke".into()));
    let result = verify_index(&backend, IndexSnapshot::from_records(records));

    assert_eq!(result.status, IndexConsistencyStatus::Inconsistent);
    assert_eq!(result.code, "LB_INDEX_AMBIGUOUS");
    assert_eq!(result.ambiguous_ids, vec![finalized.canonical_id]);
    let error = persist_index_checkpoint(&checkpoint, &result).unwrap_err();
    assert!(error.starts_with("LB_INDEX_CHECKPOINT_REFUSED:"));
    assert_eq!(fs::read(&checkpoint).unwrap(), before);
    fs::remove_dir_all(dir).ok();
}

fn finalize(raw: &str) -> lingonberry_protocol::FinalizedKnowledgeObject {
    let JsonValue::Object(request) = parse_json(raw).unwrap() else {
        panic!()
    };
    let object = request.get("object").unwrap().clone();
    let JsonValue::Object(map) = &object else {
        panic!()
    };
    let Some(JsonValue::String(canonical_id)) = map.get("id") else {
        panic!()
    };
    lingonberry_protocol::FinalizedKnowledgeObject {
        canonical_id: canonical_id.clone(),
        identity_key: derive_identity_key(&object),
        canonical_json: to_canonical_json(&object),
        object,
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lingonberry-ambiguous-smoke-{}-{nonce}",
        std::process::id()
    ))
}
