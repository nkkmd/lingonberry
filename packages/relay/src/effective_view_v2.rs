use lingonberry_core::StorageBackend;
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SUMMARY_LIMIT: usize = 20;
const PAGE_LIMIT: usize = 100;

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
    if !valid_id(target_id, "lb:obj:") {
        return error(400, "Bad Request", "LB_TARGET_ID_INVALID", "invalid target identifier");
    }
    let target = match backend.get(target_id) {
        Ok(Some(record)) => record,
        Ok(None) => return error(404, "Not Found", "LB_TARGET_NOT_FOUND", "target object not found"),
        Err(storage_error) => {
            return error(500, "Internal Server Error", storage_error.code, &storage_error.message)
        }
    };
    let original = target.object;
    let transitions = match load_transitions(state_dir, target_id) {
        Ok(value) => value,
        Err(message) => return error(500, "Internal Server Error", "LB_TRANSITION_STORAGE_ERROR", &message),
    };
    let generation = match evidence_generation(target_id, &original, &transitions) {
        Ok(value) => value,
        Err(message) => return error(500, "Internal Server Error", "LB_EVIDENCE_GENERATION_FAILED", &message),
    };
    let publisher = target_publisher_key(backend, target_id);
    let mut authorized = Vec::new();
    let mut diagnostics = Vec::new();
    for transition in &transitions {
        match publisher.as_deref() {
            Some(key) if key == transition.publisher_key => authorized.push(transition.clone()),
            Some(_) => {}
            None => diagnostics.push(diagnostic(
                &transition.id,
                "unreadable",
                "LB_EVIDENCE_BYTES_UNREADABLE",
            )),
        }
    }
    let graph = graph_heads(&authorized);
    diagnostics.extend(graph.diagnostics);
    if !diagnostics.is_empty() {
        return incomplete_response(state_dir, target_id, original, generation, diagnostics);
    }
    let projected = match project(original.clone(), graph.heads, backend, &generation) {
        Ok(value) => value,
        Err(problem) => {
            return incomplete_response(state_dir, target_id, original, generation, vec![problem])
        }
    };
    if let Err(message) = persist_snapshot(state_dir, target_id, &projected) {
        return error(500, "Internal Server Error", "LB_EFFECTIVE_VIEW_STORAGE_ERROR", &message);
    }
    EffectiveViewHttpResponse {
        status_code: 200,
        status_text: "OK",
        body: projected,
    }
}

pub fn diagnostic_page_http_response(
    target_id: &str,
    generation: &str,
    cursor: Option<&str>,
    limit: Option<usize>,
    state_dir: &Path,
) -> EffectiveViewHttpResponse {
    if !valid_id(target_id, "lb:obj:") {
        return error(400, "Bad Request", "LB_TARGET_ID_INVALID", "invalid target identifier");
    }
    if !generation.starts_with("evidence:sha256:") {
        return error(400, "Bad Request", "LB_DIAGNOSTIC_GENERATION_INVALID", "invalid generation");
    }
    let requested_limit = limit.unwrap_or(PAGE_LIMIT);
    if !(1..=PAGE_LIMIT).contains(&requested_limit) {
        return error(400, "Bad Request", "LB_DIAGNOSTIC_LIMIT_INVALID", "limit must be between 1 and 100");
    }
    let snapshot = match load_snapshot(state_dir, target_id) {
        Some(value) => value,
        None => return error(409, "Conflict", "LB_DIAGNOSTIC_GENERATION_UNAVAILABLE", "generation snapshot unavailable"),
    };
    let observation = as_object(&snapshot)
        .and_then(|root| root.get("evidenceObservation"))
        .and_then(as_object);
    let Some(observation) = observation else {
        return error(409, "Conflict", "LB_DIAGNOSTIC_GENERATION_UNAVAILABLE", "generation snapshot unavailable");
    };
    if string_field(observation, "generation") != Some(generation) {
        return error(409, "Conflict", "LB_DIAGNOSTIC_GENERATION_UNAVAILABLE", "requested generation is not retained");
    }
    let diagnostics = match observation.get("allDiagnostics") {
        Some(JsonValue::Array(items)) => items.clone(),
        _ => Vec::new(),
    };
    let offset = match cursor {
        None => 0,
        Some(value) => match decode_cursor(value, target_id, generation) {
            Some(position) => position,
            None => return error(400, "Bad Request", "LB_DIAGNOSTIC_CURSOR_INVALID", "invalid cursor"),
        },
    };
    if offset > diagnostics.len() {
        return error(400, "Bad Request", "LB_DIAGNOSTIC_CURSOR_INVALID", "invalid cursor position");
    }
    let end = (offset + requested_limit).min(diagnostics.len());
    let next = (end < diagnostics.len()).then(|| encode_cursor(target_id, generation, end));
    EffectiveViewHttpResponse {
        status_code: 200,
        status_text: "OK",
        body: object(vec![
            ("targetId", JsonValue::String(target_id.to_string())),
            ("generation", JsonValue::String(generation.to_string())),
            ("diagnostics", JsonValue::Array(diagnostics[offset..end].to_vec())),
            (
                "page",
                object(vec![
                    ("limit", JsonValue::Number(requested_limit.to_string())),
                    ("returned", JsonValue::Number((end - offset).to_string())),
                    (
                        "nextCursor",
                        next.map(JsonValue::String).unwrap_or(JsonValue::Null),
                    ),
                ]),
            ),
        ]),
    }
}

