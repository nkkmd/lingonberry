use std::collections::BTreeSet;
use std::sync::{Mutex, OnceLock};

use crate::{store_error, StoreError};

pub const QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV: &str =
    "LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION";
pub const QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV: &str =
    "LINGONBERRY_REPLACEMENT_FAILURE_POINT";

pub const FAILURE_POINT_POINTER_RENAME: &str = "publication.pointer-rename";
pub const FAILURE_POINT_INDEX_REBUILD: &str = "publication.index-rebuild";
pub const FAILURE_POINT_COMMIT_TRANSITION: &str = "publication.commit-transition";
pub const FAILURE_POINT_ROLLBACK_POINTER_RESTORE: &str = "rollback.pointer-restore";
pub const FAILURE_POINT_ROLLED_BACK_TRANSITION: &str = "rollback.rolled-back-transition";

static TRIGGERED_POINTS: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();

pub(crate) fn inject_quarantine_replacement_failure(point: &str) -> Result<(), StoreError> {
    if std::env::var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV).as_deref() != Ok("1") {
        return Ok(());
    }
    if std::env::var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV).as_deref() != Ok(point) {
        return Ok(());
    }

    let mut triggered = TRIGGERED_POINTS
        .get_or_init(|| Mutex::new(BTreeSet::new()))
        .lock()
        .map_err(|_| failure_injection_error("failure-injection state lock is poisoned"))?;
    if !triggered.insert(point.to_string()) {
        return Ok(());
    }

    Err(failure_injection_error(&format!(
        "injected replacement failure at {point}"
    )))
}

fn failure_injection_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_FAILURE_INJECTION", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn requires_explicit_double_opt_in_and_triggers_once() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV);
        std::env::remove_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV);
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

        std::env::remove_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_ENABLE_ENV);
        std::env::remove_var(QUARANTINE_REPLACEMENT_FAILURE_INJECTION_POINT_ENV);
    }
}
