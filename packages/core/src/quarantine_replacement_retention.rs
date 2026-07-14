use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use lingonberry_protocol::JsonValue;

use crate::{
    read_quarantine_replacement_transaction_journal, resolve_quarantine_active_generation,
    store_error, QuarantineReplacementTransactionState, StoreError, QUARANTINE_GENERATIONS_DIR,
    QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE, QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE,
};

pub const QUARANTINE_REPLACEMENT_RETENTION_REPORT_VERSION: &str =
    "lingonberry-quarantine-replacement-retention-report/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementGenerationInspection {
    pub generation: Option<String>,
    pub classification: String,
    pub referenced_by_pointer: bool,
    pub referenced_by_journal: bool,
    pub terminal_transaction_state: Option<String>,
    pub verification_status: String,
    pub manual_review_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementRetentionReport {
    pub layout: String,
    pub generations: Vec<QuarantineReplacementGenerationInspection>,
}

pub fn inspect_quarantine_replacement_generations(
    state_dir: impl AsRef<Path>,
    transaction_dirs: &[PathBuf],
) -> Result<QuarantineReplacementRetentionReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let active_transaction = resolve_quarantine_active_generation(state_dir)?.transaction_id;
    let journals = read_journals(transaction_dirs)?;
    let generations_dir = state_dir.join(QUARANTINE_GENERATIONS_DIR);

    if !generations_dir.exists() {
        return Ok(legacy_report());
    }
    if !generations_dir.is_dir() {
        return Err(retention_error(
            "generation container exists but is not a directory",
        ));
    }

    let mut entries = fs::read_dir(&generations_dir)
        .map_err(retention_io_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(retention_io_error)?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut seen = BTreeSet::new();
    let mut generations = Vec::new();
    for entry in entries {
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| retention_error("generation directory name is not valid UTF-8"))?;
        if !seen.insert(name.clone()) {
            return Err(retention_error("duplicate generation directory name"));
        }

        let path = entry.path();
        let referenced_by_pointer = active_transaction.as_deref() == Some(name.as_str());
        let journal_state = journals.get(&name).copied();
        let verification_status = generation_verification_status(&path);
        let classification = classify_generation(
            verification_status,
            referenced_by_pointer,
            journal_state,
        );

        generations.push(QuarantineReplacementGenerationInspection {
            generation: Some(name),
            classification: classification.to_string(),
            referenced_by_pointer,
            referenced_by_journal: journal_state.is_some(),
            terminal_transaction_state: terminal_state(journal_state),
            verification_status: verification_status.to_string(),
            manual_review_required: matches!(
                classification,
                "orphan-unreferenced-generation" | "unknown-or-corrupt"
            ),
        });
    }

    if active_transaction
        .as_ref()
        .is_some_and(|transaction| !seen.contains(transaction))
    {
        return Err(retention_error(
            "active generation pointer references a missing generation directory",
        ));
    }

    Ok(QuarantineReplacementRetentionReport {
        layout: "generation".to_string(),
        generations,
    })
}

pub fn quarantine_replacement_retention_report_json(
    report: &QuarantineReplacementRetentionReport,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "generations".to_string(),
            JsonValue::Array(
                report
                    .generations
                    .iter()
                    .map(generation_inspection_json)
                    .collect(),
            ),
        ),
        (
            "layout".to_string(),
            JsonValue::String(report.layout.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_RETENTION_REPORT_VERSION.to_string()),
        ),
    ]))
}

fn read_journals(
    transaction_dirs: &[PathBuf],
) -> Result<BTreeMap<String, QuarantineReplacementTransactionState>, StoreError> {
    let mut journals = BTreeMap::new();
    for transaction_dir in transaction_dirs {
        let report = read_quarantine_replacement_transaction_journal(transaction_dir)?;
        if journals
            .insert(report.transaction_id.clone(), report.state)
            .is_some()
        {
            return Err(retention_error(
                "duplicate transaction ID supplied to generation inspection",
            ));
        }
    }
    Ok(journals)
}

