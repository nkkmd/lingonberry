use crate::effective_view::{self, EffectiveViewHttpResponse};
use lingonberry_core::StorageBackend;
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn effective_view_http_response(
    target_id: &str,
    backend: &impl StorageBackend,
    state_dir: &Path,
) -> EffectiveViewHttpResponse {
    match authorized_graph_has_cycle(target_id, backend, state_dir) {
        Ok(false) => effective_view::effective_view_http_response(target_id, backend, state_dir),
        Ok(true) => cyclic_graph_response(target_id, backend, state_dir),
        Err(message) => EffectiveViewHttpResponse {
            status_code: 500,
            status_text: "Internal Server Error",
            body: error_body("LB_TRANSITION_STORAGE_ERROR", &message),
        },
    }
}

fn authorized_graph_has_cycle(
    target_id: &str,
    backend: &impl StorageBackend,
    state_dir: &Path,
) -> Result<bool, String> {
    let Some(target_publisher) = target_publisher_key(backend, target_id) else {
        return Ok(false);
    };
    let path = state_dir.join("transitions/append-only.jsonl");
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    let mut graph = BTreeMap::<String, Vec<String>>::new();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        let record = parse_json(line).map_err(|error| error.to_string())?;
        let record = as_object(&record).ok_or_else(|| "transition record invalid".to_string())?;
        if string_field(record, "targetId") != Some(target_id) {
            continue;
        }
        let request = record
            .get("request")
            .and_then(as_object)
            .ok_or_else(|| "transition request invalid".to_string())?;
        let transition = request
            .get("transition")
            .and_then(as_object)
            .ok_or_else(|| "transition missing".to_string())?;
        let publisher = request
            .get("publisher")
            .and_then(as_object)
            .ok_or_else(|| "publisher missing".to_string())?;
        if string_field(publisher, "publicKey") != Some(target_publisher.as_str()) {
            continue;
        }
        let id = string_field(transition, "id")
            .ok_or_else(|| "transition id missing".to_string())?
            .to_string();
        let parents = match transition.get("supersedesTransitionIds") {
            Some(JsonValue::Array(items)) => items
                .iter()
                .filter_map(|item| match item {
                    JsonValue::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        graph.insert(id, parents);
    }
    let ids = graph.keys().cloned().collect::<BTreeSet<_>>();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in &ids {
        if visit(id, &graph, &ids, &mut visiting, &mut visited) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn visit(
    id: &str,
    graph: &BTreeMap<String, Vec<String>>,
    ids: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
) -> bool {
    if visited.contains(id) {
        return false;
    }
    if !visiting.insert(id.to_string()) {
        return true;
    }
    if let Some(parents) = graph.get(id) {
        for parent in parents.iter().filter(|parent| ids.contains(*parent)) {
            if visit(parent, graph, ids, visiting, visited) {
                return true;
            }
        }
    }
    visiting.remove(id);
    visited.insert(id.to_string());
    false
}

fn cyclic_graph_response(
    target_id: &str,
    backend: &impl StorageBackend,
    state_dir: &Path,
) -> EffectiveViewHttpResponse {
    let diagnostic = object(vec![
        ("kind", JsonValue::String("transition".to_string())),
        ("evidenceId", JsonValue::String(target_id.to_string())),
        ("classification", JsonValue::String("corrupt".to_string())),
        (
            "reasonCode",
            JsonValue::String("LB_EVIDENCE_VALIDATION_FAILED".to_string()),
        ),
    ]);
    let observation = object(vec![
        ("generation", JsonValue::Null),
        (
            "snapshotClassification",
            JsonValue::String("incomplete".to_string()),
        ),
        (
            "diagnosticSummary",
            object(vec![
                ("total", JsonValue::Number("1".to_string())),
                ("returned", JsonValue::Number("1".to_string())),
                ("truncated", JsonValue::Bool(false)),
            ]),
        ),
        ("diagnostics", JsonValue::Array(vec![diagnostic.clone()])),
        ("allDiagnostics", JsonValue::Array(vec![diagnostic])),
    ]);
    if let Some(mut previous) = load_snapshot(state_dir, target_id) {
        if let Some(root) = as_object_mut(&mut previous) {
            if let Some(view) = root.get_mut("effectiveView").and_then(as_object_mut) {
                view.insert(
                    "freshness".to_string(),
                    JsonValue::String("stale".to_string()),
                );
            }
            root.insert("evidenceObservation".to_string(), observation);
        }
        let _ = persist_snapshot(state_dir, target_id, &previous);
        return EffectiveViewHttpResponse {
            status_code: 200,
            status_text: "OK",
            body: previous,
        };
    }
    let original = backend
        .get(target_id)
        .ok()
        .flatten()
        .map(|record| record.object)
        .unwrap_or(JsonValue::Null);
    let body = object(vec![
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
        ("evidenceObservation", observation),
    ]);
    let _ = persist_snapshot(state_dir, target_id, &body);
    EffectiveViewHttpResponse {
        status_code: 200,
        status_text: "OK",
        body,
    }
}

fn target_publisher_key(backend: &impl StorageBackend, target_id: &str) -> Option<String> {
    let raw = backend.get_raw_request(target_id).ok().flatten()?;
    let request = parse_json(&raw.request_json).ok()?;
    let publisher = as_object(&request)?.get("publisher").and_then(as_object)?;
    string_field(publisher, "publicKey").map(ToString::to_string)
}

fn snapshot_path(state_dir: &Path, target_id: &str) -> PathBuf {
    state_dir
        .join("transitions/effective")
        .join(format!("{:016x}.json", fnv1a64(target_id.as_bytes())))
}

fn load_snapshot(state_dir: &Path, target_id: &str) -> Option<JsonValue> {
    parse_json(&fs::read_to_string(snapshot_path(state_dir, target_id)).ok()?).ok()
}

fn persist_snapshot(state_dir: &Path, target_id: &str, body: &JsonValue) -> Result<(), String> {
    let path = snapshot_path(state_dir, target_id);
    let parent = path
        .parent()
        .ok_or_else(|| "snapshot path missing parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    fs::write(path, to_canonical_json(body)).map_err(|error| error.to_string())
}

fn error_body(code: &str, message: &str) -> JsonValue {
    object(vec![
        ("status", JsonValue::String("error".to_string())),
        ("code", JsonValue::String(code.to_string())),
        ("message", JsonValue::String(message.to_string())),
    ])
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

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
