use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

pub const IDENTITY_KEY_RULE_VERSION_V1: &str = "lb.identity.key.v1";
pub const PROTOCOL_VERSION: &str = "0.1.0";
pub const KNOWLEDGE_OBJECT_SCHEMA_VERSION: &str = "0.1.0";
pub const HTTP_PUBLISH_REQUEST_SCHEMA_VERSION: &str = "0.1.0";
pub const ARCHIVE_VERSION: &str = "1";
pub const CAPABILITY_VERSION: &str = "1";
pub const DEFAULT_ACCESS_SCOPE: &str = "public";
pub const DEFAULT_RETENTION_HINT: &str = "long-lived";
pub const CARRIER_KIND_HTTP: &str = "http";
pub const CARRIER_KIND_ARCHIVE: &str = "archive";
pub const CARRIER_KIND_RELAY: &str = "relay";
pub const MAX_JSON_INPUT_BYTES: usize = 1024 * 1024;
pub const MAX_JSON_NESTING_DEPTH: usize = 128;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

#[derive(Debug, Clone)]
pub struct JsonError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.position)
    }
}

impl std::error::Error for JsonError {}

#[derive(Debug, Clone)]
pub struct FinalizedKnowledgeObject {
    pub canonical_id: String,
    pub identity_key: String,
    pub object: JsonValue,
    pub canonical_json: String,
}

#[derive(Debug, Clone)]
pub struct ReadJsonFile {
    pub raw: String,
    pub value: JsonValue,
}

pub fn read_json_file(path: impl AsRef<Path>) -> Result<ReadJsonFile, String> {
    let raw = fs::read_to_string(path.as_ref()).map_err(|error| {
        format!(
            "failed to read JSON file {}: {}",
            path.as_ref().display(),
            error
        )
    })?;
    let value = parse_json(&raw).map_err(|error| error.to_string())?;
    Ok(ReadJsonFile { raw, value })
}

pub fn parse_json(input: &str) -> Result<JsonValue, JsonError> {
    if input.len() > MAX_JSON_INPUT_BYTES {
        return Err(JsonError {
            message: format!("JSON input exceeds {MAX_JSON_INPUT_BYTES} bytes"),
            position: MAX_JSON_INPUT_BYTES,
        });
    }
    let mut parser = Parser {
        input: input.as_bytes(),
        position: 0,
        depth: 0,
    };
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if !parser.is_eof() {
        return Err(parser.error("unexpected trailing content"));
    }
    Ok(value)
}

pub fn normalize_json(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::Array(items) => {
            JsonValue::Array(items.into_iter().map(normalize_json).collect())
        }
        JsonValue::Object(entries) => {
            let mut normalized = BTreeMap::new();
            for (key, value) in entries {
                normalized.insert(key, normalize_json(value));
            }
            JsonValue::Object(normalized)
        }
        other => other,
    }
}

pub fn to_canonical_json(value: &JsonValue) -> String {
    let mut out = String::new();
    write_json(value, &mut out);
    out
}

pub fn detect_shape(value: &JsonValue) -> &'static str {
    if let JsonValue::Object(map) = value {
        if map.contains_key("object") && map.contains_key("publisher") {
            return "publish-request";
        }
    }
    "knowledge-object"
}

pub fn validate_knowledge_object(value: &JsonValue) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(map) = as_object(value) else {
        return vec!["knowledge object must be an object".to_string()];
    };

    for key in [
        "id",
        "schemaVersion",
        "type",
        "createdAt",
        "body",
        "provenance",
        "rawRef",
    ] {
        if !map.contains_key(key) {
            errors.push(format!("missing required field: {}", key));
        }
    }

    if !matches!(map.get("id"), Some(JsonValue::String(value)) if is_lb_object_id(value)) {
        errors.push("id must match ^lb:obj:[^\\s]+$".to_string());
    }

    if !matches!(map.get("schemaVersion"), Some(JsonValue::String(value)) if value == KNOWLEDGE_OBJECT_SCHEMA_VERSION)
    {
        errors.push(format!(
            "schemaVersion must be {}",
            KNOWLEDGE_OBJECT_SCHEMA_VERSION
        ));
    }

    if !matches!(map.get("type"), Some(JsonValue::String(value)) if supported_knowledge_types().contains(&value.as_str()))
    {
        errors.push("type must be one of the supported knowledge object types".to_string());
    }

    if let Some(JsonValue::String(value)) = map.get("createdAt") {
        if !is_rfc3339_datetime(value) {
            errors.push("createdAt must be a valid date-time string".to_string());
        }
    }

    validate_body(map.get("body"), &mut errors);
    validate_contexts(map.get("contexts"), &mut errors);
    validate_relations(map.get("relations"), &mut errors);
    validate_status(map.get("status"), &mut errors);
    validate_lineage(map.get("lineage"), &mut errors);
    validate_provenance(map.get("provenance"), &mut errors);
    validate_raw_ref(map.get("rawRef"), &mut errors);
    validate_identity_claims(map.get("identityClaims"), map, &mut errors);
    validate_attachments(map.get("attachments"), &mut errors);
    validate_labels(map.get("labels"), &mut errors);
    validate_meta(map.get("meta"), &mut errors);

    let allowed_root = [
        "id",
        "schemaVersion",
        "type",
        "createdAt",
        "body",
        "contexts",
        "relations",
        "status",
        "lineage",
        "provenance",
        "rawRef",
        "identityClaims",
        "attachments",
        "labels",
        "meta",
    ];
    for key in map.keys() {
        if !allowed_root.contains(&key.as_str()) {
            errors.push(format!("unknown root field: {}", key));
        }
    }

    errors
}

