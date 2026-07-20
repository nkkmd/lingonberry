use lingonberry_core::StorageBackend;
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const TRANSITION_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone)]
pub struct TransitionHttpResponse {
    pub status_code: u16,
    pub status_text: &'static str,
    pub body: JsonValue,
}

pub fn ingest_transition_request(
    request_json: &str,
    backend: &impl StorageBackend,
    state_dir: &Path,
) -> TransitionHttpResponse {
    let request = match parse_json(request_json) {
        Ok(value) => value,
        Err(error) => return error_response(400, "Bad Request", "LB_INVALID_JSON", &error.to_string()),
    };
    let request_map = match as_object(&request) {
        Some(value) => value,
        None => return error_response(400, "Bad Request", "LB_TRANSITION_ENVELOPE_INVALID", "transition request must be an object"),
    };
    if request_map.keys().any(|key| key != "transition" && key != "publisher") {
        return error_response(400, "Bad Request", "LB_TRANSITION_ENVELOPE_INVALID", "transition request contains unknown fields");
    }
    let transition = match request_map.get("transition") {
        Some(value) => value,
        None => return error_response(400, "Bad Request", "LB_TRANSITION_ENVELOPE_INVALID", "missing transition"),
    };
    let publisher = match request_map.get("publisher").and_then(as_object) {
        Some(value) => value,
        None => return error_response(400, "Bad Request", "LB_TRANSITION_ENVELOPE_INVALID", "missing publisher"),
    };
    let transition_map = match as_object(transition) {
        Some(value) => value,
        None => return error_response(400, "Bad Request", "LB_TRANSITION_INVALID", "transition must be an object"),
    };
    if let Err(message) = validate_publisher(publisher) {
        return error_response(400, "Bad Request", "LB_TRANSITION_ENVELOPE_INVALID", &message);
    }
    if let Err(message) = validate_transition(transition_map) {
        return error_response(400, "Bad Request", "LB_TRANSITION_INVALID", &message);
    }
    if let Err(message) = verify_transition_signature(&request) {
        return error_response(401, "Unauthorized", "LB_TRANSITION_SIGNATURE_INVALID", &message);
    }

    let transition_id = string_field(transition_map, "id").unwrap_or_default();
    let target_id = string_field(transition_map, "targetId").unwrap_or_default();
    let canonical_request = to_canonical_json(&request);
    let paths = TransitionPaths::new(state_dir);
    if let Err(error) = fs::create_dir_all(&paths.root) {
        return error_response(500, "Internal Server Error", "LB_TRANSITION_STORAGE_ERROR", &error.to_string());
    }

    match find_existing(&paths.log, transition_id) {
        Ok(Some(existing)) if existing == canonical_request => {
            return success_response(200, "OK", "duplicate", "LB_TRANSITION_DUPLICATE", transition_id, target_id, target_status(backend, target_id));
        }
        Ok(Some(_)) => {
            return error_response(409, "Conflict", "LB_TRANSITION_CONFLICT", "transition id already exists with different immutable content");
        }
        Ok(None) => {}
        Err(error) => return error_response(500, "Internal Server Error", "LB_TRANSITION_STORAGE_ERROR", &error),
    }

    let stored_at = unix_seconds().to_string();
    let record = JsonValue::Object(BTreeMap::from([
        ("storedAtUnixSeconds".to_string(), JsonValue::Number(stored_at)),
        ("transitionId".to_string(), JsonValue::String(transition_id.to_string())),
        ("targetId".to_string(), JsonValue::String(target_id.to_string())),
        ("request".to_string(), request.clone()),
    ]));
    if let Err(error) = append_line(&paths.log, &to_canonical_json(&record)) {
        return error_response(500, "Internal Server Error", "LB_TRANSITION_STORAGE_ERROR", &error);
    }

    let target_status = target_status(backend, target_id);
    let intent = JsonValue::Object(BTreeMap::from([
        ("ruleVersion".to_string(), JsonValue::String("lb.transition.reevaluation.queue.v1".to_string())),
        ("targetId".to_string(), JsonValue::String(target_id.to_string())),
        ("triggerTransitionId".to_string(), JsonValue::String(transition_id.to_string())),
        ("status".to_string(), JsonValue::String("pending".to_string())),
    ]));
    if let Err(error) = append_line(&paths.queue, &to_canonical_json(&intent)) {
        return error_response(500, "Internal Server Error", "LB_TRANSITION_STORAGE_ERROR", &error);
    }

    success_response(201, "Created", "stored", "LB_TRANSITION_STORED", transition_id, target_id, target_status)
}

struct TransitionPaths {
    root: PathBuf,
    log: PathBuf,
    queue: PathBuf,
}

impl TransitionPaths {
    fn new(state_dir: &Path) -> Self {
        let root = state_dir.join("transitions");
        Self {
            log: root.join("append-only.jsonl"),
            queue: root.join("reevaluation-queue.jsonl"),
            root,
        }
    }
}

