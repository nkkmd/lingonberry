use std::collections::{BTreeMap, BTreeSet};

use lingonberry_protocol::JsonValue;

use crate::{store_error, StoreError};

pub const QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION: &str =
    "lingonberry-quarantine-replacement-retention-policy/v1";
pub const QUARANTINE_REPLACEMENT_RETENTION_DECISION_REPORT_VERSION: &str =
    "lingonberry-quarantine-replacement-retention-decision-report/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementRetentionPolicy {
    pub minimum_previous_committed_generations: usize,
    pub minimum_age_seconds: u64,
    pub allow_previous_committed_generations: bool,
    pub allow_rolled_back_generations: bool,
    pub selected_generation_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementRetentionCandidate {
    pub generation_id: String,
    pub classification: String,
    pub terminal_transaction_state: Option<String>,
    pub verification_status: String,
    pub durable_age_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementRetentionDecision {
    pub generation_id: String,
    pub classification: String,
    pub eligible: bool,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementRetentionDecisionReport {
    pub policy_version: String,
    pub decisions: Vec<QuarantineReplacementRetentionDecision>,
}

pub fn evaluate_quarantine_replacement_retention_policy(
    policy: &QuarantineReplacementRetentionPolicy,
    candidates: &[QuarantineReplacementRetentionCandidate],
) -> Result<QuarantineReplacementRetentionDecisionReport, StoreError> {
    validate_policy(policy)?;
    let selected = exact_selection(&policy.selected_generation_ids)?;
    let candidates_by_id = candidate_map(candidates)?;
    let previous_committed_total = candidates
        .iter()
        .filter(|candidate| candidate.classification == "previous-committed-generation")
        .count();

    let mut decisions = Vec::new();
    let mut provisionally_eligible_previous = Vec::new();
    for generation_id in &selected {
        let Some(candidate) = candidates_by_id.get(generation_id) else {
            decisions.push(decision(generation_id, "missing", false, "subject-not-found"));
            continue;
        };
        let result = evaluate_candidate(policy, candidate);
        if result.eligible && candidate.classification == "previous-committed-generation" {
            provisionally_eligible_previous.push(generation_id.clone());
        }
        decisions.push(result);
    }

    let maximum_removable_previous = previous_committed_total
        .saturating_sub(policy.minimum_previous_committed_generations);
    let allowed_previous = provisionally_eligible_previous
        .iter()
        .take(maximum_removable_previous)
        .cloned()
        .collect::<BTreeSet<_>>();
    for decision in &mut decisions {
        if decision.eligible
            && decision.classification == "previous-committed-generation"
            && !allowed_previous.contains(&decision.generation_id)
        {
            decision.eligible = false;
            decision.reason_code = "minimum-retention-floor".to_string();
        }
    }

    Ok(QuarantineReplacementRetentionDecisionReport {
        policy_version: QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION.to_string(),
        decisions,
    })
}

pub fn quarantine_replacement_retention_decision_report_json(
    report: &QuarantineReplacementRetentionDecisionReport,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "decisions".to_string(),
            JsonValue::Array(
                report
                    .decisions
                    .iter()
                    .map(|decision| {
                        JsonValue::Object(BTreeMap::from([
                            (
                                "classification".to_string(),
                                JsonValue::String(decision.classification.clone()),
                            ),
                            ("eligible".to_string(), JsonValue::Bool(decision.eligible)),
                            (
                                "generationId".to_string(),
                                JsonValue::String(decision.generation_id.clone()),
                            ),
                            (
                                "reasonCode".to_string(),
                                JsonValue::String(decision.reason_code.clone()),
                            ),
                        ]))
                    })
                    .collect(),
            ),
        ),
        (
            "policyVersion".to_string(),
            JsonValue::String(report.policy_version.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(
                QUARANTINE_REPLACEMENT_RETENTION_DECISION_REPORT_VERSION.to_string(),
            ),
        ),
    ]))
}

