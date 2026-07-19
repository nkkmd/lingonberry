use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BINARY: &str = env!("CARGO_BIN_EXE_lingonberry-relay");

#[test]
fn cli_publish_contract_covers_terminal_states() {
    let workspace = workspace_root();
    let state_dir = unique_temp_dir("cli");
    let minimal = workspace.join("fixtures/http-publish-request/minimal-request.json");
    let invalid = workspace.join("fixtures/http-publish-request/invalid-schema-version.json");
    let conflict = state_dir.join("conflict.json");
    let deferred = state_dir.join("deferred.json");
    fs::create_dir_all(&state_dir).expect("create CLI state directory");
    write_conflict_fixture(&minimal, &conflict);
    write_deferred_fixture(
        &workspace.join("fixtures/http-publish-request/with-identity-claim.json"),
        &deferred,
    );

    let stored = run_cli_publish(&state_dir, &minimal, &[]);
    assert_success_contract(&stored, "\"status\":\"stored\"", "LB_OBJECT_STORED");

    let duplicate = run_cli_publish(&state_dir, &minimal, &[]);
    assert_success_contract(
        &duplicate,
        "\"status\":\"duplicate\"",
        "LB_OBJECT_DUPLICATE",
    );

    let rejected = run_cli_publish(&state_dir, &invalid, &[]);
    assert_failure_contract(&rejected, "\"status\":\"rejected\"", "LB_VALIDATION_FAILED");

    let conflict_output = run_cli_publish(&state_dir, &conflict, &[]);
    assert_failure_contract(
        &conflict_output,
        "\"status\":\"conflict\"",
        "LB_OBJECT_CONFLICT",
    );

    let deferred_output = run_cli_publish(
        &state_dir,
        &deferred,
        &[("LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY", "defer")],
    );
    assert_success_contract(
        &deferred_output,
        "\"status\":\"deferred\"",
        "LB_IDENTITY_DEFERRED",
    );

    fs::remove_dir_all(state_dir).ok();
}

#[test]
fn http_publish_contract_covers_terminal_states() {
    let workspace = workspace_root();
    let state_dir = unique_temp_dir("http");
    fs::create_dir_all(&state_dir).expect("create HTTP state directory");
    let minimal = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/minimal-request.json"),
    )
    .expect("read minimal fixture");
    let rejected = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/invalid-schema-version.json"),
    )
    .expect("read rejected fixture");
    let conflict = minimal.replace(
        "What evidence supports this claim?",
        "What contradictory evidence supports this claim?",
    );
    let deferred = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/with-identity-claim.json"),
    )
    .expect("read identity fixture")
    .replace("lb.identity.key.v1", "lb.identity.key.v99");

    let port = available_port();
    let mut server = spawn_http_server(&state_dir, port, true);
    wait_until_ready(port);

    assert_http_contract(port, &minimal, 201, "stored", "LB_OBJECT_STORED");
    assert_http_contract(port, &minimal, 200, "duplicate", "LB_OBJECT_DUPLICATE");
    assert_http_contract(port, &rejected, 400, "rejected", "LB_VALIDATION_FAILED");
    assert_http_contract(port, &conflict, 409, "conflict", "LB_OBJECT_CONFLICT");
    assert_http_contract(port, &deferred, 202, "deferred", "LB_IDENTITY_DEFERRED");

    server.kill().ok();
    server.wait().ok();
    fs::remove_dir_all(state_dir).ok();
}

fn run_cli_publish(state_dir: &Path, fixture: &Path, envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(BINARY);
    command
        .arg("publish")
        .arg(fixture)
        .env("LINGONBERRY_STATE_DIR", state_dir)
        .env_remove("LINGONBERRY_REQUIRE_IDENTITY_CLAIM")
        .env_remove("LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY");
    for (name, value) in envs {
        command.env(name, value);
    }
    command.output().expect("run CLI publish")
}

fn spawn_http_server(state_dir: &Path, port: u16, defer_unsupported: bool) -> Child {
    let mut command = Command::new(BINARY);
    command
        .arg("serve-http")
        .arg(format!("127.0.0.1:{port}"))
        .env("LINGONBERRY_STATE_DIR", state_dir)
        .env_remove("LINGONBERRY_REQUIRE_IDENTITY_CLAIM")
        .env_remove("LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if defer_unsupported {
        command.env("LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY", "defer");
    }
    command.spawn().expect("spawn HTTP server")
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

fn assert_http_contract(port: u16, body: &str, status: u16, state: &str, code: &str) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect to HTTP server");
    let request = format!(
        "POST /v1/objects HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read response");
    assert!(response.starts_with(&format!("HTTP/1.1 {status} ")), "{response}");
    assert!(response.contains(&format!("\"status\":\"{state}\"")), "{response}");
    assert!(response.contains(&format!("\"code\":\"{code}\"")), "{response}");
}

fn assert_success_contract(output: &Output, status: &str, code: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(stdout.contains(status), "{stdout}");
    assert!(stdout.contains(&format!("\"code\":\"{code}\"")), "{stdout}");
}

fn assert_failure_contract(output: &Output, status: &str, code: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(stdout.contains(status), "{stdout}");
    assert!(stdout.contains(&format!("\"code\":\"{code}\"")), "{stdout}");
    assert!(stderr.starts_with(code), "{stderr}");
}

fn write_conflict_fixture(source: &Path, destination: &Path) {
    let content = fs::read_to_string(source).expect("read source fixture").replace(
        "What evidence supports this claim?",
        "What contradictory evidence supports this claim?",
    );
    fs::write(destination, content).expect("write conflict fixture");
}

fn write_deferred_fixture(source: &Path, destination: &Path) {
    let content = fs::read_to_string(source)
        .expect("read identity fixture")
        .replace("lb.identity.key.v1", "lb.identity.key.v99");
    fs::write(destination, content).expect("write deferred fixture");
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
        "lingonberry-publish-contract-{label}-{}-{nonce}",
        std::process::id()
    ))
}