fn legacy_report() -> QuarantineReplacementRetentionReport {
    QuarantineReplacementRetentionReport {
        layout: "legacy".to_string(),
        generations: vec![QuarantineReplacementGenerationInspection {
            generation: None,
            classification: "legacy-root-layout".to_string(),
            referenced_by_pointer: false,
            referenced_by_journal: false,
            terminal_transaction_state: None,
            verification_status: "not-applicable".to_string(),
            manual_review_required: false,
        }],
    }
}

fn classify_generation(
    verification_status: &str,
    referenced_by_pointer: bool,
    journal_state: Option<QuarantineReplacementTransactionState>,
) -> &'static str {
    if verification_status != "metadata-present" {
        return "unknown-or-corrupt";
    }
    if referenced_by_pointer {
        return match journal_state {
            Some(QuarantineReplacementTransactionState::Committed) => {
                "active-committed-generation"
            }
            _ => "unknown-or-corrupt",
        };
    }
    match journal_state {
        Some(QuarantineReplacementTransactionState::Committed) => {
            "previous-committed-generation"
        }
        Some(QuarantineReplacementTransactionState::RolledBack) => "rolled-back-generation",
        Some(_) => "incomplete-transaction-generation",
        None => "orphan-unreferenced-generation",
    }
}

fn terminal_state(
    state: Option<QuarantineReplacementTransactionState>,
) -> Option<String> {
    state.and_then(|state| match state {
        QuarantineReplacementTransactionState::Committed
        | QuarantineReplacementTransactionState::RolledBack => {
            Some(state.as_str().to_string())
        }
        _ => None,
    })
}

fn generation_inspection_json(inspection: &QuarantineReplacementGenerationInspection) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "classification".to_string(),
            JsonValue::String(inspection.classification.clone()),
        ),
        (
            "generation".to_string(),
            inspection
                .generation
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "manualReviewRequired".to_string(),
            JsonValue::Bool(inspection.manual_review_required),
        ),
        (
            "referencedByJournal".to_string(),
            JsonValue::Bool(inspection.referenced_by_journal),
        ),
        (
            "referencedByPointer".to_string(),
            JsonValue::Bool(inspection.referenced_by_pointer),
        ),
        (
            "terminalTransactionState".to_string(),
            inspection
                .terminal_transaction_state
                .as_ref()
                .map(|value| JsonValue::String(value.clone()))
                .unwrap_or(JsonValue::Null),
        ),
        (
            "verificationStatus".to_string(),
            JsonValue::String(inspection.verification_status.clone()),
        ),
    ]))
}

fn generation_verification_status(path: &Path) -> &'static str {
    if !path.is_dir() {
        return "not-a-directory";
    }
    if path
        .join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE)
        .is_file()
        && path
            .join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE)
            .is_file()
    {
        "metadata-present"
    } else {
        "metadata-missing"
    }
}

fn retention_io_error(error: std::io::Error) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_RETENTION", error.to_string())
}

fn retention_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_RETENTION", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-retention-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn reports_legacy_layout_without_mutation() {
        let state = temp_dir();
        fs::create_dir_all(&state).unwrap();
        fs::write(state.join("quarantine.jsonl"), b"{}\n").unwrap();

        let report = inspect_quarantine_replacement_generations(&state, &[]).unwrap();
        assert_eq!(report.layout, "legacy");
        assert_eq!(report.generations[0].classification, "legacy-root-layout");
        assert!(state.join("quarantine.jsonl").is_file());
        let _ = fs::remove_dir_all(state);
    }

    #[test]
    fn classifies_unreferenced_generation_as_manual_review() {
        let state = temp_dir();
        let generation = state.join(QUARANTINE_GENERATIONS_DIR).join("tx-orphan");
        fs::create_dir_all(&generation).unwrap();
        fs::write(
            generation.join(QUARANTINE_REPLACEMENT_GENERATION_MANIFEST_FILE),
            b"{}",
        )
        .unwrap();
        fs::write(
            generation.join(QUARANTINE_REPLACEMENT_GENERATION_DIGEST_FILE),
            b"fnv1a64:0000000000000000\n",
        )
        .unwrap();

        let report = inspect_quarantine_replacement_generations(&state, &[]).unwrap();
        assert_eq!(report.layout, "generation");
        assert_eq!(
            report.generations[0].classification,
            "orphan-unreferenced-generation"
        );
        assert!(report.generations[0].manual_review_required);
        let _ = fs::remove_dir_all(state);
    }
}
