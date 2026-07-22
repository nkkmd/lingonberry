use std::panic::{catch_unwind, AssertUnwindSafe};

use lingonberry_protocol::{normalize_json, parse_json, to_canonical_json};

const SEEDS: &[&str] = &[
    "null",
    "true",
    "false",
    "0",
    "-1.25e3",
    r#""lingonberry""#,
    "[]",
    "{}",
    r#"[null,true,false,0,"text",{"b":2,"a":1}]"#,
    r#"{"object":{"id":"lb:obj:test"},"publisher":{"publicKey":"00"}}"#,
];

#[test]
fn single_byte_mutations_never_panic_and_are_deterministic() {
    for seed in SEEDS {
        let bytes = seed.as_bytes();
        for index in 0..bytes.len() {
            for replacement in mutation_bytes(bytes[index]) {
                let mut candidate = bytes.to_vec();
                candidate[index] = replacement;
                let Ok(candidate) = String::from_utf8(candidate) else {
                    continue;
                };

                let first = catch_unwind(AssertUnwindSafe(|| parse_json(&candidate)));
                let second = catch_unwind(AssertUnwindSafe(|| parse_json(&candidate)));

                assert!(
                    first.is_ok(),
                    "parser panicked for mutation {candidate:?}"
                );
                assert!(
                    second.is_ok(),
                    "parser panicked for repeated mutation {candidate:?}"
                );

                let first = first.expect("panic checked above");
                let second = second.expect("panic checked above");
                assert_eq!(
                    result_fingerprint(first),
                    result_fingerprint(second),
                    "parser result changed across identical input {candidate:?}"
                );
            }
        }
    }
}

#[test]
fn accepted_mutations_round_trip_through_canonical_serialization() {
    for seed in SEEDS {
        let bytes = seed.as_bytes();
        for index in 0..bytes.len() {
            for replacement in mutation_bytes(bytes[index]) {
                let mut candidate = bytes.to_vec();
                candidate[index] = replacement;
                let Ok(candidate) = String::from_utf8(candidate) else {
                    continue;
                };
                let Ok(parsed) = parse_json(&candidate) else {
                    continue;
                };

                let normalized = normalize_json(parsed);
                let canonical = to_canonical_json(&normalized);
                let reparsed = parse_json(&canonical).unwrap_or_else(|error| {
                    panic!("canonical output {canonical:?} failed to parse: {error}")
                });

                assert_eq!(
                    reparsed, normalized,
                    "accepted mutation changed during canonical round trip: {candidate:?}"
                );
            }
        }
    }
}

#[test]
fn moderate_nesting_is_supported_without_panicking() {
    for depth in [1_usize, 2, 8, 16, 32, 64] {
        let input = format!("{}null{}", "[".repeat(depth), "]".repeat(depth));
        let parsed = catch_unwind(AssertUnwindSafe(|| parse_json(&input)));
        assert!(parsed.is_ok(), "parser panicked at nesting depth {depth}");
        assert!(
            parsed.expect("panic checked above").is_ok(),
            "parser rejected supported nesting depth {depth}"
        );
    }
}

fn mutation_bytes(original: u8) -> [u8; 8] {
    [
        0,
        b' ',
        b'"',
        b'\\',
        b'{',
        b'[',
        b'0',
        original.wrapping_add(1),
    ]
}

fn result_fingerprint(
    result: Result<lingonberry_protocol::JsonValue, lingonberry_protocol::JsonError>,
) -> String {
    match result {
        Ok(value) => format!("ok:{}", to_canonical_json(&normalize_json(value))),
        Err(error) => format!("err:{}:{}", error.position, error.message),
    }
}