fn validate_publisher(publisher: &BTreeMap<String, JsonValue>) -> Result<(), String> {
    if publisher.keys().any(|key| key != "publicKey" && key != "signature") {
        return Err("publisher contains unknown fields".to_string());
    }
    let public_key = string_field(publisher, "publicKey").ok_or_else(|| "publisher.publicKey is required".to_string())?;
    let signature = string_field(publisher, "signature").ok_or_else(|| "publisher.signature is required".to_string())?;
    if public_key.len() != 64 || !is_lower_hex(public_key) {
        return Err("publisher.publicKey must be 64 lowercase hex characters".to_string());
    }
    if signature.len() != 128 || !is_lower_hex(signature) {
        return Err("publisher.signature must be 128 lowercase hex characters".to_string());
    }
    Ok(())
}

fn validate_transition(transition: &BTreeMap<String, JsonValue>) -> Result<(), String> {
    let allowed = BTreeSet::from([
        "id", "schemaVersion", "objectType", "transitionType", "targetId", "replacementId",
        "supersedesTransitionIds", "issuedAt", "reason", "provenance", "rawRef", "identityClaims", "meta",
    ]);
    if transition.keys().any(|key| !allowed.contains(key.as_str())) {
        return Err("transition contains unknown fields".to_string());
    }
    for required in ["id", "schemaVersion", "objectType", "transitionType", "targetId", "issuedAt", "provenance", "rawRef"] {
        if !transition.contains_key(required) {
            return Err(format!("missing required field: {required}"));
        }
    }
    let id = string_field(transition, "id").ok_or_else(|| "id must be a string".to_string())?;
    let target_id = string_field(transition, "targetId").ok_or_else(|| "targetId must be a string".to_string())?;
    if !valid_protocol_id(id, "lb:transition:") || id.len() > 255 {
        return Err("id must be a bounded lb:transition ASCII identifier".to_string());
    }
    if !valid_protocol_id(target_id, "lb:obj:") || target_id.len() > 255 {
        return Err("targetId must be a bounded lb:obj ASCII identifier".to_string());
    }
    if string_field(transition, "schemaVersion") != Some(TRANSITION_SCHEMA_VERSION) {
        return Err("unsupported transition schemaVersion".to_string());
    }
    if string_field(transition, "objectType") != Some("transition") {
        return Err("objectType must be transition".to_string());
    }
    let transition_type = string_field(transition, "transitionType").ok_or_else(|| "transitionType must be a string".to_string())?;
    match transition_type {
        "replace" => {
            let replacement_id = string_field(transition, "replacementId").ok_or_else(|| "replace transition requires replacementId".to_string())?;
            if !valid_protocol_id(replacement_id, "lb:obj:") || replacement_id.len() > 255 {
                return Err("replacementId must be a bounded lb:obj ASCII identifier".to_string());
            }
        }
        "withdraw" => {
            if transition.contains_key("replacementId") {
                return Err("withdraw transition must not contain replacementId".to_string());
            }
        }
        _ => return Err("transitionType must be replace or withdraw".to_string()),
    }
    if !matches!(transition.get("issuedAt"), Some(JsonValue::String(value)) if valid_rfc3339_shape(value)) {
        return Err("issuedAt must be an RFC3339 timestamp with a zone".to_string());
    }
    if !matches!(transition.get("provenance"), Some(JsonValue::Object(_))) {
        return Err("provenance must be an object".to_string());
    }
    if !matches!(transition.get("rawRef"), Some(JsonValue::Object(_))) {
        return Err("rawRef must be an object".to_string());
    }
    if let Some(value) = transition.get("reason") {
        if !matches!(value, JsonValue::String(reason) if !reason.is_empty()) {
            return Err("reason must be a non-empty string".to_string());
        }
    }
    if let Some(value) = transition.get("supersedesTransitionIds") {
        let JsonValue::Array(items) = value else {
            return Err("supersedesTransitionIds must be an array".to_string());
        };
        if items.is_empty() {
            return Err("supersedesTransitionIds must not be empty".to_string());
        }
        let mut seen = BTreeSet::new();
        for item in items {
            let JsonValue::String(parent) = item else {
                return Err("supersedesTransitionIds entries must be strings".to_string());
            };
            if !valid_protocol_id(parent, "lb:transition:") || parent.len() > 255 {
                return Err("supersedesTransitionIds contains an invalid identifier".to_string());
            }
            if parent == id {
                return Err("transition must not supersede itself".to_string());
            }
            if !seen.insert(parent) {
                return Err("supersedesTransitionIds must not contain duplicates".to_string());
            }
        }
    }
    Ok(())
}

