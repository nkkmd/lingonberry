use lingonberry_core::StorageBackend;
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct EffectiveViewHttpResponse {
    pub status_code: u16,
    pub status_text: &'static str,
    pub body: JsonValue,
}

#[derive(Debug, Clone)]
struct TransitionEvidence {
    id: String,
    transition_type: String,
    target_id: String,
    replacement_id: Option<String>,
    supersedes: Vec<String>,
    publisher_key: String,
    request: JsonValue,
}

pub fn effective_view_http_response(
    target_id: &str,
    backend: &impl StorageBackend,
    state_dir: &Path,
) -> EffectiveViewHttpResponse {
    if !valid_protocol_id(target_id, "lb:obj:") || target_id.len() > 255 {
        return error_response(400, "Bad Request", "LB_TARGET_ID_INVALID", "invalid target identifier");
    }
    let target = match backend.get(target_id) {
        Ok(Some(record)) => record,
        Ok(None) => return error_response(404, "Not Found", "LB_TARGET_NOT_FOUND", "target object not found"),
        Err(error) => return error_response(500, "Internal Server Error", error.code, &error.message),
    };
    let target_publisher = target_publisher_key(backend, target_id);
    let transitions = match load_transitions(state_dir, target_id) {
        Ok(value) => value,
        Err(error) => return error_response(500, "Internal Server Error", "LB_TRANSITION_STORAGE_ERROR", &error),
    };
    let generation = match evidence_generation(target_id, &target.object, &transitions) {
        Ok(value) => value,
        Err(error) => return error_response(500, "Internal Server Error", "LB_EVIDENCE_GENERATION_FAILED", &error),
    };

    let mut diagnostics = Vec::new();
    let mut authorized = Vec::new();
    for transition in &transitions {
        match target_publisher.as_deref() {
            Some(key) if key == transition.publisher_key => authorized.push(transition.clone()),
            Some(_) => {}
            None => diagnostics.push(diagnostic(
                "transition",
                &transition.id,
                "unreadable",
                "LB_EVIDENCE_BYTES_UNREADABLE",
            )),
        }
    }

    let graph = evaluate_graph(target_id, &authorized);
    diagnostics.extend(graph.diagnostics);
    let complete = diagnostics.is_empty();
    if !complete {
        if let Some(last_known_good) = load_snapshot(state_dir, target_id) {
            return stale_response(last_known_good, generation, diagnostics);
        }
        return unavailable_response(target.object, generation, diagnostics);
    }

    let body = match project_effective_view(target.object, graph.heads, backend, &generation) {
        Ok(value) => value,
        Err(diagnostic) => {
            if let Some(last_known_good) = load_snapshot(state_dir, target_id) {
                return stale_response(last_known_good, generation, vec![diagnostic]);
            }
            return unavailable_response(target.object, generation, vec![diagnostic]);
        }
    };
    if let Err(error) = persist_snapshot(state_dir, target_id, &body) {
        return error_response(500, "Internal Server Error", "LB_EFFECTIVE_VIEW_STORAGE_ERROR", &error);
    }
    EffectiveViewHttpResponse {
        status_code: 200,
        status_text: "OK",
        body,
    }
}

struct GraphResult {
    heads: Vec<TransitionEvidence>,
    diagnostics: Vec<JsonValue>,
}

fn evaluate_graph(target_id: &str, transitions: &[TransitionEvidence]) -> GraphResult {
    let by_id = transitions
        .iter()
        .map(|transition| (transition.id.as_str(), transition))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    let mut superseded = BTreeSet::new();
    for transition in transitions {
        if transition.target_id != target_id {
            diagnostics.push(diagnostic(
                "transition",
                &transition.id,
                "corrupt",
                "LB_EVIDENCE_VALIDATION_FAILED",
            ));
            continue;
        }
        for parent in &transition.supersedes {
            match by_id.get(parent.as_str()) {
                Some(parent_transition) if parent_transition.target_id == target_id => {
                    superseded.insert(parent.clone());
                }
                _ => diagnostics.push(diagnostic(
                    "transition",
                    &transition.id,
                    "corrupt",
                    "LB_EVIDENCE_INVENTORY_CONFLICT",
                )),
            }
        }
    }
    let heads = transitions
        .iter()
        .filter(|transition| !superseded.contains(&transition.id))
        .cloned()
        .collect();
    GraphResult { heads, diagnostics }
}

