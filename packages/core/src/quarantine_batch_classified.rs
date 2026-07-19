use crate::{
    promote_quarantine_batch, promote_quarantine_record_classified, runtime_state_dir,
    QuarantineBatchReport, QuarantinePromotionOutcome, QuarantineStore, StorageBackend, StoreError,
};

pub fn promote_quarantine_batch_classified(
    limit: usize,
    dry_run: bool,
    backend: &impl StorageBackend,
) -> Result<QuarantineBatchReport, StoreError> {
    if dry_run {
        return promote_quarantine_batch(limit, true, backend);
    }

    let records = QuarantineStore::new(runtime_state_dir())
        .list()?
        .into_iter()
        .take(limit)
        .collect::<Vec<_>>();
    let mut report = QuarantineBatchReport {
        dry_run: false,
        limit,
        scanned: records.len(),
        promoted: 0,
        already_promoted: 0,
        deferred: 0,
        rejected: 0,
        outcomes: Vec::with_capacity(records.len()),
    };

    for record in records {
        let outcome = promote_quarantine_record_classified(&record.id, backend)?;
        match &outcome {
            QuarantinePromotionOutcome::Promoted { .. } => report.promoted += 1,
            QuarantinePromotionOutcome::AlreadyPromoted { .. } => report.already_promoted += 1,
            QuarantinePromotionOutcome::StillDeferred { .. } => report.deferred += 1,
            QuarantinePromotionOutcome::Rejected { .. } => report.rejected += 1,
        }
        report.outcomes.push(outcome);
    }

    Ok(report)
}
