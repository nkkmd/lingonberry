#![allow(clippy::duplicate_mod)]

mod v0_5_index {
    include!("main_v0_5_index.rs");

    pub fn run_main() {
        main();
    }
}

mod v0_6_release {
    include!("main_v0_6_release.rs");

    pub fn run_main() {
        main();
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("publish") | Some("serve-http") => v0_6_release::run_main(),
        _ => v0_5_index::run_main(),
    }
}