pub fn validate_publish_request(value: &JsonValue) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(map) = as_object(value) else {
        return vec!["publish request must be an object".to_string()];
    };

    if let Some(object) = map.get("object") {
        errors.extend(
            validate_knowledge_object(object)
                .into_iter()
                .map(|error| format!("object.{}", error)),
        );
    } else {
        errors.push("missing required field: object".to_string());
    }

    match map.get("publisher") {
        Some(JsonValue::Object(publisher)) => {
            match publisher.get("publicKey") {
                Some(JsonValue::String(value)) if is_lower_hex(value) && value.len() == 64 => {}
                _ => errors.push(
                    "publisher.publicKey must be a 64-character lowercase hex string".to_string(),
                ),
            }
            match publisher.get("signature") {
                Some(JsonValue::String(value)) if is_lower_hex(value) && value.len() == 128 => {}
                _ => errors.push(
                    "publisher.signature must be a 128-character lowercase hex string".to_string(),
                ),
            }
            let allowed = ["publicKey", "signature"];
            for key in publisher.keys() {
                if !allowed.contains(&key.as_str()) {
                    errors.push("publisher must not contain additional properties".to_string());
                    break;
                }
            }
        }
        Some(_) => errors.push("publisher must be an object".to_string()),
        None => errors.push("missing required field: publisher".to_string()),
    }

    if let Err(error) = verify_publish_request_signature(value) {
        errors.push(error);
    }

    for key in map.keys() {
        if !["object", "publisher"].contains(&key.as_str()) {
            errors.push(format!("unknown root field: {}", key));
        }
    }

    errors
}

pub fn finalize_knowledge_object(
    value: &JsonValue,
) -> Result<FinalizedKnowledgeObject, Vec<String>> {
    let errors = validate_knowledge_object(value);
    if !errors.is_empty() {
        return Err(errors);
    }
    let normalized = normalize_json(value.clone());
    let canonical_json = to_canonical_json(&normalized);
    let canonical_id = as_object(&normalized)
        .and_then(|map| map.get("id"))
        .and_then(as_string)
        .map(ToString::to_string)
        .unwrap_or_default();
    let identity_key = derive_identity_key(&normalized);
    Ok(FinalizedKnowledgeObject {
        canonical_id,
        identity_key,
        object: normalized,
        canonical_json,
    })
}

pub fn derive_identity_key(value: &JsonValue) -> String {
    let basis = identity_key_basis(value);
    let canonical_json = to_canonical_json(&basis);
    let fingerprint = fnv1a64_hex(&canonical_json);
    format!(
        "lb:key:{}:fnv1a64:{}",
        IDENTITY_KEY_RULE_VERSION_V1, fingerprint
    )
}

pub fn is_lb_object_id(value: &str) -> bool {
    value.starts_with("lb:obj:") && !value.chars().any(char::is_whitespace)
}

pub fn supported_knowledge_types() -> &'static [&'static str] {
    &[
        "inquiry",
        "observation",
        "claim",
        "evidence",
        "annotation",
        "synthesis",
        "translation",
        "reference",
        "concept",
    ]
}

pub fn build_capability_manifest(
    carrier_kind: &str,
    default_access: &str,
    default_retention: &str,
) -> JsonValue {
    let supported_schema_versions = JsonValue::Array(vec![
        JsonValue::Object(BTreeMap::from([
            (
                "schema".to_string(),
                JsonValue::String("knowledge-object".to_string()),
            ),
            (
                "versions".to_string(),
                JsonValue::Array(vec![JsonValue::String(
                    KNOWLEDGE_OBJECT_SCHEMA_VERSION.to_string(),
                )]),
            ),
            (
                "preferred".to_string(),
                JsonValue::String(KNOWLEDGE_OBJECT_SCHEMA_VERSION.to_string()),
            ),
            ("breaking".to_string(), JsonValue::Bool(false)),
        ])),
        JsonValue::Object(BTreeMap::from([
            (
                "schema".to_string(),
                JsonValue::String("http-publish-request".to_string()),
            ),
            (
                "versions".to_string(),
                JsonValue::Array(vec![JsonValue::String(
                    HTTP_PUBLISH_REQUEST_SCHEMA_VERSION.to_string(),
                )]),
            ),
            (
                "preferred".to_string(),
                JsonValue::String(HTTP_PUBLISH_REQUEST_SCHEMA_VERSION.to_string()),
            ),
            ("breaking".to_string(), JsonValue::Bool(false)),
        ])),
    ]);

    let supported_object_types = JsonValue::Array(
        supported_knowledge_types()
            .iter()
            .map(|value| JsonValue::String((*value).to_string()))
            .collect(),
    );

    JsonValue::Object(BTreeMap::from([
        (
            "capabilityVersion".to_string(),
            JsonValue::String(CAPABILITY_VERSION.to_string()),
        ),
        (
            "protocolVersion".to_string(),
            JsonValue::String(PROTOCOL_VERSION.to_string()),
        ),
        (
            "carrierKind".to_string(),
            JsonValue::String(carrier_kind.to_string()),
        ),
        (
            "supportedCarrierKinds".to_string(),
            JsonValue::Array(vec![
                JsonValue::String(CARRIER_KIND_HTTP.to_string()),
                JsonValue::String(CARRIER_KIND_ARCHIVE.to_string()),
                JsonValue::String(CARRIER_KIND_RELAY.to_string()),
            ]),
        ),
        (
            "supportedSchemaVersions".to_string(),
            supported_schema_versions,
        ),
        ("supportedObjectTypes".to_string(), supported_object_types),
        (
            "supportedContentTypes".to_string(),
            JsonValue::Array(vec![JsonValue::String("application/json".to_string())]),
        ),
        (
            "supportedAuthModes".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("public-key-signature".to_string()),
                JsonValue::String("relay-trusted-signature".to_string()),
            ]),
        ),
        (
            "validationConstraints".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("required-fields".to_string()),
                JsonValue::String("schema-version-match".to_string()),
                JsonValue::String("identity-consistency".to_string()),
            ]),
        ),
        (
            "finalizeConstraints".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("canonical-id-resolution".to_string()),
                JsonValue::String("rawref-preservation".to_string()),
                JsonValue::String("provenance-preservation".to_string()),
            ]),
        ),
        (
            "supportedAccessScopes".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("public".to_string()),
                JsonValue::String("curated".to_string()),
                JsonValue::String("private".to_string()),
            ]),
        ),
        (
            "supportedRetentionHints".to_string(),
            JsonValue::Array(vec![
                JsonValue::String(default_retention.to_string()),
                JsonValue::String("long-term".to_string()),
                JsonValue::String("ephemeral".to_string()),
            ]),
        ),
        ("multiNode".to_string(), build_multi_node_policy_manifest()),
        (
            "defaults".to_string(),
            JsonValue::Object(BTreeMap::from([
                (
                    "accessScope".to_string(),
                    JsonValue::String(default_access.to_string()),
                ),
                (
                    "retentionHint".to_string(),
                    JsonValue::String(default_retention.to_string()),
                ),
            ])),
        ),
    ]))
}

