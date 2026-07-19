#[path = "ingestion.rs"]
mod base;

use crate::{
    append_publish_request_classified, AppendOutcome, QuarantineStore, RawRequestRecord,
    StorageBackend, StoreError, StoredCatalogRecord, StoredReplayRecord,
};
use lingonberry_protocol::FinalizedKnowledgeObject;
use lingonberry_validation::AcceptancePolicy;

pub use base::{
    publish_ingestion_result_json, PublishIngestionResult, PublishIngestionStatus,
    PUBLISH_INGESTION_CONTRACT_VERSION,
};

pub fn ingest_publish_request(
    request_json: &str,
    backend: &impl StorageBackend,
    quarantine: &QuarantineStore,
    policy: &AcceptancePolicy,
) -> PublishIngestionResult {
    base::ingest_publish_request(
        request_json,
        &ClassifiedStorageBackend { inner: backend },
        quarantine,
        policy,
    )
}

struct ClassifiedStorageBackend<'a, B> {
    inner: &'a B,
}

impl<B: StorageBackend> StorageBackend for ClassifiedStorageBackend<'_, B> {
    fn append_publish_request(
        &self,
        request_json: &str,
        finalized: &FinalizedKnowledgeObject,
    ) -> Result<AppendOutcome, StoreError> {
        append_publish_request_classified(self.inner, request_json, finalized)
    }

    fn get(&self, canonical_id: &str) -> Result<Option<StoredCatalogRecord>, StoreError> {
        self.inner.get(canonical_id)
    }

    fn get_raw_request(&self, canonical_id: &str) -> Result<Option<RawRequestRecord>, StoreError> {
        self.inner.get_raw_request(canonical_id)
    }

    fn list_ids(&self) -> Result<Vec<String>, StoreError> {
        self.inner.list_ids()
    }

    fn subscribe(&self, object_type: Option<&str>) -> Result<Vec<StoredCatalogRecord>, StoreError> {
        self.inner.subscribe(object_type)
    }

    fn replay(&self) -> Result<Vec<StoredReplayRecord>, StoreError> {
        self.inner.replay()
    }
}
