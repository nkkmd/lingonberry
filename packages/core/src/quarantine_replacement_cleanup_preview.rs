use std::collections::{BTreeMap, BTreeSet};

use lingonberry_protocol::JsonValue;

use crate::{
    store_error, QuarantineReplacementRetentionDecisionReport,
    QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION, StoreError,
};

pub const QUARANTINE_REPLACEMENT_CLEANUP_PLAN_VERSION: &str =
    "lingonberry-quarantine-replacement-cleanup-plan/v1";
pub const QUARANTINE_REPLACEMENT_CLEANUP_PROOF_VERSION: &str =
    "lingonberry-quarantine-replacement-cleanup-proof/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupSubject {
    pub generation_id: String,
    pub classification: String,
    pub transaction_journal_digest: String,
    pub generation_digest: String,
    pub completion_evidence_digest: String,
    pub managed_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupPlan {
    pub policy_version: String,
    pub state_identity: String,
    pub active_pointer_digest: String,
    pub runtime_fingerprint: String,
    pub subjects: Vec<QuarantineReplacementCleanupSubject>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementCleanupProof {
    pub plan: QuarantineReplacementCleanupPlan,
    pub plan_digest: String,
}

pub fn build_quarantine_replacement_cleanup_plan(
    decisions: &QuarantineReplacementRetentionDecisionReport,
    state_identity: &str,
    active_pointer_digest: &str,
    runtime_fingerprint: &str,
    subjects: Vec<QuarantineReplacementCleanupSubject>,
) -> Result<QuarantineReplacementCleanupPlan, StoreError> {
    validate_atom(state_identity, "state identity")?;
    validate_digest(active_pointer_digest, "active pointer digest")?;
    validate_atom(runtime_fingerprint, "runtime fingerprint")?;

    if decisions.policy_version != QUARANTINE_REPLACEMENT_RETENTION_POLICY_VERSION {
        return Err(preview_error("retention policy version mismatch"));
    }

    let eligible = decisions
        .decisions
        .iter()
        .filter(|decision| decision.eligible)
        .map(|decision| decision.generation_id.as_str())
        .collect::<BTreeSet<_>>();
    if eligible.is_empty() {
        return Err(preview_error("cleanup plan requires at least one eligible subject"));
    }

    let mut seen = BTreeSet::new();
    let mut normalized = Vec::new();
    for mut subject in subjects {
        validate_generation_id(&subject.generation_id)?;
        validate_digest(&subject.transaction_journal_digest, "transaction journal digest")?;
        validate_digest(&subject.generation_digest, "generation digest")?;
        validate_digest(&subject.completion_evidence_digest, "completion evidence digest")?;
        if !eligible.contains(subject.generation_id.as_str()) {
            return Err(preview_error("cleanup subject is not eligible"));
        }
        if !seen.insert(subject.generation_id.clone()) {
            return Err(preview_error("duplicate cleanup subject"));
        }
        normalize_managed_paths(&mut subject.managed_paths)?;
        normalized.push(subject);
    }

    if seen.len() != eligible.len() {
        return Err(preview_error("eligible subject set is not fully bound"));
    }
    normalized.sort_by(|left, right| left.generation_id.cmp(&right.generation_id));

    Ok(QuarantineReplacementCleanupPlan {
        policy_version: decisions.policy_version.clone(),
        state_identity: state_identity.to_string(),
        active_pointer_digest: active_pointer_digest.to_string(),
        runtime_fingerprint: runtime_fingerprint.to_string(),
        subjects: normalized,
    })
}

pub fn quarantine_replacement_cleanup_plan_json(
    plan: &QuarantineReplacementCleanupPlan,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "activePointerDigest".to_string(),
            JsonValue::String(plan.active_pointer_digest.clone()),
        ),
        (
            "policyVersion".to_string(),
            JsonValue::String(plan.policy_version.clone()),
        ),
        (
            "runtimeFingerprint".to_string(),
            JsonValue::String(plan.runtime_fingerprint.clone()),
        ),
        (
            "stateIdentity".to_string(),
            JsonValue::String(plan.state_identity.clone()),
        ),
        (
            "subjects".to_string(),
            JsonValue::Array(plan.subjects.iter().map(subject_json).collect()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_CLEANUP_PLAN_VERSION.to_string()),
        ),
    ]))
}

pub fn quarantine_replacement_cleanup_proof_json(
    proof: &QuarantineReplacementCleanupProof,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "plan".to_string(),
            quarantine_replacement_cleanup_plan_json(&proof.plan),
        ),
        (
            "planDigest".to_string(),
            JsonValue::String(proof.plan_digest.clone()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_CLEANUP_PROOF_VERSION.to_string()),
        ),
    ]))
}

fn subject_json(subject: &QuarantineReplacementCleanupSubject) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "classification".to_string(),
            JsonValue::String(subject.classification.clone()),
        ),
        (
            "completionEvidenceDigest".to_string(),
            JsonValue::String(subject.completion_evidence_digest.clone()),
        ),
        (
            "generationDigest".to_string(),
            JsonValue::String(subject.generation_digest.clone()),
        ),
        (
            "generationId".to_string(),
            JsonValue::String(subject.generation_id.clone()),
        ),
        (
            "managedPaths".to_string(),
            JsonValue::Array(
                subject
                    .managed_paths
                    .iter()
                    .map(|path| JsonValue::String(path.clone()))
                    .collect(),
            ),
        ),
        (
            "transactionJournalDigest".to_string(),
            JsonValue::String(subject.transaction_journal_digest.clone()),
        ),
    ]))
}

fn normalize_managed_paths(paths: &mut Vec<String>) -> Result<(), StoreError> {
    if paths.is_empty() {
        return Err(preview_error("managed path inventory must not be empty"));
    }
    let mut seen = BTreeSet::new();
    for path in paths.iter() {
        if path.is_empty()
            || path.starts_with('/')
            || path.contains('\\')
            || path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err(preview_error("managed path must be a normalized relative path"));
        }
        if !seen.insert(path.clone()) {
            return Err(preview_error("duplicate managed path"));
        }
    }
    paths.sort();
    Ok(())
}

fn validate_generation_id(value: &str) -> Result<(), StoreError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains(['/', '\\', '*', '?', '[', ']'])
        || !value.is_ascii()
    {
        return Err(preview_error("invalid generation ID"));
    }
    Ok(())
}

fn validate_digest(value: &str, label: &str) -> Result<(), StoreError> {
    let Some(hex) = value.strip_prefix("fnv1a64:") else {
        return Err(preview_error(format!("invalid {label}")));
    };
    if hex.len() != 16 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(preview_error(format!("invalid {label}")));
    }
    Ok(())
}

fn validate_atom(value: &str, label: &str) -> Result<(), StoreError> {
    if value.is_empty() || value.contains(['\n', '\r']) {
        return Err(preview_error(format!("invalid {label}")));
    }
    Ok(())
}

fn preview_error(message: impl Into<String>) -> StoreError {
    store_error("LB_QUARANTINE_REPLACEMENT_CLEANUP_PREVIEW", message)
}