pub fn build_multi_node_policy_manifest() -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "supportedNodeRoles".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("public relay".to_string()),
                JsonValue::String("curated relay".to_string()),
                JsonValue::String("archive relay".to_string()),
                JsonValue::String("gateway relay".to_string()),
                JsonValue::String("storage node".to_string()),
                JsonValue::String("archive node".to_string()),
            ]),
        ),
        (
            "discovery".to_string(),
            JsonValue::Object(BTreeMap::from([
                ("registryFree".to_string(), JsonValue::Bool(true)),
                (
                    "helperSurfaces".to_string(),
                    JsonValue::Array(vec![
                        JsonValue::String("signed-manifest".to_string()),
                        JsonValue::String("capability-endpoint".to_string()),
                        JsonValue::String("relay-discovery".to_string()),
                        JsonValue::String("indexer-cache".to_string()),
                    ]),
                ),
            ])),
        ),
        (
            "sync".to_string(),
            JsonValue::Object(BTreeMap::from([
                (
                    "relay".to_string(),
                    JsonValue::String("subscription".to_string()),
                ),
                (
                    "storageNode".to_string(),
                    JsonValue::Array(vec![
                        JsonValue::String("replay".to_string()),
                        JsonValue::String("export/import".to_string()),
                    ]),
                ),
                (
                    "archive".to_string(),
                    JsonValue::Array(vec![JsonValue::String("export/import".to_string())]),
                ),
                ("semanticTranslation".to_string(), JsonValue::Bool(false)),
            ])),
        ),
        (
            "conflict".to_string(),
            JsonValue::Object(BTreeMap::from([
                (
                    "exactDuplicate".to_string(),
                    JsonValue::String("idempotent".to_string()),
                ),
                (
                    "conflictingRePublish".to_string(),
                    JsonValue::String("reject-or-quarantine".to_string()),
                ),
                (
                    "identityCollision".to_string(),
                    JsonValue::String("keep-both".to_string()),
                ),
                (
                    "revision".to_string(),
                    JsonValue::String("lineage".to_string()),
                ),
            ])),
        ),
        (
            "capacity".to_string(),
            JsonValue::Object(BTreeMap::from([
                (
                    "placementOrder".to_string(),
                    JsonValue::Array(vec![
                        JsonValue::String("role-requirements".to_string()),
                        JsonValue::String("availability".to_string()),
                        JsonValue::String("storage".to_string()),
                        JsonValue::String("replay".to_string()),
                        JsonValue::String("connectivity".to_string()),
                    ]),
                ),
                (
                    "pressurePriority".to_string(),
                    JsonValue::Array(vec![
                        JsonValue::String("replay-possible".to_string()),
                        JsonValue::String("provenance-retained".to_string()),
                        JsonValue::String("public-entry-point".to_string()),
                        JsonValue::String("storage-reconstruction".to_string()),
                        JsonValue::String("archive-spool".to_string()),
                    ]),
                ),
            ])),
        ),
    ]))
}

fn validate_body(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    let Some(JsonValue::Object(map)) = value else {
        errors.push("body must be an object".to_string());
        return;
    };

    match map.get("text") {
        Some(JsonValue::String(text)) if !text.is_empty() => {}
        _ => errors.push("body.text must be a non-empty string".to_string()),
    }
    match map.get("language") {
        Some(JsonValue::String(language)) if is_bcp47_language_tag(language) => {}
        _ => errors.push("body.language must be a BCP47-style language tag".to_string()),
    }

    let allowed = ["text", "language"];
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            errors.push("body must not contain additional properties".to_string());
            break;
        }
    }
}

fn validate_contexts(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    if let Some(value) = value {
        if !matches!(value, JsonValue::Object(_)) {
            errors.push("contexts must be an object".to_string());
        }
    }
}

fn validate_relations(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    let Some(JsonValue::Array(items)) = value else {
        if value.is_some() {
            errors.push("relations must be an array".to_string());
        }
        return;
    };

    for (index, item) in items.iter().enumerate() {
        let Some(map) = as_object(item) else {
            errors.push(format!("relations[{}] must be an object", index));
            continue;
        };
        match map.get("source") {
            Some(JsonValue::String(value)) if !value.is_empty() => {}
            _ => errors.push(format!(
                "relations[{}].source must be a non-empty string",
                index
            )),
        }
        match map.get("target") {
            Some(JsonValue::String(value)) if !value.is_empty() => {}
            _ => errors.push(format!(
                "relations[{}].target must be a non-empty string",
                index
            )),
        }
        if let Some(JsonValue::String(value)) = map.get("kind") {
            if value.is_empty() {
                errors.push(format!(
                    "relations[{}].kind must be a non-empty string when present",
                    index
                ));
            }
        } else if map.contains_key("kind") {
            errors.push(format!(
                "relations[{}].kind must be a non-empty string when present",
                index
            ));
        }
        let allowed = ["source", "target", "kind"];
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                errors.push(format!(
                    "relations[{}] must not contain additional properties",
                    index
                ));
                break;
            }
        }
    }
}

fn validate_status(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    if let Some(JsonValue::String(status)) = value {
        if !["draft", "active", "superseded", "deprecated", "archived"].contains(&status.as_str()) {
            errors.push(
                "status must be one of draft, active, superseded, deprecated, archived".to_string(),
            );
        }
    } else if value.is_some() {
        errors.push("status must be a string".to_string());
    }
}

fn validate_lineage(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    let Some(JsonValue::Array(items)) = value else {
        if value.is_some() {
            errors.push("lineage must be an array".to_string());
        }
        return;
    };

    for (index, item) in items.iter().enumerate() {
        let Some(map) = as_object(item) else {
            errors.push(format!("lineage[{}] must be an object", index));
            continue;
        };
        match map.get("type") {
            Some(JsonValue::String(value))
                if [
                    "derived_from",
                    "revises",
                    "supersedes",
                    "translates",
                    "synthesizes",
                ]
                .contains(&value.as_str()) => {}
            _ => errors.push(format!(
                "lineage[{}].type must be one of the supported lineage types",
                index
            )),
        }
        match map.get("target") {
            Some(JsonValue::String(value)) if !value.is_empty() => {}
            _ => errors.push(format!(
                "lineage[{}].target must be a non-empty string",
                index
            )),
        }
        let allowed = ["type", "target"];
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                errors.push(format!(
                    "lineage[{}] must not contain additional properties",
                    index
                ));
                break;
            }
        }
    }
}

