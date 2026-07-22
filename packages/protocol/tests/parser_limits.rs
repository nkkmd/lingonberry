use lingonberry_protocol::{parse_json, MAX_JSON_INPUT_BYTES, MAX_JSON_NESTING_DEPTH};

#[test]
fn rejects_input_larger_than_limit() {
    let input = " ".repeat(MAX_JSON_INPUT_BYTES + 1);
    let error = parse_json(&input).unwrap_err();
    assert!(error.message.contains("exceeds"));
}

#[test]
fn accepts_input_at_limit_when_valid() {
    let padding = " ".repeat(MAX_JSON_INPUT_BYTES - 4);
    let input = format!("null{padding}");
    assert_eq!(input.len(), MAX_JSON_INPUT_BYTES);
    assert!(parse_json(&input).is_ok());
}

#[test]
fn accepts_maximum_nesting_depth() {
    let input = format!(
        "{}null{}",
        "[".repeat(MAX_JSON_NESTING_DEPTH),
        "]".repeat(MAX_JSON_NESTING_DEPTH)
    );
    assert!(parse_json(&input).is_ok());
}

#[test]
fn rejects_nesting_above_limit_without_panicking() {
    let input = format!(
        "{}null{}",
        "[".repeat(MAX_JSON_NESTING_DEPTH + 1),
        "]".repeat(MAX_JSON_NESTING_DEPTH + 1)
    );
    let error = parse_json(&input).unwrap_err();
    assert!(error.message.contains("nesting exceeds"));
}

#[test]
fn rejects_mixed_nesting_above_limit() {
    let mut input = String::new();
    for index in 0..=MAX_JSON_NESTING_DEPTH {
        if index % 2 == 0 {
            input.push_str(r#"{"x":"#);
        } else {
            input.push('[');
        }
    }
    input.push_str("null");
    for index in (0..=MAX_JSON_NESTING_DEPTH).rev() {
        if index % 2 == 0 {
            input.push('}');
        } else {
            input.push(']');
        }
    }
    assert!(parse_json(&input).is_err());
}
