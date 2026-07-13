use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::{
    create_quarantine_replacement_preview, export_complete_quarantine_backup,
    verify_quarantine_replacement_proof, QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_PLAN_FILE, QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE,
    QUARANTINE_REPLACEMENT_PROOF_FILE,
};
use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

const VALID_VECTOR: &str = include_str!(
    "../../../docs/operations/fixtures/quarantine-replacement-policy/valid-canonical-representation.json"
);
const DUPLICATE_VECTOR: &str = include_str!(
    "../../../docs/operations/fixtures/quarantine-replacement-policy/reject-duplicate-terminal-key.json"
);
const SEMANTIC_CHANGE_VECTOR: &str = include_str!(
    "../../../docs/operations/fixtures/quarantine-replacement-policy/reject-semantic-change.json"
);

fn temp_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "lingonberry-integration-{label}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos()
    ))
}

fn digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn object(value: &JsonValue) -> &BTreeMap<String, JsonValue> {
    match value {
        JsonValue::Object(map) => map,
        _ => panic!("expected JSON object"),
    }
}

fn object_mut(value: &mut JsonValue) -> &mut BTreeMap<String, JsonValue> {
    match value {
        JsonValue::Object(map) => map,
        _ => panic!("expected JSON object"),
    }
}

fn array_mut(value: &mut JsonValue) -> &mut Vec<JsonValue> {
    match value {
        JsonValue::Array(values) => values,
        _ => panic!("expected JSON array"),
    }
}

fn write_artifact(path: &Path, value: &JsonValue, digest_path: &Path) -> String {
    let text = to_canonical_json(value);
    let value_digest = digest(text.as_bytes());
    fs::write(path, text).expect("write artifact");
    fs::write(digest_path, format!("{value_digest}\n")).expect("write digest");
    value_digest
}

#[test]
fn accepts_valid_canonical_representation_vector() {
    assert!(VALID_VECTOR.contains("valid canonical representation replacement"));

    let state = temp_dir("valid-state");
    let backup = temp_dir("valid-backup");
    let proof = temp_dir("valid-proof");
    fs::create_dir_all(&state).expect("create state");
    fs::write(
        state.join("quarantine-dismissals.jsonl"),
        "{\"note\":\"reviewed\",\"quarantineId\":\"q-001\",\"dismissedAt\":\"2026-07-13T00:00:00Z\"}\n",
    )
    .expect("write source ledger");

    export_complete_quarantine_backup(&state, &backup).expect("export backup");
    let report = create_quarantine_replacement_preview(&state, &backup, &proof)
        .expect("create replacement preview");

    assert_eq!(report.replacement_lines, 1);
    assert!(!report.mutation_allowed);
    assert_eq!(
        verify_quarantine_replacement_proof(&proof).expect("verify proof"),
        report
    );

    let _ = fs::remove_dir_all(state);
    let _ = fs::remove_dir_all(backup);
    let _ = fs::remove_dir_all(proof);
}

#[test]
fn rejects_duplicate_terminal_key_vector() {
    assert!(DUPLICATE_VECTOR.contains("duplicate terminal key"));

    let state = temp_dir("duplicate-state");
    let backup = temp_dir("duplicate-backup");
    let proof = temp_dir("duplicate-proof");
    fs::create_dir_all(&state).expect("create state");
    fs::write(
        state.join("quarantine-resolutions.jsonl"),
        "{\"quarantineId\":\"q-duplicate\"}\n{\"quarantineId\":\"q-duplicate\"}\n",
    )
    .expect("write duplicate ledger");

    export_complete_quarantine_backup(&state, &backup).expect("export backup");
    let error = create_quarantine_replacement_preview(&state, &backup, &proof)
        .expect_err("duplicate terminal key must fail");
    assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_CORRUPT");

    let _ = fs::remove_dir_all(state);
    let _ = fs::remove_dir_all(backup);
    let _ = fs::remove_dir_all(proof);
}

#[test]
fn rejects_semantic_change_even_with_recomputed_digests() {
    assert!(SEMANTIC_CHANGE_VECTOR.contains("semantic change"));

    let state = temp_dir("semantic-state");
    let backup = temp_dir("semantic-backup");
    let proof_dir = temp_dir("semantic-proof");
    fs::create_dir_all(&state).expect("create state");
    fs::write(
        state.join("quarantine-dismissals.jsonl"),
        "{\"quarantineId\":\"q-semantic\",\"reason\":\"not-actionable\"}\n",
    )
    .expect("write source ledger");

    export_complete_quarantine_backup(&state, &backup).expect("export backup");
    create_quarantine_replacement_preview(&state, &backup, &proof_dir)
        .expect("create replacement preview");

    let plan_path = proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_FILE);
    let mut plan =
        parse_json(&fs::read_to_string(&plan_path).expect("read plan")).expect("parse plan");
    let ledgers = array_mut(
        object_mut(&mut plan)
            .get_mut("ledgers")
            .expect("plan ledgers"),
    );
    let dismissal = ledgers
        .iter_mut()
        .find(|ledger| {
            matches!(
                object(ledger).get("ledger"),
                Some(JsonValue::String(name)) if name == "quarantine-dismissals.jsonl"
            )
        })
        .expect("dismissal ledger");
    let entries = array_mut(
        object_mut(dismissal)
            .get_mut("entries")
            .expect("ledger entries"),
    );
    let replacement = object_mut(
        object_mut(entries.first_mut().expect("first entry"))
            .get_mut("replacement")
            .expect("replacement object"),
    );
    replacement.insert(
        "valueDigest".to_string(),
        JsonValue::String("fnv1a64:0000000000000000".to_string()),
    );

    let plan_digest = write_artifact(
        &plan_path,
        &plan,
        &proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE),
    );

    let proof_path = proof_dir.join(QUARANTINE_REPLACEMENT_PROOF_FILE);
    let mut proof =
        parse_json(&fs::read_to_string(&proof_path).expect("read proof")).expect("parse proof");
    object_mut(&mut proof).insert("planDigest".to_string(), JsonValue::String(plan_digest));
    write_artifact(
        &proof_path,
        &proof,
        &proof_dir.join(QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE),
    );

    let error =
        verify_quarantine_replacement_proof(&proof_dir).expect_err("semantic mismatch must fail");
    assert_eq!(error.code, "LB_QUARANTINE_REPLACEMENT_PROOF");

    let _ = fs::remove_dir_all(state);
    let _ = fs::remove_dir_all(backup);
    let _ = fs::remove_dir_all(proof_dir);
}
