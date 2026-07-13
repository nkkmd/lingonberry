use std::env;
use std::path::PathBuf;
use std::process;

use lingonberry_core::{
    build_quarantine_ledger_index, create_quarantine_compaction_preview,
    create_quarantine_replacement_preview, plan_quarantine_ledger_maintenance,
    quarantine_compaction_proof_report_json, quarantine_ledger_index_report_json,
    quarantine_ledger_maintenance_plan_json, quarantine_replacement_proof_report_json,
    quarantine_rotation_report_json, quarantine_segment_report_json, rotate_quarantine_ledger,
    runtime_state_dir, verify_quarantine_compaction_proof, verify_quarantine_ledger_index,
    verify_quarantine_replacement_proof, verify_quarantine_segments,
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
        "verify-segments" => {
            if args.len() != 1 {
                return Err(usage());
            }
            let report = verify_quarantine_segments(runtime_state_dir())
                .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_segment_report_json(&report))
            );
        }
        "rotate" => {
            let ledger = args.get(1).ok_or_else(usage)?;
            if args.len() != 2 {
                return Err(usage());
            }
            let report = rotate_quarantine_ledger(runtime_state_dir(), ledger)
                .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_rotation_report_json(&report))
            );
        }
        "compaction-preview" => {
            let backup_dir = args.get(1).ok_or_else(usage)?;
            let output_dir = args.get(2).ok_or_else(usage)?;
            if args.len() != 3 {
                return Err(usage());
            }
            let report = create_quarantine_compaction_preview(
                runtime_state_dir(),
                PathBuf::from(backup_dir),
                PathBuf::from(output_dir),
            )
            .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_compaction_proof_report_json(&report))
            );
        }
        "verify-compaction-proof" => {
            let proof_dir = args.get(1).ok_or_else(usage)?;
            if args.len() != 2 {
                return Err(usage());
            }
            let report = verify_quarantine_compaction_proof(PathBuf::from(proof_dir))
                .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_compaction_proof_report_json(&report))
            );
        }
        "replacement-preview" => {
            let backup_dir = args.get(1).ok_or_else(usage)?;
            let output_dir = args.get(2).ok_or_else(usage)?;
            if args.len() != 3 {
                return Err(usage());
            }
            let report = create_quarantine_replacement_preview(
                runtime_state_dir(),
                PathBuf::from(backup_dir),
                PathBuf::from(output_dir),
            )
            .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_replacement_proof_report_json(&report))
            );
        }
        "verify-replacement-proof" => {
            let proof_dir = args.get(1).ok_or_else(usage)?;
            if args.len() != 2 {
                return Err(usage());
            }
            let report = verify_quarantine_replacement_proof(PathBuf::from(proof_dir))
                .map_err(|error| error.to_string())?;
            println!(
                "{}",
                to_canonical_json(&quarantine_replacement_proof_report_json(&report))
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
    "usage:\n  lingonberry-quarantine-maintenance build-index\n  lingonberry-quarantine-maintenance verify-index\n  lingonberry-quarantine-maintenance verify-segments\n  lingonberry-quarantine-maintenance rotate <managed-ledger-name>\n  lingonberry-quarantine-maintenance compaction-preview <verified-backup-v2-dir> <empty-output-dir>\n  lingonberry-quarantine-maintenance verify-compaction-proof <proof-dir>\n  lingonberry-quarantine-maintenance replacement-preview <verified-backup-v2-dir> <empty-output-dir>\n  lingonberry-quarantine-maintenance verify-replacement-proof <proof-dir>\n  lingonberry-quarantine-maintenance plan <byte-threshold> <line-threshold>"
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
        assert!(run(vec!["rotate".to_string()]).is_err());
        assert!(run(vec!["compaction-preview".to_string()]).is_err());
        assert!(run(vec!["verify-compaction-proof".to_string()]).is_err());
        assert!(run(vec!["replacement-preview".to_string()]).is_err());
        assert!(run(vec!["verify-replacement-proof".to_string()]).is_err());
    }
}