struct GraphResult {
    heads: Vec<TransitionEvidence>,
    diagnostics: Vec<JsonValue>,
}

fn graph_heads(transitions: &[TransitionEvidence]) -> GraphResult {
    let ids = transitions.iter().map(|item| item.id.as_str()).collect::<BTreeSet<_>>();
    let mut superseded = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for transition in transitions {
        for parent in &transition.supersedes {
            if ids.contains(parent.as_str()) {
                superseded.insert(parent.clone());
            } else {
                diagnostics.push(diagnostic(
                    &transition.id,
                    "corrupt",
                    "LB_EVIDENCE_INVENTORY_CONFLICT",
                ));
            }
        }
    }
    let heads = transitions
        .iter()
        .filter(|item| !superseded.contains(&item.id))
        .cloned()
        .collect();
    GraphResult { heads, diagnostics }
}

fn project(
    original: JsonValue,
    heads: Vec<TransitionEvidence>,
    backend: &impl StorageBackend,
    generation: &str,
) -> Result<JsonValue, JsonValue> {
    match heads.as_slice() {
        [] => Ok(view(original.clone(), original, "original", generation)),
        [head] if head.transition_type == "withdraw" => {
            Ok(view(JsonValue::Null, original, "withdrawn", generation))
        }
        [head] if head.transition_type == "replace" => {
            let replacement_id = head.replacement_id.as_deref().unwrap_or_default();
            match backend.get(replacement_id) {
                Ok(Some(replacement)) => Ok(view(replacement.object, original, "replaced", generation)),
                _ => Err(diagnostic(
                    &head.id,
                    "unreadable",
                    "LB_EVIDENCE_BYTES_UNREADABLE",
                )),
            }
        }
        [_] => Err(diagnostic(
            &heads[0].id,
            "corrupt",
            "LB_EVIDENCE_VALIDATION_FAILED",
        )),
        _ => Ok(view(original.clone(), original, "ambiguous", generation)),
    }
}