fn project_effective_view(
    original: JsonValue,
    heads: Vec<TransitionEvidence>,
    backend: &impl StorageBackend,
    generation: &str,
) -> Result<JsonValue, JsonValue> {
    match heads.as_slice() {
        [] => Ok(read_body(original.clone(), original, "original", generation)),
        [head] if head.transition_type == "withdraw" => Ok(read_body(
            JsonValue::Null,
            original,
            "withdrawn",
            generation,
        )),
        [head] if head.transition_type == "replace" => {
            let replacement_id = head.replacement_id.as_deref().unwrap_or_default();
            match backend.get(replacement_id) {
                Ok(Some(replacement)) => Ok(read_body(
                    replacement.object,
                    original,
                    "replaced",
                    generation,
                )),
                _ => Err(diagnostic(
                    "transition",
                    &head.id,
                    "unreadable",
                    "LB_EVIDENCE_BYTES_UNREADABLE",
                )),
            }
        }
        [_] => Err(diagnostic(
            "transition",
            &heads[0].id,
            "corrupt",
            "LB_EVIDENCE_VALIDATION_FAILED",
        )),
        _ => Ok(read_body(original.clone(), original, "ambiguous", generation)),
    }
}

fn read_body(
    effective_object: JsonValue,
    original_object: JsonValue,
    classification: &str,
    generation: &str,
) -> JsonValue {
    object(vec![
        ("effectiveObject", effective_object),
        ("originalObject", original_object),
        (
            "effectiveView",
            object(vec![
                (
                    "classification",
                    JsonValue::String(classification.to_string()),
                ),
                (
                    "generation",
                    JsonValue::String(generation.to_string()),
                ),
                ("freshness", JsonValue::String("current".to_string())),
            ]),
        ),
        (
            "evidenceObservation",
            object(vec![
                (
                    "generation",
                    JsonValue::String(generation.to_string()),
                ),
                (
                    "snapshotClassification",
                    JsonValue::String("complete".to_string()),
                ),
                (
                    "diagnosticSummary",
                    diagnostic_summary(&[]),
                ),
                ("diagnostics", JsonValue::Array(Vec::new())),
            ]),
        ),
    ])
}

fn stale_response(
    mut last_known_good: JsonValue,
    observation_generation: String,
    diagnostics: Vec<JsonValue>,
) -> EffectiveViewHttpResponse {
    if let Some(root) = as_object_mut(&mut last_known_good) {
        if let Some(effective_view) = root.get_mut("effectiveView").and_then(as_object_mut) {
            effective_view.insert(
                "freshness".to_string(),
                JsonValue::String("stale".to_string()),
            );
        }
        root.insert(
            "evidenceObservation".to_string(),
            observation("incomplete", observation_generation, diagnostics),
        );
    }
    EffectiveViewHttpResponse {
        status_code: 200,
        status_text: "OK",
        body: last_known_good,
    }
}

fn unavailable_response(
    original: JsonValue,
    observation_generation: String,
    diagnostics: Vec<JsonValue>,
) -> EffectiveViewHttpResponse {
    EffectiveViewHttpResponse {
        status_code: 200,
        status_text: "OK",
        body: object(vec![
            ("effectiveObject", original.clone()),
            ("originalObject", original),
            (
                "effectiveView",
                object(vec![
                    (
                        "classification",
                        JsonValue::String("unresolved".to_string()),
                    ),
                    ("generation", JsonValue::Null),
                    (
                        "freshness",
                        JsonValue::String("unavailable".to_string()),
                    ),
                ]),
            ),
            (
                "evidenceObservation",
                observation("incomplete", observation_generation, diagnostics),
            ),
        ]),
    }
}

