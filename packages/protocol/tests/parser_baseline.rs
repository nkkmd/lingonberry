use std::panic::{catch_unwind, AssertUnwindSafe};

use lingonberry_protocol::{normalize_json, parse_json, to_canonical_json};

#[test]
fn rejects_trailing_content() {
    assert!(parse_json("null true").is_err());
    assert!(parse_json("{}[]").is_err());
}

#[test]
fn rejects_truncated_structures() {
    assert!(parse_json("{").is_err());
    assert!(parse_json("[").is_err());
    assert!(parse_json("[1,").is_err());
}

#[test]
fn rejects_invalid_numbers() {
    assert!(parse_json("+1").is_err());
    assert!(parse_json("01").is_err());
    assert!(parse_json("1.").is_err());
    assert!(parse_json("1e").is_err());
}

#[test]
fn canonical_output_sorts_object_keys() {
    let value = parse_json(r#"{"z":0,"a":1}"#).unwrap();
    assert_eq!(to_canonical_json(&value), r#"{"a":1,"z":0}"#);
}

#[test]
fn canonical_round_trip_is_idempotent() {
    let value = parse_json(r#"{"z":[3,2,1],"a":{"y":2,"x":1}}"#).unwrap();
    let canonical = to_canonical_json(&normalize_json(value));
    let reparsed = parse_json(&canonical).unwrap();
    let repeated = to_canonical_json(&normalize_json(reparsed));
    assert_eq!(canonical, repeated);
}

#[test]
fn representative_inputs_never_panic() {
    let inputs = [
        "",
        "null",
        "true",
        "false",
        "[]",
        "{}",
        "[1,2,3]",
        r#"{"a":1,"b":[true,null]}"#,
        "[",
        "{",
        r#""unterminated"#,
        r#""bad\\x""#,
    ];

    for input in inputs {
        let result = catch_unwind(AssertUnwindSafe(|| parse_json(input)));
        assert!(result.is_ok(), "parser panicked for {input:?}");
    }
}

#[test]
fn repeated_parse_results_are_deterministic() {
    let inputs = ["null", "[]", "{}", "[1,2,3]", "[", "{", "01"];

    for input in inputs {
        let first = parse_json(input);
        let second = parse_json(input);
        let first = first.map(|value| to_canonical_json(&normalize_json(value)));
        let second = second.map(|value| to_canonical_json(&normalize_json(value)));
        assert_eq!(first, second);
    }
}

#[test]
fn moderate_nesting_remains_supported() {
    let depth = 64;
    let input = format!("{}null{}", "[".repeat(depth), "]".repeat(depth));
    assert!(parse_json(&input).is_ok());
}
