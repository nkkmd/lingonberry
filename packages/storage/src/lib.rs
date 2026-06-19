use lingonberry_core::{runtime_state_dir, SqliteStorageBackend};
use std::path::Path;

pub fn build_storage_backend() -> SqliteStorageBackend {
    SqliteStorageBackend::new(runtime_state_dir())
}

pub fn build_storage_backend_at(base_dir: impl AsRef<Path>) -> SqliteStorageBackend {
    SqliteStorageBackend::new(base_dir)
}
