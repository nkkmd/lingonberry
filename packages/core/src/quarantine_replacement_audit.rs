use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{to_canonical_json, JsonValue};

use crate::{
    acquire_quarantine_lock, store_error, QuarantineReplacementTransactionState, StoreError,
};

pub const QUARANTINE_REPLACEMENT_AUDIT_FILE: &str = "quarantine-replacement-audit.jsonl";
pub const QUARANTINE_REPLACEMENT_AUDIT_VERSION: &str =
    "lingonberry-quarantine-replacement-audit/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarantineReplacementAuditEventType {
    OperationStarted,
    OperationCompleted,
    OperationRejected,
    RecoveryRequired,
    GenerationSwitched,
    Committed,
    RolledBack,
    StatusCorrupt,
}

impl QuarantineReplacementAuditEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OperationStarted => "replacement-operation-started",
            Self::OperationCompleted => "replacement-operation-completed",
            Self::OperationRejected => "replacement-operation-rejected",
            Self::RecoveryRequired => "replacement-recovery-required",
            Self::GenerationSwitched => "replacement-generation-switched",
            Self::Committed => "replacement-committed",
            Self::RolledBack => "replacement-rolled-back",
            Self::StatusCorrupt => "replacement-status-corrupt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarantineReplacementAuditOperation {
    Apply,
    Resume,
    Rollback,
    Status,
}

impl QuarantineReplacementAuditOperation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Apply => "apply",
            Self::Resume => "resume",
            Self::Rollback => "rollback",
            Self::Status => "status",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuarantineReplacementAuditOutcome {
    Started,
    Success,
    Rejected,
    Failed,
}

impl QuarantineReplacementAuditOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Success => "success",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementAuditEvent {
    pub occurred_at: String,
    pub event_type: QuarantineReplacementAuditEventType,
    pub operation: QuarantineReplacementAuditOperation,
    pub outcome: QuarantineReplacementAuditOutcome,
    pub transaction_state: Option<QuarantineReplacementTransactionState>,
    pub classification: Option<String>,
    pub bounded_error_code: Option<String>,
}

pub fn quarantine_replacement_audit_path(state_dir: impl AsRef<Path>) -> PathBuf {
    state_dir.as_ref().join(QUARANTINE_REPLACEMENT_AUDIT_FILE)
}

pub fn append_quarantine_replacement_audit_event(
    state_dir: impl AsRef<Path>,
    event_type: QuarantineReplacementAuditEventType,
    operation: QuarantineReplacementAuditOperation,
    outcome: QuarantineReplacementAuditOutcome,
    transaction_state: Option<QuarantineReplacementTransactionState>,
    classification: Option<&str>,
    bounded_error_code: Option<&str>,
) -> Result<QuarantineReplacementAuditEvent, StoreError> {
    let state_dir = state_dir.as_ref();
    let classification = classification.map(validate_classification).transpose()?;
    let bounded_error_code = bounded_error_code.map(validate_error_code).transpose()?;
    let occurred_at = timestamp()?;
    let event = QuarantineReplacementAuditEvent {
        occurred_at,
        event_type,
        operation,
        outcome,
        transaction_state,
        classification,
        bounded_error_code,
    };

    let _lock = acquire_quarantine_lock(state_dir, "quarantine-replacement-audit-v1")?;
    fs::create_dir_all(state_dir).map_err(audit_io_error)?;
    let path = quarantine_replacement_audit_path(state_dir);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(audit_io_error)?;
    let line = to_canonical_json(&quarantine_replacement_audit_event_json(&event));
    file.write_all(line.as_bytes()).map_err(audit_io_error)?;
    file.write_all(b"\n").map_err(audit_io_error)?;
    file.sync_all().map_err(audit_io_error)?;
    if let Some(parent) = path.parent() {
        std::fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(audit_io_error)?;
    }
    Ok(event)
}

