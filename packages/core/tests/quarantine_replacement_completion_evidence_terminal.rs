use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    advance_quarantine_replacement_transaction_journal, apply_quarantine_replacement_transaction,
    create_quarantine_replacement_preview, export_complete_quarantine_backup,
    prepare_quarantine_replacement_transaction, resume_quarantine_replacement_transaction,
    rollback_quarantine_replacement_transaction, QuarantineReplacementTransactionState,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE,
};

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn serial_test_guard() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK
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
    let state = temp_dir("terminal-evidence-state");
    let backup = temp_dir("terminal-evidence-backup");
    let proof = temp_dir("terminal-evidence-proof");
    let transaction = temp_dir("terminal-evidence-transaction");
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

fn evidence_pair(transaction: &Path) -> (String, String) {
    let evidence =
        fs::read_to_string(transaction.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE))
            .unwrap();
    let digest = fs::read_to_string(
        transaction.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_DIGEST_FILE),
    )
    .unwrap();
    (evidence, digest)
}

#[test]
fn committed_transaction_publishes_stable_completion_evidence() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();

    let first = apply_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-terminal-evidence-committed",
    )
    .unwrap();
    assert_eq!(
        first.state,
        QuarantineReplacementTransactionState::Committed
    );
    let first_pair = evidence_pair(&transaction);

    let resumed = resume_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        resumed.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(evidence_pair(&transaction), first_pair);

    cleanup([state, backup, proof, transaction]);
}

#[test]
fn rolled_back_transaction_publishes_stable_completion_evidence() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();

    prepare_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-terminal-evidence-rolled-back",
    )
    .unwrap();
    advance_quarantine_replacement_transaction_journal(
        &transaction,
        QuarantineReplacementTransactionState::RecoveryRequired,
    )
    .unwrap();

    let first = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        first.state,
        QuarantineReplacementTransactionState::RolledBack
    );
    let first_pair = evidence_pair(&transaction);

    let second = rollback_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        second.state,
        QuarantineReplacementTransactionState::RolledBack
    );
    assert_eq!(evidence_pair(&transaction), first_pair);

    cleanup([state, backup, proof, transaction]);
}

#[test]
fn tampered_terminal_completion_evidence_fails_closed_on_resume() {
    let _serial = serial_test_guard();
    let (state, backup, proof, transaction) = fixture();

    apply_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-terminal-evidence-tampered",
    )
    .unwrap();
    fs::write(
        transaction.join(QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_FILE),
        b"{}\n",
    )
    .unwrap();

    let error = resume_quarantine_replacement_transaction(&state, &transaction).unwrap_err();
    assert_eq!(
        error.code,
        "LB_QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE_PUBLICATION"
    );

    cleanup([state, backup, proof, transaction]);
}
