use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{store_error, StoreError};

pub const QUARANTINE_LOCK_FILE: &str = ".quarantine-operation.lock";
pub const QUARANTINE_LOCK_STALE_AFTER: Duration = Duration::from_secs(15 * 60);

#[derive(Debug)]
pub struct QuarantineOperationLock {
    path: PathBuf,
}

impl Drop for QuarantineOperationLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn acquire_quarantine_lock(
    state_dir: impl AsRef<Path>,
    operation: &str,
) -> Result<QuarantineOperationLock, StoreError> {
    let state_dir = state_dir.as_ref();
    fs::create_dir_all(state_dir)
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    let path = state_dir.join(QUARANTINE_LOCK_FILE);
    let operation = sanitize_operation(operation)?;

    for attempt in 0..=1 {
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                let acquired_at = now_seconds()?;
                writeln!(
                    file,
                    "operation={operation}\npid={}\nacquiredAt={acquired_at}",
                    std::process::id()
                )
                .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
                file.sync_all()
                    .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
                return Ok(QuarantineOperationLock { path });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if attempt == 0 && lock_is_stale(&path)? {
                    match fs::remove_file(&path) {
                        Ok(()) => continue,
                        Err(remove_error)
                            if remove_error.kind() == std::io::ErrorKind::NotFound =>
                        {
                            continue
                        }
                        Err(remove_error) => {
                            return Err(store_error("LB_QUARANTINE_IO", remove_error.to_string()))
                        }
                    }
                }
                return Err(store_error(
                    "LB_QUARANTINE_BUSY",
                    format!("another quarantine operation holds {}", path.display()),
                ));
            }
            Err(error) => {
                return Err(store_error("LB_QUARANTINE_IO", error.to_string()));
            }
        }
    }
    Err(store_error(
        "LB_QUARANTINE_BUSY",
        "failed to acquire quarantine operation lock",
    ))
}

fn sanitize_operation(operation: &str) -> Result<String, StoreError> {
    let operation = operation.trim();
    if operation.is_empty()
        || operation.len() > 64
        || !operation.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        return Err(store_error(
            "LB_QUARANTINE_LOCK",
            "operation must be a bounded ASCII identifier",
        ));
    }
    Ok(operation.to_string())
}

fn lock_is_stale(path: &Path) -> Result<bool, StoreError> {
    if let Ok(contents) = fs::read_to_string(path) {
        if let Some(acquired_at) = contents
            .lines()
            .find_map(|line| line.strip_prefix("acquiredAt="))
            .and_then(|value| value.parse::<u64>().ok())
        {
            return Ok(
                now_seconds()?.saturating_sub(acquired_at) >= QUARANTINE_LOCK_STALE_AFTER.as_secs()
            );
        }
    }
    let metadata =
        fs::metadata(path).map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    let modified = metadata
        .modified()
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))?;
    Ok(SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default()
        >= QUARANTINE_LOCK_STALE_AFTER)
}

fn now_seconds() -> Result<u64, StoreError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| store_error("LB_QUARANTINE_IO", error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-quarantine-lock-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn excludes_concurrent_holders_and_releases_on_drop() {
        let dir = temp_dir();
        let first = acquire_quarantine_lock(&dir, "first-operation").unwrap();
        assert_eq!(
            acquire_quarantine_lock(&dir, "second-operation")
                .unwrap_err()
                .code,
            "LB_QUARANTINE_BUSY"
        );
        drop(first);
        acquire_quarantine_lock(&dir, "second-operation").unwrap();
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn recovers_stale_lock_once() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(QUARANTINE_LOCK_FILE),
            "operation=crashed\npid=1\nacquiredAt=0\n",
        )
        .unwrap();
        let guard = acquire_quarantine_lock(&dir, "replacement").unwrap();
        let metadata = fs::read_to_string(dir.join(QUARANTINE_LOCK_FILE)).unwrap();
        assert!(metadata.contains("operation=replacement"));
        drop(guard);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn metadata_is_bounded_and_contains_no_payload_fields() {
        let dir = temp_dir();
        let guard = acquire_quarantine_lock(&dir, "backup-export").unwrap();
        let metadata = fs::read_to_string(dir.join(QUARANTINE_LOCK_FILE)).unwrap();
        assert!(metadata.contains("operation=backup-export"));
        assert!(metadata.contains("pid="));
        assert!(metadata.contains("acquiredAt="));
        assert!(!metadata.contains("note"));
        assert!(!metadata.contains("token"));
        assert!(!metadata.contains("payload"));
        drop(guard);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_unbounded_operation_metadata() {
        let dir = temp_dir();
        assert_eq!(
            acquire_quarantine_lock(&dir, "contains whitespace")
                .unwrap_err()
                .code,
            "LB_QUARANTINE_LOCK"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
