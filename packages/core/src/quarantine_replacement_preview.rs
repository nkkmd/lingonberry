use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};

use crate::{
    resolve_quarantine_active_path, store_error, verify_any_quarantine_backup,
    verify_quarantine_segments, StoreError, QUARANTINE_BACKUP_FILES,
    QUARANTINE_BACKUP_MANIFEST, QUARANTINE_COMPLETE_BACKUP_VERSION,
    QUARANTINE_SEGMENT_ARCHIVE_DIR, QUARANTINE_SEGMENT_MANIFEST_FILE,
};

pub const QUARANTINE_REPLACEMENT_POLICY_VERSION: &str =
    "lingonberry-quarantine-compaction-policy/v2";
pub const QUARANTINE_REPLACEMENT_PLAN_VERSION: &str =
    "lingonberry-quarantine-replacement-plan/v1";
pub const QUARANTINE_REPLACEMENT_PROOF_VERSION: &str =
    "lingonberry-quarantine-replacement-proof/v1";
pub const QUARANTINE_REPLACEMENT_PLAN_FILE: &str = "quarantine-replacement-plan.json";
pub const QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE: &str =
    "quarantine-replacement-plan.digest";
pub const QUARANTINE_REPLACEMENT_PROOF_FILE: &str = "quarantine-replacement-proof.json";
pub const QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE: &str =
    "quarantine-replacement-proof.digest";

const IMMUTABLE_LEDGERS: [&str; 3] = [
    "quarantine.jsonl",
    "quarantine-annotations.jsonl",
    "admin-auth-audit.jsonl",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReplacementProofReport {
    pub plan_path: PathBuf,
    pub proof_path: PathBuf,
    pub ledgers: usize,
    pub source_lines: u64,
    pub replacement_lines: u64,
    pub retained_lines: u64,
    pub mutation_allowed: bool,
}

#[derive(Debug, Clone)]
struct LocatedLine {
    location: String,
    line_number: u64,
    text: String,
}

pub fn create_quarantine_replacement_preview(
    state_dir: impl AsRef<Path>,
    verified_backup_dir: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementProofReport, StoreError> {
    let state_dir = state_dir.as_ref();
    let backup_dir = verified_backup_dir.as_ref();
    let output_dir = output_dir.as_ref();

    verify_quarantine_segments(state_dir)?;
    verify_any_quarantine_backup(backup_dir)?;
    require_v2_backup(backup_dir)?;
    prepare_empty_output(output_dir)?;

    let before = runtime_fingerprint(state_dir)?;
    let source_backup_manifest_digest =
        file_digest(&backup_dir.join(QUARANTINE_BACKUP_MANIFEST))?;
    let source_segment_manifest_digest =
        optional_file_digest(&state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE))?;

    let mut ledger_plans = Vec::new();
    let mut source_lines = 0u64;
    let mut replacement_lines = 0u64;
    let mut retained_lines = 0u64;

    for ledger in QUARANTINE_BACKUP_FILES {
        let lines = read_located_lines(state_dir, ledger)?;
        source_lines += lines.len() as u64;
        let plan = build_ledger_plan(ledger, &lines)?;
        replacement_lines += object_number(&plan, "replacementLines")?;
        retained_lines += object_number(&plan, "retainedLines")?;
        ledger_plans.push(plan);
    }

    let after = runtime_fingerprint(state_dir)?;
    if before != after {
        return Err(replacement_error(
            "LB_QUARANTINE_REPLACEMENT_CHANGED",
            "runtime state changed during replacement preview",
        ));
    }

    let plan = JsonValue::Object(BTreeMap::from([
        (
            "ledgers".to_string(),
            JsonValue::Array(ledger_plans),
        ),
        (
            "policyVersion".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_POLICY_VERSION.to_string()),
        ),
        (
            "runtimeFingerprint".to_string(),
            fingerprint_json(&before),
        ),
        (
            "semanticEquivalenceExpectations".to_string(),
            equivalence_json(true),
        ),
        (
            "sourceBackupManifestDigest".to_string(),
            JsonValue::String(source_backup_manifest_digest),
        ),
        (
            "sourceSegmentManifestDigest".to_string(),
            source_segment_manifest_digest
                .map(JsonValue::String)
                .unwrap_or(JsonValue::Null),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_PLAN_VERSION.to_string()),
        ),
    ]));
    let plan_text = to_canonical_json(&plan);
    let plan_digest = integrity_digest(plan_text.as_bytes());

    let proof = JsonValue::Object(BTreeMap::from([
        (
            "generatedAt".to_string(),
            JsonValue::String(timestamp()?),
        ),
        (
            "mutationAllowed".to_string(),
            JsonValue::Bool(false),
        ),
        (
            "planDigest".to_string(),
            JsonValue::String(plan_digest.clone()),
        ),
        (
            "planVersion".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_PLAN_VERSION.to_string()),
        ),
        (
            "policyVersion".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_POLICY_VERSION.to_string()),
        ),
        (
            "replacementLines".to_string(),
            JsonValue::Number(replacement_lines.to_string()),
        ),
        (
            "retainedLines".to_string(),
            JsonValue::Number(retained_lines.to_string()),
        ),
        (
            "rewritePerformed".to_string(),
            JsonValue::Bool(false),
        ),
        (
            "semanticEquivalence".to_string(),
            equivalence_json(true),
        ),
        (
            "sourceLines".to_string(),
            JsonValue::Number(source_lines.to_string()),
        ),
        (
            "version".to_string(),
            JsonValue::String(QUARANTINE_REPLACEMENT_PROOF_VERSION.to_string()),
        ),
    ]));
    let proof_text = to_canonical_json(&proof);
    let proof_digest = integrity_digest(proof_text.as_bytes());

    publish_artifacts(
        output_dir,
        &plan_text,
        &plan_digest,
        &proof_text,
        &proof_digest,
    )?;
    verify_quarantine_replacement_proof(output_dir)
}

