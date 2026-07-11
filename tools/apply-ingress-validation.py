from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text()
    if old not in content:
        raise SystemExit(f"expected text not found in {path}: {old[:80]!r}")
    target.write_text(content.replace(old, new, 1))


Path("packages/validation/src/lib.rs").write_text(r'''use lingonberry_identity::validate_identity_claim_versions;
use lingonberry_protocol::{
    derive_identity_key, normalize_json, to_canonical_json, validate_knowledge_object,
    validate_publish_request, FinalizedKnowledgeObject, JsonValue,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityValidationStatus {
    Valid,
    Invalid,
    Unsupported,
    NotPresent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub schema_errors: Vec<String>,
    pub identity_errors: Vec<String>,
    pub unsupported_identity_rules: Vec<String>,
    pub identity_status: IdentityValidationStatus,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.schema_errors.is_empty()
            && self.identity_errors.is_empty()
            && self.unsupported_identity_rules.is_empty()
    }

    pub fn combined_errors(&self) -> Vec<String> {
        self.schema_errors
            .iter()
            .chain(self.identity_errors.iter())
            .chain(self.unsupported_identity_rules.iter())
            .cloned()
            .collect()
    }
}

fn is_legacy_identity_rule_error(error: &str) -> bool {
    let error = error.strip_prefix("object.").unwrap_or(error);
    error.starts_with("identityClaims[")
        && (error.contains(".ruleVersion must be lb.identity.key.v1")
            || error.contains(".identityKey must match the derived identity key"))
}

fn identity_report(value: &JsonValue) -> (Vec<String>, Vec<String>, IdentityValidationStatus) {
    let identity_results = validate_identity_claim_versions(value);
    let mut identity_errors = Vec::new();
    let mut unsupported_identity_rules = Vec::new();

    for error in identity_results {
        if error.contains(".ruleVersion is unsupported:") {
            unsupported_identity_rules.push(error);
        } else {
            identity_errors.push(error);
        }
    }

    let has_identity_claims = matches!(
        value,
        JsonValue::Object(object)
            if matches!(object.get("identityClaims"), Some(JsonValue::Array(items)) if !items.is_empty())
    );

    let status = if !unsupported_identity_rules.is_empty() {
        IdentityValidationStatus::Unsupported
    } else if !identity_errors.is_empty() {
        IdentityValidationStatus::Invalid
    } else if has_identity_claims {
        IdentityValidationStatus::Valid
    } else {
        IdentityValidationStatus::NotPresent
    };

    (identity_errors, unsupported_identity_rules, status)
}

pub fn validate_knowledge_object_full(value: &JsonValue) -> ValidationReport {
    let schema_errors = validate_knowledge_object(value)
        .into_iter()
        .filter(|error| !is_legacy_identity_rule_error(error))
        .collect();
    let (identity_errors, unsupported_identity_rules, identity_status) = identity_report(value);

    ValidationReport {
        schema_errors,
        identity_errors,
        unsupported_identity_rules,
        identity_status,
    }
}

pub fn validate_publish_request_full(value: &JsonValue) -> ValidationReport {
    let schema_errors = validate_publish_request(value)
        .into_iter()
        .filter(|error| !is_legacy_identity_rule_error(error))
        .collect::<Vec<_>>();

    let object = match value {
        JsonValue::Object(request) => request.get("object"),
        _ => None,
    };
    let (identity_errors, unsupported_identity_rules, identity_status) = match object {
        Some(object) => identity_report(object),
        None => (Vec::new(), Vec::new(), IdentityValidationStatus::NotPresent),
    };

    ValidationReport {
        schema_errors,
        identity_errors,
        unsupported_identity_rules,
        identity_status,
    }
}

pub fn finalize_knowledge_object_full(
    value: &JsonValue,
) -> Result<FinalizedKnowledgeObject, ValidationReport> {
    let report = validate_knowledge_object_full(value);
    if !report.is_valid() {
        return Err(report);
    }

    let normalized = normalize_json(value.clone());
    let canonical_json = to_canonical_json(&normalized);
    let canonical_id = match &normalized {
        JsonValue::Object(object) => match object.get("id") {
            Some(JsonValue::String(value)) => value.clone(),
            _ => String::new(),
        },
        _ => String::new(),
    };
    let identity_key = derive_identity_key(&normalized);

    Ok(FinalizedKnowledgeObject {
        canonical_id,
        identity_key,
        object: normalized,
        canonical_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::parse_json;

    fn parse(raw: &str) -> JsonValue {
        parse_json(raw).expect("fixture must parse")
    }

    #[test]
    fn validates_v1_claim_through_the_facade() {
        let value = parse(include_str!(
            "../../../fixtures/knowledge-object/with-identity-claim.json"
        ));
        let report = validate_knowledge_object_full(&value);

        assert!(report.is_valid(), "{report:?}");
        assert_eq!(report.identity_status, IdentityValidationStatus::Valid);
    }

    #[test]
    fn validates_and_finalizes_v2_claim_through_the_facade() {
        let value = parse(include_str!(
            "../../../conformance/identity-claims/valid-v2.json"
        ));
        let report = validate_knowledge_object_full(&value);

        assert!(report.is_valid(), "{report:?}");
        assert_eq!(report.identity_status, IdentityValidationStatus::Valid);
        let finalized = finalize_knowledge_object_full(&value).expect("v2 object must finalize");
        assert_eq!(finalized.canonical_id, "lb:obj:identity-v2-claim");
    }

    #[test]
    fn separates_unsupported_rules_from_mismatches() {
        let value = parse(include_str!(
            "../../../conformance/identity-claims/unsupported-rule.json"
        ));
        let report = validate_knowledge_object_full(&value);

        assert_eq!(report.identity_status, IdentityValidationStatus::Unsupported);
        assert!(report.identity_errors.is_empty());
        assert_eq!(report.unsupported_identity_rules.len(), 1);
    }

    #[test]
    fn reports_schema_and_identity_errors_together() {
        let value = parse(include_str!(
            "../../../fixtures/knowledge-object/invalid-identity-claim-mismatch.json"
        ));
        let report = validate_knowledge_object_full(&value);

        assert!(!report.is_valid());
        assert_eq!(report.identity_status, IdentityValidationStatus::Invalid);
        assert!(!report.identity_errors.is_empty());
    }
}
''')

