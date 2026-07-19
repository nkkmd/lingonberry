include!("lib_entry.rs");
#[path = "ingestion_classified.rs"]
#[rustfmt::skip]
pub mod ingestion;
pub mod classified_append;
pub mod duplicate_conflict;
pub mod quarantine_batch_classified;
pub mod quarantine_promotion_classified;

pub use classified_append::append_publish_request_classified;
pub use duplicate_conflict::{
    classify_duplicate_or_conflict, DuplicateConflictClassification, ExistingObjectIdentity,
    IncomingObjectIdentity, DUPLICATE_CONFLICT_CONTRACT_VERSION,
};
pub use ingestion::{
    ingest_publish_request, publish_ingestion_result_json, PublishIngestionResult,
    PublishIngestionStatus, PUBLISH_INGESTION_CONTRACT_VERSION,
};
pub use quarantine_batch_classified::promote_quarantine_batch_classified;
pub use quarantine_promotion_classified::promote_quarantine_record_classified;
