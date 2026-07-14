use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    advance_quarantine_replacement_transaction_journal, apply_quarantine_replacement_transaction,
    create_quarantine_replacement_preview, export_complete_quarantine_backup,
    read_quarantine_replacement_transaction_journal, resume_quarantine_replacement_transaction,
    rollback_quarantine_replacement_transaction, QuarantineReplacementTransactionState,
    QUARANTINE_CURRENT_GENERATION_POINTER_FILE, QUARANTINE_LEDGER_INDEX_FILE,
};

const FAILURE_ENABLE_ENV: &str = "LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION";
const FAILURE_POINT_ENV: &str = "LINGONBERRY_REPLACEMENT_FAILURE_POINT";
const POINTER_RENAME_FAILURE: &str = "publication.pointer-rename";
const INDEX_REBUILD_FAILURE: &str = "publication.index-rebuild";

static FAILURE_ENV_LOCK: Mutex<()> = Mutex::new(());

struct FailureInjectionGuard;

impl FailureInjectionGuard {
    fn new(point: &str) -> Self {
        std::env::set_var(FAILURE_ENABLE_ENV, "1");
        std::env::set_var(FAILURE_POINT_ENV, point);
        Self
    }
}

impl Drop for FailureInjectionGuard {
    fn drop(&mut self) {
        std::env::remove_var(FAILURE_ENABLE_ENV);
        std::env::remove_var(FAILURE_POINT_ENV);
    }
}

fn serial_test_guard() -> std::sync::MutexGuard<'static, ()> {
    FAILURE_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "lingonberry-{label}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn fixture() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let state = temp_dir("transaction-state");
    let backup = temp_dir("transaction-backup");
    let proof = temp_dir("transaction-proof");
    let transaction = temp_dir("transaction-workspace");
    fs::create_dir_all(&state).unwrap();
    fs::write(state.join("quarantine.jsonl"), b"{\"id\":\"q0\"}\n").unwrap();
    fs::write(
        state.join("quarantine-resolutions.jsonl"),
        b"{\"canonicalId\":\"c1\", \"quarantineId\":\"q1\"}\n",
    )
    .unwrap();
    export_complete_quarantine_backup(&state, &backup).unwrap();
    create_quarantine_replacement_preview(&state, &backup, &proof).unwrap();
    (state, backup, proof, transaction)
}

fn cleanup(paths: impl IntoIterator<Item = PathBuf>) {
    for path in paths {
        let _ = fs::remove_dir_all(path);
    }
}

#[test]
fn repeated_apply_and_resume_are_idempotent_after_commit() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    let first = apply_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-integration-idempotent",
    )
    .unwrap();
    let second = apply_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-integration-idempotent",
    )
    .unwrap();
    let resumed = resume_quarantine_replacement_transaction(&state, &transaction).unwrap();

    assert_eq!(
        first.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        second.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        resumed.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        resumed.active_generation.as_deref(),
        Some("tx-integration-idempotent")
    );
    cleanup([state, backup, proof, transaction]);
}

#[test]
fn resumes_after_failure_following_atomic_pointer_switch() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    fs::create_dir(state.join(QUARANTINE_LEDGER_INDEX_FILE)).unwrap();

    let error = apply_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-integration-after-switch",
    )
    .unwrap_err();
    assert!(matches!(
        error.code,
        "LB_QUARANTINE_IO" | "LB_QUARANTINE_REPLACEMENT_PUBLICATION"
    ));
    assert!(state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .is_file());
    assert_eq!(
        read_quarantine_replacement_transaction_journal(&transaction)
            .unwrap()
            .state,
        QuarantineReplacementTransactionState::RecoveryRequired
    );

    fs::remove_dir(state.join(QUARANTINE_LEDGER_INDEX_FILE)).unwrap();
    let report = resume_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        report.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        report.active_generation.as_deref(),
        Some("tx-integration-after-switch")
    );
    cleanup([state, backup, proof, transaction]);
}

#[test]
fn injected_pointer_rename_failure_preserves_legacy_root_and_resumes() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    {
        let _failure = FailureInjectionGuard::new(POINTER_RENAME_FAILURE);
        let error = apply_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-integration-pointer-rename-failure",
        )
        .unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
    }

    assert!(!state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .exists());
    assert!(state.join("quarantine.jsonl").is_file());
    assert_eq!(
        read_quarantine_replacement_transaction_journal(&transaction)
            .unwrap()
            .state,
        QuarantineReplacementTransactionState::RecoveryRequired
    );

    let report = resume_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        report.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        report.active_generation.as_deref(),
        Some("tx-integration-pointer-rename-failure")
    );
    cleanup([state, backup, proof, transaction]);
}

#[test]
fn injected_index_rebuild_failure_is_resumable_after_switch() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    {
        let _failure = FailureInjectionGuard::new(INDEX_REBUILD_FAILURE);
        let error = apply_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-integration-index-rebuild-failure",
        )
        .unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
    }

    assert!(state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .is_file());
    assert_eq!(
        read_quarantine_replacement_transaction_journal(&transaction)
            .unwrap()
            .state,
        QuarantineReplacementTransactionState::RecoveryRequired
    );

    let report = resume_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        report.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        report.active_generation.as_deref(),
        Some("tx-integration-index-rebuild-failure")
    );
    cleanup([state, backup, proof, transaction]);
}

#[test]
fn rollback_before_publication_is_idempotent_and_preserves_legacy_root() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    lingonberry_core::prepare_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-integration-rollback",
    )
    .unwrap();
    advance_quarantine_replacement_transaction_journal(
        &transaction,
        QuarantineReplacementTransactionState::RecoveryRequired,
    )
    .unwrap();

    let first = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap();
    let second = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        first.state,
        QuarantineReplacementTransactionState::RolledBack
    );
    assert_eq!(
        second.state,
        QuarantineReplacementTransactionState::RolledBack
    );
    assert!(!state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .exists());
    assert!(state.join("quarantine.jsonl").is_file());
    cleanup([state, backup, proof, transaction]);
}
