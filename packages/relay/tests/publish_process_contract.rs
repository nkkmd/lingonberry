use lingonberry_protocol::{parse_json, to_canonical_json, JsonValue};
use std::collections::BTreeMap;
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
    let conflict = workspace.join("fixtures/http-publish-request/with-identity-claim.json");
    let deferred = state_dir.join("deferred.json");
    fs::create_dir_all(&state_dir).expect("create CLI state directory");
    fs::write(
        &deferred,
        signed_unsupported_request(
            &workspace.join("conformance/identity-claims/unsupported-rule.json"),
            &state_dir,
        ),
    )
    .expect("write deferred fixture");

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
fn cli_query_contract_covers_success_and_empty() {
    let workspace = workspace_root();
    let state_dir = unique_temp_dir("query");
    fs::create_dir_all(&state_dir).expect("create query state directory");
    let minimal = workspace.join("fixtures/http-publish-request/minimal-request.json");

    let empty = run_cli_query(&state_dir, Some("question"));
    assert_success_contract(&empty, "\"status\":\"empty\"", "LB_QUERY_EMPTY");
    let empty_stdout = String::from_utf8_lossy(&empty.stdout);
    assert!(
        empty_stdout.contains("\"contractVersion\":\"1\""),
        "{empty_stdout}"
    );
    assert!(
        empty_stdout.contains("\"ordering\":\"canonicalId-ascending\""),
        "{empty_stdout}"
    );

    let published = run_cli_publish(&state_dir, &minimal, &[]);
    assert_success_contract(&published, "\"status\":\"stored\"", "LB_OBJECT_STORED");

    let success = run_cli_query(&state_dir, None);
    assert_success_contract(&success, "\"status\":\"success\"", "LB_QUERY_SUCCESS");
    let success_stdout = String::from_utf8_lossy(&success.stdout);
    assert!(
        success_stdout.contains("\"contractVersion\":\"1\""),
        "{success_stdout}"
    );
    assert!(success_stdout.contains("\"count\":1"), "{success_stdout}");
    assert!(
        success_stdout.contains("\"ordering\":\"canonicalId-ascending\""),
        "{success_stdout}"
    );

    fs::remove_dir_all(state_dir).ok();
}

#[test]
fn http_publish_contract_covers_terminal_states() {
    let workspace = workspace_root();
    let state_dir = unique_temp_dir("http");
    fs::create_dir_all(&state_dir).expect("create HTTP state directory");
    let minimal =
        fs::read_to_string(workspace.join("fixtures/http-publish-request/minimal-request.json"))
            .expect("read minimal fixture");
    let rejected = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/invalid-schema-version.json"),
    )
    .expect("read rejected fixture");
    let conflict = fs::read_to_string(
        workspace.join("fixtures/http-publish-request/with-identity-claim.json"),
    )
    .expect("read conflict fixture");
    let deferred = signed_unsupported_request(
        &workspace.join("conformance/identity-claims/unsupported-rule.json"),
        &state_dir,
    );

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

#[test]
fn http_get_contract_covers_found_and_not_found() {
    let workspace = workspace_root();
    let state_dir = unique_temp_dir("http-get");
    fs::create_dir_all(&state_dir).expect("create HTTP GET state directory");
    let minimal =
        fs::read_to_string(workspace.join("fixtures/http-publish-request/minimal-request.json"))
            .expect("read minimal fixture");

    let port = available_port();
    let mut server = spawn_http_server(&state_dir, port, false);
    wait_until_ready(port);

    let publish_response = http_request(port, "POST /v1/objects", Some(&minimal));
    assert!(
        publish_response.starts_with("HTTP/1.1 201 "),
        "{publish_response}"
    );
    let publish_body = publish_response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .expect("publish response body");
    let publish_json = parse_json(publish_body).expect("parse publish response");
    let JsonValue::Object(publish_map) = publish_json else {
        panic!("publish response must be an object");
    };
    let canonical_id = match publish_map.get("canonicalId") {
        Some(JsonValue::String(value)) => value.clone(),
        other => panic!("missing canonicalId: {other:?}"),
    };

    let found = http_request(port, &format!("GET /v1/objects/{canonical_id}"), None);
    assert!(found.starts_with("HTTP/1.1 200 "), "{found}");
    assert!(found.contains("\"contractVersion\":\"1\""), "{found}");
    assert!(found.contains("\"status\":\"found\""), "{found}");
    assert!(found.contains("\"code\":\"LB_OBJECT_FOUND\""), "{found}");
    assert!(
        found.contains(&format!("\"canonicalId\":\"{canonical_id}\"")),
        "{found}"
    );

    let missing = http_request(port, "GET /v1/objects/lb_missing", None);
    assert!(missing.starts_with("HTTP/1.1 404 "), "{missing}");
    assert!(missing.contains("\"contractVersion\":\"1\""), "{missing}");
    assert!(missing.contains("\"status\":\"not-found\""), "{missing}");
    assert!(
        missing.contains("\"code\":\"LB_OBJECT_NOT_FOUND\""),
        "{missing}"
    );

    server.kill().ok();
    server.wait().ok();
    fs::remove_dir_all(state_dir).ok();
}

