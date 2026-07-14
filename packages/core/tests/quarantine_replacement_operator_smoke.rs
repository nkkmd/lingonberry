use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    apply_quarantine_replacement_transaction, create_quarantine_replacement_preview,
    export_complete_quarantine_backup, inspect_quarantine_replacement_generations,
    quarantine_replacement_metrics_text, quarantine_replacement_status,
    resume_quarantine_replacement_transaction, verify_any_quarantine_backup,
    verify_quarantine_ledger_index, verify_quarantine_replacement_proof,
    verify_quarantine_segments, QuarantineReplacementTransactionState,
};

fn temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "lingonberry-{label}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn operator_smoke_covers_backup_preview_apply_observe_and_verify() {
    let state = temp_dir("operator-smoke-state");
    let backup = temp_dir("operator-smoke-backup");
    let proof = temp_dir("operator-smoke-proof");
    let transaction = temp_dir("operator-smoke-transaction");
    fs::create_dir_all(&state).unwrap();
    fs::write(state.join("quarantine.jsonl"), b"{\"id\":\"q0\"}\n").unwrap();
    fs::write(
        state.join("quarantine-resolutions.jsonl"),
        b"{\"canonicalId\":\"c1\", \"quarantineId\":\"q1\"}\n",
    )
    .unwrap();

    export_complete_quarantine_backup(&state, &backup).unwrap();
    verify_any_quarantine_backup(&backup).unwrap();
    create_quarantine_replacement_preview(&state, &backup, &proof).unwrap();
    verify_quarantine_replacement_proof(&proof).unwrap();

    let applied = apply_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-operator-smoke",
    )
    .unwrap();
    assert_eq!(
        applied.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        applied.active_generation.as_deref(),
        Some("tx-operator-smoke")
    );

    let status = quarantine_replacement_status(&state, &transaction).unwrap();
    assert_eq!(
        status.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(status.classification, "committed");

    let metrics = quarantine_replacement_metrics_text(&status);
    assert!(metrics.contains("lingonberry_quarantine_replacement_transactions"));
    assert!(metrics.contains("state=\"committed\""));
    assert!(!metrics.contains("tx-operator-smoke"));

    verify_quarantine_ledger_index(&state).unwrap();
    verify_quarantine_segments(&state).unwrap();

    let retention =
        inspect_quarantine_replacement_generations(&state, std::slice::from_ref(&transaction))
            .unwrap();
    assert_eq!(retention.layout, "generation");
    assert_eq!(retention.generations.len(), 1);
    assert_eq!(
        retention.generations[0].classification,
        "active-committed-generation"
    );
    assert!(!retention.generations[0].manual_review_required);

    let repeated = apply_quarantine_replacement_transaction(
        &state,
        &backup,
        &proof,
        &transaction,
        "tx-operator-smoke",
    )
    .unwrap();
    let resumed = resume_quarantine_replacement_transaction(&state, &transaction).unwrap();
    assert_eq!(
        repeated.state,
        QuarantineReplacementTransactionState::Committed
    );
    assert_eq!(
        resumed.state,
        QuarantineReplacementTransactionState::Committed
    );

    let _ = fs::remove_dir_all(state);
    let _ = fs::remove_dir_all(backup);
    let _ = fs::remove_dir_all(proof);
    let _ = fs::remove_dir_all(transaction);
}
