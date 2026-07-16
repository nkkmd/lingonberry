use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    advance_quarantine_replacement_cleanup_transaction_journal,
    create_quarantine_replacement_cleanup_transaction_journal,
    move_quarantine_replacement_cleanup_to_tomb,
    read_quarantine_replacement_cleanup_transaction_details,
    resume_quarantine_replacement_cleanup_deletion,
    rollback_quarantine_replacement_cleanup_tomb, QuarantineReplacementCleanupPlan,
    QuarantineReplacementCleanupProof, QuarantineReplacementCleanupSubject,
    QuarantineReplacementCleanupTransactionJournal, QuarantineReplacementCleanupTransactionState,
    QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_FILE,
    QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION,
};

fn temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "lingonberry-cleanup-tomb-{label}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn proof() -> QuarantineReplacementCleanupProof {
    QuarantineReplacementCleanupProof {
        plan: QuarantineReplacementCleanupPlan {
            policy_version: QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION.to_string(),
            state_identity: "state-v1".to_string(),
            active_pointer_digest: "fnv1a64:1111111111111111".to_string(),
            runtime_fingerprint: "runtime-v1".to_string(),
            subjects: vec![QuarantineReplacementCleanupSubject {
                generation_id: "generation-old".to_string(),
                classification: "previous-committed-generation".to_string(),
                transaction_journal_digest: "fnv1a64:2222222222222222".to_string(),
                generation_digest: "fnv1a64:3333333333333333".to_string(),
                completion_evidence_digest: "fnv1a64:4444444444444444".to_string(),
                managed_paths: vec!["nested/b.txt".to_string(), "a.txt".to_string()],
            }],
        },
        plan_digest: "fnv1a64:5555555555555555".to_string(),
    }
}

fn prepare(label: &str) -> (PathBuf, PathBuf, QuarantineReplacementCleanupProof) {
    let root = temp_dir(label);
    let state = root.join("state");
    let transaction = root.join("cleanup-transaction");
    fs::create_dir_all(state.join("generation-old/nested")).unwrap();
    fs::write(state.join("generation-old/a.txt"), "a").unwrap();
    fs::write(state.join("generation-old/nested/b.txt"), "b").unwrap();
    let proof = proof();
    create_quarantine_replacement_cleanup_transaction_journal(
        &transaction,
        &QuarantineReplacementCleanupTransactionJournal {
            transaction_id: "cleanup-1".to_string(),
            state: QuarantineReplacementCleanupTransactionState::Prepared,
            sequence: 0,
            cleanup_proof_digest: proof.plan_digest.clone(),
            runtime_fingerprint: proof.plan.runtime_fingerprint.clone(),
            tomb_inventory_digest: None,
            deleted_paths: Vec::new(),
        },
    )
    .unwrap();
    advance_quarantine_replacement_cleanup_transaction_journal(
        &transaction,
        QuarantineReplacementCleanupTransactionState::Revalidated,
        None,
    )
    .unwrap();
    (state, transaction, proof)
}

#[test]
fn moves_seals_and_deletes_in_deterministic_order() {
    let (state, transaction, proof) = prepare("delete");
    let report = move_quarantine_replacement_cleanup_to_tomb(&state, &transaction, &proof).unwrap();
    assert_eq!(
        report.managed_paths,
        ["generation-old/a.txt", "generation-old/nested/b.txt"]
    );
    assert!(!state.join("generation-old/a.txt").exists());
    resume_quarantine_replacement_cleanup_deletion(&transaction).unwrap();
    let journal = read_quarantine_replacement_cleanup_transaction_details(&transaction).unwrap();
    assert_eq!(journal.state, QuarantineReplacementCleanupTransactionState::Committed);
    assert_eq!(journal.deleted_paths, report.managed_paths);
    assert!(!report.tomb_dir.join("generation-old/a.txt").exists());
    let _ = fs::remove_dir_all(state.parent().unwrap());
}

#[test]
fn rollback_restores_all_paths_before_deletion() {
    let (state, transaction, proof) = prepare("rollback");
    move_quarantine_replacement_cleanup_to_tomb(&state, &transaction, &proof).unwrap();
    rollback_quarantine_replacement_cleanup_tomb(&state, &transaction).unwrap();
    assert_eq!(fs::read_to_string(state.join("generation-old/a.txt")).unwrap(), "a");
    assert_eq!(
        fs::read_to_string(state.join("generation-old/nested/b.txt")).unwrap(),
        "b"
    );
    let journal = read_quarantine_replacement_cleanup_transaction_details(&transaction).unwrap();
    assert_eq!(journal.state, QuarantineReplacementCleanupTransactionState::RolledBack);
    let _ = fs::remove_dir_all(state.parent().unwrap());
}

#[test]
fn inventory_tamper_fails_closed_before_deletion() {
    let (state, transaction, proof) = prepare("tamper");
    move_quarantine_replacement_cleanup_to_tomb(&state, &transaction, &proof).unwrap();
    fs::write(
        transaction.join(QUARANTINE_REPLACEMENT_CLEANUP_TOMB_INVENTORY_FILE),
        "{}",
    )
    .unwrap();
    assert!(resume_quarantine_replacement_cleanup_deletion(&transaction).is_err());
    assert!(transaction.join("tomb/generation-old/a.txt").exists());
    let journal = read_quarantine_replacement_cleanup_transaction_details(&transaction).unwrap();
    assert_eq!(journal.state, QuarantineReplacementCleanupTransactionState::TombSealed);
    let _ = fs::remove_dir_all(state.parent().unwrap());
}
