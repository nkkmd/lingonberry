use lingonberry_core::{
    classify_duplicate_or_conflict, DuplicateConflictClassification, ExistingObjectIdentity,
    IncomingObjectIdentity, DUPLICATE_CONFLICT_CONTRACT_VERSION,
};
use lingonberry_protocol::{parse_json, to_canonical_json};

fn object(text: &str) -> lingonberry_protocol::JsonValue {
    parse_json(&format!(
        r#"{{"id":"lb:obj:contract","body":{{"text":"{text}"}}}}"#
    ))
    .expect("fixture parses")
}

#[test]
fn contract_version_is_stable() {
    assert_eq!(DUPLICATE_CONFLICT_CONTRACT_VERSION, "1");
}

#[test]
fn canonical_json_equivalence_is_an_exact_duplicate() {
    let existing = parse_json(r#"{"body":{"text":"same"},"id":"lb:obj:contract"}"#)
        .expect("existing parses");
    let incoming = parse_json(r#"{"id":"lb:obj:contract","body":{"text":"same"}}"#)
        .expect("incoming parses");
    let canonical_json = to_canonical_json(&incoming);

    assert_eq!(
        classify_duplicate_or_conflict(
            None,
            Some(ExistingObjectIdentity {
                canonical_id: "lb:obj:contract",
                carrier_identity: "carrier:contract",
                object: &existing,
            }),
            IncomingObjectIdentity {
                canonical_id: "lb:obj:contract",
                carrier_identity: "carrier:contract",
                canonical_json: &canonical_json,
            },
        ),
        DuplicateConflictClassification::ExactDuplicate
    );
}

#[test]
fn canonical_id_cannot_be_rebound_to_another_carrier_identity() {
    let existing = object("same");
    let canonical_json = to_canonical_json(&existing);

    assert_eq!(
        classify_duplicate_or_conflict(
            Some(ExistingObjectIdentity {
                canonical_id: "lb:obj:contract",
                carrier_identity: "carrier:original",
                object: &existing,
            }),
            None,
            IncomingObjectIdentity {
                canonical_id: "lb:obj:contract",
                carrier_identity: "carrier:replacement",
                canonical_json: &canonical_json,
            },
        ),
        DuplicateConflictClassification::CrossIdentityConflict
    );
}

#[test]
fn carrier_identity_cannot_be_rebound_to_another_canonical_id() {
    let existing = object("same");
    let canonical_json = to_canonical_json(&existing);

    assert_eq!(
        classify_duplicate_or_conflict(
            None,
            Some(ExistingObjectIdentity {
                canonical_id: "lb:obj:original",
                carrier_identity: "carrier:contract",
                object: &existing,
            }),
            IncomingObjectIdentity {
                canonical_id: "lb:obj:replacement",
                carrier_identity: "carrier:contract",
                canonical_json: &canonical_json,
            },
        ),
        DuplicateConflictClassification::CrossIdentityConflict
    );
}
