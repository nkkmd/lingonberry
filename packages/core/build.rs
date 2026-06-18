use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let link_dir = Path::new("/tmp/lingonberry-sqlite-link");
    let _ = fs::create_dir_all(link_dir);
    let link_path = link_dir.join("libsqlite3.so");
    let target = Path::new("/lib/x86_64-linux-gnu/libsqlite3.so.0");
    if !link_path.exists() {
        let _ = std::os::unix::fs::symlink(target, &link_path);
    }
    println!("cargo:rustc-link-search=native={}", link_dir.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LIBSQLITE3_PATH");
    let _ = env::var("LIBSQLITE3_PATH");
}
