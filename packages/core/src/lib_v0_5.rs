include!("lib_entry.rs");
#[path = "ingestion_classified.rs"]
#[rustfmt::skip]
pub mod ingestion;
pub mod archive_import_classified;
pub mod classified_append;
pub mod duplicate_conflict;
pub mod object_retrieval;
pub mod quarantine_batch_classified;
pub mod quarantine_promotion_classified;
pub mod query_contract;

pub use archive_import_classified::import_archive_classified;
pub use classified_append::append_publish_request_classified;
pub use duplicate_conflict::{
    classify_duplicate_or_conflict, DuplicateConflictClassification, ExistingObjectIdentity,
    IncomingObjectIdentity, DUPLICATE_CONFLICT_CONTRACT_VERSION,
};
pub use ingestion::{
    ingest_publish_request, publish_ingestion_result_json, PublishIngestionResult,
    PublishIngestionStatus, PUBLISH_INGESTION_CONTRACT_VERSION,
};
pub use object_retrieval::{
    object_retrieval_result_json, retrieve_object, ObjectRetrievalResult, ObjectRetrievalStatus,
    OBJECT_RETRIEVAL_CONTRACT_VERSION,
};
pub use quarantine_batch_classified::promote_quarantine_batch_classified;
pub use quarantine_promotion_classified::promote_quarantine_record_classified;
pub use query_contract::{
    basic_query_result_json, execute_basic_query, BasicQueryResult, BasicQueryStatus,
    BASIC_QUERY_CONTRACT_VERSION,
};