fn observation(
    classification: &str,
    generation: String,
    diagnostics: Vec<JsonValue>,
) -> JsonValue {
    let returned = diagnostics.len().min(20);
    object(vec![
        ("generation", JsonValue::String(generation)),
        (
            "snapshotClassification",
            JsonValue::String(classification.to_string()),
        ),
        (
            "diagnosticSummary",
            diagnostic_summary(&diagnostics),
        ),
        (
            "diagnostics",
            JsonValue::Array(diagnostics.into_iter().take(returned).collect()),
        ),
    ])
}

fn diagnostic_summary(diagnostics: &[JsonValue]) -> JsonValue {
    let mut unsupported = 0usize;
    let mut corrupt = 0usize;
    let mut unreadable = 0usize;
    for diagnostic in diagnostics {
        match as_object(diagnostic)
            .and_then(|map| string_field(map, "classification"))
            .unwrap_or_default()
        {
            "unsupported" => unsupported += 1,
            "corrupt" => corrupt += 1,
            "unreadable" => unreadable += 1,
            _ => {}
        }
    }
    let returned = diagnostics.len().min(20);
    object(vec![
        (
            "total",
            JsonValue::Number(diagnostics.len().to_string()),
        ),
        ("returned", JsonValue::Number(returned.to_string())),
        (
            "truncated",
            JsonValue::Bool(returned < diagnostics.len()),
        ),
        (
            "byClassification",
            object(vec![
                (
                    "unsupported",
                    JsonValue::Number(unsupported.to_string()),
                ),
                ("corrupt", JsonValue::Number(corrupt.to_string())),
                (
                    "unreadable",
                    JsonValue::Number(unreadable.to_string()),
                ),
            ]),
        ),
    ])
}

