use lingonberry_protocol::{normalize_json, parse_json, to_canonical_json, JsonValue};

#[test]
fn rejects_trailing_content_and_truncated_structures() {
    let invalid = [
        "null true",
        "{}[]",
        "{",
        "[",
        "{\"key\":",
        "[1,",
        "{\"key\":1",
        "[1,2",
        "\"unterminated",
        "\"bad\\",
    ];

    for input in invalid {
        assert!(
            parse_json(input).is_err(),
            "parser unexpectedly accepted {input:?}"
        );
    }
}

#[test]
fn rejects_invalid_escape_sequences_and_control_characters() {
    let invalid = [
        r#""\x""#,
        r#""\u12""#,
        r#""\uZZZZ""#,
        "\"line\nfeed\"",
        "\"tab\tcharacter\"",
        "\"nul\u{0}character\"",
    ];

    for input in invalid {
        assert!(
            parse_json(input).is_err(),
            "parser unexpectedly accepted {input:?}"
        );
    }
}

#[test]
fn accepts_valid_json_number_boundaries_and_rejects_invalid_forms() {
    let valid = ["0", "-0", "1", "-1", "1.0", "1e3", "1E-3", "0.001"];
    for input in valid {
        assert!(
            parse_json(input).is_ok(),
            "parser unexpectedly rejected valid number {input:?}"
        );
    }

    let invalid = ["+1", "01", "-01", ".1", "1.", "1e", "1e+", "--1"];
    for input in invalid {
        assert!(
            parse_json(input).is_err(),
            "parser unexpectedly accepted invalid number {input:?}"
        );
    }
}

#[test]
fn canonical_serialization_is_stable_across_object_key_order() {
    let left = parse_json(r#"{"z":0,"a":{"y":2,"x":1},"items":[3,2,1]}"#)
        .expect("left input must parse");
    let right = parse_json(r#"{"items":[3,2,1],"a":{"x":1,"y":2},"z":0}"#)
        .expect("right input must parse");

    let left = to_canonical_json(&normalize_json(left));
    let right = to_canonical_json(&normalize_json(right));

    assert_eq!(left, right);
    assert_eq!(left, r#"{"a":{"x":1,"y":2},"items":[3,2,1],"z":0}"#);
}

#[test]
fn canonical_round_trip_preserves_normalized_value() {
    let corpus = [
        "null",
        "true",
        "false",
        "0",
        r#""plain""#,
        r#""escaped \" quote \\ slash \b\f\n\r\t""#,
        "[]",
        "{}",
        r#"[null,true,false,0,-1,1.25,1e3,"text"]"#,
        r#"{"nested":{"array":[{"b":2,"a":1}],"empty":{}},"unicode":"日本語"}"#,
    ];

    for input in corpus {
        let parsed = parse_json(input).unwrap_or_else(|error| {
            panic!("corpus entry {input:?} must parse: {error}")
        });
        let normalized = normalize_json(parsed);
        let canonical = to_canonical_json(&normalized);
        let reparsed = parse_json(&canonical).unwrap_or_else(|error| {
            panic!("canonical output {canonical:?} must parse: {error}")
        });

        assert_eq!(
            reparsed, normalized,
            "canonical round trip changed value for {input:?}"
        );
    }
}

#[test]
fn duplicate_object_keys_have_deterministic_last_value_semantics() {
    let parsed = parse_json(r#"{"key":1,"key":2}"#).expect("input must parse");
    let canonical = to_canonical_json(&normalize_json(parsed));

    assert_eq!(canonical, r#"{"key":2}"#);
    assert_eq!(
        parse_json(&canonical).expect("canonical output must parse"),
        JsonValue::Object(std::collections::BTreeMap::from([(
            "key".to_string(),
            JsonValue::Number("2".to_string()),
        )]))
    );
}
