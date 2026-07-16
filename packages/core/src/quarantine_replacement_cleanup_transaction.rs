use std::collections::BTreeMap;

use lingonberry_protocol::JsonValue;

use crate::{store_error, StoreError};

pub const QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_VERSION: &str =
    "lingonberry-quarantine-replacement-cleanup-transaction/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarantineReplacementCleanupTransactionState {
    Prepared,
    Revalidated,
    RenamingToTomb,
    TombSealed,
    Deleting,
    Committed,
    RecoveryRequired,
    RolledBack,
    PartiallyDeleted,
}

impl QuarantineReplacementCleanupTransactionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Revalidated => "revalidated",
            Self::RenamingToTomb => "renaming-to-tomb",
            Self::TombSealed => "tomb-sealed",
            Self::Deleting => "deleting",
            Self::Committed => "committed",
            Self::RecoveryRequired => "recovery-required",
            Self::RolledBack => "rolled-back",
            Self::PartiallyDeleted => "partially-deleted",
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Committed | Self::RolledBack | Self::PartiallyDeleted
        )
    }

    pub fn deletion_has_started(self) -> bool {
        matches!(
            self,
            Self::Deleting | Self::Committed | Self::PartiallyDeleted
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupTransactionJournal {
    pub transaction_id: String,
    pub state: QuarantineReplacementCleanupTransactionState,
    pub sequence: u64,
    pub cleanup_proof_digest: String,
    pub runtime_fingerprint: String,
    pub tomb_inventory_digest: Option<String>,
    pub deleted_paths: Vec<String>,
}

pub fn validate_quarantine_replacement_cleanup_transaction_transition(
    current: QuarantineReplacementCleanupTransactionState,
    next: QuarantineReplacementCleanupTransactionState,
) -> Result<(), StoreError> {
    let allowed = matches!(
        (current, next),
        (
            QuarantineReplacementCleanupTransactionState::Prepared,
            QuarantineReplacementCleanupTransactionState::Revalidated
        ) | (
            QuarantineReplacementCleanupTransactionState::Revalidated,
            QuarantineReplacementCleanupTransactionState::RenamingToTomb
        ) | (
            QuarantineReplacementCleanupTransactionState::RenamingToTomb,
            QuarantineReplacementCleanupTransactionState::TombSealed
        ) | (
            QuarantineReplacementCleanupTransactionState::TombSealed,
            QuarantineReplacementCleanupTransactionState::Deleting
        ) | (
            QuarantineReplacementCleanupTransactionState::Deleting,
            QuarantineReplacementCleanupTransactionState::Committed
        ) | (
            QuarantineReplacementCleanupTransactionState::Prepared
                | QuarantineReplacementCleanupTransactionState::Revalidated
                | QuarantineReplacementCleanupTransactionState::RenamingToTomb
                | QuarantineReplacementCleanupTransactionState::TombSealed
                | QuarantineReplacementCleanupTransactionState::Deleting,
            QuarantineReplacementCleanupTransactionState::RecoveryRequired
        ) | (
            QuarantineReplacementCleanupTransactionState::RecoveryRequired,
            QuarantineReplacementCleanupTransactionState::Revalidated
                | QuarantineReplacementCleanupTransactionState::RenamingToTomb
                | QuarantineReplacementCleanupTransactionState::TombSealed
                | QuarantineReplacementCleanupTransactionState::Deleting
        ) | (
            QuarantineReplacementCleanupTransactionState::RecoveryRequired,
            QuarantineReplacementCleanupTransactionState::RolledBack
        ) | (
            QuarantineReplacementCleanupTransactionState::RecoveryRequired,
            QuarantineReplacementCleanupTransactionState::PartiallyDeleted
        )
    );
    if !allowed {
        return Err(cleanup_transaction_error(format!(
            "invalid cleanup transaction transition: {} -> {}",
            current.as_str(),
            next.as_str()
        )));
    }
    if current.deletion_has_started()
        && next == QuarantineReplacementCleanupTransactionState::RolledBack
    {
        return Err(cleanup_transaction_error(
            "rollback is forbidden after irreversible deletion begins",
        ));
    }
    Ok(())
}

pub fn quarantine_replacement_cleanup_transaction_journal_json(
    journal: &QuarantineReplacementCleanupTransactionJournal,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "cleanupProofDigest".to_string(),
            JsonValue::String(journal.cleanup_proof_digest.clone()),
        ),
        (
            "deletedPaths".to_string(),
            JsonValue::Array(
                journal
                    .deleted_paths
                    .iter()
                    .map(|value| JsonValue::String(value.clone()))
                    .collect(),
            ),
        ),
        (
            "runtimeFingerprint".to_string(),
            JsonValue::String(journal.runtime_fingerprint.clone()),
        ),
        (
            "sequence".to_string(),
            JsonValue::Number(journal.sequence.to_string()),
        ),
        (
            "state".to_string(),
            JsonValue::String(journal.state.as_str().to_string()),
        ),
        (
            "tombInventoryDigest".to_string(),
            journal
                .tomb_inventory_digest
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(journal.transaction_id.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION_VERSION.to_string()),
        ),
    ]))
}

fn cleanup_transaction_error(message: impl Into<String>) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_CLEANUP_TRANSACTION", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_cutoff_is_explicit() {
        assert!(
            validate_quarantine_replacement_cleanup_transaction_transition(
                QuarantineReplacementCleanupTransactionState::RecoveryRequired,
                QuarantineReplacementCleanupTransactionState::RolledBack,
            )
            .is_ok()
        );
        assert!(
            validate_quarantine_replacement_cleanup_transaction_transition(
                QuarantineReplacementCleanupTransactionState::Deleting,
                QuarantineReplacementCleanupTransactionState::RolledBack,
            )
            .is_err()
        );
        assert!(
            validate_quarantine_replacement_cleanup_transaction_transition(
                QuarantineReplacementCleanupTransactionState::RecoveryRequired,
                QuarantineReplacementCleanupTransactionState::PartiallyDeleted,
            )
            .is_ok()
        );
    }
}