fn incomplete_response(
    state_dir: &Path,
    target_id: &str,
    original: JsonValue,
    generation: String,
    diagnostics: Vec<JsonValue>,
) -> EffectiveViewHttpResponse {
    let observation = observation(&generation, diagnostics);
    if let Some(mut previous) = load_snapshot(state_dir, target_id) {
        if let Some(root) = as_object_mut(&mut previous) {
            if let Some(view) = root.get_mut("effectiveView").and_then(as_object_mut) {
                view.insert("freshness".to_string(), JsonValue::String("stale".to_string()));
            }
            root.insert("evidenceObservation".to_string(), observation);
        }
        let _ = persist_snapshot(state_dir, target_id, &previous);
        return EffectiveViewHttpResponse { status_code: 200, status_text: "OK", body: previous };
    }
    let body = object(vec![
        ("effectiveObject", original.clone()),
        ("originalObject", original),
        (
            "effectiveView",
            object(vec![
                ("classification", JsonValue::String("unresolved".to_string())),
                ("generation", JsonValue::Null),
                ("freshness", JsonValue::String("unavailable".to_string())),
            ]),
        ),
        ("evidenceObservation", observation),
    ]);
    let _ = persist_snapshot(state_dir, target_id, &body);
    EffectiveViewHttpResponse { status_code: 200, status_text: "OK", body }
}

fn view(effective: JsonValue, original: JsonValue, classification: &str, generation: &str) -> JsonValue {
    object(vec![
        ("effectiveObject", effective),
        ("originalObject", original),
        (
            "effectiveView",
            object(vec![
                ("classification", JsonValue::String(classification.to_string())),
                ("generation", JsonValue::String(generation.to_string())),
                ("freshness", JsonValue::String("current".to_string())),
            ]),
        ),
        ("evidenceObservation", observation(generation, Vec::new())),
    ])
}

fn observation(generation: &str, mut diagnostics: Vec<JsonValue>) -> JsonValue {
    diagnostics.sort_by(|left, right| to_canonical_json(left).cmp(&to_canonical_json(right)));
    diagnostics.dedup();
    let total = diagnostics.len();
    let visible = diagnostics.iter().take(SUMMARY_LIMIT).cloned().collect::<Vec<_>>();
    object(vec![
        ("generation", JsonValue::String(generation.to_string())),
        (
            "snapshotClassification",
            JsonValue::String(if total == 0 { "complete" } else { "incomplete" }.to_string()),
        ),
        (
            "diagnosticSummary",
            object(vec![
                ("total", JsonValue::Number(total.to_string())),
                ("returned", JsonValue::Number(visible.len().to_string())),
                ("truncated", JsonValue::Bool(visible.len() < total)),
            ]),
        ),
        ("diagnostics", JsonValue::Array(visible)),
        ("allDiagnostics", JsonValue::Array(diagnostics)),
    ])
}

fn load_transitions(state_dir: &Path, target_id: &str) -> Result<Vec<TransitionEvidence>, String> {
    let path = state_dir.join("transitions/append-only.jsonl");
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut result = Vec::new();
    for line in BufReader::new(file).lines() {
        let record = parse_json(&line.map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        let record_map = as_object(&record).ok_or_else(|| "transition record invalid".to_string())?;
        if string_field(record_map, "targetId") != Some(target_id) {
            continue;
        }
        let request = record_map.get("request").cloned().ok_or_else(|| "transition request missing".to_string())?;
        let request_map = as_object(&request).ok_or_else(|| "transition request invalid".to_string())?;
        let transition = request_map.get("transition").and_then(as_object).ok_or_else(|| "transition missing".to_string())?;
        let publisher = request_map.get("publisher").and_then(as_object).ok_or_else(|| "publisher missing".to_string())?;
        let supersedes = match transition.get("supersedesTransitionIds") {
            Some(JsonValue::Array(items)) => items
                .iter()
                .filter_map(|item| match item { JsonValue::String(value) => Some(value.clone()), _ => None })
                .collect(),
            _ => Vec::new(),
        };
        result.push(TransitionEvidence {
            id: string_field(transition, "id").unwrap_or_default().to_string(),
            transition_type: string_field(transition, "transitionType").unwrap_or_default().to_string(),
            replacement_id: string_field(transition, "replacementId").map(ToString::to_string),
            supersedes,
            publisher_key: string_field(publisher, "publicKey").unwrap_or_default().to_string(),
            request,
        });
    }
    result.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
    Ok(result)
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
        ("classification", JsonValue::String("supported".to_string())),
        ("digest", JsonValue::String(format!("sha256:{}", sha256_hex(to_canonical_json(target).as_bytes())?))),
    ])];
    for transition in transitions {
        evidence.push(object(vec![
            ("kind", JsonValue::String("transition".to_string())),
            ("id", JsonValue::String(transition.id.clone())),
            ("classification", JsonValue::String("supported".to_string())),
            ("digest", JsonValue::String(format!("sha256:{}", sha256_hex(to_canonical_json(&transition.request).as_bytes())?))),
        ]));
    }
    let basis = object(vec![
        ("ruleVersion", JsonValue::String("lb.transition.evidence-generation.v1".to_string())),
        ("targetId", JsonValue::String(target_id.to_string())),
        ("evidence", JsonValue::Array(evidence)),
    ]);
    Ok(format!("evidence:sha256:{}", sha256_hex(to_canonical_json(&basis).as_bytes())?))
}

