mod existing_v0_5 {
    include!("main_v0_5.rs");

    pub fn run_main() {
        main();
    }
}

use lingonberry_core::{
    build_runtime_storage_backend, ingest_publish_request, retrieve_object, runtime_state_dir,
    QuarantineStore, StorageBackend,
};
use lingonberry_protocol::{
    build_capability_manifest, to_canonical_json, JsonValue, CARRIER_KIND_HTTP,
    DEFAULT_ACCESS_SCOPE, DEFAULT_RETENTION_HINT,
};
use lingonberry_relay::{
    ingest_transition_request, ingestion_http_response, retrieval_http_response,
};
use lingonberry_validation::AcceptancePolicy;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("serve-http") => {
            let addr = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:8787");
            if let Err(error) = serve_http(addr) {
                eprintln!("{error}");
                std::process::exit(70);
            }
        }
        _ => existing_v0_5::run_main(),
    }
}

fn serve_http(addr: &str) -> Result<(), String> {
    let backend = build_runtime_storage_backend();
    let listener = TcpListener::bind(addr)
        .map_err(|error| format!("failed to bind {addr}: {error}"))?;
    eprintln!("public relay v0.6 listening on http://{addr}");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(error) = handle_connection(stream, &backend) {
                    eprintln!("{error}");
                }
            }
            Err(error) => eprintln!("accept error: {error}"),
        }
    }
    Ok(())
}

fn handle_connection(
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
    let (method, path) = parse_request_line(&request_line)?;
    let headers = read_headers(&mut reader)?;
    let body = read_body(&mut reader, &headers)?;
    let (status_code, status_text, response_body) = route(&method, &path, &body, backend)?;
    write_response(&mut stream, status_code, status_text, &response_body)
        .map_err(|error| error.to_string())
}

fn route(
    method: &str,
    path: &str,
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    match (method, path) {
        ("GET", "/v1/ready") => Ok((
            200,
            "OK",
            object(vec![
                ("status", JsonValue::String("ok".to_string())),
                ("service", JsonValue::String("relay".to_string())),
                ("version", JsonValue::String("0.6.0".to_string())),
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
        ("POST", "/v1/objects") => publish_object(body, backend),
        ("POST", "/v1/transitions") => publish_transition(body, backend),
        ("GET", path) if path.starts_with("/v1/objects/") => {
            get_object(path.trim_start_matches("/v1/objects/"), backend)
        }
        _ => Ok((
            404,
            "Not Found",
            error_body("LB_ROUTE_NOT_FOUND", "route not found"),
        )),
    }
}

fn publish_object(
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    let quarantine = QuarantineStore::new(runtime_state_dir());
    let policy = AcceptancePolicy::from_env()?;
    let result = ingest_publish_request(body, backend, &quarantine, &policy);
    let response = ingestion_http_response(&result);
    Ok((response.status_code, response.status_text, response.body))
}

fn publish_transition(
    body: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    let response = ingest_transition_request(body, backend, &runtime_state_dir());
    Ok((response.status_code, response.status_text, response.body))
}

fn get_object(
    canonical_id: &str,
    backend: &impl StorageBackend,
) -> Result<(u16, &'static str, JsonValue), String> {
    let result = retrieve_object(canonical_id, backend);
    let response = retrieval_http_response(&result);
    Ok((response.status_code, response.status_text, response.body))
}

fn parse_request_line(line: &str) -> Result<(String, String), String> {
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

fn read_headers(
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

fn read_body(
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

fn write_response(
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

fn error_body(code: &str, message: &str) -> JsonValue {
    object(vec![
        ("status", JsonValue::String("error".to_string())),
        ("code", JsonValue::String(code.to_string())),
        ("message", JsonValue::String(message.to_string())),
    ])
}

fn object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}
