mod existing_classified {
    include!("main_v0_5_classified.rs");
    pub fn run_main() {
        main();
    }
}

use lingonberry_core::{build_runtime_storage_backend, runtime_state_dir};
use lingonberry_indexer::{
    index_rebuild_result_json, persist_index_checkpoint, rebuild_index, IndexConsistencyStatus,
};
use lingonberry_protocol::to_canonical_json;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) != Some("rebuild-index") {
        existing_classified::run_main();
        return;
    }
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
