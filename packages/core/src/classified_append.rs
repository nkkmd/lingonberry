use crate::{
    carrier_identity_for_request, classify_duplicate_or_conflict, store_error, AppendOutcome,
    DuplicateConflictClassification, ExistingObjectIdentity, IncomingObjectIdentity, StorageBackend,
    StoreError, DUPLICATE_CONFLICT_CONTRACT_VERSION,
};
use lingonberry_protocol::FinalizedKnowledgeObject;

pub fn append_publish_request_classified(
    backend: &impl StorageBackend,
    request_json: &str,
    finalized: &FinalizedKnowledgeObject,
) -> Result<AppendOutcome, StoreError> {
    let carrier_identity = carrier_identity_for_request(request_json)?;
    let existing_by_canonical_id = backend.get(&finalized.canonical_id)?;
    let existing_by_carrier_identity = backend
        .subscribe(None)?
        .into_iter()
        .find(|record| record.carrier_identity == carrier_identity);

    let classification = classify_duplicate_or_conflict(
        existing_by_canonical_id
            .as_ref()
            .map(|existing| ExistingObjectIdentity {
                canonical_id: &existing.canonical_id,
                carrier_identity: &existing.carrier_identity,
                object: &existing.object,
            }),
        existing_by_carrier_identity
            .as_ref()
            .map(|existing| ExistingObjectIdentity {
                canonical_id: &existing.canonical_id,
                carrier_identity: &existing.carrier_identity,
                object: &existing.object,
            }),
        IncomingObjectIdentity {
            canonical_id: &finalized.canonical_id,
            carrier_identity: &carrier_identity,
            canonical_json: &finalized.canonical_json,
        },
    );

    match classification {
        DuplicateConflictClassification::New => {
            backend.append_publish_request(request_json, finalized)
        }
        DuplicateConflictClassification::ExactDuplicate => {
            let existing = existing_by_carrier_identity
                .or(existing_by_canonical_id)
                .ok_or_else(|| {
                    store_error(
                        "LB_OBJECT_CONFLICT",
                        "duplicate classification missing existing record",
                    )
                })?;
            Ok(AppendOutcome {
                stored_at: Some(existing.stored_at),
                canonical_id: existing.canonical_id,
                carrier_identity: existing.carrier_identity,
                object: existing.object,
                duplicate: true,
            })
        }
        conflict => Err(store_error(
            conflict.code(),
            format!(
                "duplicate/conflict contract {} classified write as {:?}",
                DUPLICATE_CONFLICT_CONTRACT_VERSION, conflict
            ),
        )),
    }
}
