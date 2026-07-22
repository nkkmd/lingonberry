use lingonberry_protocol::{parse_json, to_canonical_json};

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
fn moderate_nesting_remains_supported() {
    let depth = 64;
    let input = format!("{}null{}", "[".repeat(depth), "]".repeat(depth));
    assert!(parse_json(&input).is_ok());
}
