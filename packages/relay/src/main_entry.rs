mod legacy {
    include!("main.rs");

    pub(crate) fn run_existing(args: Vec<String>) -> Result<(), String> {
        run(args)
    }

    pub(crate) fn exit_code(error: &str) -> i32 {
        exit_code_for_error(error)
    }

    pub(crate) fn serve_http_with_quarantine_status(
        addr: &str,
        backend: &impl StorageBackend,
    ) -> Result<(), String> {
        let listener = TcpListener::bind(addr)
            .map_err(|error| format!("failed to bind {}: {}", addr, error))?;
        eprintln!("listening on http://{}", addr);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(error) = handle_http_connection_with_quarantine_status(stream, backend)
                    {
                        eprintln!("{}", error);
                    }
                }
                Err(error) => eprintln!("accept error: {}", error),
            }
        }
        Ok(())
    }

    fn handle_http_connection_with_quarantine_status(
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
        let (method, path, _version) = parse_http_request_line(&request_line)?;
        let headers = read_http_headers(&mut reader)?;
        let body = read_http_body(&mut reader, &headers)?;

        if is_quarantine_metrics_route(&method, &path) {
            let metrics = QuarantineStore::new(runtime_state_dir())
                .metrics_text()
                .map_err(|error| error.to_string())?;
            return write_metrics_response(&mut stream, &metrics).map_err(|error| error.to_string());
        }

        let (status_code, status_text, response_body) = if let Some(quarantine_id) =
            annotation_route_id(&method, &path)
        {
            handle_http_quarantine_annotations(&method, quarantine_id, &body)?
        } else if is_quarantine_status_route(&method, &path) {
            let status = QuarantineStore::new(runtime_state_dir())
                .status_json()
                .map_err(|error| error.to_string())?;
            (200, "OK", status)
        } else {
            route_http_request(&method, &path, &body, backend)?
        };
        write_http_response(&mut stream, status_code, status_text, &response_body)
            .map_err(|error| error.to_string())
    }

    fn handle_http_quarantine_annotations(
        method: &str,
        quarantine_id: &str,
        body: &str,
    ) -> Result<(u16, &'static str, JsonValue), String> {
        let store = QuarantineStore::new(runtime_state_dir());
        match method {
            "GET" => {
                if store
                    .get(quarantine_id)
                    .map_err(|error| error.to_string())?
                    .is_none()
                {
                    return Ok((
                        404,
                        "Not Found",
                        http_error("not_found", "quarantine record not found"),
                    ));
                }
                let annotations = store
                    .list_annotations(Some(quarantine_id))
                    .map_err(|error| error.to_string())?;
                Ok((
                    200,
                    "OK",
                    json_object(vec![
                        (
                            "count",
                            JsonValue::Number(annotations.len().to_string()),
                        ),
                        (
                            "annotations",
                            JsonValue::Array(
                                annotations
                                    .iter()
                                    .map(lingonberry_core::quarantine_annotation_json)
                                    .collect(),
                            ),
                        ),
                    ]),
                ))
            }
            "POST" => {
                let value = lingonberry_protocol::parse_json(body)
                    .map_err(|error| error.to_string())?;
                let map = as_object(&value)
                    .ok_or_else(|| "annotation request must be an object".to_string())?;
                let operator = map
                    .get("operator")
                    .and_then(as_string)
                    .ok_or_else(|| "annotation request missing operator".to_string())?;
                let note = map
                    .get("note")
                    .and_then(as_string)
                    .ok_or_else(|| "annotation request missing note".to_string())?;
                match store.append_annotation(quarantine_id, operator, note) {
                    Ok(annotation) => Ok((
                        201,
                        "Created",
                        lingonberry_core::quarantine_annotation_json(&annotation),
                    )),
                    Err(error) if error.code == "LB_QUARANTINE_NOT_FOUND" => Ok((
                        404,
                        "Not Found",
                        http_error("not_found", &error.message),
                    )),
                    Err(error) if error.code == "LB_QUARANTINE_ANNOTATION" => Ok((
                        400,
                        "Bad Request",
                        http_error("validation_error", &error.message),
                    )),
                    Err(error) => Err(error.to_string()),
                }
            }
            _ => Ok((
                405,
                "Method Not Allowed",
                http_error("method_not_allowed", "method not allowed"),
            )),
        }
    }

    fn write_metrics_response(stream: &mut TcpStream, body: &str) -> std::io::Result<()> {
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.as_bytes().len(),
            body
        )
    }

    pub(crate) fn is_quarantine_status_route(method: &str, path: &str) -> bool {
        method == "GET" && path == "/v1/quarantine-status"
    }

    pub(crate) fn is_quarantine_metrics_route(method: &str, path: &str) -> bool {
        method == "GET" && path == "/metrics"
    }

    pub(crate) fn annotation_route_id<'a>(method: &str, path: &'a str) -> Option<&'a str> {
        if method != "GET" && method != "POST" {
            return None;
        }
        let prefix = "/v1/quarantine/";
        let suffix = "/annotations";
        if !path.starts_with(prefix) || !path.ends_with(suffix) {
            return None;
        }
        let id = &path[prefix.len()..path.len() - suffix.len()];
        (!id.is_empty() && !id.contains('/')).then_some(id)
    }
}

