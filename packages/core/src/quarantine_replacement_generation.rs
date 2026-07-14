mod inner {
    include!("quarantine_replacement_generation.inc");
}

use std::fs::{self, File};
use std::path::Path;

pub use inner::{
    validate_quarantine_current_generation_pointer, verify_quarantine_replacement_generation,
    QuarantineReplacementGenerationReport, QUARANTINE_CURRENT_GENERATION_POINTER_FILE,
    QUARANTINE_CURRENT_GENERATION_POINTER_VERSION, QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE, QUARANTINE_REPLACEMENT_GENERATION_VERSION,
    QUARANTINE_REPLACEMENT_PUBLICATION_DIR,
};

use crate::{
    advance_quarantine_replacement_transaction_journal,
    quarantine_replacement_failure_injection::{
        inject_quarantine_replacement_failure, FAILURE_POINT_GENERATION_MANIFEST_FSYNC,
        FAILURE_POINT_GENERATION_MANIFEST_WRITE,
    },
    read_quarantine_replacement_transaction_journal, QuarantineReplacementTransactionState,
    StoreError,
};

pub fn seal_quarantine_replacement_generation(
    transaction_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementGenerationReport, StoreError> {
    let transaction_dir = transaction_dir.as_ref();

    inject_quarantine_replacement_failure(FAILURE_POINT_GENERATION_MANIFEST_WRITE)?;

    let report = inner::seal_quarantine_replacement_generation(transaction_dir)?;

    if let Err(error) = inject_quarantine_replacement_failure(FAILURE_POINT_GENERATION_MANIFEST_FSYNC)
    {
        recover_after_generation_durability_failure(transaction_dir, &report);
        return Err(error);
    }

    Ok(report)
}

fn recover_after_generation_durability_failure(
    transaction_dir: &Path,
    report: &QuarantineReplacementGenerationReport,
) {
    let _ = fs::remove_dir_all(&report.publication_dir);
    let _ = File::open(transaction_dir).and_then(|directory| directory.sync_all());

    if let Ok(journal) = read_quarantine_replacement_transaction_journal(transaction_dir) {
        if journal.state == QuarantineReplacementTransactionState::Verified {
            let _ = advance_quarantine_replacement_transaction_journal(
                transaction_dir,
                QuarantineReplacementTransactionState::RecoveryRequired,
            );
        }
    }
}
