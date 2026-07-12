use std::env;
use std::path::PathBuf;
use std::process;

use lingonberry_core::{
    export_complete_quarantine_backup, quarantine_backup_report_json,
    restore_any_quarantine_backup, runtime_state_dir, verify_any_quarantine_backup,
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
    let report = match command {
        "export" => {
            let backup_dir = args.get(1).ok_or_else(usage)?;
            if args.len() != 2 {
                return Err(usage());
            }
            export_complete_quarantine_backup(runtime_state_dir(), PathBuf::from(backup_dir))
                .map_err(|error| error.to_string())?
        }
        "verify" => {
            let backup_dir = args.get(1).ok_or_else(usage)?;
            if args.len() != 2 {
                return Err(usage());
            }
            verify_any_quarantine_backup(PathBuf::from(backup_dir))
                .map_err(|error| error.to_string())?
        }
        "restore" => {
            let backup_dir = args.get(1).ok_or_else(usage)?;
            let destination = args.get(2).ok_or_else(usage)?;
            if args.len() != 3 {
                return Err(usage());
            }
            restore_any_quarantine_backup(
                PathBuf::from(backup_dir),
                PathBuf::from(destination),
            )
            .map_err(|error| error.to_string())?
        }
        _ => return Err(usage()),
    };
    println!(
        "{}",
        to_canonical_json(&quarantine_backup_report_json(&report))
    );
    Ok(())
}

fn usage() -> String {
    "usage:\n  lingonberry-quarantine-backup export <backup-dir>\n  lingonberry-quarantine-backup verify <backup-dir>\n  lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_or_incomplete_commands() {
        assert!(run(vec![]).is_err());
        assert!(run(vec!["unknown".to_string()]).is_err());
        assert!(run(vec!["export".to_string()]).is_err());
        assert!(run(vec!["restore".to_string(), "backup".to_string()]).is_err());
    }
}