use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    apply_quarantine_replacement_transaction, create_quarantine_replacement_preview,
    export_complete_quarantine_backup, read_quarantine_replacement_transaction_journal,
    resume_quarantine_replacement_transaction, rollback_quarantine_replacement_transaction,
    QuarantineReplacementTransactionState, QUARANTINE_CURRENT_GENERATION_POINTER_FILE,
};

const FAILURE_ENABLE_ENV: &str = "LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION";
const FAILURE_POINT_ENV: &str = "LINGONBERRY_REPLACEMENT_FAILURE_POINT";
const INDEX_REBUILD_FAILURE: &str = "publication.index-rebuild";
const COMMIT_TRANSITION_FAILURE: &str = "publication.commit-transition";
const ROLLBACK_POINTER_RESTORE_FAILURE: &str = "rollback.pointer-restore";
const ROLLED_BACK_TRANSITION_FAILURE: &str = "rollback.rolled-back-transition";

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
    let state = temp_dir("crash-matrix-state");
    let backup = temp_dir("crash-matrix-backup");
    let proof = temp_dir("crash-matrix-proof");
    let transaction = temp_dir("crash-matrix-transaction");
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

fn leave_target_active_recovery_required(
    state: &PathBuf,
    backup: &PathBuf,
    proof: &PathBuf,
    transaction: &PathBuf,
    transaction_id: &str,
) {
    let _failure = FailureInjectionGuard::new(INDEX_REBUILD_FAILURE);
    let error =
        apply_quarantine_replacement_transaction(state, backup, proof, transaction, transaction_id)
            .unwrap_err();
    assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
    assert!(state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .is_file());
    assert_eq!(
        read_quarantine_replacement_transaction_journal(transaction)
            .unwrap()
            .state,
        QuarantineReplacementTransactionState::RecoveryRequired
    );
}

fn cleanup(paths: impl IntoIterator<Item = PathBuf>) {
    for path in paths {
        let _ = fs::remove_dir_all(path);
    }
}

#[test]
fn injected_commit_transition_failure_resumes_without_second_switch() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    {
        let _failure = FailureInjectionGuard::new(COMMIT_TRANSITION_FAILURE);
        let error = apply_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            "tx-crash-commit-transition",
        )
        .unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
    }

    let pointer_before = fs::read(state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)).unwrap();
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
        pointer_before,
        fs::read(state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)).unwrap()
    );
    cleanup([state, backup, proof, transaction]);
}

#[test]
fn injected_rollback_pointer_restore_failure_keeps_target_until_retry() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    leave_target_active_recovery_required(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-crash-rollback-pointer",
    );
    let target_pointer = fs::read(state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)).unwrap();

    {
        let _failure = FailureInjectionGuard::new(ROLLBACK_POINTER_RESTORE_FAILURE);
        let error = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
    }
    assert_eq!(
        target_pointer,
        fs::read(state.join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)).unwrap()
    );
    assert_eq!(
        read_quarantine_replacement_transaction_journal(&transaction)
            .unwrap()
            .state,
        QuarantineReplacementTransactionState::RecoveryRequired
    );

    let report = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        report.state,
        QuarantineReplacementTransactionState::RolledBack
    );
    assert!(!state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .exists());
    cleanup([state, backup, proof, transaction]);
}

#[test]
fn injected_rolled_back_transition_failure_retries_after_pointer_restore() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();
    leave_target_active_recovery_required(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-crash-rolled-back-transition",
    );

    {
        let _failure = FailureInjectionGuard::new(ROLLED_BACK_TRANSITION_FAILURE);
        let error = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
    }
    assert!(!state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .exists());
    assert_eq!(
        read_quarantine_replacement_transaction_journal(&transaction)
            .unwrap()
            .state,
        QuarantineReplacementTransactionState::RecoveryRequired
    );

    let report = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        report.state,
        QuarantineReplacementTransactionState::RolledBack
    );
    assert!(!state
        .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
        .exists());
    cleanup([state, backup, proof, transaction]);
}