fn validate_provenance(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    let Some(JsonValue::Object(map)) = value else {
        errors.push("provenance must be an object".to_string());
        return;
    };

    match map.get("sources") {
        Some(JsonValue::Array(items)) if !items.is_empty() => {
            for (index, item) in items.iter().enumerate() {
                let Some(source) = as_object(item) else {
                    errors.push(format!("provenance.sources[{}] must be an object", index));
                    continue;
                };
                match source.get("protocol") {
                    Some(JsonValue::String(value)) if !value.is_empty() => {}
                    _ => errors.push(format!(
                        "provenance.sources[{}].protocol must be a non-empty string",
                        index
                    )),
                }
                match source.get("sourceId") {
                    Some(JsonValue::String(value)) if !value.is_empty() => {}
                    _ => errors.push(format!(
                        "provenance.sources[{}].sourceId must be a non-empty string",
                        index
                    )),
                }
                if let Some(JsonValue::String(value)) = source.get("authorId") {
                    if value.is_empty() {
                        errors.push(format!("provenance.sources[{}].authorId must be a non-empty string when present", index));
                    }
                } else if source.contains_key("authorId") {
                    errors.push(format!(
                        "provenance.sources[{}].authorId must be a non-empty string when present",
                        index
                    ));
                }
                if let Some(JsonValue::String(value)) = source.get("observedAt") {
                    if !is_rfc3339_datetime(value) {
                        errors.push(format!(
                            "provenance.sources[{}].observedAt must be a valid date-time string",
                            index
                        ));
                    }
                } else if source.contains_key("observedAt") {
                    errors.push(format!(
                        "provenance.sources[{}].observedAt must be a valid date-time string",
                        index
                    ));
                }
                let allowed = ["protocol", "sourceId", "authorId", "observedAt"];
                for key in source.keys() {
                    if !allowed.contains(&key.as_str()) {
                        errors.push(format!(
                            "provenance.sources[{}] must not contain additional properties",
                            index
                        ));
                        break;
                    }
                }
            }
        }
        Some(JsonValue::Array(_)) => {
            errors.push("provenance.sources must be a non-empty array".to_string())
        }
        _ => errors.push("provenance.sources must be a non-empty array".to_string()),
    }

    let allowed = ["sources"];
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            errors.push("provenance must not contain additional properties".to_string());
            break;
        }
    }
}

fn validate_raw_ref(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    let Some(JsonValue::Object(map)) = value else {
        errors.push("rawRef must be an object".to_string());
        return;
    };

    match map.get("protocol") {
        Some(JsonValue::String(value)) if !value.is_empty() => {}
        _ => errors.push("rawRef.protocol must be a non-empty string".to_string()),
    }
    match map.get("sourceId") {
        Some(JsonValue::String(value)) if !value.is_empty() => {}
        _ => errors.push("rawRef.sourceId must be a non-empty string".to_string()),
    }
    if let Some(JsonValue::String(value)) = map.get("locator") {
        if value.is_empty() {
            errors.push("rawRef.locator must be a non-empty string when present".to_string());
        }
    } else if map.contains_key("locator") {
        errors.push("rawRef.locator must be a non-empty string when present".to_string());
    }
    if let Some(JsonValue::String(value)) = map.get("payloadHash") {
        if value.is_empty() {
            errors.push("rawRef.payloadHash must be a non-empty string when present".to_string());
        }
    } else if map.contains_key("payloadHash") {
        errors.push("rawRef.payloadHash must be a non-empty string when present".to_string());
    }
    let allowed = ["protocol", "sourceId", "locator", "payloadHash"];
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            errors.push("rawRef must not contain additional properties".to_string());
            break;
        }
    }
}

