use lingonberry_identity::validate_identity_claim_versions;
use lingonberry_protocol::{validate_knowledge_object, JsonValue};

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
}

pub fn validate_knowledge_object_full(value: &JsonValue) -> ValidationReport {
    let schema_errors = validate_knowledge_object(value);
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

    let identity_status = if !unsupported_identity_rules.is_empty() {
        IdentityValidationStatus::Unsupported
    } else if !identity_errors.is_empty() {
        IdentityValidationStatus::Invalid
    } else if has_identity_claims {
        IdentityValidationStatus::Valid
    } else {
        IdentityValidationStatus::NotPresent
    };

    ValidationReport {
        schema_errors,
        identity_errors,
        unsupported_identity_rules,
        identity_status,
    }
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
    fn validates_v2_claim_through_the_facade() {
        let value = parse(include_str!(
            "../../../conformance/identity-claims/valid-v2.json"
        ));
        let report = validate_knowledge_object_full(&value);

        assert!(report.is_valid(), "{report:?}");
        assert_eq!(report.identity_status, IdentityValidationStatus::Valid);
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