pub fn quarantine_replacement_audit_event_json(
    event: &QuarantineReplacementAuditEvent,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "boundedErrorCode".to_string(),
            optional_string_json(&event.bounded_error_code),
        ),
        (
            "classification".to_string(),
            optional_string_json(&event.classification),
        ),
        (
            "eventType".to_string(),
            JsonValue::String(event.event_type.as_str().to_string()),
        ),
        (
            "occurredAt".to_string(),
            JsonValue::String(event.occurred_at.clone()),
        ),
        (
            "operation".to_string(),
            JsonValue::String(event.operation.as_str().to_string()),
        ),
        (
            "outcome".to_string(),
            JsonValue::String(event.outcome.as_str().to_string()),
        ),
        (
            "transactionState".to_string(),
            event
                .transaction_state
                .map(|state| JsonValue::String(state.as_str().to_string()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_AUDIT_VERSION.to_string()),
        ),
    ]))
}

fn validate_classification(value: &str) -> Result<String, StoreError> {
    const ALLOWED: &[&str] = &[
        "resumable-before-publication",
        "resumable-before-switch",
        "resumable-after-switch",
        "resumable-or-rollback-before-publication",
        "resumable-or-rollback-before-switch",
        "resumable-or-rollback-after-switch",
        "committed",
        "rolled-back",
        "corrupt",
    ];
    if ALLOWED.contains(&value) {
        Ok(value.to_string())
    } else {
        Err(audit_error(
            "audit classification is not in the bounded allowlist",
        ))
    }
}

fn validate_error_code(value: &str) -> Result<String, StoreError> {
    let valid = value.starts_with("LB_")
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_');
    if valid {
        Ok(value.to_string())
    } else {
        Err(audit_error("audit error code is not a bounded stable code"))
    }
}

fn optional_string_json(value: &Option<String>) -> JsonValue {
    value
        .as_ref()
        .map(|value| JsonValue::String(value.clone()))
        .unwrap_or(JsonValue::Null)
}

fn timestamp() -> Result<String, StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| audit_error("system clock is before the Unix epoch"))?;
    Ok(format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()))
}

fn audit_io_error(error: std::io::Error) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_AUDIT", error.to_string())
}

fn audit_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_AUDIT", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-replacement-audit-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn appends_canonical_secret_free_events() {
        let state = temp_dir();
        let first = append_quarantine_replacement_audit_event(
            &state,
            QuarantineReplacementAuditEventType::OperationStarted,
            QuarantineReplacementAuditOperation::Apply,
            QuarantineReplacementAuditOutcome::Started,
            Some(QuarantineReplacementTransactionState::Prepared),
            Some("resumable-before-publication"),
            None,
        )
        .unwrap();
        append_quarantine_replacement_audit_event(
            &state,
            QuarantineReplacementAuditEventType::OperationCompleted,
            QuarantineReplacementAuditOperation::Apply,
            QuarantineReplacementAuditOutcome::Success,
            Some(QuarantineReplacementTransactionState::Committed),
            Some("committed"),
            None,
        )
        .unwrap();

        let contents = fs::read_to_string(quarantine_replacement_audit_path(&state)).unwrap();
        let lines = contents.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains(QUARANTINE_REPLACEMENT_AUDIT_VERSION));
        assert!(lines[1].contains("replacement-operation-completed"));
        assert!(!contents.contains("Bearer "));
        assert!(!contents.contains("/tmp/"));
        assert!(!contents.contains("transactionId"));
        assert!(!contents.contains("generationDigest"));
        assert!(!first.occurred_at.is_empty());
        let _ = fs::remove_dir_all(state);
    }

    #[test]
    fn rejects_unbounded_values_before_writing() {
        let state = temp_dir();
        let classification = append_quarantine_replacement_audit_event(
            &state,
            QuarantineReplacementAuditEventType::OperationRejected,
            QuarantineReplacementAuditOperation::Status,
            QuarantineReplacementAuditOutcome::Rejected,
            None,
            Some("user-controlled-value"),
            None,
        )
        .unwrap_err();
        assert_eq!(classification.code, "LB_QUARANTINE_REPLACEMENT_AUDIT");

        let error_code = append_quarantine_replacement_audit_event(
            &state,
            QuarantineReplacementAuditEventType::OperationRejected,
            QuarantineReplacementAuditOperation::Status,
            QuarantineReplacementAuditOutcome::Rejected,
            None,
            Some("corrupt"),
            Some("free form message"),
        )
        .unwrap_err();
        assert_eq!(error_code.code, "LB_QUARANTINE_REPLACEMENT_AUDIT");
        assert!(!quarantine_replacement_audit_path(&state).exists());
        let _ = fs::remove_dir_all(state);
    }
}
