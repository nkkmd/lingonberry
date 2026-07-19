mod existing_v0_5 {
    include!("main_v0_5.rs");

    pub fn run_main() {
        main();
    }
}

use lingonberry_core::{
    build_runtime_storage_backend, promote_quarantine_record_classified, QuarantinePromotionOutcome,
};
use lingonberry_protocol::{to_canonical_json, JsonValue};
use std::env;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("quarantine-promote") {
        existing_v0_5::run_main();
        return;
    }

    if let Err(error) = handle_quarantine_promote(&args) {
        eprintln!("{error}");
        process::exit(if error.starts_with("usage:") { 64 } else { 70 });
    }
}

fn handle_quarantine_promote(args: &[String]) -> Result<(), String> {
    let quarantine_id = args
        .get(1)
        .ok_or_else(|| "usage: lingonberry quarantine-promote <quarantine-id>".to_string())?;
    let backend = build_runtime_storage_backend();
    let outcome = promote_quarantine_record_classified(quarantine_id, &backend)
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&promotion_outcome_json(outcome)));
    Ok(())
}

fn promotion_outcome_json(outcome: QuarantinePromotionOutcome) -> JsonValue {
    match outcome {
        QuarantinePromotionOutcome::Promoted {
            quarantine_id,
            canonical_id,
            duplicate,
        } => json_object(vec![
            ("status", JsonValue::String("promoted".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("canonicalId", JsonValue::String(canonical_id)),
            ("duplicate", JsonValue::Bool(duplicate)),
        ]),
        QuarantinePromotionOutcome::AlreadyPromoted {
            quarantine_id,
            canonical_id,
            duplicate,
        } => json_object(vec![
            ("status", JsonValue::String("already-promoted".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("canonicalId", JsonValue::String(canonical_id)),
            ("duplicate", JsonValue::Bool(duplicate)),
        ]),
        QuarantinePromotionOutcome::StillDeferred {
            quarantine_id,
            code,
            errors,
        } => json_object(vec![
            ("status", JsonValue::String("deferred".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("code", JsonValue::String(code.to_string())),
            (
                "errors",
                JsonValue::Array(errors.into_iter().map(JsonValue::String).collect()),
            ),
        ]),
        QuarantinePromotionOutcome::Rejected {
            quarantine_id,
            code,
            errors,
        } => json_object(vec![
            ("status", JsonValue::String("rejected".to_string())),
            ("quarantineId", JsonValue::String(quarantine_id)),
            ("code", JsonValue::String(code.to_string())),
            (
                "errors",
                JsonValue::Array(errors.into_iter().map(JsonValue::String).collect()),
            ),
        ]),
    }
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}