replace_once(
    "packages/relay/Cargo.toml",
    'lingonberry-protocol = { path = "../protocol" }\n',
    'lingonberry-protocol = { path = "../protocol" }\nlingonberry-validation = { path = "../validation" }\n',
)
replace_once(
    "packages/core/Cargo.toml",
    'lingonberry-protocol = { path = "../protocol" }\n',
    'lingonberry-protocol = { path = "../protocol" }\nlingonberry-validation = { path = "../validation" }\n',
)

replace_once(
    "packages/relay/src/main.rs",
    '''use lingonberry_protocol::{
    build_capability_manifest, derive_identity_key, detect_shape, finalize_knowledge_object,
    read_json_file, to_canonical_json, validate_knowledge_object, validate_publish_request,
    JsonValue, CARRIER_KIND_HTTP, DEFAULT_ACCESS_SCOPE, DEFAULT_RETENTION_HINT,
};
''',
    '''use lingonberry_protocol::{
    build_capability_manifest, derive_identity_key, detect_shape, read_json_file,
    to_canonical_json, JsonValue, CARRIER_KIND_HTTP, DEFAULT_ACCESS_SCOPE,
    DEFAULT_RETENTION_HINT,
};
use lingonberry_validation::{
    finalize_knowledge_object_full, validate_knowledge_object_full,
    validate_publish_request_full, IdentityValidationStatus, ValidationReport,
};
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''    let errors = match detect_shape(&loaded.value) {
        "publish-request" => validate_publish_request(&loaded.value),
        _ => validate_knowledge_object(&loaded.value),
    };
    if !errors.is_empty() {
        return Err(format_validation_error("validation failed", &errors));
    }
''',
    '''    let report = match detect_shape(&loaded.value) {
        "publish-request" => validate_publish_request_full(&loaded.value),
        _ => validate_knowledge_object_full(&loaded.value),
    };
    if !report.is_valid() {
        return Err(format_validation_error(
            "validation failed",
            &report.combined_errors(),
        ));
    }
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''    let errors = validate_publish_request(&loaded.value);
    if !errors.is_empty() {
        return Err(format_validation_error("validation failed", &errors));
    }
''',
    '''    let report = validate_publish_request_full(&loaded.value);
    if !report.is_valid() {
        return Err(format_validation_error(
            "validation failed",
            &report.combined_errors(),
        ));
    }
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''    let finalized = finalize_knowledge_object(object)
        .map_err(|errors| format_validation_error("validation failed", &errors))?;
''',
    '''    let finalized = finalize_knowledge_object_full(object).map_err(|report| {
        format_validation_error("validation failed", &report.combined_errors())
    })?;
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''            let errors = validate_publish_request(&loaded.value);
            if !errors.is_empty() {
                return Err(format_validation_error("validation failed", &errors));
            }
''',
    '''            let report = validate_publish_request_full(&loaded.value);
            if !report.is_valid() {
                return Err(format_validation_error(
                    "validation failed",
                    &report.combined_errors(),
                ));
            }
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''            let errors = validate_knowledge_object(&loaded.value);
            if !errors.is_empty() {
                return Err(format_validation_error("validation failed", &errors));
            }
''',
    '''            let report = validate_knowledge_object_full(&loaded.value);
            if !report.is_valid() {
                return Err(format_validation_error(
                    "validation failed",
                    &report.combined_errors(),
                ));
            }
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''    let errors = validate_publish_request(&value);
    if !errors.is_empty() {
        return Ok((
            400,
            "Bad Request",
            http_error("validation_error", &errors.join("; ")),
        ));
    }
''',
    '''    let report = validate_publish_request_full(&value);
    if !report.is_valid() {
        let (status, text, kind) = validation_http_error(&report);
        return Ok((
            status,
            text,
            http_error(kind, &report.combined_errors().join("; ")),
        ));
    }
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''    let finalized = finalize_knowledge_object(object)
        .map_err(|errors| format_validation_error("validation failed", &errors))?;
''',
    '''    let finalized = finalize_knowledge_object_full(object).map_err(|report| {
        format_validation_error("validation failed", &report.combined_errors())
    })?;
''',
)
replace_once(
    "packages/relay/src/main.rs",
    '''fn http_error(kind: &str, message: &str) -> JsonValue {
''',
    '''fn validation_http_error(
    report: &ValidationReport,
) -> (u16, &'static str, &'static str) {
    if report.identity_status == IdentityValidationStatus::Unsupported {
        (422, "Unprocessable Entity", "unsupported_identity_rule")
    } else {
        (400, "Bad Request", "validation_error")
    }
}

fn http_error(kind: &str, message: &str) -> JsonValue {
''',
)

replace_once(
    "packages/core/src/lib.rs",
    '''use lingonberry_protocol::{
''',
    '''use lingonberry_validation::{
    finalize_knowledge_object_full, IdentityValidationStatus,
};
use lingonberry_protocol::{
''',
)
replace_once(
    "packages/core/src/lib.rs",
    '''        let finalized = finalize_knowledge_object(object_value)
            .map_err(|errors| store_error("LB_ARCHIVE_IMPORT", errors.join("; ")))?;
''',
    '''        let finalized = finalize_knowledge_object_full(object_value).map_err(|report| {
            let code = if report.identity_status == IdentityValidationStatus::Unsupported {
                "LB_UNSUPPORTED_IDENTITY_RULE"
            } else {
                "LB_ARCHIVE_IMPORT"
            };
            store_error(code, report.combined_errors().join("; "))
        })?;
''',
)