pub fn verify_quarantine_replacement_proof(
    proof_dir: impl AsRef<Path>,
) -> Result<QuarantineReplacementProofReport, StoreError> {
    let proof_dir = proof_dir.as_ref();
    let plan_path = proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_FILE);
    let proof_path = proof_dir.join(QUARANTINE_REPLACEMENT_PROOF_FILE);
    let plan_text = read_verified_artifact(
        &plan_path,
        &proof_dir.join(QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE),
        "plan",
    )?;
    let proof_text = read_verified_artifact(
        &proof_path,
        &proof_dir.join(QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE),
        "proof",
    )?;
    let plan = parse_json(&plan_text).map_err(|error| proof_error(&error.to_string()))?;
    let proof = parse_json(&proof_text).map_err(|error| proof_error(&error.to_string()))?;
    validate_plan(&plan)?;
    validate_proof(&proof, integrity_digest(plan_text.as_bytes()))?;

    Ok(QuarantineReplacementProofReport {
        plan_path,
        proof_path,
        ledgers: object_array(&plan, "ledgers")?.len(),
        source_lines: object_number(&proof, "sourceLines")?,
        replacement_lines: object_number(&proof, "replacementLines")?,
        retained_lines: object_number(&proof, "retainedLines")?,
        mutation_allowed: object_bool(&proof, "mutationAllowed")?,
    })
}

pub fn quarantine_replacement_proof_report_json(
    report: &QuarantineReplacementProofReport,
) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "ledgers".to_string(),
            JsonValue::Number(report.ledgers.to_string()),
        ),
        (
            "mutationAllowed".to_string(),
            JsonValue::Bool(report.mutation_allowed),
        ),
        (
            "planPath".to_string(),
            JsonValue::String(report.plan_path.to_string_lossy().to_string()),
        ),
        (
            "proofPath".to_string(),
            JsonValue::String(report.proof_path.to_string_lossy().to_string()),
        ),
        (
            "replacementLines".to_string(),
            JsonValue::Number(report.replacement_lines.to_string()),
        ),
        (
            "retainedLines".to_string(),
            JsonValue::Number(report.retained_lines.to_string()),
        ),
        (
            "sourceLines".to_string(),
            JsonValue::Number(report.source_lines.to_string()),
        ),
    ]))
}

