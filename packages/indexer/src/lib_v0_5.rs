include!("lib.rs");
pub mod checkpoint;
pub mod lifecycle;
pub use checkpoint::{index_checkpoint_json, load_index_checkpoint, persist_index_checkpoint, IndexCheckpoint, INDEX_CHECKPOINT_VERSION};
pub use lifecycle::{index_rebuild_result_json, rebuild_index, verify_index, IndexConsistencyStatus, IndexGeneration, IndexRebuildResult, INDEX_LIFECYCLE_CONTRACT_VERSION};
