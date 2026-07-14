use std::env;
use std::path::{Path, PathBuf};
use std::process;

use lingonberry_core::{
    append_quarantine_replacement_audit_event, apply_quarantine_replacement_transaction,
    build_quarantine_ledger_index, create_quarantine_compaction_preview,
    create_quarantine_replacement_preview, plan_quarantine_ledger_maintenance,
    quarantine_compaction_proof_report_json, quarantine_ledger_index_report_json,
    quarantine_ledger_maintenance_plan_json, quarantine_replacement_metrics_text,
    quarantine_replacement_proof_report_json, quarantine_replacement_status,
    quarantine_replacement_status_v1_json, quarantine_rotation_report_json,
    quarantine_segment_report_json, resume_quarantine_replacement_transaction,
    rollback_quarantine_replacement_transaction, rotate_quarantine_ledger, runtime_state_dir,
    verify_quarantine_compaction_proof, verify_quarantine_ledger_index,
    verify_quarantine_replacement_proof, verify_quarantine_segments,
    QuarantineReplacementAuditEventType, QuarantineReplacementAuditOperation,
    QuarantineReplacementAuditOutcome, QuarantineReplacementStatusReport,
    QuarantineReplacementTransactionState, StoreError,
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
        "replacement-apply" => {
            let backup_dir = args.get(1).ok_or_else(usage)?;
            let proof_dir = args.get(2).ok_or_else(usage)?;
            let transaction_dir = args.get(3).ok_or_else(usage)?;
            if args.len() != 4 {
                return Err(usage());
            }
            let transaction_id = transaction_id_from_dir(Path::new(transaction_dir))?;
            let state_dir = runtime_state_dir();
            audit_started(&state_dir, QuarantineReplacementAuditOperation::Apply)?;
            let result = apply_quarantine_replacement_transaction(
                &state_dir,
                PathBuf::from(backup_dir),
                PathBuf::from(proof_dir),
                PathBuf::from(transaction_dir),
                &transaction_id,
            );
            let report = audit_result(
                &state_dir,
                QuarantineReplacementAuditOperation::Apply,
                result,
            )?;
            print_status(&report);
        }
        "replacement-status" => {
            let transaction_dir = args.get(1).ok_or_else(usage)?;
            if args.len() != 2 {
                return Err(usage());
            }
            let state_dir = runtime_state_dir();
            let result = quarantine_replacement_status(&state_dir, PathBuf::from(transaction_dir));
            let report = audit_result(
                &state_dir,
                QuarantineReplacementAuditOperation::Status,
                result,
            )?;
            print_status(&report);
        }
        "replacement-metrics" => {
            let transaction_dir = args.get(1).ok_or_else(usage)?;
            if args.len() != 2 {
                return Err(usage());
            }
            let report =
                quarantine_replacement_status(runtime_state_dir(), PathBuf::from(transaction_dir))
                    .map_err(|error| error.to_string())?;
            print!("{}", quarantine_replacement_metrics_text(&report));
        }
        "replacement-recover" => {
            let transaction_dir = args.get(1).ok_or_else(usage)?;
            let mode = args.get(2).map(String::as_str).ok_or_else(usage)?;
            if args.len() != 3 {
                return Err(usage());
            }
            let state_dir = runtime_state_dir();
            let operation = match mode {
                "--resume" => QuarantineReplacementAuditOperation::Resume,
                "--rollback" => QuarantineReplacementAuditOperation::Rollback,
                _ => return Err(usage()),
            };
            audit_started(&state_dir, operation)?;
            let result = match operation {
                QuarantineReplacementAuditOperation::Resume => {
                    resume_quarantine_replacement_transaction(
                        &state_dir,
                        PathBuf::from(transaction_dir),
                    )
                }
                QuarantineReplacementAuditOperation::Rollback => {
                    rollback_quarantine_replacement_transaction(
                        &state_dir,
                        PathBuf::from(transaction_dir),
                    )
                }
                QuarantineReplacementAuditOperation::Apply
                | QuarantineReplacementAuditOperation::Status => unreachable!(),
            };
            let report = audit_result(&state_dir, operation, result)?;
            print_status(&report);
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

fn audit_started(
    state_dir: &Path,
    operation: QuarantineReplacementAuditOperation,
) -> Result<(), String> {
    append_quarantine_replacement_audit_event(
        state_dir,
        QuarantineReplacementAuditEventType::OperationStarted,
        operation,
        QuarantineReplacementAuditOutcome::Started,
        None,
        None,
        None,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn audit_result(
    state_dir: &Path,
    operation: QuarantineReplacementAuditOperation,
    result: Result<QuarantineReplacementStatusReport, StoreError>,
) -> Result<QuarantineReplacementStatusReport, String> {
    match result {
        Ok(report) => {
            let event_type = match report.state {
                QuarantineReplacementTransactionState::Committed => {
                    QuarantineReplacementAuditEventType::Committed
                }
                QuarantineReplacementTransactionState::RolledBack => {
                    QuarantineReplacementAuditEventType::RolledBack
                }
                QuarantineReplacementTransactionState::RecoveryRequired => {
                    QuarantineReplacementAuditEventType::RecoveryRequired
                }
                _ => QuarantineReplacementAuditEventType::OperationCompleted,
            };
            let outcome = if report.state == QuarantineReplacementTransactionState::RecoveryRequired
            {
                QuarantineReplacementAuditOutcome::Failed
            } else {
                QuarantineReplacementAuditOutcome::Success
            };
            append_quarantine_replacement_audit_event(
                state_dir,
                event_type,
                operation,
                outcome,
                Some(report.state),
                Some(&report.classification),
                None,
            )
            .map_err(|error| error.to_string())?;
            Ok(report)
        }
        Err(error) => {
            let event_type = if operation == QuarantineReplacementAuditOperation::Status {
                QuarantineReplacementAuditEventType::StatusCorrupt
            } else {
                QuarantineReplacementAuditEventType::OperationRejected
            };
            append_quarantine_replacement_audit_event(
                state_dir,
                event_type,
                operation,
                QuarantineReplacementAuditOutcome::Rejected,
                None,
                if operation == QuarantineReplacementAuditOperation::Status {
                    Some("corrupt")
                } else {
                    None
                },
                Some(&error.code),
            )
            .map_err(|audit_error| {
                format!("{}; audit failure: {}", error, audit_error)
            })?;
            Err(error.to_string())
        }
    }
}

fn print_status(report: &QuarantineReplacementStatusReport) {
    println!(
        "{}",
        to_canonical_json(&quarantine_replacement_status_v1_json(report))
    );
}

fn transaction_id_from_dir(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "transaction directory must have a UTF-8 basename".to_string())
}

fn usage() -> String {
    "usage:\n  lingonberry-quarantine-maintenance build-index\n  lingonberry-quarantine-maintenance verify-index\n  lingonberry-quarantine-maintenance verify-segments\n  lingonberry-quarantine-maintenance rotate <managed-ledger-name>\n  lingonberry-quarantine-maintenance compaction-preview <verified-backup-v2-dir> <empty-output-dir>\n  lingonberry-quarantine-maintenance verify-compaction-proof <proof-dir>\n  lingonberry-quarantine-maintenance replacement-preview <verified-backup-v2-dir> <empty-output-dir>\n  lingonberry-quarantine-maintenance verify-replacement-proof <proof-dir>\n  lingonberry-quarantine-maintenance replacement-apply <verified-backup-v2-dir> <verified-proof-dir> <transaction-dir>\n  lingonberry-quarantine-maintenance replacement-status <transaction-dir>\n  lingonberry-quarantine-maintenance replacement-metrics <transaction-dir>\n  lingonberry-quarantine-maintenance replacement-recover <transaction-dir> --resume|--rollback\n  lingonberry-quarantine-maintenance plan <byte-threshold> <line-threshold>"
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
        assert!(run(vec!["replacement-apply".to_string()]).is_err());
        assert!(run(vec!["replacement-status".to_string()]).is_err());
        assert!(run(vec!["replacement-metrics".to_string()]).is_err());
        assert!(run(vec!["replacement-recover".to_string()]).is_err());
    }

    #[test]
    fn derives_transaction_id_from_workspace_basename() {
        assert_eq!(
            transaction_id_from_dir(Path::new("/tmp/tx-example-001")).unwrap(),
            "tx-example-001"
        );
    }

    #[test]
    fn usage_lists_replacement_observability_commands() {
        let usage = usage();
        assert!(usage.contains("replacement-status <transaction-dir>"));
        assert!(usage.contains("replacement-metrics <transaction-dir>"));
    }
}
