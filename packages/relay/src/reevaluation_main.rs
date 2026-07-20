use lingonberry_core::{build_runtime_storage_backend, runtime_state_dir};
use lingonberry_protocol::to_canonical_json;
use lingonberry_relay::{
    process_reevaluation_queue, reconcile_reevaluation_queue, reevaluation_report_json,
};

fn main() {
    let reconcile = std::env::args().skip(1).any(|argument| argument == "--reconcile");
    let backend = build_runtime_storage_backend();
    let state_dir = runtime_state_dir();
    let result = if reconcile {
        reconcile_reevaluation_queue(&backend, &state_dir)
    } else {
        process_reevaluation_queue(&backend, &state_dir)
    };
    match result {
        Ok(report) => println!("{}", to_canonical_json(&reevaluation_report_json(&report))),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(70);
        }
    }
}
