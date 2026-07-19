mod existing_classified {
    include!("main_v0_5_classified.rs");
    pub fn run_main() {
        main();
    }
}

use lingonberry_core::{build_runtime_storage_backend, runtime_state_dir};
use lingonberry_indexer::{
    catch_up_index, index_catch_up_result_json, index_rebuild_result_json,
    persist_index_checkpoint, rebuild_index, IndexCatchUpStatus, IndexConsistencyStatus,
};
use lingonberry_protocol::to_canonical_json;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("rebuild-index") => run_rebuild_index(&args),
        Some("catch-up-index") => run_catch_up_index(&args),
        _ => existing_classified::run_main(),
    }
}

fn run_rebuild_index(args: &[String]) {
    if args.len() != 1 {
        eprintln!("usage: lingonberry rebuild-index");
        std::process::exit(64);
    }
    let backend = build_runtime_storage_backend();
    let result = rebuild_index(&backend);
    println!("{}", to_canonical_json(&index_rebuild_result_json(&result)));
    if result.status != IndexConsistencyStatus::Consistent {
        eprintln!("{}: {}", result.code, result.message);
        std::process::exit(70);
    }
    let path = runtime_state_dir().join("index/checkpoint.json");
    if let Err(error) = persist_index_checkpoint(path, &result) {
        eprintln!("{error}");
        std::process::exit(70);
    }
}

fn run_catch_up_index(args: &[String]) {
    if args.len() != 1 {
        eprintln!("usage: lingonberry catch-up-index");
        std::process::exit(64);
    }
    let backend = build_runtime_storage_backend();
    let path = runtime_state_dir().join("index/checkpoint.json");
    let result = catch_up_index(&backend, path);
    println!(
        "{}",
        to_canonical_json(&index_catch_up_result_json(&result))
    );
    if result.status == IndexCatchUpStatus::Failed {
        eprintln!("{}: {}", result.code, result.message);
        std::process::exit(70);
    }
}