fn validate_policy(policy: &QuarantineReplacementRetentionPolicy) -> Result<(), StoreError> {
    if policy.minimum_previous_committed_generations < 1 {
        return Err(policy_error(
            "minimum previous committed generations must be at least one",
        ));
    }
    if policy.selected_generation_ids.is_empty() {
        return Err(policy_error("at least one exact generation must be selected"));
    }
    Ok(())
}

fn exact_selection(values: &[String]) -> Result<BTreeSet<String>, StoreError> {
    let mut selected = BTreeSet::new();
    for value in values {
        if value.is_empty()
            || value == "."
            || value == ".."
            || value.contains('/')
            || value.contains('\\')
            || value.contains('*')
            || value.contains('?')
            || value.contains('[')
            || value.contains(']')
        {
            return Err(policy_error("selection must be an exact generation ID"));
        }
        if !selected.insert(value.clone()) {
            return Err(policy_error("duplicate generation selection"));
        }
    }
    Ok(selected)
}

fn candidate_map(
    candidates: &[QuarantineReplacementRetentionCandidate],
) -> Result<BTreeMap<String, &QuarantineReplacementRetentionCandidate>, StoreError> {
    let mut result = BTreeMap::new();
    for candidate in candidates {
        if result
            .insert(candidate.generation_id.clone(), candidate)
            .is_some()
        {
            return Err(policy_error("duplicate retention candidate generation ID"));
        }
    }
    Ok(result)
}

fn evaluate_candidate(
    policy: &QuarantineReplacementRetentionPolicy,
    candidate: &QuarantineReplacementRetentionCandidate,
) -> QuarantineReplacementRetentionDecision {
    let expected_terminal_state = match candidate.classification.as_str() {
        "previous-committed-generation" if policy.allow_previous_committed_generations => {
            "committed"
        }
        "rolled-back-generation" if policy.allow_rolled_back_generations => "rolled-back",
        "previous-committed-generation" | "rolled-back-generation" => {
            return rejected(candidate, "classification-disabled-by-policy");
        }
        "active-committed-generation" => return rejected(candidate, "active-generation"),
        "incomplete-transaction-generation" => {
            return rejected(candidate, "non-terminal-transaction");
        }
        "orphan-unreferenced-generation" => {
            return rejected(candidate, "orphan-requires-manual-review");
        }
        "unknown-or-corrupt" => return rejected(candidate, "unknown-or-corrupt"),
        "legacy-root-layout" => return rejected(candidate, "legacy-root-layout"),
        _ => return rejected(candidate, "unsupported-classification"),
    };

    if candidate.terminal_transaction_state.as_deref() != Some(expected_terminal_state) {
        return rejected(candidate, "terminal-state-mismatch");
    }
    if candidate.verification_status != "verified" {
        return rejected(candidate, "generation-not-verified");
    }
    let Some(age) = candidate.durable_age_seconds else {
        return rejected(candidate, "durable-age-evidence-missing");
    };
    if age < policy.minimum_age_seconds {
        return rejected(candidate, "minimum-age-not-satisfied");
    }
    decision(
        &candidate.generation_id,
        &candidate.classification,
        true,
        "eligible",
    )
}

fn rejected(
    candidate: &QuarantineReplacementRetentionCandidate,
    reason_code: &str,
) -> QuarantineReplacementRetentionDecision {
    decision(
        &candidate.generation_id,
        &candidate.classification,
        false,
        reason_code,
    )
}

fn decision(
    generation_id: &str,
    classification: &str,
    eligible: bool,
    reason_code: &str,
) -> QuarantineReplacementRetentionDecision {
    QuarantineReplacementRetentionDecision {
        generation_id: generation_id.to_string(),
        classification: classification.to_string(),
        eligible,
        reason_code: reason_code.to_string(),
    }
}

