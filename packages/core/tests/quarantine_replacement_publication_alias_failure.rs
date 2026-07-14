use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    apply_quarantine_replacement_transaction, create_quarantine_replacement_preview,
    export_complete_quarantine_backup, read_quarantine_replacement_transaction_journal,
    resume_quarantine_replacement_transaction, QuarantineReplacementTransactionState,
    QUARANTINE_CURRENT_GENERATION_POINTER_FILE,
};

const FAILURE_ENABLE_ENV: &str = "LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION";
const FAILURE_POINT_ENV: &str = "LINGONBERRY_REPLACEMENT_FAILURE_POINT";
const PUBLICATION_INTENT_WRITE_FAILURE: &str = "publication.intent-write";
const GENERATION_MATERIALIZE_RENAME_FAILURE: &str = "publication.generation-materialize-rename";
const STATE_DIRECTORY_FSYNC_FAILURE: &str = "publication.state-directory-fsync";
const INDEX_VERIFICATION_FAILURE: &str = "publication.index-verification";
const SEGMENT_VERIFICATION_FAILURE: &str = "publication.segment-verification";

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
    let state = temp_dir("publication-alias-state");
    let backup = temp_dir("publication-alias-backup");
    let proof = temp_dir("publication-alias-proof");
    let transaction = temp_dir("publication-alias-transaction");
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
fn injected_publication_post_boundary_failures_resume_to_committed() {
    let _serial = FAILURE_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    for (point, transaction_id, pointer_visible) in [
        (
            PUBLICATION_INTENT_WRITE_FAILURE,
            "tx-publication-intent-write",
            false,
        ),
        (
            GENERATION_MATERIALIZE_RENAME_FAILURE,
            "tx-generation-materialize-rename",
            false,
        ),
        (
            STATE_DIRECTORY_FSYNC_FAILURE,
            "tx-state-directory-fsync",
            true,
        ),
        (INDEX_VERIFICATION_FAILURE, "tx-index-verification", true),
        (
            SEGMENT_VERIFICATION_FAILURE,
            "tx-segment-verification",
            true,
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

        assert_eq!(
            state
                .join(QUARANTINE_CURRENT_GENERATION_POINTER_FILE)
                .exists(),
            pointer_visible,
            "unexpected pointer visibility after {point}"
        );
        assert_eq!(
            read_quarantine_replacement_transaction_journal(&transaction)
                .unwrap()
                .state,
            QuarantineReplacementTransactionState::RecoveryRequired,
            "unexpected journal state after {point}"
        );

        let report = resume_quarantine_replacement_transaction(&state, &transaction).unwrap();
        assert_eq!(
            report.state,
            QuarantineReplacementTransactionState::Committed,
            "failure point {point} did not recover"
        );
        cleanup([state, backup, proof, transaction]);
    }
}
