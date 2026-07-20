mod existing_v0_5 {
    include!("main_v0_5.rs");

    pub fn run_main() {
        main();
    }
}

use lingonberry_core::{build_runtime_storage_backend, runtime_state_dir, StorageBackend};
use lingonberry_protocol::{to_canonical_json, JsonValue};
use lingonberry_relay::ingest_transition_request;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("serve-http-v0.6") => {
            let addr = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:8787");
            if let Err(error) = serve_http_v0_6(addr) {
                eprintln!("{error}");
                std::process::exit(70);
            }
        }
        _ => existing_v0_5::run_main(),
    }
}

fn serve_http_v0_6(addr: &str) -> Result<(), String> {
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
    if method == "POST" && path == "/v1/transitions" {
        let response = ingest_transition_request(&body, backend, &runtime_state_dir());
        return write_response(
            &mut stream,
            response.status_code,
            response.status_text,
            &response.body,
        )
        .map_err(|error| error.to_string());
    }
    write_response(
        &mut stream,
        404,
        "Not Found",
        &JsonValue::Object(BTreeMap::from([
            ("status".to_string(), JsonValue::String("error".to_string())),
            (
                "code".to_string(),
                JsonValue::String("LB_ROUTE_NOT_FOUND".to_string()),
            ),
            (
                "message".to_string(),
                JsonValue::String(
                    "serve-http-v0.6 currently exposes the dedicated transition route"
                        .to_string(),
                ),
            ),
        ])),
    )
    .map_err(|error| error.to_string())
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