fn policy_error(message: &str) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_RETENTION_POLICY", message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(selected: &[&str]) -> QuarantineReplacementRetentionPolicy {
        QuarantineReplacementRetentionPolicy {
            minimum_previous_committed_generations: 1,
            minimum_age_seconds: 100,
            allow_previous_committed_generations: true,
            allow_rolled_back_generations: true,
            selected_generation_ids: selected.iter().map(|value| value.to_string()).collect(),
        }
    }

    fn candidate(
        id: &str,
        classification: &str,
        state: Option<&str>,
        verified: bool,
        age: Option<u64>,
    ) -> QuarantineReplacementRetentionCandidate {
        QuarantineReplacementRetentionCandidate {
            generation_id: id.to_string(),
            classification: classification.to_string(),
            terminal_transaction_state: state.map(str::to_string),
            verification_status: if verified { "verified" } else { "metadata-present" }.to_string(),
            durable_age_seconds: age,
        }
    }

    #[test]
    fn rejects_active_generation() {
        let report = evaluate_quarantine_replacement_retention_policy(
            &policy(&["active"]),
            &[candidate(
                "active",
                "active-committed-generation",
                Some("committed"),
                true,
                Some(1000),
            )],
        )
        .unwrap();
        assert_eq!(report.decisions[0].reason_code, "active-generation");
    }

    #[test]
    fn requires_durable_age_evidence() {
        let report = evaluate_quarantine_replacement_retention_policy(
            &policy(&["old"]),
            &[
                candidate(
                    "old",
                    "previous-committed-generation",
                    Some("committed"),
                    true,
                    None,
                ),
                candidate(
                    "kept",
                    "previous-committed-generation",
                    Some("committed"),
                    true,
                    Some(1000),
                ),
            ],
        )
        .unwrap();
        assert_eq!(
            report.decisions[0].reason_code,
            "durable-age-evidence-missing"
        );
    }

    #[test]
    fn rejects_duplicate_selection() {
        let error = evaluate_quarantine_replacement_retention_policy(
            &policy(&["same", "same"]),
            &[],
        )
        .unwrap_err();
        assert!(error.to_string().contains("duplicate generation selection"));
    }

    #[test]
    fn rejects_terminal_state_mismatch_and_unverified_metadata() {
        let report = evaluate_quarantine_replacement_retention_policy(
            &policy(&["wrong-state", "unverified"]),
            &[
                candidate(
                    "wrong-state",
                    "previous-committed-generation",
                    Some("rolled-back"),
                    true,
                    Some(1000),
                ),
                candidate(
                    "unverified",
                    "rolled-back-generation",
                    Some("rolled-back"),
                    false,
                    Some(1000),
                ),
            ],
        )
        .unwrap();
        assert_eq!(report.decisions[0].reason_code, "terminal-state-mismatch");
        assert_eq!(report.decisions[1].reason_code, "generation-not-verified");
    }

    #[test]
    fn enforces_previous_committed_retention_floor_deterministically() {
        let report = evaluate_quarantine_replacement_retention_policy(
            &policy(&["a", "b"]),
            &[
                candidate(
                    "a",
                    "previous-committed-generation",
                    Some("committed"),
                    true,
                    Some(1000),
                ),
                candidate(
                    "b",
                    "previous-committed-generation",
                    Some("committed"),
                    true,
                    Some(1000),
                ),
            ],
        )
        .unwrap();
        assert!(report.decisions[0].eligible);
        assert_eq!(report.decisions[1].reason_code, "minimum-retention-floor");
    }

    #[test]
    fn allows_verified_old_rolled_back_and_previous_committed_generations() {
        let report = evaluate_quarantine_replacement_retention_policy(
            &policy(&["committed", "rolled-back"]),
            &[
                candidate(
                    "committed",
                    "previous-committed-generation",
                    Some("committed"),
                    true,
                    Some(1000),
                ),
                candidate(
                    "kept",
                    "previous-committed-generation",
                    Some("committed"),
                    true,
                    Some(1000),
                ),
                candidate(
                    "rolled-back",
                    "rolled-back-generation",
                    Some("rolled-back"),
                    true,
                    Some(1000),
                ),
            ],
        )
        .unwrap();
        assert!(report.decisions.iter().all(|decision| decision.eligible));
    }
}
