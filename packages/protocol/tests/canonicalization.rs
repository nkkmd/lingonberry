use lingonberry_protocol::{normalize_json, parse_json, to_canonical_json};

const INPUT: &str =
    include_str!("../../../conformance/canonicalization/object-key-order.input.json");
const EXPECTED: &str =
    include_str!("../../../conformance/canonicalization/object-key-order.expected.json");

#[test]
fn canonicalization_matches_shared_fixture() {
    let value = parse_json(INPUT).expect("canonicalization input fixture must parse");
    let normalized = normalize_json(value);
    let canonical = to_canonical_json(&normalized);

    assert_eq!(canonical.as_bytes(), EXPECTED.as_bytes());
}

#[test]
fn canonicalization_is_idempotent() {
    let value = parse_json(INPUT).expect("canonicalization input fixture must parse");
    let first = to_canonical_json(&normalize_json(value));
    let reparsed = parse_json(&first).expect("canonical output must parse");
    let second = to_canonical_json(&normalize_json(reparsed));

    assert_eq!(first, second);
}
