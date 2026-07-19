include!("lib_entry.rs");
#[rustfmt::skip]
pub mod ingestion;
pub mod duplicate_conflict;

pub use duplicate_conflict::{
    classify_duplicate_or_conflict, DuplicateConflictClassification, ExistingObjectIdentity,
    IncomingObjectIdentity, DUPLICATE_CONFLICT_CONTRACT_VERSION,
};
pub use ingestion::{
    ingest_publish_request, publish_ingestion_result_json, PublishIngestionResult,
    PublishIngestionStatus, PUBLISH_INGESTION_CONTRACT_VERSION,
};