fn validate_identity_claims(
    value: Option<&JsonValue>,
    object: &BTreeMap<String, JsonValue>,
    errors: &mut Vec<String>,
) {
    let Some(JsonValue::Array(items)) = value else {
        if value.is_some() {
            errors.push("identityClaims must be an array".to_string());
        }
        return;
    };

    let expected_identity_key = derive_identity_key(&JsonValue::Object(object.clone()));
    let expected_canonical_id = object
        .get("id")
        .and_then(as_string)
        .unwrap_or_default()
        .to_string();

    for (index, item) in items.iter().enumerate() {
        let Some(map) = as_object(item) else {
            errors.push(format!("identityClaims[{}] must be an object", index));
            continue;
        };

        match map.get("schemaVersion") {
            Some(JsonValue::String(value)) if value == "1" => {}
            _ => errors.push(format!("identityClaims[{}].schemaVersion must be 1", index)),
        }
        match map.get("claimType") {
            Some(JsonValue::String(value)) if value == "identity" => {}
            _ => errors.push(format!(
                "identityClaims[{}].claimType must be identity",
                index
            )),
        }
        match map.get("ruleVersion") {
            Some(JsonValue::String(value)) if value == IDENTITY_KEY_RULE_VERSION_V1 => {}
            Some(JsonValue::String(_)) => errors.push(format!(
                "identityClaims[{}].ruleVersion must be {}",
                index, IDENTITY_KEY_RULE_VERSION_V1
            )),
            _ => errors.push(format!(
                "identityClaims[{}].ruleVersion must be a non-empty string",
                index
            )),
        }
        match map.get("identityKey") {
            Some(JsonValue::String(value))
                if value.starts_with("lb:key:") && !value.chars().any(char::is_whitespace) => {}
            _ => errors.push(format!(
                "identityClaims[{}].identityKey must match ^lb:key:[^\\s]+$",
                index
            )),
        }
        match map.get("canonicalId") {
            Some(JsonValue::String(value)) if is_lb_object_id(value) => {}
            _ => errors.push(format!(
                "identityClaims[{}].canonicalId must match ^lb:obj:[^\\s]+$",
                index
            )),
        }
        match map.get("issuer") {
            Some(JsonValue::Object(issuer)) => {
                match issuer.get("protocol") {
                    Some(JsonValue::String(value)) if !value.is_empty() => {}
                    _ => errors.push(format!(
                        "identityClaims[{}].issuer.protocol must be a non-empty string",
                        index
                    )),
                }
                match issuer.get("sourceId") {
                    Some(JsonValue::String(value)) if !value.is_empty() => {}
                    _ => errors.push(format!(
                        "identityClaims[{}].issuer.sourceId must be a non-empty string",
                        index
                    )),
                }
                if let Some(JsonValue::String(value)) = issuer.get("signerId") {
                    if value.is_empty() {
                        errors.push(format!("identityClaims[{}].issuer.signerId must be a non-empty string when present", index));
                    }
                } else if issuer.contains_key("signerId") {
                    errors.push(format!("identityClaims[{}].issuer.signerId must be a non-empty string when present", index));
                }
                let allowed = ["protocol", "sourceId", "signerId"];
                for key in issuer.keys() {
                    if !allowed.contains(&key.as_str()) {
                        errors.push(format!(
                            "identityClaims[{}].issuer must not contain additional properties",
                            index
                        ));
                        break;
                    }
                }
            }
            _ => errors.push(format!(
                "identityClaims[{}].issuer must be an object",
                index
            )),
        }
        match map.get("issuedAt") {
            Some(JsonValue::String(value)) if is_rfc3339_datetime(value) => {}
            _ => errors.push(format!(
                "identityClaims[{}].issuedAt must be a valid date-time string",
                index
            )),
        }
        match map.get("verification") {
            Some(JsonValue::Object(verification)) => {
                match verification.get("method") {
                    Some(JsonValue::String(value)) if !value.is_empty() => {}
                    _ => errors.push(format!(
                        "identityClaims[{}].verification.method must be a non-empty string",
                        index
                    )),
                }
                match verification.get("payloadHash") {
                    Some(JsonValue::String(value)) if is_sha256_hash(value) => {}
                    _ => errors.push(format!("identityClaims[{}].verification.payloadHash must match ^sha256:[0-9a-f]{{64}}$", index)),
                }
                if let Some(JsonValue::String(value)) = verification.get("signature") {
                    if value.is_empty() {
                        errors.push(format!("identityClaims[{}].verification.signature must be a non-empty string when present", index));
                    }
                } else if verification.contains_key("signature") {
                    errors.push(format!("identityClaims[{}].verification.signature must be a non-empty string when present", index));
                }
                if let Some(JsonValue::String(value)) = verification.get("status") {
                    if !["pending", "verified", "rejected"].contains(&value.as_str()) {
                        errors.push(format!("identityClaims[{}].verification.status must be one of pending, verified, rejected", index));
                    }
                } else if verification.contains_key("status") {
                    errors.push(format!("identityClaims[{}].verification.status must be one of pending, verified, rejected", index));
                }
                let allowed = ["method", "payloadHash", "signature", "status"];
                for key in verification.keys() {
                    if !allowed.contains(&key.as_str()) {
                        errors.push(format!("identityClaims[{}].verification must not contain additional properties", index));
                        break;
                    }
                }
            }
            _ => errors.push(format!(
                "identityClaims[{}].verification must be an object",
                index
            )),
        }

        match map.get("canonicalId") {
            Some(JsonValue::String(value)) if value == &expected_canonical_id => {}
            Some(JsonValue::String(_)) => errors.push(format!(
                "identityClaims[{}].canonicalId must match the enclosing object id",
                index
            )),
            _ => {}
        }
        match map.get("identityKey") {
            Some(JsonValue::String(value)) if value == &expected_identity_key => {}
            Some(JsonValue::String(_)) => errors.push(format!(
                "identityClaims[{}].identityKey must match the derived identity key",
                index
            )),
            _ => {}
        }
        let allowed = [
            "schemaVersion",
            "claimType",
            "ruleVersion",
            "identityKey",
            "canonicalId",
            "issuer",
            "issuedAt",
            "verification",
        ];
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                errors.push(format!(
                    "identityClaims[{}] must not contain additional properties",
                    index
                ));
                break;
            }
        }
    }
}

fn validate_attachments(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    let Some(JsonValue::Array(items)) = value else {
        if value.is_some() {
            errors.push("attachments must be an array".to_string());
        }
        return;
    };

    for (index, item) in items.iter().enumerate() {
        let Some(map) = as_object(item) else {
            errors.push(format!("attachments[{}] must be an object", index));
            continue;
        };
        match map.get("type") {
            Some(JsonValue::String(value)) if !value.is_empty() => {}
            _ => errors.push(format!(
                "attachments[{}].type must be a non-empty string",
                index
            )),
        }
        match map.get("uri") {
            Some(JsonValue::String(value)) if !value.is_empty() => {}
            _ => errors.push(format!(
                "attachments[{}].uri must be a non-empty string",
                index
            )),
        }
        if let Some(JsonValue::String(value)) = map.get("title") {
            if value.is_empty() {
                errors.push(format!(
                    "attachments[{}].title must be a non-empty string when present",
                    index
                ));
            }
        } else if map.contains_key("title") {
            errors.push(format!(
                "attachments[{}].title must be a non-empty string when present",
                index
            ));
        }
        if let Some(JsonValue::String(value)) = map.get("mimeType") {
            if value.is_empty() {
                errors.push(format!(
                    "attachments[{}].mimeType must be a non-empty string when present",
                    index
                ));
            }
        } else if map.contains_key("mimeType") {
            errors.push(format!(
                "attachments[{}].mimeType must be a non-empty string when present",
                index
            ));
        }
        let allowed = ["type", "uri", "title", "mimeType"];
        for key in map.keys() {
            if !allowed.contains(&key.as_str()) {
                errors.push(format!(
                    "attachments[{}] must not contain additional properties",
                    index
                ));
                break;
            }
        }
    }
}

fn validate_labels(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    let Some(JsonValue::Array(items)) = value else {
        if value.is_some() {
            errors.push("labels must be an array".to_string());
        }
        return;
    };
    for (index, item) in items.iter().enumerate() {
        match item {
            JsonValue::String(value) if !value.is_empty() => {}
            _ => errors.push(format!("labels[{}] must be a non-empty string", index)),
        }
    }
}

fn validate_meta(value: Option<&JsonValue>, errors: &mut Vec<String>) {
    if let Some(value) = value {
        if !matches!(value, JsonValue::Object(_)) {
            errors.push("meta must be an object".to_string());
        }
    }
}

