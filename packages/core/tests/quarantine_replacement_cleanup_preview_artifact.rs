use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    build_quarantine_replacement_cleanup_plan,
    publish_quarantine_replacement_cleanup_preview_artifacts,
    verify_quarantine_replacement_cleanup_preview_artifacts,
    QuarantineReplacementCleanupPlan, QuarantineReplacementCleanupSubject,
    QuarantineReplacementRetentionDecision, QuarantineReplacementRetentionDecisionReport,
    QUARANTINE_REPLACEMENT_CLEANUP_PLAN_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_CLEANUP_PLAN_FILE,
    QUARANTINE_REPLACEMENT_CLEANUP_PROOF_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_CLEANUP_PROOF_FILE,
    QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION,
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

fn plan() -> QuarantineReplacementCleanupPlan {
    let decisions = QuarantineReplacementRetentionDecisionReport {
        policy_version: QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION.to_string(),
        decisions: vec![QuarantineReplacementRetentionDecision {
            generation_id: "tx-a".to_string(),
            classification: "previous-committed-generation".to_string(),
            eligible: true,
            reason_code: "eligible".to_string(),
        }],
    };
    build_quarantine_replacement_cleanup_plan(
        &decisions,
        "state-v1",
        "fnv1a64:4444444444444444",
        "runtime-v1",
        vec![QuarantineReplacementCleanupSubject {
            generation_id: "tx-a".to_string(),
            classification: "previous-committed-generation".to_string(),
            transaction_journal_digest: "fnv1a64:1111111111111111".to_string(),
            generation_digest: "fnv1a64:2222222222222222".to_string(),
            completion_evidence_digest: "fnv1a64:3333333333333333".to_string(),
            managed_paths: vec!["generations/tx-a".to_string()],
        }],
    )
    .unwrap()
}

#[test]
fn publishes_complete_artifact_set_idempotently() {
    let dir = temp_dir("cleanup-preview-artifact");
    let plan = plan();

    let first = publish_quarantine_replacement_cleanup_preview_artifacts(&dir, &plan).unwrap();
    let second = publish_quarantine_replacement_cleanup_preview_artifacts(&dir, &plan).unwrap();

    assert_eq!(first, second);
    verify_quarantine_replacement_cleanup_preview_artifacts(&dir, &first).unwrap();
    for name in [
        QUARANTINE_REPLACEMENT_CLEANUP_PLAN_FILE,
        QUARANTINE_REPLACEMENT_CLEANUP_PLAN_DIGEST_FILE,
        QUARANTINE_REPLACEMENT_CLEANUP_PROOF_FILE,
        QUARANTINE_REPLACEMENT_CLEANUP_PROOF_DIGEST_FILE,
    ] {
        assert!(dir.join(name).is_file());
    }
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn rejects_tampered_or_partial_artifacts() {
    let dir = temp_dir("cleanup-preview-tamper");
    let proof = publish_quarantine_replacement_cleanup_preview_artifacts(&dir, &plan()).unwrap();
    fs::write(dir.join(QUARANTINE_REPLACEMENT_CLEANUP_PLAN_FILE), "{}") .unwrap();
    assert!(verify_quarantine_replacement_cleanup_preview_artifacts(&dir, &proof).is_err());

    let _ = fs::remove_dir_all(&dir);
    let proof = publish_quarantine_replacement_cleanup_preview_artifacts(&dir, &plan()).unwrap();
    fs::remove_file(dir.join(QUARANTINE_REPLACEMENT_CLEANUP_PROOF_DIGEST_FILE)).unwrap();
    assert!(verify_quarantine_replacement_cleanup_preview_artifacts(&dir, &proof).is_err());
    assert!(publish_quarantine_replacement_cleanup_preview_artifacts(&dir, &plan()).is_err());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn rejects_stale_temporary_artifact() {
    let dir = temp_dir("cleanup-preview-stale-temp");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(".quarantine-replacement-cleanup-plan.json.tmp"),
        "stale",
    )
    .unwrap();

    assert!(publish_quarantine_replacement_cleanup_preview_artifacts(&dir, &plan()).is_err());
    let _ = fs::remove_dir_all(dir);
}
