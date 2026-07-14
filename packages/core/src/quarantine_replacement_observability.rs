use std::collections::BTreeMap;

use lingonberry_protocol::JsonValue;

use crate::{QuarantineReplacementStatusReport, QuarantineReplacementTransactionState};

pub const QUARANTINE_REPLACEMENT_STATUS_VERSION: &str =
    "lingonberry-quarantine-replacement-status/v1";

pub fn quarantine_replacement_status_v1_json(
    report: &QuarantineReplacementStatusReport,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "activeGeneration".to_string(),
            optional_string_json(&report.active_generation),
        ),
        (
            "activeGenerationPresent".to_string(),
            JsonValue::Bool(report.active_generation.is_some()),
        ),
        (
            "classification".to_string(),
            JsonValue::String(report.classification.clone()),
        ),
        (
            "generationDigest".to_string(),
            optional_string_json(&report.generation_digest),
        ),
        (
            "publicationPhase".to_string(),
            JsonValue::String(publication_phase(report).to_string()),
        ),
        (
            "recoveryRequired".to_string(),
            JsonValue::Bool(
                report.state == QuarantineReplacementTransactionState::RecoveryRequired,
            ),
        ),
        (
            "state".to_string(),
            JsonValue::String(report.state.as_str().to_string()),
        ),
        (
            "targetGenerationActive".to_string(),
            JsonValue::Bool(
                report.active_generation.as_deref() == Some(report.transaction_id.as_str()),
            ),
        ),
        (
            "terminal".to_string(),
            JsonValue::Bool(matches!(
                report.state,
                QuarantineReplacementTransactionState::Committed
                    | QuarantineReplacementTransactionState::RolledBack
            )),
        ),
        (
            "transactionId".to_string(),
            JsonValue::String(report.transaction_id.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_STATUS_VERSION.to_string()),
        ),
    ]))
}

pub fn quarantine_replacement_metrics_text(report: &QuarantineReplacementStatusReport) -> String {
    let layout = if report.active_generation.is_some() {
        "generation"
    } else {
        "legacy"
    };
    let recovery_required =
        usize::from(report.state == QuarantineReplacementTransactionState::RecoveryRequired);
    let target_active =
        usize::from(report.active_generation.as_deref() == Some(report.transaction_id.as_str()));

    format!(
        "# HELP lingonberry_quarantine_replacement_transactions Replacement transactions by bounded journal state.\n\
# TYPE lingonberry_quarantine_replacement_transactions gauge\n\
lingonberry_quarantine_replacement_transactions{{state=\"{}\"}} 1\n\
# HELP lingonberry_quarantine_replacement_active_generation Active ledger layout and target-generation state.\n\
# TYPE lingonberry_quarantine_replacement_active_generation gauge\n\
lingonberry_quarantine_replacement_active_generation{{layout=\"{layout}\",target=\"active\"}} {target_active}\n\
# HELP lingonberry_quarantine_replacement_recovery_required Whether the transaction requires recovery.\n\
# TYPE lingonberry_quarantine_replacement_recovery_required gauge\n\
lingonberry_quarantine_replacement_recovery_required {recovery_required}\n\
# HELP lingonberry_quarantine_replacement_publication_phase Current bounded publication phase.\n\
# TYPE lingonberry_quarantine_replacement_publication_phase gauge\n\
lingonberry_quarantine_replacement_publication_phase{{phase=\"{}\"}} 1\n",
        report.state.as_str(),
        publication_phase(report),
    )
}

fn publication_phase(report: &QuarantineReplacementStatusReport) -> &'static str {
    match report.state {
        QuarantineReplacementTransactionState::Prepared => "prepared",
        QuarantineReplacementTransactionState::Writing => "writing",
        QuarantineReplacementTransactionState::Staged => "staged",
        QuarantineReplacementTransactionState::Verified => "verified",
        QuarantineReplacementTransactionState::Publishing => {
            if report.classification.contains("after-switch") {
                "switched"
            } else {
                "materialized"
            }
        }
        QuarantineReplacementTransactionState::Committed => "committed",
        QuarantineReplacementTransactionState::RolledBack => "rolled-back",
        QuarantineReplacementTransactionState::RecoveryRequired => {
            if report.classification.contains("after-switch") {
                "switched"
            } else {
                "recovery-required"
            }
        }
    }
}

fn optional_string_json(value: &Option<String>) -> JsonValue {
    value
        .as_ref()
        .map(|value| JsonValue::String(value.clone()))
        .unwrap_or(JsonValue::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::to_canonical_json;

    fn report(state: QuarantineReplacementTransactionState) -> QuarantineReplacementStatusReport {
        QuarantineReplacementStatusReport {
            transaction_id: "tx-sensitive-id".to_string(),
            state,
            classification: "resumable-after-switch".to_string(),
            generation_digest: Some("fnv1a64:sensitive-digest".to_string()),
            active_generation: Some("tx-sensitive-id".to_string()),
        }
    }

    #[test]
    fn status_json_is_versioned_and_explicit() {
        let value = quarantine_replacement_status_v1_json(&report(
            QuarantineReplacementTransactionState::RecoveryRequired,
        ));
        let canonical = to_canonical_json(&value);
        assert!(canonical.contains(QUARANTINE_REPLACEMENT_STATUS_VERSION));
        assert!(canonical.contains("\"recoveryRequired\":true"));
        assert!(canonical.contains("\"targetGenerationActive\":true"));
        assert!(canonical.contains("\"publicationPhase\":\"switched\""));
    }

    #[test]
    fn metrics_use_only_bounded_labels() {
        let metrics = quarantine_replacement_metrics_text(&report(
            QuarantineReplacementTransactionState::RecoveryRequired,
        ));
        assert!(metrics.contains("state=\"recovery-required\""));
        assert!(metrics.contains("layout=\"generation\""));
        assert!(metrics.contains("phase=\"switched\""));
        assert!(!metrics.contains("tx-sensitive-id"));
        assert!(!metrics.contains("sensitive-digest"));
        assert!(!metrics.contains("/tmp/"));
    }
}