pub fn verify_publish_request_signature(value: &JsonValue) -> Result<(), String> {
    let Some(map) = as_object(value) else {
        return Err("publish request must be an object".to_string());
    };
    let Some(JsonValue::Object(publisher)) = map.get("publisher") else {
        return Ok(());
    };
    let (Some(JsonValue::String(public_key_hex)), Some(JsonValue::String(signature_hex))) =
        (publisher.get("publicKey"), publisher.get("signature"))
    else {
        return Ok(());
    };
    if public_key_hex.len() != 64 || !is_lower_hex(public_key_hex) {
        return Ok(());
    }
    if signature_hex.len() != 128 || !is_lower_hex(signature_hex) {
        return Ok(());
    }

    let payload = canonical_publish_request_payload(value)?;
    let public_key_bytes = decode_lower_hex(public_key_hex)
        .ok_or_else(|| "publisher.publicKey is not valid hex".to_string())?;
    if public_key_bytes.len() != 32 {
        return Err("publisher.publicKey must decode to 32 bytes".to_string());
    }
    let signature_bytes = decode_lower_hex(signature_hex)
        .ok_or_else(|| "publisher.signature is not valid hex".to_string())?;
    if signature_bytes.len() != 64 {
        return Err("publisher.signature must decode to 64 bytes".to_string());
    }
    verify_publish_request_signature_with_openssl(
        payload.as_bytes(),
        &public_key_bytes,
        &signature_bytes,
    )?;
    Ok(())
}

fn canonical_publish_request_payload(value: &JsonValue) -> Result<String, String> {
    let Some(map) = as_object(value) else {
        return Err("publish request must be an object".to_string());
    };
    let Some(object) = map.get("object") else {
        return Err("missing required field: object".to_string());
    };
    let Some(JsonValue::Object(publisher)) = map.get("publisher") else {
        return Err("missing required field: publisher".to_string());
    };
    let mut publisher = publisher.clone();
    publisher.remove("signature");
    let mut request = BTreeMap::new();
    request.insert("object".to_string(), object.clone());
    request.insert("publisher".to_string(), JsonValue::Object(publisher));
    Ok(to_canonical_json(&JsonValue::Object(request)))
}

static SIGNATURE_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct SignatureWorkspace {
    path: PathBuf,
}

impl SignatureWorkspace {
    fn create() -> Result<Self, String> {
        for _ in 0..32 {
            let counter = SIGNATURE_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "lingonberry-signature-{}-{timestamp}-{counter}",
                std::process::id()
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => {
                    return Err("failed to create signature verification workspace".to_string())
                }
            }
        }
        Err("failed to create unique signature verification workspace".to_string())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for SignatureWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "failed to create signature verification artifact".to_string())?;
    file.write_all(bytes)
        .map_err(|_| "failed to write signature verification artifact".to_string())
}

fn verify_publish_request_signature_with_openssl(
    message: &[u8],
    public_key: &[u8],
    signature: &[u8],
) -> Result<(), String> {
    let workspace = SignatureWorkspace::create()?;
    let key_path = workspace.path().join("public-key.der");
    let sig_path = workspace.path().join("signature.bin");
    let msg_path = workspace.path().join("message.bin");

    let mut der = vec![
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ];
    der.extend_from_slice(public_key);
    write_new_file(&key_path, &der)?;
    write_new_file(&sig_path, signature)?;
    write_new_file(&msg_path, message)?;

    let output = Command::new("openssl")
        .arg("pkeyutl")
        .arg("-verify")
        .arg("-pubin")
        .arg("-inkey")
        .arg(&key_path)
        .arg("-keyform")
        .arg("DER")
        .arg("-rawin")
        .arg("-in")
        .arg(&msg_path)
        .arg("-sigfile")
        .arg(&sig_path)
        .output()
        .map_err(|_| "failed to run signature verification command".to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err("publisher.signature does not verify the canonical request payload".to_string())
    }
}

fn identity_key_basis(value: &JsonValue) -> JsonValue {
    let Some(map) = as_object(value) else {
        return JsonValue::Object(BTreeMap::new());
    };
    let mut basis = BTreeMap::new();
    for key in [
        "type",
        "createdAt",
        "body",
        "contexts",
        "relations",
        "status",
        "lineage",
        "attachments",
        "labels",
    ] {
        if let Some(value) = map.get(key) {
            basis.insert(key.to_string(), value.clone());
        }
    }
    JsonValue::Object(basis)
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn as_string(value: &JsonValue) -> Option<&str> {
    match value {
        JsonValue::String(value) => Some(value.as_str()),
        _ => None,
    }
}

fn decode_lower_hex(value: &str) -> Option<Vec<u8>> {
    if !is_lower_hex(value) || !value.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    let chars: Vec<char> = value.chars().collect();
    for index in (0..chars.len()).step_by(2) {
        let hi = chars[index].to_digit(16)?;
        let lo = chars[index + 1].to_digit(16)?;
        bytes.push(((hi << 4) | lo) as u8);
    }
    Some(bytes)
}

fn fnv1a64_hex(input: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

fn is_lower_hex(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn is_sha256_hash(value: &str) -> bool {
    value.starts_with("sha256:")
        && value.len() == "sha256:".len() + 64
        && is_lower_hex(&value["sha256:".len()..])
}

fn is_bcp47_language_tag(value: &str) -> bool {
    let mut segments = value.split('-');
    let Some(first) = segments.next() else {
        return false;
    };
    if !(2..=8).contains(&first.len()) || !first.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return false;
    }
    for segment in segments {
        if !(1..=8).contains(&segment.len())
            || !segment.chars().all(|ch| ch.is_ascii_alphanumeric())
        {
            return false;
        }
    }
    true
}

fn is_rfc3339_datetime(value: &str) -> bool {
    let Some((date, time)) = value.split_once('T') else {
        return false;
    };
    if !is_date(date) {
        return false;
    }
    parse_time_with_zone(time)
}

fn is_date(value: &str) -> bool {
    let parts: Vec<&str> = value.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let year = parts[0];
    let month = parts[1];
    let day = parts[2];
    year.len() == 4
        && month.len() == 2
        && day.len() == 2
        && year.chars().all(|ch| ch.is_ascii_digit())
        && month.chars().all(|ch| ch.is_ascii_digit())
        && day.chars().all(|ch| ch.is_ascii_digit())
        && month
            .parse::<u32>()
            .is_ok_and(|value| (1..=12).contains(&value))
        && day
            .parse::<u32>()
            .is_ok_and(|value| (1..=31).contains(&value))
}

fn parse_time_with_zone(value: &str) -> bool {
    if let Some(time) = value.strip_suffix('Z') {
        return is_time(time);
    }
    if let Some((time, offset)) = value.rsplit_once('+') {
        return is_time(time) && is_offset(offset);
    }
    if let Some((time, offset)) = value.rsplit_once('-') {
        return is_time(time) && is_offset(offset);
    }
    false
}

fn is_time(value: &str) -> bool {
    let (hour, rest) = match value.split_once(':') {
        Some(parts) => parts,
        None => return false,
    };
    let (minute, second_part) = match rest.split_once(':') {
        Some(parts) => parts,
        None => return false,
    };
    let (second, fraction_ok) = match second_part.split_once('.') {
        Some((second, fraction)) => (
            second,
            !fraction.is_empty() && fraction.chars().all(|ch| ch.is_ascii_digit()),
        ),
        None => (second_part, true),
    };
    fraction_ok
        && hour.len() == 2
        && minute.len() == 2
        && second.len() == 2
        && hour.chars().all(|ch| ch.is_ascii_digit())
        && minute.chars().all(|ch| ch.is_ascii_digit())
        && second.chars().all(|ch| ch.is_ascii_digit())
        && hour.parse::<u32>().is_ok_and(|value| value <= 23)
        && minute.parse::<u32>().is_ok_and(|value| value <= 59)
        && second.parse::<u32>().is_ok_and(|value| value <= 60)
}

fn is_offset(value: &str) -> bool {
    let (hour, minute) = match value.split_once(':') {
        Some(parts) => parts,
        None => return false,
    };
    hour.len() == 2
        && minute.len() == 2
        && hour.chars().all(|ch| ch.is_ascii_digit())
        && minute.chars().all(|ch| ch.is_ascii_digit())
        && hour.parse::<u32>().is_ok_and(|value| value <= 23)
        && minute.parse::<u32>().is_ok_and(|value| value <= 59)
}

fn write_json(value: &JsonValue, out: &mut String) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Number(number) => out.push_str(number),
        JsonValue::String(value) => write_string(value, out),
        JsonValue::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_json(item, out);
            }
            out.push(']');
        }
        JsonValue::Object(map) => {
            out.push('{');
            for (index, (key, value)) in map.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                write_json(value, out);
            }
            out.push('}');
        }
    }
}

