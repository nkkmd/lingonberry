use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    apply_quarantine_replacement_transaction, create_quarantine_replacement_preview,
    export_complete_quarantine_backup, read_quarantine_replacement_transaction_journal,
    QuarantineReplacementTransactionState, QUARANTINE_REPLACEMENT_PUBLICATION_DIR,
};

const FAILURE_ENABLE_ENV: &str = "LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION";
const FAILURE_POINT_ENV: &str = "LINGONBERRY_REPLACEMENT_FAILURE_POINT";
const GENERATION_MANIFEST_WRITE_FAILURE: &str = "generation.manifest-write";
const GENERATION_MANIFEST_FSYNC_FAILURE: &str = "generation.manifest-fsync";

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
    let state = temp_dir("generation-failure-state");
    let backup = temp_dir("generation-failure-backup");
    let proof = temp_dir("generation-failure-proof");
    let transaction = temp_dir("generation-failure-transaction");
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
fn injected_generation_failures_remove_publication_state_and_retry_to_committed() {
    let _serial = FAILURE_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    for (point, transaction_id, expected_state) in [
        (
            GENERATION_MANIFEST_WRITE_FAILURE,
            "tx-generation-manifest-write",
            QuarantineReplacementTransactionState::Verified,
        ),
        (
            GENERATION_MANIFEST_FSYNC_FAILURE,
            "tx-generation-manifest-fsync",
            QuarantineReplacementTransactionState::RecoveryRequired,
        ),
    ] {
        let (state, backup, proof, transaction) = fixture();
        {
            let _failure = FailureInjectionGuard::new(point);
            let error = apply_quarantine_replacement_transaction(
                &state,
                &backup,
                &proof,
                &transaction,
                transaction_id,
            )
            .unwrap_err();
            assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
        }

        assert!(!transaction
            .join(QUARANTINE_REPLACEMENT_PUBLICATION_DIR)
            .exists());
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            expected_state,
            "unexpected journal state after {point}"
        );

        let report = apply_quarantine_replacement_transaction(
            &state,
            &backup,
            &proof,
            &transaction,
            transaction_id,
        )
        .unwrap();
        assert_eq!(
            report.state,
            QuarantineReplacementTransactionState::Committed,
            "failure point {point} did not recover"
        );
        cleanup([state, backup, proof, transaction]);
    }
}
