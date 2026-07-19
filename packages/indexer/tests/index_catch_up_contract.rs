use lingonberry_core::FileStorageBackend;
use lingonberry_indexer::{catch_up_index, IndexCatchUpStatus, INDEX_CATCH_UP_CONTRACT_VERSION};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn catch_up_rebuilds_then_reports_up_to_date_and_fails_closed() {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let state = std::env::temp_dir().join(format!("lingonberry-catch-up-{}-{nonce}", std::process::id()));
    let backend = FileStorageBackend::new(state.join("storage"));
    let checkpoint = state.join("index/checkpoint.json");

    let rebuilt = catch_up_index(&backend, &checkpoint);
    assert_eq!(rebuilt.contract_version, INDEX_CATCH_UP_CONTRACT_VERSION);
    assert_eq!(rebuilt.status, IndexCatchUpStatus::Rebuilt);
    assert_eq!(rebuilt.code, "LB_INDEX_REBUILT");

    let current = catch_up_index(&backend, &checkpoint);
    assert_eq!(current.status, IndexCatchUpStatus::UpToDate);
    assert_eq!(current.code, "LB_INDEX_UP_TO_DATE");

    fs::write(&checkpoint, "{not-json").unwrap();
    let corrupt = catch_up_index(&backend, &checkpoint);
    assert_eq!(corrupt.status, IndexCatchUpStatus::Failed);
    assert_eq!(corrupt.code, "LB_INDEX_CHECKPOINT_CORRUPT");
    assert_eq!(fs::read_to_string(&checkpoint).unwrap(), "{not-json");

    fs::remove_dir_all(state).ok();
}