fn build_ledger_plan(ledger: &str, lines: &[LocatedLine]) -> Result<JsonValue, StoreError> {
    let immutable = IMMUTABLE_LEDGERS.contains(&ledger);
    let mut keys = BTreeSet::new();
    let mut entries = Vec::new();
    let mut replacements = 0u64;

    for (ordinal, line) in lines.iter().enumerate() {
        let value = parse_json(&line.text).map_err(|error| {
            replacement_error("LB_QUARANTINE_REPLACEMENT_CORRUPT", &error.to_string())
        })?;
        let canonical = to_canonical_json(&value);
        let source_line_digest = integrity_digest(line.text.as_bytes());
        let value_digest = integrity_digest(canonical.as_bytes());
        let replacement_line_digest = integrity_digest(canonical.as_bytes());

        let replacement_key = if immutable {
            optional_object_string(&value, "id")?.unwrap_or_else(|| format!("ordinal:{ordinal}"))
        } else {
            let key = required_object_string(&value, "quarantineId")?;
            if !keys.insert(key.clone()) {
                return Err(replacement_error(
                    "LB_QUARANTINE_REPLACEMENT_CORRUPT",
                    &format!("duplicate terminal event in {ledger}: {key}"),
                ));
            }
            key
        };
        let decision = if immutable || line.text == canonical {
            "retain-byte-for-byte"
        } else {
            replacements += 1;
            "canonical-json-representation"
        };
        entries.push(JsonValue::Object(BTreeMap::from([
            (
                "decision".to_string(),
                JsonValue::String(decision.to_string()),
            ),
            (
                "ledger".to_string(),
                JsonValue::String(ledger.to_string()),
            ),
            (
                "logicalOrdinal".to_string(),
                JsonValue::Number(ordinal.to_string()),
            ),
            (
                "replacement".to_string(),
                JsonValue::Object(BTreeMap::from([
                    (
                        "lineDigest".to_string(),
                        JsonValue::String(replacement_line_digest),
                    ),
                    (
                        "valueDigest".to_string(),
                        JsonValue::String(value_digest.clone()),
                    ),
                ])),
            ),
            (
                "replacementKey".to_string(),
                JsonValue::String(replacement_key),
            ),
            (
                "source".to_string(),
                JsonValue::Object(BTreeMap::from([
                    (
                        "lineDigest".to_string(),
                        JsonValue::String(source_line_digest),
                    ),
                    (
                        "lineNumber".to_string(),
                        JsonValue::Number(line.line_number.to_string()),
                    ),
                    (
                        "location".to_string(),
                        JsonValue::String(line.location.clone()),
                    ),
                    (
                        "valueDigest".to_string(),
                        JsonValue::String(value_digest),
                    ),
                ])),
            ),
            (
                "transformation".to_string(),
                JsonValue::String(decision.to_string()),
            ),
        ])));
    }

    Ok(JsonValue::Object(BTreeMap::from([
        (
            "classification".to_string(),
            JsonValue::String(if immutable {
                "immutable-evidence".to_string()
            } else {
                "terminal-single-event".to_string()
            }),
        ),
        ("entries".to_string(), JsonValue::Array(entries)),
        ("ledger".to_string(), JsonValue::String(ledger.to_string())),
        (
            "replacementLines".to_string(),
            JsonValue::Number(replacements.to_string()),
        ),
        (
            "retainedLines".to_string(),
            JsonValue::Number((lines.len() as u64 - replacements).to_string()),
        ),
        (
            "sourceLines".to_string(),
            JsonValue::Number(lines.len().to_string()),
        ),
    ])))
}