use lingonberry_core::{
    build_runtime_storage_backend, quarantine_annotation_json, quarantine_dismissal_json,
    runtime_state_dir, QuarantineStore, OPERATOR_DISMISSED_REASON_CODE,
};
use lingonberry_protocol::{to_canonical_json, JsonValue};
use std::collections::BTreeMap;
use std::env;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("quarantine-status") => handle_quarantine_status(),
        Some("quarantine-metrics") => handle_quarantine_metrics(),
        Some("quarantine-annotate") => handle_quarantine_annotate(&args),
        Some("quarantine-annotations") => handle_quarantine_annotations(&args),
        Some("quarantine-dismiss") => handle_quarantine_dismiss(&args),
        Some("quarantine-dismissals") => handle_quarantine_dismissals(&args),
        Some("serve-http") => {
            let backend = build_runtime_storage_backend();
            let addr = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:8787");
            legacy::serve_http_with_quarantine_status(addr, &backend)
        }
        _ => legacy::run_existing(args),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        process::exit(legacy::exit_code(&error));
    }
}

fn handle_quarantine_status() -> Result<(), String> {
    let status = QuarantineStore::new(runtime_state_dir())
        .status_json()
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&status));
    Ok(())
}

fn handle_quarantine_metrics() -> Result<(), String> {
    let metrics = QuarantineStore::new(runtime_state_dir())
        .metrics_text()
        .map_err(|error| error.to_string())?;
    print!("{}", metrics);
    Ok(())
}

fn handle_quarantine_annotate(args: &[String]) -> Result<(), String> {
    let quarantine_id = args.get(1).ok_or_else(|| {
        "usage: lingonberry quarantine-annotate <quarantine-id> <operator> <note>".to_string()
    })?;
    let operator = args.get(2).ok_or_else(|| {
        "usage: lingonberry quarantine-annotate <quarantine-id> <operator> <note>".to_string()
    })?;
    let note = args.get(3).ok_or_else(|| {
        "usage: lingonberry quarantine-annotate <quarantine-id> <operator> <note>".to_string()
    })?;
    let annotation = QuarantineStore::new(runtime_state_dir())
        .append_annotation(quarantine_id, operator, note)
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&quarantine_annotation_json(&annotation)));
    Ok(())
}

fn handle_quarantine_annotations(args: &[String]) -> Result<(), String> {
    let quarantine_id = args.get(1).map(String::as_str);
    let annotations = QuarantineStore::new(runtime_state_dir())
        .list_annotations(quarantine_id)
        .map_err(|error| error.to_string())?;
    let output = JsonValue::Object(BTreeMap::from([
        (
            "count".to_string(),
            JsonValue::Number(annotations.len().to_string()),
        ),
        (
            "annotations".to_string(),
            JsonValue::Array(annotations.iter().map(quarantine_annotation_json).collect()),
        ),
    ]));
    println!("{}", to_canonical_json(&output));
    Ok(())
}

fn handle_quarantine_dismiss(args: &[String]) -> Result<(), String> {
    let quarantine_id = args.get(1).ok_or_else(|| {
        "usage: lingonberry quarantine-dismiss <quarantine-id> <operator> <note>".to_string()
    })?;
    let operator = args.get(2).ok_or_else(|| {
        "usage: lingonberry quarantine-dismiss <quarantine-id> <operator> <note>".to_string()
    })?;
    let note = args.get(3).ok_or_else(|| {
        "usage: lingonberry quarantine-dismiss <quarantine-id> <operator> <note>".to_string()
    })?;
    let dismissal = QuarantineStore::new(runtime_state_dir())
        .dismiss(
            quarantine_id,
            operator,
            OPERATOR_DISMISSED_REASON_CODE,
            note,
        )
        .map_err(|error| error.to_string())?;
    println!("{}", to_canonical_json(&quarantine_dismissal_json(&dismissal)));
    Ok(())
}

fn handle_quarantine_dismissals(args: &[String]) -> Result<(), String> {
    let quarantine_id = args.get(1).map(String::as_str);
    let dismissals = QuarantineStore::new(runtime_state_dir())
        .list_dismissals(quarantine_id)
        .map_err(|error| error.to_string())?;
    let output = JsonValue::Object(BTreeMap::from([
        (
            "count".to_string(),
            JsonValue::Number(dismissals.len().to_string()),
        ),
        (
            "dismissals".to_string(),
            JsonValue::Array(dismissals.iter().map(quarantine_dismissal_json).collect()),
        ),
    ]));
    println!("{}", to_canonical_json(&output));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::legacy::{
        annotation_route_id, is_quarantine_metrics_route, is_quarantine_status_route,
    };

    #[test]
    fn quarantine_status_http_route_is_exact() {
        assert!(is_quarantine_status_route("GET", "/v1/quarantine-status"));
        assert!(!is_quarantine_status_route("POST", "/v1/quarantine-status"));
        assert!(!is_quarantine_status_route("GET", "/v1/quarantine-status/"));
        assert!(!is_quarantine_status_route("GET", "/v1/quarantine"));
    }

    #[test]
    fn quarantine_metrics_http_route_is_exact() {
        assert!(is_quarantine_metrics_route("GET", "/metrics"));
        assert!(!is_quarantine_metrics_route("POST", "/metrics"));
        assert!(!is_quarantine_metrics_route("GET", "/metrics/"));
        assert!(!is_quarantine_metrics_route("GET", "/v1/metrics"));
    }

    #[test]
    fn quarantine_annotation_route_is_exact() {
        assert_eq!(
            annotation_route_id("GET", "/v1/quarantine/lb:q:123/annotations"),
            Some("lb:q:123")
        );
        assert_eq!(
            annotation_route_id("POST", "/v1/quarantine/lb:q:123/annotations"),
            Some("lb:q:123")
        );
        assert_eq!(
            annotation_route_id("DELETE", "/v1/quarantine/lb:q:123/annotations"),
            None
        );
        assert_eq!(
            annotation_route_id("GET", "/v1/quarantine/lb:q:123/annotations/"),
            None
        );
    }
}