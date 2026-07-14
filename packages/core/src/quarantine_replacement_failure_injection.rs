use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};

use crate::{store_error, StoreError};

pub const QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV: &str =
    "LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION";
pub const QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV: &str =
    "LINGONBERRY_REPLACEMENT_FAILURE_POINT";

pub const FAILURE_POINT_JOURNAL_WRITE: &str = "journal.write";
pub const FAILURE_POINT_JOURNAL_FSYNC: &str = "journal.fsync";
pub const FAILURE_POINT_STAGED_LEDGER_WRITE: &str = "staging.ledger-write";
pub const FAILURE_POINT_STAGED_LEDGER_FSYNC: &str = "staging.ledger-fsync";
pub const FAILURE_POINT_STAGING_DIRECTORY_FSYNC: &str = "staging.directory-fsync";
pub const FAILURE_POINT_GENERATION_MANIFEST_WRITE: &str = "generation.manifest-write";
pub const FAILURE_POINT_GENERATION_MANIFEST_FSYNC: &str = "generation.manifest-fsync";
pub const FAILURE_POINT_PUBLICATION_INTENT_WRITE: &str = "publication.intent-write";
pub const FAILURE_POINT_GENERATION_MATERIALIZE_RENAME: &str =
    "publication.generation-materialize-rename";
pub const FAILURE_POINT_POINTER_TEMPORARY_WRITE: &str = "publication.pointer-temporary-write";
pub const FAILURE_POINT_POINTER_RENAME: &str = "publication.pointer-rename";
pub const FAILURE_POINT_STATE_DIRECTORY_FSYNC: &str = "publication.state-directory-fsync";
pub const FAILURE_POINT_INDEX_REBUILD: &str = "publication.index-rebuild";
pub const FAILURE_POINT_INDEX_VERIFICATION: &str = "publication.index-verification";
pub const FAILURE_POINT_SEGMENT_VERIFICATION: &str = "publication.segment-verification";
pub const FAILURE_POINT_COMMIT_TRANSITION: &str = "publication.commit-transition";
pub const FAILURE_POINT_ROLLBACK_POINTER_RESTORE: &str = "rollback.pointer-restore";
pub const FAILURE_POINT_ROLLED_BACK_TRANSITION: &str = "rollback.rolled-back-transition";

pub const QUARANTINE_REPLACEMENT_FAILURE_POINTS: &[&str] = &[
    FAILURE_POINT_JOURNAL_WRITE,
    FAILURE_POINT_JOURNAL_FSYNC,
    FAILURE_POINT_STAGED_LEDGER_WRITE,
    FAILURE_POINT_STAGED_LEDGER_FSYNC,
    FAILURE_POINT_STAGING_DIRECTORY_FSYNC,
    FAILURE_POINT_GENERATION_MANIFEST_WRITE,
    FAILURE_POINT_GENERATION_MANIFEST_FSYNC,
    FAILURE_POINT_PUBLICATION_INTENT_WRITE,
    FAILURE_POINT_GENERATION_MATERIALIZE_RENAME,
    FAILURE_POINT_POINTER_TEMPORARY_WRITE,
    FAILURE_POINT_POINTER_RENAME,
    FAILURE_POINT_STATE_DIRECTORY_FSYNC,
    FAILURE_POINT_INDEX_REBUILD,
    FAILURE_POINT_INDEX_VERIFICATION,
    FAILURE_POINT_SEGMENT_VERIFICATION,
    FAILURE_POINT_COMMIT_TRANSITION,
    FAILURE_POINT_ROLLBACK_POINTER_RESTORE,
    FAILURE_POINT_ROLLED_BACK_TRANSITION,
];

static TRIGGERED_POINTS: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();

pub(crate) fn inject_quarantine_replacement_failure(point: &str) -> Result<(), StoreError> {
    if !QUARANTINE_REPLACEMENT_FAILURE_POINTS.contains(&point) {
        return Err(failure_injection_error("unknown replacement failure point"));
    }
    if std::env::var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV).as_deref() != Ok("1") {
        return Ok(());
    }

    let requested = match std::env::var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV) {
        Ok(requested) => requested,
        Err(_) => return Ok(()),
    };
    let triggered_point = match (point, requested.as_str()) {
        (FAILURE_POINT_POINTER_RENAME, FAILURE_POINT_POINTER_TEMPORARY_WRITE) => {
            FAILURE_POINT_POINTER_TEMPORARY_WRITE
        }
        (_, requested) if requested == point => point,
        _ => return Ok(()),
    };

    let mut triggered = TRIGGERED_POINTS
        .get_or_init(|| Mutex::new(BTreeSet::new()))
        .lock()
        .map_err(|_| failure_injection_error("failure-injection state lock is poisoned"))?;
    if !triggered.insert(triggered_point.to_string()) {
        return Ok(());
    }

    Err(failure_injection_error(&format!(
        "injected replacement failure at {triggered_point}"
    )))
}

fn failure_injection_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        std::env::remove_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV);
        std::env::remove_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV);
    }

    #[test]
    fn registry_is_unique_and_bounded() {
        let points = QUARANTINE_REPLACEMENT_FAILURE_POINTS
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        assert_eq!(points.len(), QUARANTINE_REPLACEMENT_FAILURE_POINTS.len());
        assert!(points.iter().all(|point| {
            !point.is_empty()
                && point.len() <= 64
                && point
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'.' || byte == b'-')
        }));
    }

    #[test]
    fn rejects_unknown_failure_point() {
        let error = inject_quarantine_replacement_failure("unknown.point").unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
    }

    #[test]
    fn temporary_pointer_write_uses_the_atomic_pointer_boundary() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        std::env::set_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV, "1");
        std::env::set_var(
            QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV,
            FAILURE_POINT_POINTER_TEMPORARY_WRITE,
        );

        let error = inject_quarantine_replacement_failure(FAILURE_POINT_POINTER_RENAME).unwrap_err();
        assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
        assert!(error.message.contains(FAILURE_POINT_POINTER_TEMPORARY_WRITE));
        assert!(inject_quarantine_replacement_failure(FAILURE_POINT_POINTER_RENAME).is_ok());
        clear_env();
    }

    #[test]
    fn requires_explicit_double_opt_in_and_triggers_once() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        assert!(inject_quarantine_replacement_failure(FAILURE_POINT_POINTER_RENAME).is_ok());

        std::env::set_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV, "1");
        std::env::set_var(
            QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV,
            FAILURE_POINT_POINTER_RENAME,
        );
        let first =
            inject_quarantine_replacement_failure(FAILURE_POINT_POINTER_RENAME).unwrap_err();
        assert_eq!(first.code, "LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION");
        assert!(inject_quarantine_replacement_failure(FAILURE_POINT_POINTER_RENAME).is_ok());
        clear_env();
    }
}