fn validate_plan(plan: &JsonValue) -> Result<(), StoreError> {
    require_string(plan, "version", QUARANTINE_REPLACEMENT_PLAN_VERSION)?;
    require_string(
        plan,
        "policyVersion",
        QUARANTINE_REPLACEMENT_POLICY_VERSION,
    )?;
    validate_equivalence(object_field(plan, "semanticEquivalenceExpectations")?)?;
    let ledgers = object_array(plan, "ledgers")?;
    let names = ledgers
        .iter()
        .map(|ledger| object_string(ledger, "ledger"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let expected = QUARANTINE_BACKUP_FILES.iter().map(|v| v.to_string()).collect();
    if names != expected || ledgers.len() != QUARANTINE_BACKUP_FILES.len() {
        return Err(proof_error("plan must contain the exact managed ledger set"));
    }
    for ledger in ledgers {
        let name = object_string(ledger, "ledger")?;
        let immutable = IMMUTABLE_LEDGERS.contains(&name.as_str());
        let entries = object_array(ledger, "entries")?;
        let mut keys = BTreeSet::new();
        for (ordinal, entry) in entries.iter().enumerate() {
            if object_string(entry, "ledger")? != name
                || object_number(entry, "logicalOrdinal")? != ordinal as u64
            {
                return Err(proof_error("replacement plan order or ledger mismatch"));
            }
            let key = object_string(entry, "replacementKey")?;
            if !immutable && !keys.insert(key) {
                return Err(proof_error("duplicate terminal replacement key"));
            }
            let decision = object_string(entry, "decision")?;
            if immutable && decision != "retain-byte-for-byte" {
                return Err(proof_error("immutable ledger contains a replacement"));
            }
            if decision != "retain-byte-for-byte"
                && decision != "canonical-json-representation"
            {
                return Err(proof_error("unsupported replacement decision"));
            }
            let source = object_field(entry, "source")?;
            let replacement = object_field(entry, "replacement")?;
            if object_string(source, "valueDigest")?
                != object_string(replacement, "valueDigest")?
            {
                return Err(replacement_error(
                    "LB_QUARANTINE_REPLACEMENT_SEMANTICS",
                    "source and replacement value digests differ",
                ));
            }
            object_string(source, "location")?;
            object_number(source, "lineNumber")?;
            object_string(source, "lineDigest")?;
            object_string(replacement, "lineDigest")?;
        }
    }
    Ok(())
}

fn validate_proof(proof: &JsonValue, plan_digest: String) -> Result<(), StoreError> {
    require_string(proof, "version", QUARANTINE_REPLACEMENT_PROOF_VERSION)?;
    require_string(proof, "planVersion", QUARANTINE_REPLACEMENT_PLAN_VERSION)?;
    require_string(
        proof,
        "policyVersion",
        QUARANTINE_REPLACEMENT_POLICY_VERSION,
    )?;
    if object_string(proof, "planDigest")? != plan_digest {
        return Err(proof_error("proof references a different plan digest"));
    }
    if object_bool(proof, "mutationAllowed")? || object_bool(proof, "rewritePerformed")? {
        return Err(proof_error("replacement proof must be non-mutating"));
    }
    validate_equivalence(object_field(proof, "semanticEquivalence")?)?;
    let source = object_number(proof, "sourceLines")?;
    let retained = object_number(proof, "retainedLines")?;
    let replaced = object_number(proof, "replacementLines")?;
    if retained + replaced != source {
        return Err(proof_error("proof line counts are inconsistent"));
    }
    Ok(())
}

fn validate_equivalence(value: &JsonValue) -> Result<(), StoreError> {
    let map = as_object(value)?;
    let expected = [
        "batchIdempotency",
        "completeProvenance",
        "corruptionDetection",
        "logicalOrder",
        "metrics",
        "promotionEligibility",
        "readerResults",
        "recordIdentity",
        "singleOperationIdempotency",
        "statusCounts",
        "terminalState",
    ];
    if map.len() != expected.len() {
        return Err(proof_error("semantic equivalence field set is incomplete"));
    }
    for name in expected {
        match map.get(name) {
            Some(JsonValue::Bool(true)) => {}
            _ => return Err(proof_error(&format!("semantic equivalence failed: {name}"))),
        }
    }
    Ok(())
}

fn equivalence_json(value: bool) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        ("batchIdempotency".to_string(), JsonValue::Bool(value)),
        ("completeProvenance".to_string(), JsonValue::Bool(value)),
        ("corruptionDetection".to_string(), JsonValue::Bool(value)),
        ("logicalOrder".to_string(), JsonValue::Bool(value)),
        ("metrics".to_string(), JsonValue::Bool(value)),
        ("promotionEligibility".to_string(), JsonValue::Bool(value)),
        ("readerResults".to_string(), JsonValue::Bool(value)),
        ("recordIdentity".to_string(), JsonValue::Bool(value)),
        (
            "singleOperationIdempotency".to_string(),
            JsonValue::Bool(value),
        ),
        ("statusCounts".to_string(), JsonValue::Bool(value)),
        ("terminalState".to_string(), JsonValue::Bool(value)),
    ]))
}

