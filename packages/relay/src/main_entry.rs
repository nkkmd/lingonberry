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

        let (status_code, status_text, response_body) = if is_quarantine_status_route(&method, &path)
        {
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
}

use lingonberry_core::{build_runtime_storage_backend, runtime_state_dir, QuarantineStore};
use lingonberry_protocol::to_canonical_json;
use std::env;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("quarantine-status") => handle_quarantine_status(),
        Some("quarantine-metrics") => handle_quarantine_metrics(),
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

#[cfg(test)]
mod tests {
    use super::legacy::{is_quarantine_metrics_route, is_quarantine_status_route};

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
}
