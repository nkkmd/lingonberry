use std::collections::BTreeMap;

use lingonberry_protocol::{
    derive_identity_key, to_canonical_json, JsonValue, IDENTITY_KEY_RULE_VERSION_V1,
};
use sha2::{Digest, Sha256};

pub const IDENTITY_KEY_RULE_VERSION_V2: &str = "lb.identity.key.v2";
pub const CANONICALIZATION_RULE_VERSION_V1: &str = "lb.canonical.json.v1";

const SEMANTIC_FIELDS: [&str; 9] = [
    "type",
    "createdAt",
    "body",
    "contexts",
    "relations",
    "status",
    "lineage",
    "attachments",
    "labels",
];

pub fn identity_key_basis(value: &JsonValue) -> JsonValue {
    let JsonValue::Object(object) = value else {
        return JsonValue::Object(BTreeMap::new());
    };

    let mut basis = BTreeMap::new();
    for field in SEMANTIC_FIELDS {
        if let Some(value) = object.get(field) {
            basis.insert(field.to_string(), value.clone());
        }
    }

    JsonValue::Object(basis)
}

pub fn derive_identity_key_v2(value: &JsonValue) -> String {
    let basis = identity_key_basis(value);
    let canonical_json = to_canonical_json(&basis);
    let digest = Sha256::digest(canonical_json.as_bytes());
    format!(
        "lb:key:{}:sha256:{:x}",
        IDENTITY_KEY_RULE_VERSION_V2, digest
    )
}

pub fn derive_identity_key_for_rule(
    value: &JsonValue,
    rule_version: &str,
) -> Result<String, String> {
    match rule_version {
        IDENTITY_KEY_RULE_VERSION_V1 => Ok(derive_identity_key(value)),
        IDENTITY_KEY_RULE_VERSION_V2 => Ok(derive_identity_key_v2(value)),
        other => Err(format!("unsupported identity rule version: {}", other)),
    }
}

pub fn validate_identity_claim_versions(value: &JsonValue) -> Vec<String> {
    let JsonValue::Object(object) = value else {
        return vec!["knowledge object must be an object".to_string()];
    };
    let Some(JsonValue::Array(claims)) = object.get("identityClaims") else {
        return if object.contains_key("identityClaims") {
            vec!["identityClaims must be an array".to_string()]
        } else {
            Vec::new()
        };
    };

    let canonical_id = match object.get("id") {
        Some(JsonValue::String(value)) => Some(value.as_str()),
        _ => None,
    };
    let mut errors = Vec::new();

    for (index, claim) in claims.iter().enumerate() {
        let JsonValue::Object(claim) = claim else {
            errors.push(format!("identityClaims[{}] must be an object", index));
            continue;
        };
        let rule_version = match claim.get("ruleVersion") {
            Some(JsonValue::String(value)) if !value.is_empty() => value.as_str(),
            _ => {
                errors.push(format!(
                    "identityClaims[{}].ruleVersion must be a non-empty string",
                    index
                ));
                continue;
            }
        };
        let expected = match derive_identity_key_for_rule(value, rule_version) {
            Ok(value) => value,
            Err(_) => {
                errors.push(format!(
                    "identityClaims[{}].ruleVersion is unsupported: {}",
                    index, rule_version
                ));
                continue;
            }
        };
        match claim.get("identityKey") {
            Some(JsonValue::String(actual)) if actual == &expected => {}
            Some(JsonValue::String(_)) => errors.push(format!(
                "identityClaims[{}].identityKey must match the derived identity key for {}",
                index, rule_version
            )),
            _ => errors.push(format!(
                "identityClaims[{}].identityKey must be a string",
                index
            )),
        }
        if let (Some(expected_id), Some(JsonValue::String(actual_id))) =
            (canonical_id, claim.get("canonicalId"))
        {
            if actual_id != expected_id {
                errors.push(format!(
                    "identityClaims[{}].canonicalId must match the enclosing object id",
                    index
                ));
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::parse_json;

    #[test]
    fn derives_shared_identity_key_v2_fixture() {
        let raw = include_str!("../../../conformance/identity-key-v2/minimal-object.input.json");
        let expected =
            include_str!("../../../conformance/identity-key-v2/minimal-object.expected.txt");
        let value = parse_json(raw).expect("fixture must parse");
        assert_eq!(derive_identity_key_v2(&value), expected);
    }

    #[test]
    fn excludes_transport_and_provenance_fields() {
        let first = parse_json(include_str!(
            "../../../conformance/identity-key-v2/minimal-object.input.json"
        ))
        .expect("fixture must parse");
        let second = parse_json(include_str!(
            "../../../conformance/identity-key-v2/minimal-object-alternate-origin.input.json"
        ))
        .expect("fixture must parse");
        assert_eq!(
            derive_identity_key_v2(&first),
            derive_identity_key_v2(&second)
        );
    }

    #[test]
    fn validates_v1_and_v2_claims() {
        for raw in [
            include_str!("../../../fixtures/knowledge-object/with-identity-claim.json"),
            include_str!("../../../conformance/identity-claims/valid-v2.json"),
        ] {
            let value = parse_json(raw).expect("fixture must parse");
            assert!(validate_identity_claim_versions(&value).is_empty());
        }
    }

    #[test]
    fn reports_unsupported_rule_separately() {
        let value = parse_json(include_str!(
            "../../../conformance/identity-claims/unsupported-rule.json"
        ))
        .expect("fixture must parse");
        let errors = validate_identity_claim_versions(&value);
        assert!(errors.iter().any(|error| error.contains("is unsupported")));
    }
}
