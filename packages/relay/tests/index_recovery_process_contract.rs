use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const BINARY: &str = env!("CARGO_BIN_EXE_lingonberry-relay");

#[test]
fn catch_up_index_rebuilds_stale_state_and_rejects_corruption() {
    let state_dir = unique_temp_dir();
    fs::create_dir_all(&state_dir).expect("create state directory");
    let fixture = workspace_root().join("fixtures/http-publish-request/minimal-request.json");

    assert_success(&run_cli(
        &state_dir,
        &["publish", fixture.to_str().expect("fixture path")],
    ));

    let rebuilt = run_cli(&state_dir, &["catch-up-index"]);
    assert_contract(&rebuilt, true, "rebuilt", "LB_INDEX_REBUILT");

    let up_to_date = run_cli(&state_dir, &["catch-up-index"]);
    assert_contract(
        &up_to_date,
        true,
        "up-to-date",
        "LB_INDEX_UP_TO_DATE",
    );

    let checkpoint = state_dir.join("index/checkpoint.json");
    fs::write(&checkpoint, "{not-json").expect("corrupt checkpoint");
    let failed = run_cli(&state_dir, &["catch-up-index"]);
    assert_contract(
        &failed,
        false,
        "failed",
        "LB_INDEX_CHECKPOINT_CORRUPT",
    );
    assert_eq!(
        fs::read_to_string(&checkpoint).expect("read checkpoint"),
        "{not-json"
    );

    fs::remove_dir_all(state_dir).ok();
}

fn run_cli(state_dir: &Path, args: &[&str]) -> Output {
    Command::new(BINARY)
        .args(args)
        .env("LINGONBERRY_STATE_DIR", state_dir)
        .env_remove("LINGONBERRY_REQUIRE_IDENTITY_CLAIM")
        .env_remove("LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY")
        .output()
        .expect("run CLI")
}

fn assert_success(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
}

fn assert_contract(output: &Output, success: bool, status: &str, code: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.success(), success, "stdout={stdout}\nstderr={stderr}");
    assert!(stdout.contains("\"contractVersion\":\"1\""), "{stdout}");
    assert!(stdout.contains(&format!("\"status\":\"{status}\"")), "{stdout}");
    assert!(stdout.contains(&format!("\"code\":\"{code}\"")), "{stdout}");
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lingonberry-index-recovery-{}-{nonce}",
        std::process::id()
    ))
}
