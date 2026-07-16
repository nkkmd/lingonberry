use lingonberry_core::{
    build_quarantine_replacement_cleanup_plan, quarantine_replacement_cleanup_plan_json,
    QuarantineReplacementCleanupSubject, QuarantineReplacementRetentionDecision,
    QuarantineReplacementRetentionDecisionReport, QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION,
};

fn decisions(ids: &[&str]) -> QuarantineReplacementRetentionDecisionReport {
    QuarantineReplacementRetentionDecisionReport {
        policy_version: QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION.to_string(),
        decisions: ids
            .iter()
            .map(|id| QuarantineReplacementRetentionDecision {
                generation_id: (*id).to_string(),
                classification: "previous-committed-generation".to_string(),
                eligible: true,
                reason_code: "eligible".to_string(),
            })
            .collect(),
    }
}

fn subject(id: &str, paths: &[&str]) -> QuarantineReplacementCleanupSubject {
    QuarantineReplacementCleanupSubject {
        generation_id: id.to_string(),
        classification: "previous-committed-generation".to_string(),
        transaction_journal_digest: "fnv1a64:1111111111111111".to_string(),
        generation_digest: "fnv1a64:2222222222222222".to_string(),
        completion_evidence_digest: "fnv1a64:3333333333333333".to_string(),
        managed_paths: paths.iter().map(|path| (*path).to_string()).collect(),
    }
}

fn build(
    report: &QuarantineReplacementRetentionDecisionReport,
    subjects: Vec<QuarantineReplacementCleanupSubject>,
) -> Result<lingonberry_core::QuarantineReplacementCleanupPlan, lingonberry_core::StoreError> {
    build_quarantine_replacement_cleanup_plan(
        report,
        "state-v1",
        "fnv1a64:4444444444444444",
        "runtime-v1",
        subjects,
    )
}

#[test]
fn normalizes_subject_and_managed_path_order_deterministically() {
    let report = decisions(&["tx-b", "tx-a"]);
    let first = build(
        &report,
        vec![
            subject("tx-b", &["z/file", "a/file"]),
            subject("tx-a", &["b/file", "a/file"]),
        ],
    )
    .unwrap();
    let second = build(
        &report,
        vec![
            subject("tx-a", &["a/file", "b/file"]),
            subject("tx-b", &["a/file", "z/file"]),
        ],
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.subjects[0].generation_id, "tx-a");
    assert_eq!(first.subjects[0].managed_paths, ["a/file", "b/file"]);
    assert_eq!(
        quarantine_replacement_cleanup_plan_json(&first),
        quarantine_replacement_cleanup_plan_json(&second)
    );
}

#[test]
fn rejects_incomplete_or_duplicate_eligible_subject_binding() {
    let report = decisions(&["tx-a", "tx-b"]);
    assert!(build(&report, vec![subject("tx-a", &["a/file"])]).is_err());
    assert!(build(
        &report,
        vec![subject("tx-a", &["a/file"]), subject("tx-a", &["b/file"])],
    )
    .is_err());
}

#[test]
fn rejects_traversal_absolute_and_duplicate_managed_paths() {
    let report = decisions(&["tx-a"]);
    for invalid in ["../escape", "a/../escape", "/absolute", "a//file", "a/./file"] {
        assert!(build(&report, vec![subject("tx-a", &[invalid])]).is_err());
    }
    assert!(build(
        &report,
        vec![subject("tx-a", &["a/file", "a/file"])],
    )
    .is_err());
}

#[test]
fn rejects_unapproved_subject_and_invalid_generation_id() {
    let report = decisions(&["tx-a"]);
    assert!(build(&report, vec![subject("tx-b", &["a/file"])]).is_err());
    assert!(build(&report, vec![subject("../tx-a", &["a/file"])]).is_err());
}