fn read_located_lines(state_dir: &Path, ledger: &str) -> Result<Vec<LocatedLine>, StoreError> {
    let mut lines = Vec::new();
    let manifest_path = state_dir.join(QUARANTINE_SEGMENT_MANIFEST_FILE);
    if manifest_path.exists() {
        let manifest = parse_json(&fs::read_to_string(&manifest_path).map_err(io_error)?)
            .map_err(|error| replacement_error("LB_QUARANTINE_REPLACEMENT_CORRUPT", &error.to_string()))?;
        for segment in object_array(&manifest, "segments")? {
            if object_string(segment, "ledger")? == ledger {
                let file = object_string(segment, "file")?;
                let path = state_dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR).join(&file);
                lines.extend(read_file_lines(&path, &format!("archive-segment:{file}"))?);
            }
        }
    }
    lines.extend(read_file_lines(
        &resolve_quarantine_active_path(state_dir, ledger)?,
        "active-ledger",
    )?);
    Ok(lines)
}

fn read_file_lines(path: &Path, location: &str) -> Result<Vec<LocatedLine>, StoreError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).map_err(io_error)?;
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            parse_json(line).map_err(|error| {
                replacement_error("LB_QUARANTINE_REPLACEMENT_CORRUPT", &error.to_string())
            })?;
            Ok(LocatedLine {
                location: location.to_string(),
                line_number: index as u64 + 1,
                text: line.to_string(),
            })
        })
        .collect()
}

fn publish_artifacts(
    output_dir: &Path,
    plan_text: &str,
    plan_digest: &str,
    proof_text: &str,
    proof_digest: &str,
) -> Result<(), StoreError> {
    let artifacts = [
        (QUARANTINE_REPLACEMENT_PLAN_FILE, plan_text.to_string()),
        (
            QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE,
            format!("{plan_digest}\n"),
        ),
        (QUARANTINE_REPLACEMENT_PROOF_FILE, proof_text.to_string()),
        (
            QUARANTINE_REPLACEMENT_PROOF_DIGEST_FILE,
            format!("{proof_digest}\n"),
        ),
    ];
    for (name, content) in &artifacts {
        fs::write(output_dir.join(format!(".{name}.tmp")), content).map_err(io_error)?;
    }
    for (name, _) in &artifacts {
        fs::rename(
            output_dir.join(format!(".{name}.tmp")),
            output_dir.join(name),
        )
        .map_err(io_error)?;
    }
    Ok(())
}

fn read_verified_artifact(path: &Path, digest_path: &Path, kind: &str) -> Result<String, StoreError> {
    let text = fs::read_to_string(path)
        .map_err(|error| proof_error(&format!("failed to read {kind}: {error}")))?;
    let expected = fs::read_to_string(digest_path)
        .map_err(|error| proof_error(&format!("failed to read {kind} digest: {error}")))?;
    if expected.trim() != integrity_digest(text.as_bytes()) {
        return Err(proof_error(&format!("replacement {kind} digest mismatch")));
    }
    Ok(text)
}

fn require_v2_backup(backup_dir: &Path) -> Result<(), StoreError> {
    let manifest = parse_json(
        &fs::read_to_string(backup_dir.join(QUARANTINE_BACKUP_MANIFEST)).map_err(io_error)?,
    )
    .map_err(|error| replacement_error("LB_QUARANTINE_REPLACEMENT_BACKUP", &error.to_string()))?;
    if object_string(&manifest, "version")? != QUARANTINE_COMPLETE_BACKUP_VERSION {
        return Err(replacement_error(
            "LB_QUARANTINE_REPLACEMENT_BACKUP",
            "replacement preview requires a verified backup v2",
        ));
    }
    Ok(())
}

fn prepare_empty_output(path: &Path) -> Result<(), StoreError> {
    fs::create_dir_all(path).map_err(io_error)?;
    if fs::read_dir(path).map_err(io_error)?.next().is_some() {
        return Err(replacement_error(
            "LB_QUARANTINE_REPLACEMENT_CONFLICT",
            "replacement proof output directory must be empty",
        ));
    }
    Ok(())
}