fn load_transitions(state_dir: &Path, target_id: &str) -> Result<Vec<TransitionEvidence>, String> {
    let path = state_dir.join("transitions/append-only.jsonl");
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut transitions = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| error.to_string())?;
        let record = parse_json(&line).map_err(|error| error.to_string())?;
        let record_map = as_object(&record).ok_or_else(|| "transition record is not an object".to_string())?;
        if string_field(record_map, "targetId") != Some(target_id) {
            continue;
        }
        let request = record_map
            .get("request")
            .cloned()
            .ok_or_else(|| "transition record missing request".to_string())?;
        let request_map = as_object(&request).ok_or_else(|| "transition request is not an object".to_string())?;
        let transition = request_map
            .get("transition")
            .and_then(as_object)
            .ok_or_else(|| "transition request missing transition".to_string())?;
        let publisher = request_map
            .get("publisher")
            .and_then(as_object)
            .ok_or_else(|| "transition request missing publisher".to_string())?;
        let supersedes = match transition.get("supersedesTransitionIds") {
            Some(JsonValue::Array(items)) => items
                .iter()
                .filter_map(|item| match item {
                    JsonValue::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        transitions.push(TransitionEvidence {
            id: string_field(transition, "id").unwrap_or_default().to_string(),
            transition_type: string_field(transition, "transitionType")
                .unwrap_or_default()
                .to_string(),
            target_id: target_id.to_string(),
            replacement_id: string_field(transition, "replacementId").map(ToString::to_string),
            supersedes,
            publisher_key: string_field(publisher, "publicKey")
                .unwrap_or_default()
                .to_string(),
            request,
        });
    }
    transitions.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
    Ok(transitions)
}

fn target_publisher_key(backend: &impl StorageBackend, target_id: &str) -> Option<String> {
    let raw = backend.get_raw_request(target_id).ok().flatten()?;
    let request = parse_json(&raw.request_json).ok()?;
    let publisher = as_object(&request)?.get("publisher").and_then(as_object)?;
    string_field(publisher, "publicKey").map(ToString::to_string)
}

fn evidence_generation(
    target_id: &str,
    target: &JsonValue,
    transitions: &[TransitionEvidence],
) -> Result<String, String> {
    let mut evidence = vec![object(vec![
        ("kind", JsonValue::String("target".to_string())),
        ("id", JsonValue::String(target_id.to_string())),
        (
            "classification",
            JsonValue::String("supported".to_string()),
        ),
        (
            "digest",
            JsonValue::String(format!(
                "sha256:{}",
                sha256_hex(to_canonical_json(target).as_bytes())?
            )),
        ),
    ])];
    for transition in transitions {
        evidence.push(object(vec![
            ("kind", JsonValue::String("transition".to_string())),
            ("id", JsonValue::String(transition.id.clone())),
            (
                "classification",
                JsonValue::String("supported".to_string()),
            ),
            (
                "digest",
                JsonValue::String(format!(
                    "sha256:{}",
                    sha256_hex(to_canonical_json(&transition.request).as_bytes())?
                )),
            ),
        ]));
    }
    let basis = object(vec![
        (
            "ruleVersion",
            JsonValue::String("lb.transition.evidence-generation.v1".to_string()),
        ),
        ("targetId", JsonValue::String(target_id.to_string())),
        ("evidence", JsonValue::Array(evidence)),
    ]);
    Ok(format!(
        "evidence:sha256:{}",
        sha256_hex(to_canonical_json(&basis).as_bytes())?
    ))
}

fn sha256_hex(bytes: &[u8]) -> Result<String, String> {
    let root = std::env::temp_dir().join(format!(
        "lingonberry-effective-view-hash-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let input = root.join("input.bin");
    fs::write(&input, bytes).map_err(|error| error.to_string())?;
    let output = Command::new("openssl")
        .args(["dgst", "-sha256"])
        .arg(&input)
        .output()
        .map_err(|error| error.to_string())?;
    let _ = fs::remove_dir_all(root);
    if !output.status.success() {
        return Err("openssl sha256 failed".to_string());
    }
    let text = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    text.split_whitespace()
        .last()
        .map(ToString::to_string)
        .ok_or_else(|| "openssl sha256 output missing digest".to_string())
}

fn snapshot_path(state_dir: &Path, target_id: &str) -> PathBuf {
    state_dir
        .join("transitions/effective")
        .join(format!("{:016x}.json", fnv1a64(target_id.as_bytes())))
}

fn persist_snapshot(state_dir: &Path, target_id: &str, body: &JsonValue) -> Result<(), String> {
    let path = snapshot_path(state_dir, target_id);
    let parent = path.parent().ok_or_else(|| "snapshot path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    file.write_all(to_canonical_json(body).as_bytes())
        .map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn load_snapshot(state_dir: &Path, target_id: &str) -> Option<JsonValue> {
    let raw = fs::read_to_string(snapshot_path(state_dir, target_id)).ok()?;
    parse_json(&raw).ok()
}

fn diagnostic(kind: &str, evidence_id: &str, classification: &str, reason_code: &str) -> JsonValue {
    object(vec![
        ("kind", JsonValue::String(kind.to_string())),
        (
            "evidenceId",
            JsonValue::String(evidence_id.to_string()),
        ),
        (
            "classification",
            JsonValue::String(classification.to_string()),
        ),
        (
            "reasonCode",
            JsonValue::String(reason_code.to_string()),
        ),
    ])
}

fn error_response(
    status_code: u16,
    status_text: &'static str,
    code: &str,
    message: &str,
) -> EffectiveViewHttpResponse {
    EffectiveViewHttpResponse {
        status_code,
        status_text,
        body: object(vec![
            ("status", JsonValue::String("error".to_string())),
            ("code", JsonValue::String(code.to_string())),
            ("message", JsonValue::String(message.to_string())),
        ]),
    }
}

fn object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn as_object_mut(value: &mut JsonValue) -> Option<&mut BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn string_field<'a>(map: &'a BTreeMap<String, JsonValue>, key: &str) -> Option<&'a str> {
    match map.get(key) {
        Some(JsonValue::String(value)) => Some(value),
        _ => None,
    }
}

fn valid_protocol_id(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.is_ascii()
        && value.as_bytes().iter().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b':' | b'-')
        })
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
