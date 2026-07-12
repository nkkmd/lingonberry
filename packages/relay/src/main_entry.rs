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

    pub(crate) fn is_quarantine_status_route(method: &str, path: &str) -> bool {
        method == "GET" && path == "/v1/quarantine-status"
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

#[cfg(test)]
mod tests {
    use super::legacy::is_quarantine_status_route;

    #[test]
    fn quarantine_status_http_route_is_exact() {
        assert!(is_quarantine_status_route("GET", "/v1/quarantine-status"));
        assert!(!is_quarantine_status_route("POST", "/v1/quarantine-status"));
        assert!(!is_quarantine_status_route("GET", "/v1/quarantine-status/"));
        assert!(!is_quarantine_status_route("GET", "/v1/quarantine"));
    }
}
