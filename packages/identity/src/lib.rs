use std::collections::BTreeMap;

use lingonberry_protocol::{to_canonical_json, JsonValue};
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

#[cfg(test)]
mod tests {
    use super::*;
    use lingonberry_protocol::parse_json;

    #[test]
    fn derives_shared_identity_key_v2_fixture() {
        let raw = include_str!(
            "../../../conformance/identity-key-v2/minimal-object.input.json"
        );
        let expected = include_str!(
            "../../../conformance/identity-key-v2/minimal-object.expected.txt"
        );
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

        assert_eq!(derive_identity_key_v2(&first), derive_identity_key_v2(&second));
    }
}
