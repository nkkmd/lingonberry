use crate::effective_view_http_response;
use lingonberry_core::StorageBackend;
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReevaluationReport {
    pub scanned_intents: usize,
    pub unique_targets: usize,
    pub succeeded: usize,
    pub failed: usize,
}

pub fn process_reevaluation_queue(
    backend: &impl StorageBackend,
    state_dir: &Path,
) -> Result<ReevaluationReport, String> {
    let queue_path = state_dir.join("transitions/reevaluation-queue.jsonl");
    let targets = load_pending_targets(&queue_path)?;
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    for target_id in &targets.unique_targets {
        let response = effective_view_http_response(target_id, backend, state_dir);
        let status = if response.status_code == 200 {
            succeeded += 1;
            "succeeded"
        } else {
            failed += 1;
            "retryable-failed"
        };
        append_checkpoint(
            state_dir,
            target_id,
            status,
            response.status_code,
            &response.body,
        )?;
    }
    Ok(ReevaluationReport {
        scanned_intents: targets.scanned,
        unique_targets: targets.unique_targets.len(),
        succeeded,
        failed,
    })
}

pub fn reconcile_reevaluation_queue(
    backend: &impl StorageBackend,
    state_dir: &Path,
) -> Result<ReevaluationReport, String> {
    process_reevaluation_queue(backend, state_dir)
}

pub fn reevaluation_report_json(report: &ReevaluationReport) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "status".to_string(),
            JsonValue::String("completed".to_string()),
        ),
        (
            "scannedIntents".to_string(),
            JsonValue::Number(report.scanned_intents.to_string()),
        ),
        (
            "uniqueTargets".to_string(),
            JsonValue::Number(report.unique_targets.to_string()),
        ),
        (
            "succeeded".to_string(),
            JsonValue::Number(report.succeeded.to_string()),
        ),
        (
            "failed".to_string(),
            JsonValue::Number(report.failed.to_string()),
        ),
    ]))
}

struct PendingTargets {
    scanned: usize,
    unique_targets: BTreeSet<String>,
}

fn load_pending_targets(path: &Path) -> Result<PendingTargets, String> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(PendingTargets {
                scanned: 0,
                unique_targets: BTreeSet::new(),
            })
        }
        Err(error) => return Err(error.to_string()),
    };
    let mut scanned = 0usize;
    let mut unique_targets = BTreeSet::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        scanned += 1;
        let value = parse_json(&line).map_err(|error| error.to_string())?;
        let JsonValue::Object(map) = value else {
            return Err("reevaluation intent must be an object".to_string());
        };
        if let Some(JsonValue::String(target_id)) = map.get("targetId") {
            unique_targets.insert(target_id.clone());
        }
    }
    Ok(PendingTargets {
        scanned,
        unique_targets,
    })
}

fn append_checkpoint(
    state_dir: &Path,
    target_id: &str,
    status: &str,
    http_status: u16,
    body: &JsonValue,
) -> Result<(), String> {
    let path = state_dir.join("transitions/reevaluation-checkpoints.jsonl");
    let parent = path
        .parent()
        .ok_or_else(|| "checkpoint path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let generation = extract_generation(body).unwrap_or_else(|| "unknown".to_string());
    let record = JsonValue::Object(BTreeMap::from([
        (
            "ruleVersion".to_string(),
            JsonValue::String("lb.transition.reevaluation.queue.v1".to_string()),
        ),
        (
            "targetId".to_string(),
            JsonValue::String(target_id.to_string()),
        ),
        ("status".to_string(), JsonValue::String(status.to_string())),
        ("generation".to_string(), JsonValue::String(generation)),
        (
            "httpStatus".to_string(),
            JsonValue::Number(http_status.to_string()),
        ),
        (
            "completedAtUnixSeconds".to_string(),
            JsonValue::Number(unix_seconds().to_string()),
        ),
    ]));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{}", to_canonical_json(&record)).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

fn extract_generation(body: &JsonValue) -> Option<String> {
    let JsonValue::Object(root) = body else {
        return None;
    };
    let JsonValue::Object(observation) = root.get("evidenceObservation")? else {
        return None;
    };
    match observation.get("generation") {
        Some(JsonValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
