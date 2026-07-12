use std::env;
use std::process;

use lingonberry_core::{
    build_quarantine_ledger_index, plan_quarantine_ledger_maintenance,
    quarantine_ledger_index_report_json, quarantine_ledger_maintenance_plan_json,
    runtime_state_dir, verify_quarantine_ledger_index,
};
use lingonberry_protocol::to_canonical_json;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let command = args.first().map(String::as_str).ok_or_else(usage)?;
    match command {
        "build-index" => {
            if args.len() != 1 {
                return Err(usage());
            }
            let report = build_quarantine_ledger_index(runtime_state_dir())
                .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_ledger_index_report_json(&report))
            );
        }
        "verify-index" => {
            if args.len() != 1 {
                return Err(usage());
            }
            let report = verify_quarantine_ledger_index(runtime_state_dir())
                .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_ledger_index_report_json(&report))
            );
        }
        "plan" => {
            let byte_threshold = args
                .get(1)
                .ok_or_else(usage)?
                .parse::<u64>()
                .map_err(|_| "byte threshold must be an unsigned integer".to_string())?;
            let line_threshold = args
                .get(2)
                .ok_or_else(usage)?
                .parse::<u64>()
                .map_err(|_| "line threshold must be an unsigned integer".to_string())?;
            if args.len() != 3 {
                return Err(usage());
            }
            let plan = plan_quarantine_ledger_maintenance(
                runtime_state_dir(),
                byte_threshold,
                line_threshold,
            )
            .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_ledger_maintenance_plan_json(&plan))
            );
        }
        _ => return Err(usage()),
    }
    Ok(())
}

fn usage() -> String {
    "usage:\n  lingonberry-quarantine-maintenance build-index\n  lingonberry-quarantine-maintenance verify-index\n  lingonberry-quarantine-maintenance plan <byte-threshold> <line-threshold>"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_and_incomplete_commands() {
        assert!(run(vec![]).is_err());
        assert!(run(vec!["unknown".to_string()]).is_err());
        assert!(run(vec!["plan".to_string(), "100".to_string()]).is_err());
    }
}