fn verify_transition_signature(request: &JsonValue) -> Result<(), String> {
    let map = as_object(request).ok_or_else(|| "transition request must be an object".to_string())?;
    let transition = map.get("transition").ok_or_else(|| "missing transition".to_string())?;
    let publisher = map.get("publisher").and_then(as_object).ok_or_else(|| "missing publisher".to_string())?;
    let public_key_hex = string_field(publisher, "publicKey").ok_or_else(|| "missing public key".to_string())?;
    let signature_hex = string_field(publisher, "signature").ok_or_else(|| "missing signature".to_string())?;
    let mut unsigned_publisher = publisher.clone();
    unsigned_publisher.remove("signature");
    let payload = to_canonical_json(&JsonValue::Object(BTreeMap::from([
        ("publisher".to_string(), JsonValue::Object(unsigned_publisher)),
        ("transition".to_string(), transition.clone()),
    ])));
    let public_key = decode_lower_hex(public_key_hex).ok_or_else(|| "invalid public key hex".to_string())?;
    let signature = decode_lower_hex(signature_hex).ok_or_else(|| "invalid signature hex".to_string())?;
    verify_ed25519_with_openssl(payload.as_bytes(), &public_key, &signature)
}

fn verify_ed25519_with_openssl(message: &[u8], public_key: &[u8], signature: &[u8]) -> Result<(), String> {
    if public_key.len() != 32 || signature.len() != 64 {
        return Err("invalid Ed25519 key or signature length".to_string());
    }
    let temp_root = std::env::temp_dir().join(format!("lingonberry-transition-signature-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()));
    fs::create_dir_all(&temp_root).map_err(|error| error.to_string())?;
    let key_path = temp_root.join("public-key.der");
    let signature_path = temp_root.join("signature.bin");
    let message_path = temp_root.join("message.bin");
    let mut der = vec![0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00];
    der.extend_from_slice(public_key);
    fs::write(&key_path, der).map_err(|error| error.to_string())?;
    fs::write(&signature_path, signature).map_err(|error| error.to_string())?;
    fs::write(&message_path, message).map_err(|error| error.to_string())?;
    let output = Command::new("openssl")
        .args(["pkeyutl", "-verify", "-pubin", "-inkey"])
        .arg(&key_path)
        .args(["-keyform", "DER", "-rawin", "-in"])
        .arg(&message_path)
        .arg("-sigfile")
        .arg(&signature_path)
        .output()
        .map_err(|error| format!("failed to run openssl: {error}"))?;
    let _ = fs::remove_dir_all(&temp_root);
    if output.status.success() {
        Ok(())
    } else {
        Err("publisher.signature does not verify the canonical transition request".to_string())
    }
}

fn find_existing(path: &Path, transition_id: &str) -> Result<Option<String>, String> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|error| error.to_string())?;
        let record = parse_json(&line).map_err(|error| error.to_string())?;
        let Some(record_map) = as_object(&record) else { continue };
        if string_field(record_map, "transitionId") == Some(transition_id) {
            return Ok(record_map.get("request").map(to_canonical_json));
        }
    }
    Ok(None)
}

fn append_line(path: &Path, line: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())?;
    file.sync_data().map_err(|error| error.to_string())
}

fn target_status(backend: &impl StorageBackend, target_id: &str) -> &'static str {
    match backend.get(target_id) {
        Ok(Some(_)) => "available",
        Ok(None) | Err(_) => "missing",
    }
}

fn success_response(status_code: u16, status_text: &'static str, status: &str, code: &str, transition_id: &str, target_id: &str, target_status: &str) -> TransitionHttpResponse {
    TransitionHttpResponse {
        status_code,
        status_text,
        body: JsonValue::Object(BTreeMap::from([
            ("status".to_string(), JsonValue::String(status.to_string())),
            ("code".to_string(), JsonValue::String(code.to_string())),
            ("transitionId".to_string(), JsonValue::String(transition_id.to_string())),
            ("targetId".to_string(), JsonValue::String(target_id.to_string())),
            ("targetStatus".to_string(), JsonValue::String(target_status.to_string())),
        ])),
    }
}

fn error_response(status_code: u16, status_text: &'static str, code: &str, message: &str) -> TransitionHttpResponse {
    TransitionHttpResponse {
        status_code,
        status_text,
        body: JsonValue::Object(BTreeMap::from([
            ("status".to_string(), JsonValue::String("error".to_string())),
            ("code".to_string(), JsonValue::String(code.to_string())),
            ("message".to_string(), JsonValue::String(message.to_string())),
        ])),
    }
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
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
        && value.as_bytes().iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'~' | b':' | b'-'))
}

fn valid_rfc3339_shape(value: &str) -> bool {
    value.contains('T') && (value.ends_with('Z') || value.rfind('+').is_some() || value.rfind('-').is_some_and(|index| index > 9))
}

fn is_lower_hex(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn decode_lower_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 || !is_lower_hex(value) {
        return None;
    }
    (0..value.len()).step_by(2).map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok()).collect()
}

fn unix_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
