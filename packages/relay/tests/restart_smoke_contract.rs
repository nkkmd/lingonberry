use lingonberry_protocol::{parse_json, JsonValue};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BINARY: &str = env!("CARGO_BIN_EXE_lingonberry-relay");

#[test]
fn publish_restart_query_and_consistency_smoke_succeeds() {
    let state_dir = unique_temp_dir("restart");
    fs::create_dir_all(&state_dir).expect("create state directory");
    let fixture = workspace_root().join("fixtures/http-publish-request/minimal-request.json");

    let publish = run_cli(
        &state_dir,
        &["publish", fixture.to_str().expect("fixture path")],
    );
    assert_success(&publish);
    let canonical_id = response_string(&publish, "canonicalId");

    let before_restart = run_cli(&state_dir, &["subscribe"]);
    assert_success(&before_restart);
    assert_contract(&before_restart, "success", "LB_QUERY_SUCCESS");

    let first_port = available_port();
    let mut first_server = spawn_http_server(&state_dir, first_port);
    wait_until_ready(first_port);
    let first_get = http_get(first_port, &canonical_id);
    assert!(first_get.starts_with("HTTP/1.1 200 "), "{first_get}");
    first_server.kill().ok();
    first_server.wait().ok();

    let second_port = available_port();
    let mut restarted_server = spawn_http_server(&state_dir, second_port);
    wait_until_ready(second_port);
    let restarted_get = http_get(second_port, &canonical_id);
    assert!(
        restarted_get.starts_with("HTTP/1.1 200 "),
        "{restarted_get}"
    );
    assert!(
        restarted_get.contains("\"status\":\"found\""),
        "{restarted_get}"
    );
    assert!(
        restarted_get.contains("\"code\":\"LB_OBJECT_FOUND\""),
        "{restarted_get}"
    );
    restarted_server.kill().ok();
    restarted_server.wait().ok();

    let after_restart = run_cli(&state_dir, &["subscribe"]);
    assert_success(&after_restart);
    assert_contract(&after_restart, "success", "LB_QUERY_SUCCESS");
    assert!(String::from_utf8_lossy(&after_restart.stdout).contains("\"count\":1"));

    let rebuild = run_cli(&state_dir, &["rebuild-index"]);
    assert_success(&rebuild);
    assert_contract(&rebuild, "consistent", "LB_INDEX_CONSISTENT");
    assert!(state_dir.join("index/checkpoint.json").is_file());

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

fn spawn_http_server(state_dir: &Path, port: u16) -> Child {
    Command::new(BINARY)
        .arg("serve-http")
        .arg(format!("127.0.0.1:{port}"))
        .env("LINGONBERRY_STATE_DIR", state_dir)
        .env_remove("LINGONBERRY_REQUIRE_IDENTITY_CLAIM")
        .env_remove("LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn HTTP server")
}

fn wait_until_ready(port: u16) {
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("HTTP server did not become ready");
}

fn http_get(port: u16, canonical_id: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect HTTP server");
    let request = format!(
        "GET /v1/objects/{canonical_id} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    response
}

fn response_string(output: &Output, key: &str) -> String {
    let stdout = String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout");
    let JsonValue::Object(map) = parse_json(stdout.trim()).expect("parse response") else {
        panic!("response must be object");
    };
    match map.get(key) {
        Some(JsonValue::String(value)) => value.clone(),
        other => panic!("missing {key}: {other:?}"),
    }
}

fn assert_success(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
}

fn assert_contract(output: &Output, status: &str, code: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"contractVersion\":\"1\""), "{stdout}");
    assert!(
        stdout.contains(&format!("\"status\":\"{status}\"")),
        "{stdout}"
    );
    assert!(stdout.contains(&format!("\"code\":\"{code}\"")), "{stdout}");
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn available_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind ephemeral port")
        .local_addr()
        .expect("read ephemeral port")
        .port()
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "lingonberry-phase5-{label}-{}-{nonce}",
        std::process::id()
    ))
}