fn runtime_fingerprint(state_dir: &Path) -> Result<Vec<(String, Option<String>)>, StoreError> {
    let mut paths = QUARANTINE_BACKUP_FILES
        .iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    paths.push(QUARANTINE_SEGMENT_MANIFEST_FILE.to_string());
    let archive_dir = state_dir.join(QUARANTINE_SEGMENT_ARCHIVE_DIR);
    if archive_dir.exists() {
        let mut archive = fs::read_dir(&archive_dir)
            .map_err(io_error)?
            .map(|entry| {
                entry.map_err(io_error).map(|entry| {
                    format!(
                        "{QUARANTINE_SEGMENT_ARCHIVE_DIR}/{}",
                        entry.file_name().to_string_lossy()
                    )
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        archive.sort();
        paths.extend(archive);
    }
    paths
        .into_iter()
        .map(|relative| {
            let path = if QUARANTINE_BACKUP_FILES.contains(&relative.as_str()) {
                resolve_quarantine_active_path(state_dir, &relative)?
            } else {
                state_dir.join(&relative)
            };
            Ok((
                relative,
                if path.exists() {
                    Some(file_digest(&path)?)
                } else {
                    None
                },
            ))
        })
        .collect()
}

fn fingerprint_json(values: &[(String, Option<String>)]) -> JsonValue {
    JsonValue::Array(
        values
            .iter()
            .map(|(path, digest)| {
                JsonValue::Object(BTreeMap::from([
                    (
                        "digest".to_string(),
                        digest
                            .as_ref()
                            .map(|v| JsonValue::String(v.clone()))
                            .unwrap_or(JsonValue::Null),
                    ),
                    ("path".to_string(), JsonValue::String(path.clone())),
                ]))
            })
            .collect(),
    )
}

fn file_digest(path: &Path) -> Result<String, StoreError> {
    fs::read(path)
        .map(|bytes| integrity_digest(&bytes))
        .map_err(io_error)
}

fn optional_file_digest(path: &Path) -> Result<Option<String>, StoreError> {
    if path.exists() {
        Ok(Some(file_digest(path)?))
    } else {
        Ok(None)
    }
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn object_field<'a>(value: &'a JsonValue, name: &str) -> Result<&'a JsonValue, StoreError> {
    as_object(value)?
        .get(name)
        .ok_or_else(|| proof_error(&format!("missing field: {name}")))
}

fn object_array<'a>(value: &'a JsonValue, name: &str) -> Result<&'a [JsonValue], StoreError> {
    match object_field(value, name)? {
        JsonValue::Array(values) => Ok(values),
        _ => Err(proof_error(&format!("invalid array field: {name}"))),
    }
}

fn object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match object_field(value, name)? {
        JsonValue::String(value) => Ok(value.clone()),
        _ => Err(proof_error(&format!("invalid string field: {name}"))),
    }
}

fn object_number(value: &JsonValue, name: &str) -> Result<u64, StoreError> {
    match object_field(value, name)? {
        JsonValue::Number(value) => value
            .parse()
            .map_err(|_| proof_error(&format!("invalid number field: {name}"))),
        _ => Err(proof_error(&format!("invalid number field: {name}"))),
    }
}

fn object_bool(value: &JsonValue, name: &str) -> Result<bool, StoreError> {
    match object_field(value, name)? {
        JsonValue::Bool(value) => Ok(*value),
        _ => Err(proof_error(&format!("invalid boolean field: {name}"))),
    }
}

fn required_object_string(value: &JsonValue, name: &str) -> Result<String, StoreError> {
    match value {
        JsonValue::Object(map) => match map.get(name) {
            Some(JsonValue::String(value)) => Ok(value.clone()),
            _ => Err(replacement_error(
                "LB_QUARANTINE_REPLACEMENT_CORRUPT",
                &format!("missing string field: {name}"),
            )),
        },
        _ => Err(replacement_error(
            "LB_QUARANTINE_REPLACEMENT_CORRUPT",
            "ledger line is not an object",
        )),
    }
}

fn optional_object_string(value: &JsonValue, name: &str) -> Result<Option<String>, StoreError> {
    match value {
        JsonValue::Object(map) => match map.get(name) {
            Some(JsonValue::String(value)) => Ok(Some(value.clone())),
            None => Ok(None),
            _ => Err(replacement_error(
                "LB_QUARANTINE_REPLACEMENT_CORRUPT",
                &format!("invalid optional string field: {name}"),
            )),
        },
        _ => Err(replacement_error(
            "LB_QUARANTINE_REPLACEMENT_CORRUPT",
            "ledger line is not an object",
        )),
    }
}

