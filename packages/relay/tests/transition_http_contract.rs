use lingonberry_core::{
    AppendOutcome, RawRequestRecord, StorageBackend, StoreError, StoredCatalogRecord,
    StoredReplayRecord,
};
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use lingonberry_relay::ingest_transition_request;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
struct EmptyBackend;

impl StorageBackend for EmptyBackend {
    fn append_publish_request(
        &self,
        _request_json: &str,
        _finalized: &lingonberry_protocol::FinalizedKnowledgeObject,
    ) -> Result<AppendOutcome, StoreError> {
        unreachable!()
    }

    fn get(&self, _canonical_id: &str) -> Result<Option<StoredCatalogRecord>, StoreError> {
        Ok(None)
    }

    fn get_raw_request(&self, _canonical_id: &str) -> Result<Option<RawRequestRecord>, StoreError> {
        Ok(None)
    }

    fn list_ids(&self) -> Result<Vec<String>, StoreError> {
        Ok(Vec::new())
    }

    fn subscribe(
        &self,
        _object_type: Option<&str>,
    ) -> Result<Vec<StoredCatalogRecord>, StoreError> {
        Ok(Vec::new())
    }

    fn replay(&self) -> Result<Vec<StoredReplayRecord>, StoreError> {
        Ok(Vec::new())
    }
}

#[test]
fn signed_orphan_transition_is_append_only_idempotent_and_conflict_safe() {
    let state_dir = unique_temp_dir("transition-contract");
    fs::create_dir_all(&state_dir).expect("create state directory");
    let backend = EmptyBackend;
    let request = signed_request("withdraw", None, "first");

    let stored = ingest_transition_request(&request, &backend, &state_dir);
    assert_eq!(stored.status_code, 201);
    assert!(to_canonical_json(&stored.body).contains("LB_TRANSITION_STORED"));
    assert!(to_canonical_json(&stored.body).contains("\"targetStatus\":\"missing\""));

    let duplicate = ingest_transition_request(&request, &backend, &state_dir);
    assert_eq!(duplicate.status_code, 200);
    assert!(to_canonical_json(&duplicate.body).contains("LB_TRANSITION_DUPLICATE"));

    let conflict = signed_request("withdraw", None, "changed");
    let conflicting = ingest_transition_request(&conflict, &backend, &state_dir);
    assert_eq!(conflicting.status_code, 409);
    assert!(to_canonical_json(&conflicting.body).contains("LB_TRANSITION_CONFLICT"));

    let log = fs::read_to_string(state_dir.join("transitions/append-only.jsonl"))
        .expect("read append-only log");
    assert_eq!(log.lines().count(), 1);
    let queue = fs::read_to_string(state_dir.join("transitions/reevaluation-queue.jsonl"))
        .expect("read queue log");
    assert_eq!(queue.lines().count(), 1);
    fs::remove_dir_all(state_dir).ok();
}

fn signed_request(transition_type: &str, replacement_id: Option<&str>, reason: &str) -> String {
    let key_dir = unique_temp_dir("transition-key");
    fs::create_dir_all(&key_dir).expect("create key directory");
    let private_key = key_dir.join("private.pem");
    let public_der = key_dir.join("public.der");
    let message_path = key_dir.join("message.bin");
    let signature_path = key_dir.join("signature.bin");
    run(Command::new("openssl")
        .args(["genpkey", "-algorithm", "ED25519", "-out"])
        .arg(&private_key));
    run(Command::new("openssl")
        .args(["pkey", "-in"])
        .arg(&private_key)
        .args(["-pubout", "-outform", "DER", "-out"])
        .arg(&public_der));
    let der = fs::read(&public_der).expect("read public key");
    let public_key = hex(&der[der.len() - 32..]);

    let mut transition = BTreeMap::from([
        (
            "id".to_string(),
            JsonValue::String("lb:transition:integration-contract".to_string()),
        ),
        (
            "schemaVersion".to_string(),
            JsonValue::String("0.1.0".to_string()),
        ),
        (
            "objectType".to_string(),
            JsonValue::String("transition".to_string()),
        ),
        (
            "transitionType".to_string(),
            JsonValue::String(transition_type.to_string()),
        ),
        (
            "targetId".to_string(),
            JsonValue::String("lb:obj:missing-target".to_string()),
        ),
        (
            "issuedAt".to_string(),
            JsonValue::String("2026-07-20T08:00:00Z".to_string()),
        ),
        ("reason".to_string(), JsonValue::String(reason.to_string())),
        ("provenance".to_string(), JsonValue::Object(BTreeMap::new())),
        ("rawRef".to_string(), JsonValue::Object(BTreeMap::new())),
    ]);
    if let Some(replacement_id) = replacement_id {
        transition.insert(
            "replacementId".to_string(),
            JsonValue::String(replacement_id.to_string()),
        );
    }
    let unsigned = JsonValue::Object(BTreeMap::from([
        (
            "publisher".to_string(),
            JsonValue::Object(BTreeMap::from([(
                "publicKey".to_string(),
                JsonValue::String(public_key.clone()),
            )])),
        ),
        (
            "transition".to_string(),
            JsonValue::Object(transition.clone()),
        ),
    ]));
    fs::write(&message_path, to_canonical_json(&unsigned)).expect("write message");
    run(Command::new("openssl")
        .args(["pkeyutl", "-sign", "-inkey"])
        .arg(&private_key)
        .args(["-rawin", "-in"])
        .arg(&message_path)
        .arg("-out")
        .arg(&signature_path));
    let signature = hex(&fs::read(&signature_path).expect("read signature"));
    let request = JsonValue::Object(BTreeMap::from([
        ("transition".to_string(), JsonValue::Object(transition)),
        (
            "publisher".to_string(),
            JsonValue::Object(BTreeMap::from([
                ("publicKey".to_string(), JsonValue::String(public_key)),
                ("signature".to_string(), JsonValue::String(signature)),
            ])),
        ),
    ]));
    let json = to_canonical_json(&request);
    parse_json(&json).expect("signed request is JSON");
    fs::remove_dir_all(key_dir).ok();
    json
}

fn run(command: &mut Command) {
    let output = command.output().expect("run openssl");
    assert!(
        output.status.success(),
        "openssl failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lingonberry-{label}-{}-{nonce}",
        std::process::id()
    ))
}