fn sha256_hex(bytes: &[u8]) -> Result<String, String> {
    let root = std::env::temp_dir().join(format!(
        "lingonberry-effective-view-hash-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()
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
    text.split_whitespace().last().map(ToString::to_string).ok_or_else(|| "sha256 digest missing".to_string())
}

fn snapshot_path(state_dir: &Path, target_id: &str) -> PathBuf {
    state_dir.join("transitions/effective").join(format!("{:016x}.json", fnv1a64(target_id.as_bytes())))
}

fn persist_snapshot(state_dir: &Path, target_id: &str, body: &JsonValue) -> Result<(), String> {
    let path = snapshot_path(state_dir, target_id);
    let parent = path.parent().ok_or_else(|| "snapshot path missing parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.tmp");
    let mut file = OpenOptions::new().create(true).truncate(true).write(true).open(&temporary).map_err(|error| error.to_string())?;
    file.write_all(to_canonical_json(body).as_bytes()).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    fs::rename(temporary, path).map_err(|error| error.to_string())
}

fn load_snapshot(state_dir: &Path, target_id: &str) -> Option<JsonValue> {
    parse_json(&fs::read_to_string(snapshot_path(state_dir, target_id)).ok()?).ok()
}

fn diagnostic(evidence_id: &str, classification: &str, reason_code: &str) -> JsonValue {
    object(vec![
        ("kind", JsonValue::String("transition".to_string())),
        ("evidenceId", JsonValue::String(evidence_id.to_string())),
        ("classification", JsonValue::String(classification.to_string())),
        ("reasonCode", JsonValue::String(reason_code.to_string())),
    ])
}

fn encode_cursor(target_id: &str, generation: &str, offset: usize) -> String {
    format!("{:016x}.{:016x}.{offset}", fnv1a64(target_id.as_bytes()), fnv1a64(generation.as_bytes()))
}

fn decode_cursor(cursor: &str, target_id: &str, generation: &str) -> Option<usize> {
    let expected_prefix = format!("{:016x}.{:016x}.", fnv1a64(target_id.as_bytes()), fnv1a64(generation.as_bytes()));
    cursor.strip_prefix(&expected_prefix)?.parse().ok()
}

fn error(status_code: u16, status_text: &'static str, code: &str, message: &str) -> EffectiveViewHttpResponse {
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

fn valid_id(value: &str, prefix: &str) -> bool {
    value.starts_with(prefix)
        && value.len() <= 255
        && value.is_ascii()
        && value.as_bytes().iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b':' | b'-'))
}

fn object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(entries.into_iter().map(|(key, value)| (key.to_string(), value)).collect())
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value { JsonValue::Object(map) => Some(map), _ => None }
}

fn as_object_mut(value: &mut JsonValue) -> Option<&mut BTreeMap<String, JsonValue>> {
    match value { JsonValue::Object(map) => Some(map), _ => None }
}

fn string_field<'a>(map: &'a BTreeMap<String, JsonValue>, key: &str) -> Option<&'a str> {
    match map.get(key) { Some(JsonValue::String(value)) => Some(value), _ => None }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
