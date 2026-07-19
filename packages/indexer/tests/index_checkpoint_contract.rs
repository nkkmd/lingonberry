use lingonberry_core::FileStorageBackend;
use lingonberry_indexer::{
    load_index_checkpoint, persist_index_checkpoint, rebuild_index, INDEX_CHECKPOINT_VERSION,
    INDEX_LIFECYCLE_CONTRACT_VERSION,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn checkpoint_round_trip_and_corruption_are_fail_closed() {
    let state_dir = unique_temp_dir();
    let backend = FileStorageBackend::new(state_dir.join("storage"));
    let checkpoint_path = state_dir.join("index/checkpoint.json");

    let result = rebuild_index(&backend);
    let persisted =
        persist_index_checkpoint(&checkpoint_path, &result).expect("persist checkpoint");
    assert_eq!(persisted.checkpoint_version, INDEX_CHECKPOINT_VERSION);
    assert_eq!(
        persisted.lifecycle_contract_version,
        INDEX_LIFECYCLE_CONTRACT_VERSION
    );

    let loaded = load_index_checkpoint(&checkpoint_path)
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(loaded, persisted);

    fs::write(&checkpoint_path, "{not-json").expect("corrupt checkpoint");
    let error = load_index_checkpoint(&checkpoint_path).expect_err("reject corruption");
    assert!(error.starts_with("LB_INDEX_CHECKPOINT_CORRUPT"), "{error}");

    fs::remove_dir_all(state_dir).ok();
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lingonberry-index-checkpoint-{}-{nonce}",
        std::process::id()
    ))
}