fn signed_unsupported_request(object_path: &Path, state_dir: &Path) -> String {
    let object = parse_json(&fs::read_to_string(object_path).expect("read unsupported object"))
        .expect("parse unsupported object");
    let private_key = state_dir.join("publisher-private.pem");
    let public_der = state_dir.join("publisher-public.der");
    let payload_path = state_dir.join("publisher-payload.json");
    let signature_path = state_dir.join("publisher-signature.bin");

    assert!(Command::new("openssl")
        .args(["genpkey", "-algorithm", "ED25519", "-out"])
        .arg(&private_key)
        .status()
        .expect("generate Ed25519 key")
        .success());
    assert!(Command::new("openssl")
        .args(["pkey", "-in"])
        .arg(&private_key)
        .args(["-pubout", "-outform", "DER", "-out"])
        .arg(&public_der)
        .status()
        .expect("export Ed25519 public key")
        .success());
    let public_der_bytes = fs::read(&public_der).expect("read public key DER");
    let public_key = hex(&public_der_bytes[public_der_bytes.len() - 32..]);

    let mut publisher = BTreeMap::new();
    publisher.insert(
        "publicKey".to_string(),
        JsonValue::String(public_key.clone()),
    );
    let mut unsigned = BTreeMap::new();
    unsigned.insert("object".to_string(), object.clone());
    unsigned.insert(
        "publisher".to_string(),
        JsonValue::Object(publisher.clone()),
    );
    let payload = to_canonical_json(&JsonValue::Object(unsigned));
    fs::write(&payload_path, payload).expect("write canonical publish payload");

    assert!(Command::new("openssl")
        .args(["pkeyutl", "-sign", "-inkey"])
        .arg(&private_key)
        .args(["-rawin", "-in"])
        .arg(&payload_path)
        .args(["-out"])
        .arg(&signature_path)
        .status()
        .expect("sign publish payload")
        .success());
    publisher.insert(
        "signature".to_string(),
        JsonValue::String(hex(&fs::read(signature_path).expect("read signature"))),
    );
    let mut request = BTreeMap::new();
    request.insert("object".to_string(), object);
    request.insert("publisher".to_string(), JsonValue::Object(publisher));
    to_canonical_json(&JsonValue::Object(request))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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

fn run_cli_query(state_dir: &Path, object_type: Option<&str>) -> Output {
    let mut command = Command::new(BINARY);
    command
        .arg("subscribe")
        .env("LINGONBERRY_STATE_DIR", state_dir);
    if let Some(object_type) = object_type {
        command.arg(object_type);
    }
    command.output().expect("run CLI query")
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

fn http_request(port: u16, request_target: &str, body: Option<&str>) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect to HTTP server");
    let body = body.unwrap_or("");
    let request = format!(
        "{request_target} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(request.as_bytes())
        .expect("write HTTP request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read HTTP response");
    response
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
    assert!(
        response.starts_with(&format!("HTTP/1.1 {status} ")),
        "{response}"
    );
    assert!(
        response.contains(&format!("\"status\":\"{state}\"")),
        "{response}"
    );
    assert!(
        response.contains(&format!("\"code\":\"{code}\"")),
        "{response}"
    );
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
