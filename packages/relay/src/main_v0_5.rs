mod existing {
    include!("main_entry.rs");

    pub fn run_main() {
        main();
    }
}

use lingonberry_core::{
    build_runtime_storage_backend, ingest_publish_request, publish_ingestion_result_json,
    runtime_state_dir, QuarantineStore, StorageBackend,
};
use lingonberry_protocol::{
    build_capability_manifest, derive_identity_key, to_canonical_json, JsonValue,
    CARRIER_KIND_HTTP, DEFAULT_ACCESS_SCOPE, DEFAULT_RETENTION_HINT,
};
use lingonberry_relay::{ingestion_cli_error, ingestion_http_response};
use lingonberry_validation::AcceptancePolicy;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("publish") => handle_publish(&args),
        Some("serve-http") => {
            let addr = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:8787");
            handle_serve_http(addr)
        }
        _ => {
            existing::run_main();
            return;
        }
    };

    if let Err(error) = result {
        eprintln!("{error}");
        process::exit(exit_code_for_ingestion_error(&error));
    }
}

fn handle_publish(args: &[String]) -> Result<(), String> {
    let pathname = args
        .get(1)
        .ok_or_else(|| "usage: lingonberry publish <json-file>".to_string())?;
    let request_json = fs::read_to_string(pathname)
        .map_err(|error| format!("failed to read {pathname}: {error}"))?;
    let backend = build_runtime_storage_backend();
    let quarantine = QuarantineStore::new(runtime_state_dir());
    let policy = AcceptancePolicy::from_env()?;
    let result = ingest_publish_request(&request_json, &backend, &quarantine, &policy);
    println!(
        "{}",
        to_canonical_json(&publish_ingestion_result_json(&result))
    );
    match ingestion_cli_error(&result) {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

fn handle_serve_http(addr: &str) -> Result<(), String> {
    let backend = build_runtime_storage_backend();
    let listener =
        TcpListener::bind(addr).map_err(|error| format!("failed to bind {addr}: {error}"))?;
    eprintln!("public relay listening on http://{addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_http_connection(stream, &backend) {
                    eprintln!("{error}");
                }
            }
            Err(error) => eprintln!("accept error: {error}"),
        }
    }
    Ok(())
}

fn handle_http_connection(
    mut stream: TcpStream,
    backend: &impl StorageBackend,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|error| error.to_string())?);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|error| error.to_string())?;
    if request_line.trim().is_empty() {
        return Ok(());
    }
    let (method, path) = parse_http_request_line(&request_line)?;
    let headers = read_http_headers(&mut reader)?;
    let body = read_http_body(&mut reader, &headers)?;
    let (status_code, status_text, response_body) = route_http(&method, &path, &body, backend)?;
    write_http_response(&mut stream, status_code, status_text, &response_body)
        .map_err(|error| error.to_string())
}

fn route_http(
    method: &str,
    path: &str,
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    match (method, path) {
        ("GET", "/v1/ready") => Ok((
            200,
            "OK",
            json_object(vec![
                ("status", JsonValue::String("ok".to_string())),
                ("service", JsonValue::String("relay".to_string())),
            ]),
        )),
        ("GET", "/v1/capabilities") => Ok((
            200,
            "OK",
            build_capability_manifest(
                CARRIER_KIND_HTTP,
                DEFAULT_ACCESS_SCOPE,
                DEFAULT_RETENTION_HINT,
            ),
        )),
        ("POST", "/v1/objects") => handle_http_publish(body, backend),
        ("GET", path) if path.starts_with("/v1/objects/") => {
            handle_http_get(path.trim_start_matches("/v1/objects/"), backend)
        }
        _ => Ok((
            404,
            "Not Found",
            http_error("LB_ROUTE_NOT_FOUND", "route not found"),
        )),
    }
}

fn handle_http_publish(
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    let quarantine = QuarantineStore::new(runtime_state_dir());
    let policy = AcceptancePolicy::from_env()?;
    let result = ingest_publish_request(body, backend, &quarantine, &policy);
    let response = ingestion_http_response(&result);
    Ok((response.status_code, response.status_text, response.body))
}

fn handle_http_get(
    canonical_id: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    if canonical_id.trim().is_empty() {
        return Ok((
            400,
            "Bad Request",
            http_error("LB_CANONICAL_ID_REQUIRED", "missing canonical id"),
        ));
    }
    match backend
        .get(canonical_id)
        .map_err(|error| error.to_string())?
    {
        Some(record) => Ok((
            200,
            "OK",
            json_object(vec![
                ("status", JsonValue::String("ok".to_string())),
                ("id", JsonValue::String(record.canonical_id)),
                (
                    "identityKey",
                    JsonValue::String(derive_identity_key(&record.object)),
                ),
                ("storedAt", JsonValue::String(record.stored_at)),
                (
                    "carrierIdentity",
                    JsonValue::String(record.carrier_identity),
                ),
                ("canonical", record.object),
            ]),
        )),
        None => Ok((
            404,
            "Not Found",
            http_error("LB_OBJECT_NOT_FOUND", "object not found"),
        )),
    }
}

fn parse_http_request_line(line: &str) -> Result<(String, String), String> {
    let mut parts = line.trim_end_matches(['\r', '\n']).split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "invalid HTTP request line".to_string())?;
    let path = parts
        .next()
        .ok_or_else(|| "invalid HTTP request line".to_string())?;
    let version = parts
        .next()
        .ok_or_else(|| "invalid HTTP request line".to_string())?;
    if !version.starts_with("HTTP/") || parts.next().is_some() {
        return Err("invalid HTTP request line".to_string());
    }
    Ok((method.to_string(), path.to_string()))
}

fn read_http_headers(
    reader: &mut BufReader<TcpStream>,
) -> Result<BTreeMap<String, String>, String> {
    let mut headers = BTreeMap::new();
    loop {
        let mut line = String::new();
        let bytes = reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        if bytes == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    Ok(headers)
}

fn read_http_body(
    reader: &mut BufReader<TcpStream>,
    headers: &BTreeMap<String, String>,
) -> Result<String, String> {
    let Some(content_length) = headers.get("content-length") else {
        return Ok(String::new());
    };
    let length = content_length
        .parse::<usize>()
        .map_err(|_| "invalid content-length".to_string())?;
    let mut buffer = vec![0_u8; length];
    reader
        .read_exact(&mut buffer)
        .map_err(|error| error.to_string())?;
    String::from_utf8(buffer).map_err(|error| error.to_string())
}

fn write_http_response(
    stream: &mut TcpStream,
    status_code: u16,
    status_text: &str,
    body: &JsonValue,
) -> std::io::Result<()> {
    let body_json = to_canonical_json(body);
    write!(
        stream,
        "HTTP/1.1 {status_code} {status_text}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_json}",
        body_json.len()
    )
}

fn http_error(code: &str, message: &str) -> JsonValue {
    json_object(vec![
        ("status", JsonValue::String("error".to_string())),
        ("code", JsonValue::String(code.to_string())),
        ("message", JsonValue::String(message.to_string())),
    ])
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn exit_code_for_ingestion_error(error: &str) -> i32 {
    if error.starts_with("usage:") {
        64
    } else if error.starts_with("LB_INVALID_JSON")
        || error.starts_with("LB_EMPTY_REQUEST")
        || error.starts_with("LB_VALIDATION_FAILED")
        || error.starts_with("LB_IDENTITY_CLAIM_REQUIRED")
    {
        65
    } else {
        70
    }
}