fn write_string(value: &str, out: &mut String) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch < '\u{20}' => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

struct Parser<'a> {
    input: &'a [u8],
    position: usize,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn parse_value(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        let Some(byte) = self.peek() else {
            return Err(self.error("unexpected end of input"));
        };
        match byte {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => self.parse_string().map(JsonValue::String),
            b't' => {
                self.expect_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            b'f' => {
                self.expect_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            b'n' => {
                self.expect_literal("null")?;
                Ok(JsonValue::Null)
            }
            b'-' | b'0'..=b'9' => self.parse_number().map(JsonValue::Number),
            _ => Err(self.error("unexpected character")),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = self.parse_object_inner();
        self.depth -= 1;
        result
    }

    fn parse_object_inner(&mut self) -> Result<JsonValue, JsonError> {
        self.consume(b'{')?;
        self.skip_whitespace();
        let mut map = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.consume(b':')?;
            let value = self.parse_value()?;
            map.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                }
                Some(b'}') => {
                    self.position += 1;
                    break;
                }
                _ => return Err(self.error("expected , or } in object")),
            }
        }
        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        self.enter_nesting()?;
        let result = self.parse_array_inner();
        self.depth -= 1;
        result
    }

    fn parse_array_inner(&mut self) -> Result<JsonValue, JsonError> {
        self.consume(b'[')?;
        self.skip_whitespace();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                }
                Some(b']') => {
                    self.position += 1;
                    break;
                }
                _ => return Err(self.error("expected , or ] in array")),
            }
        }
        Ok(JsonValue::Array(items))
    }

    fn parse_string(&mut self) -> Result<String, JsonError> {
        self.consume(b'"')?;
        let mut out = String::new();
        while let Some(byte) = self.peek() {
            self.position += 1;
            match byte {
                b'"' => return Ok(out),
                b'\\' => {
                    let Some(escape) = self.peek() else {
                        return Err(self.error("incomplete escape sequence"));
                    };
                    self.position += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{08}'),
                        b'f' => out.push('\u{0c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let code = self.parse_hex_quad()?;
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                            } else {
                                return Err(self.error("invalid unicode escape"));
                            }
                        }
                        _ => return Err(self.error("unknown escape sequence")),
                    }
                }
                byte if byte < 0x20 => {
                    return Err(self.error("unescaped control character in string"))
                }
                _ => {
                    self.position -= 1;
                    let remaining = &self.input[self.position..];
                    let ch = std::str::from_utf8(remaining)
                        .ok()
                        .and_then(|value| value.chars().next())
                        .ok_or_else(|| self.error("invalid utf-8 sequence"))?;
                    self.position += ch.len_utf8();
                    out.push(ch);
                }
            }
        }
        Err(self.error("unterminated string"))
    }

    fn parse_hex_quad(&mut self) -> Result<u32, JsonError> {
        let start = self.position;
        if self.position + 4 > self.input.len() {
            return Err(self.error("incomplete unicode escape"));
        }
        let slice = &self.input[self.position..self.position + 4];
        self.position += 4;
        let text = std::str::from_utf8(slice).map_err(|_| self.error("invalid unicode escape"))?;
        u32::from_str_radix(text, 16).map_err(|_| JsonError {
            message: "invalid unicode escape".to_string(),
            position: start,
        })
    }

    fn parse_number(&mut self) -> Result<String, JsonError> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.position += 1;
            }
            Some(b'1'..=b'9') => {
                self.position += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(self.error("invalid number")),
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("invalid number"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("invalid number"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
        }
        let slice = &self.input[start..self.position];
        let text = std::str::from_utf8(slice).map_err(|_| self.error("invalid number"))?;
        Ok(text.to_string())
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), JsonError> {
        for expected in literal.bytes() {
            match self.peek() {
                Some(byte) if byte == expected => self.position += 1,
                _ => return Err(self.error("unexpected literal")),
            }
        }
        Ok(())
    }

    fn enter_nesting(&mut self) -> Result<(), JsonError> {
        if self.depth >= MAX_JSON_NESTING_DEPTH {
            return Err(self.error(format!("JSON nesting exceeds {MAX_JSON_NESTING_DEPTH}")));
        }
        self.depth += 1;
        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    fn consume(&mut self, expected: u8) -> Result<(), JsonError> {
        match self.peek() {
            Some(byte) if byte == expected => {
                self.position += 1;
                Ok(())
            }
            _ => Err(self.error("unexpected character")),
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn error(&self, message: impl Into<String>) -> JsonError {
        JsonError {
            message: message.into(),
            position: self.position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_minimal_knowledge_object_fixture() {
        let raw = include_str!("../../../fixtures/knowledge-object/minimal-wire-object.json");
        let value = parse_json(raw).expect("fixture must parse");
        assert!(validate_knowledge_object(&value).is_empty());
        let finalized = finalize_knowledge_object(&value).expect("fixture must finalize");
        assert_eq!(finalized.canonical_id, "lb:obj:example-0001");
        assert_eq!(finalized.identity_key, derive_identity_key(&value));
    }

    #[test]
    fn rejects_missing_raw_ref_fixture() {
        let raw = include_str!("../../../fixtures/knowledge-object/invalid-missing-rawref.json");
        let value = parse_json(raw).expect("fixture must parse");
        let errors = validate_knowledge_object(&value);
        assert!(errors
            .iter()
            .any(|error| error.contains("missing required field: rawRef")));
    }

    #[test]
    fn rejects_schema_version_mismatch_fixture() {
        let raw = include_str!("../../../fixtures/knowledge-object/invalid-schema-version.json");
        let value = parse_json(raw).expect("fixture must parse");
        let errors = validate_knowledge_object(&value);
        assert!(errors
            .iter()
            .any(|error| error.contains("schemaVersion must be 0.1.0")));
    }

    #[test]
    fn validates_minimal_publish_request_fixture() {
        let raw = include_str!("../../../fixtures/http-publish-request/minimal-request.json");
        let value = parse_json(raw).expect("fixture must parse");
        assert_eq!(detect_shape(&value), "publish-request");
        assert!(validate_publish_request(&value).is_empty());
    }

    #[test]
    fn rejects_publish_request_schema_version_mismatch_fixture() {
        let raw =
            include_str!("../../../fixtures/http-publish-request/invalid-schema-version.json");
        let value = parse_json(raw).expect("fixture must parse");
        let errors = validate_publish_request(&value);
        assert!(errors
            .iter()
            .any(|error| error.contains("object.schemaVersion must be 0.1.0")));
    }

    #[test]
    fn validates_identity_claim_fixture() {
        let raw = include_str!("../../../fixtures/knowledge-object/with-identity-claim.json");
        let value = parse_json(raw).expect("fixture must parse");
        let errors = validate_knowledge_object(&value);
        assert!(errors.is_empty(), "{:?}", errors);
    }

    #[test]
    fn rejects_mismatched_identity_claim_fixture() {
        let raw =
            include_str!("../../../fixtures/knowledge-object/invalid-identity-claim-mismatch.json");
        let value = parse_json(raw).expect("fixture must parse");
        let errors = validate_knowledge_object(&value);
        assert!(errors
            .iter()
            .any(|error| error.contains("enclosing object id")));
    }

    #[test]
    fn capability_manifest_exposes_schema_and_validation_constraints() {
        let manifest = build_capability_manifest("http", "public", "long-term");
        let map = as_object(&manifest).expect("manifest must be an object");
        assert_eq!(map.get("carrierKind").and_then(as_string), Some("http"));
        assert_eq!(
            map.get("protocolVersion").and_then(as_string),
            Some(PROTOCOL_VERSION)
        );

        let schema_versions = match map.get("supportedSchemaVersions") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected supportedSchemaVersions: {:?}", other),
        };
        assert_eq!(schema_versions.len(), 2);

        let first = as_object(&schema_versions[0]).expect("first schema version entry");
        assert_eq!(
            first.get("schema").and_then(as_string),
            Some("knowledge-object")
        );
        assert_eq!(
            first.get("preferred").and_then(as_string),
            Some(KNOWLEDGE_OBJECT_SCHEMA_VERSION)
        );
        assert_eq!(first.get("breaking"), Some(&JsonValue::Bool(false)));

        let auth_modes = match map.get("supportedAuthModes") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected supportedAuthModes: {:?}", other),
        };
        assert!(auth_modes
            .iter()
            .any(|value| as_string(value) == Some("public-key-signature")));

        let validation_constraints = match map.get("validationConstraints") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected validationConstraints: {:?}", other),
        };
        assert!(validation_constraints
            .iter()
            .any(|value| as_string(value) == Some("schema-version-match")));

        let finalize_constraints = match map.get("finalizeConstraints") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected finalizeConstraints: {:?}", other),
        };
        assert!(finalize_constraints
            .iter()
            .any(|value| as_string(value) == Some("rawref-preservation")));

        let multi_node = as_object(map.get("multiNode").expect("multiNode manifest"))
            .expect("multiNode must be an object");
        let discovery = as_object(multi_node.get("discovery").expect("discovery"))
            .expect("discovery must be an object");
        assert_eq!(discovery.get("registryFree"), Some(&JsonValue::Bool(true)));
        let helper_surfaces = match discovery.get("helperSurfaces") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected helperSurfaces: {:?}", other),
        };
        assert!(helper_surfaces
            .iter()
            .any(|value| as_string(value) == Some("capability-endpoint")));

        let sync =
            as_object(multi_node.get("sync").expect("sync")).expect("sync must be an object");
        assert_eq!(sync.get("relay").and_then(as_string), Some("subscription"));
        let storage_node = match sync.get("storageNode") {
            Some(JsonValue::Array(values)) => values,
            other => panic!("unexpected storageNode: {:?}", other),
        };
        assert!(storage_node
            .iter()
            .any(|value| as_string(value) == Some("replay")));

        let conflict = as_object(multi_node.get("conflict").expect("conflict"))
            .expect("conflict must be an object");
        assert_eq!(
            conflict.get("exactDuplicate").and_then(as_string),
            Some("idempotent")
        );
        assert_eq!(
            conflict.get("revision").and_then(as_string),
            Some("lineage")
        );
    }
}
