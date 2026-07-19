include!("lib.rs");

pub mod checkpoint;
pub use checkpoint::{
    index_checkpoint_json, load_index_checkpoint, persist_index_checkpoint, IndexCheckpoint,
    INDEX_CHECKPOINT_VERSION,
};

pub mod catch_up;
pub use catch_up::{
    catch_up_index, index_catch_up_result_json, IndexCatchUpResult, IndexCatchUpStatus,
    INDEX_CATCH_UP_CONTRACT_VERSION,
};
