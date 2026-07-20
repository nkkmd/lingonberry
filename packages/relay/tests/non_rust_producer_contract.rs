use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BINARY: &str = env!("CARGO_BIN_EXE_lingonberry-relay");
const OBJECT_ID: &str = "lb:obj:js-producer-http-contract";
const CREATED_AT: &str = "2026-07-20T00:00:00Z";

#[test]
fn javascript_producer_publishes_through_real_http_path() {
    let workspace = workspace_root();
    let state_dir = unique_temp_dir("js-producer");
    fs::create_dir_all(&state_dir).expect("create state directory");

    let produced = Command::new("node")
        .arg(workspace.join("conformance/minimal-producer.mjs"))
        .args(["--id", OBJECT_ID, "--created-at", CREATED_AT])
        .output()
        .expect("run JavaScript producer");
    assert!(
        produced.status.success(),
        "producer stderr={}",
        String::from_utf8_lossy(&produced.stderr),
    );
    let request = String::from_utf8(produced.stdout).expect("producer output must be UTF-8");

    let port = available_port();
    let mut server = spawn_http_server(&state_dir, port);
    wait_until_ready(port);

    let response = http_post(port, request.trim_end());
    assert!(response.starts_with("HTTP/1.1 201 "), "{response}");
    assert!(response.contains("\"status\":\"stored\""), "{response}");
    assert!(response.contains("\"code\":\"LB_OBJECT_STORED\""), "{response}");
    let expected_id = format!("\"canonicalId\":\"{OBJECT_ID}\"");
    assert!(response.contains(&expected_id), "{response}");

    server.kill().ok();
    server.wait().ok();
    fs::remove_dir_all(state_dir).ok();
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

fn http_post(port: u16, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect to HTTP server");
    let request = format!(
        "POST /v1/objects HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body,
    );
    stream.write_all(request.as_bytes()).expect("write request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read response");
    response
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
    let process_id = std::process::id();
    let name = format!("lingonberry-non-rust-producer-{label}-{process_id}-{nonce}");
    std::env::temp_dir().join(name)
}