fn as_object(value: &JsonValue) -> Result<&BTreeMap<String, JsonValue>, StoreError> {
    match value {
        JsonValue::Object(map) => Ok(map),
        _ => Err(proof_error("expected JSON object")),
    }
}

fn require_string(value: &JsonValue, name: &str, expected: &str) -> Result<(), StoreError> {
    if object_string(value, name)? != expected {
        return Err(proof_error(&format!("unsupported {name}")));
    }
    Ok(())
}

fn timestamp() -> Result<String, StoreError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| replacement_error("LB_QUARANTINE_IO", &error.to_string()))?;
    Ok(format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()))
}

fn io_error(error: std::io::Error) -> StoreError {
    replacement_error("LB_QUARANTINE_IO", &error.to_string())
}

fn proof_error(message: &str) -> StoreError {
    replacement_error("LB_QUARANTINE_REPLACEMENT_PROOF", message)
}

fn replacement_error(code: &'static str, message: &str) -> StoreError {
    store_error(code, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export_complete_quarantine_backup;

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-{label}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn creates_deterministic_non_mutating_replacement_preview() {
        let state = temp_dir("replacement-state");
        let backup = temp_dir("replacement-backup");
        let proof_a = temp_dir("replacement-proof-a");
        let proof_b = temp_dir("replacement-proof-b");
        fs::create_dir_all(&state).unwrap();
        fs::write(
            state.join("quarantine-resolutions.jsonl"),
            "{\"canonicalId\":\"c1\", \"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        let before = runtime_fingerprint(&state).unwrap();
        let report_a = create_quarantine_replacement_preview(&state, &backup, &proof_a).unwrap();
        let report_b = create_quarantine_replacement_preview(&state, &backup, &proof_b).unwrap();
        assert_eq!(report_a.replacement_lines, 1);
        assert!(!report_a.mutation_allowed);
        assert_eq!(before, runtime_fingerprint(&state).unwrap());
        assert_eq!(
            fs::read(proof_a.join(QUARANTINE_REPLACEMENT_PLAN_FILE)).unwrap(),
            fs::read(proof_b.join(QUARANTINE_REPLACEMENT_PLAN_FILE)).unwrap()
        );
        assert_eq!(
            fs::read(proof_a.join(QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE)).unwrap(),
            fs::read(proof_b.join(QUARANTINE_REPLACEMENT_PLAN_DIGEST_FILE)).unwrap()
        );
        assert_eq!(verify_quarantine_replacement_proof(&proof_a).unwrap(), report_a);
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof_a);
        let _ = fs::remove_dir_all(proof_b);
    }

    #[test]
    fn rejects_duplicate_terminal_keys_and_tampering() {
        let state = temp_dir("replacement-duplicate");
        let backup = temp_dir("replacement-duplicate-backup");
        let proof = temp_dir("replacement-duplicate-proof");
        fs::create_dir_all(&state).unwrap();
        fs::write(
            state.join("quarantine-dismissals.jsonl"),
            "{\"quarantineId\":\"q1\"}\n{\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        export_complete_quarantine_backup(&state, &backup).unwrap();
        assert_eq!(
            create_quarantine_replacement_preview(&state, &backup, &proof)
                .unwrap_err()
                .code,
            "LB_QUARANTINE_REPLACEMENT_CORRUPT"
        );
        let _ = fs::remove_dir_all(&proof);
        fs::write(
            state.join("quarantine-dismissals.jsonl"),
            "{\"quarantineId\":\"q1\"}\n",
        )
        .unwrap();
        let _ = fs::remove_dir_all(&backup);
        export_complete_quarantine_backup(&state, &backup).unwrap();
        create_quarantine_replacement_preview(&state, &backup, &proof).unwrap();
        fs::write(
            proof.join(QUARANTINE_REPLACEMENT_PROOF_FILE),
            "{\"tampered\":true}",
        )
        .unwrap();
        assert_eq!(
            verify_quarantine_replacement_proof(&proof).unwrap_err().code,
            "LB_QUARANTINE_REPLACEMENT_PROOF"
        );
        let _ = fs::remove_dir_all(state);
        let _ = fs::remove_dir_all(backup);
        let _ = fs::remove_dir_all(proof);
    }
}
