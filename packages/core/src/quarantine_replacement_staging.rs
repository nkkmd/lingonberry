mod inner {
    include!("quarantine_replacement_staging.inc");
}

use std::fs::{self, File};
use std::path::Path;

pub use inner::{
    QuarantineReplacementStagedLedger, QuarantineReplacementStagingReport,
    QUARANTINE_REPLACEMENT_STAGING_DIR, QUARANTINE_REPLACEMENT_STAGING_MANIFEST_FILE,
    QUARANTINE_REPLACEMENT_STAGING_VERSION,
};

use crate::{
    advance_quarantine_replacement_transaction_journal,
    quarantine_replacement_failure_injection::{
        inject_quarantine_replacement_failure, FAILURE_POINT_STAGED_LEDGER_FSYNC,
        FAILURE_POINT_STAGED_LEDGER_WRITE, FAILURE_POINT_STAGING_DIRECTORY_FSYNC,
    },
    read_quarantine_replacement_transaction_journal, QuarantineReplacementTransactionState,
    StoreError,
};

pub fn stage_quarantine_replacement_ledgers(
    state_dir: impl AsRef<Path>,
    verified_proof_dir: impl AsRef<Path>,
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementStagingReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let proof_dir = verified_proof_dir.as_ref();
    let transaction_dir = transaction_dir.as_ref();

    inject_quarantine_replacement_failure(FAILURE_POINT_STAGED_LEDGER_WRITE)?;

    let report = inner::stage_quarantine_replacement_ledgers(state_dir, proof_dir, transaction_dir)?;

    if let Err(error) = inject_quarantine_replacement_failure(FAILURE_POINT_STAGED_LEDGER_FSYNC) {
        recover_after_durable_staging_failure(transaction_dir, &report);
        return Err(error);
    }
    if let Err(error) = inject_quarantine_replacement_failure(FAILURE_POINT_STAGING_DIRECTORY_FSYNC)
    {
        recover_after_durable_staging_failure(transaction_dir, &report);
        return Err(error);
    }

    Ok(report)
}

fn recover_after_durable_staging_failure(
    transaction_dir: &Path,
    report: &QuarantineReplacementStagingReport,
) {
    let _ = fs::remove_dir_all(&report.staging_dir);
    let _ = File::open(transaction_dir).and_then(|directory| directory.sync_all());

    if let Ok(journal) = read_quarantine_replacement_transaction_journal(transaction_dir) {
        if journal.state == QuarantineReplacementTransactionState::Staged {
            let _ = advance_quarantine_replacement_transaction_journal(
                transaction_dir,
                QuarantineReplacementTransactionState::RecoveryRequired,
            );
        }
    }
}
