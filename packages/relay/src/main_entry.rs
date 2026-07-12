mod admin_auth;

mod legacy {
    include!("main.rs");

    use super::admin_auth::{
        admin_request_allowed, append_admin_auth_failure, append_admin_authorization_failure,
        configured_admin_credentials, resolve_admin_role, AdminCredentials, AdminRole,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum AdminHttpAccess {
        NotFound,
        Unauthorized,
        Forbidden(AdminRole),
        Authorized(AdminRole),
    }

    pub(crate) fn run_existing(args: Vec<String>) -> Result<(), String> {
        run(args)
    }

    pub(crate) fn exit_code(error: &str) -> i32 {
        exit_code_for_error(error)
    }

    pub(crate) fn serve_public_http(
        addr: &str,
        backend: &impl StorageBackend,
    ) -> Result<(), String> {
        let listener = TcpListener::bind(addr)
            .map_err(|error| format!("failed to bind {}: {}", addr, error))?;
        eprintln!("public relay listening on http://{}", addr);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(error) = handle_public_http_connection(stream, backend) {
                        eprintln!("{}", error);
                    }
                }
                Err(error) => eprintln!("accept error: {}", error),
            }
        }
        Ok(())
    }

    pub(crate) fn serve_admin_http(
        addr: &str,
        backend: &impl StorageBackend,
    ) -> Result<(), String> {
        let credentials = configured_admin_credentials()?;
        if credentials.used_legacy_operator_fallback {
            eprintln!("warning: LINGONBERRY_ADMIN_TOKEN is active as the legacy operator fallback");
        }
        let listener = TcpListener::bind(addr)
            .map_err(|error| format!("failed to bind {}: {}", addr, error))?;
        eprintln!("admin API listening on http://{}", addr);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(error) = handle_admin_http_connection(stream, backend, &credentials)
                    {
                        eprintln!("{}", error);
                    }
                }
                Err(error) => eprintln!("accept error: {}", error),
            }
        }
        Ok(())
    }

    fn handle_public_http_connection(
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
        let response = if is_admin_path(&path) {
            (404, "Not Found", http_error("not_found", "route not found"))
        } else {
            route_http_request(&method, &path, &body, backend)?
        };
        write_http_response(&mut stream, response.0, response.1, &response.2)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn classify_admin_http_access(
        method: &str,
        path: &str,
        headers: &BTreeMap<String, String>,
        credentials: &AdminCredentials,
    ) -> AdminHttpAccess {
        if !is_admin_path(path) {
            return AdminHttpAccess::NotFound;
        }
        let Some(role) = resolve_admin_role(headers, credentials) else {
            return AdminHttpAccess::Unauthorized;
        };
        if !admin_request_allowed(role, method, path) {
            return AdminHttpAccess::Forbidden(role);
        }
        AdminHttpAccess::Authorized(role)
    }

    fn handle_admin_http_connection(
        mut stream: TcpStream,
        backend: &impl StorageBackend,
        credentials: &AdminCredentials,
    ) -> Result<(), String> {
        let remote_addr = stream
            .peer_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
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

        match classify_admin_http_access(&method, &path, &headers, credentials) {
            AdminHttpAccess::NotFound => {
                return write_http_response(
                    &mut stream,
                    404,
                    "Not Found",
                    &http_error("not_found", "route not found"),
                )
                .map_err(|error| error.to_string());
            }
            AdminHttpAccess::Unauthorized => {
                append_admin_auth_failure(
                    runtime_state_dir(),
                    &remote_addr,
                    &method,
                    &path,
                    "LB_ADMIN_AUTH_FAILED",
                )?;
                return write_http_response(
                    &mut stream,
                    401,
                    "Unauthorized",
                    &http_error("unauthorized", "admin authentication required"),
                )
                .map_err(|error| error.to_string());
            }
            AdminHttpAccess::Forbidden(role) => {
                append_admin_authorization_failure(
                    runtime_state_dir(),
                    &remote_addr,
                    &method,
                    &path,
                    role,
                )?;
                return write_http_response(
                    &mut stream,
                    403,
                    "Forbidden",
                    &http_error("forbidden", "admin permission denied"),
                )
                .map_err(|error| error.to_string());
            }
            AdminHttpAccess::Authorized(_) => {}
        }

        let body = read_http_body(&mut reader, &headers)?;

        if is_quarantine_metrics_route(&method, &path) {
            let metrics = QuarantineStore::new(runtime_state_dir())
                .metrics_text()
                .map_err(|error| error.to_string())?;
            return write_metrics_response(&mut stream, &metrics)
                .map_err(|error| error.to_string());
        }

        let (status_code, status_text, response_body) =
            if let Some(quarantine_id) = permanent_rejection_route_id(&method, &path) {
                handle_http_permanent_rejection(&method, quarantine_id, &body)?
            } else if let Some(quarantine_id) = annotation_route_id(&method, &path) {
                handle_http_quarantine_annotations(&method, quarantine_id, &body)?
            } else if is_quarantine_status_route(&method, &path) {
                let status = QuarantineStore::new(runtime_state_dir())
                    .status_json()
                    .map_err(|error| error.to_string())?;
                (200, "OK", status)
            } else if let Some(quarantine_id) = promotion_route_id(&method, &path) {
                if QuarantineStore::new(runtime_state_dir())
                    .get_permanent_rejection(quarantine_id)
                    .map_err(|error| error.to_string())?
                    .is_some()
                {
                    (
                        409,
                        "Conflict",
                        http_error(
                            "conflict",
                            "permanently rejected quarantine record cannot be promoted",
                        ),
                    )
                } else {
                    route_http_request(&method, &path, &body, backend)?
                }
            } else {
                route_http_request(&method, &path, &body, backend)?
            };
        write_http_response(&mut stream, status_code, status_text, &response_body)
            .map_err(|error| error.to_string())
    }

    fn handle_http_permanent_rejection(
        method: &str,
        quarantine_id: &str,
        body: &str,
    ) -> Result<(u16, &'static str, JsonValue), String> {
        let store = QuarantineStore::new(runtime_state_dir());
        match method {
            "GET" => match store
                .get_permanent_rejection(quarantine_id)
                .map_err(|error| error.to_string())?
            {
                Some(event) => Ok((
                    200,
                    "OK",
                    lingonberry_core::quarantine_permanent_rejection_json(&event),
                )),
                None => Ok((
                    404,
                    "Not Found",
                    http_error("not_found", "permanent rejection not found"),
                )),
            },
            "POST" => {
                let value =
                    lingonberry_protocol::parse_json(body).map_err(|error| error.to_string())?;
                let map = as_object(&value)
                    .ok_or_else(|| "permanent rejection request must be an object".to_string())?;
                let operator = map
                    .get("operator")
                    .and_then(as_string)
                    .ok_or_else(|| "permanent rejection request missing operator".to_string())?;
                let note = map
                    .get("note")
                    .and_then(as_string)
                    .ok_or_else(|| "permanent rejection request missing note".to_string())?;
                match store.permanently_reject(
                    quarantine_id,
                    operator,
                    lingonberry_core::OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
                    note,
                ) {
                    Ok(event) => Ok((
                        201,
                        "Created",
                        lingonberry_core::quarantine_permanent_rejection_json(&event),
                    )),
                    Err(error) if error.code == "LB_QUARANTINE_NOT_FOUND" => {
                        Ok((404, "Not Found", http_error("not_found", &error.message)))
                    }
                    Err(error)
                        if error.code == "LB_QUARANTINE_ALREADY_PROMOTED"
                            || error.code == "LB_QUARANTINE_ALREADY_DISMISSED" =>
                    {
                        Ok((409, "Conflict", http_error("conflict", &error.message)))
                    }
                    Err(error) if error.code == "LB_QUARANTINE_PERMANENT_REJECTION" => Ok((
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
                        ("count", JsonValue::Number(annotations.len().to_string())),
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
                let value =
                    lingonberry_protocol::parse_json(body).map_err(|error| error.to_string())?;
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
                    Err(error) if error.code == "LB_QUARANTINE_NOT_FOUND" => {
                        Ok((404, "Not Found", http_error("not_found", &error.message)))
                    }
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
            body.len(),
            body
        )
    }

    pub(crate) fn is_admin_path(path: &str) -> bool {
        path == "/metrics"
            || path == "/v1/quarantine-status"
            || path == "/v1/quarantine"
            || path == "/v1/quarantine-resolutions"
            || path.starts_with("/v1/quarantine/")
    }

    pub(crate) fn is_quarantine_status_route(method: &str, path: &str) -> bool {
        method == "GET" && path == "/v1/quarantine-status"
    }

    pub(crate) fn is_quarantine_metrics_route(method: &str, path: &str) -> bool {
        method == "GET" && path == "/metrics"
    }

    pub(crate) fn annotation_route_id<'a>(method: &str, path: &'a str) -> Option<&'a str> {
        route_id(method, path, "/annotations")
    }

    pub(crate) fn permanent_rejection_route_id<'a>(method: &str, path: &'a str) -> Option<&'a str> {
        route_id(method, path, "/permanent-rejection")
    }

    pub(crate) fn promotion_route_id<'a>(method: &str, path: &'a str) -> Option<&'a str> {
        if method != "POST" || !path.ends_with("/promote") {
            return None;
        }
        let prefix = "/v1/quarantine/";
        if !path.starts_with(prefix) {
            return None;
        }
        let id = &path[prefix.len()..path.len() - "/promote".len()];
        (!id.is_empty() && !id.contains('/')).then_some(id)
    }

    fn route_id<'a>(method: &str, path: &'a str, suffix: &str) -> Option<&'a str> {
        if method != "GET" && method != "POST" {
            return None;
        }
        let prefix = "/v1/quarantine/";
        if !path.starts_with(prefix) || !path.ends_with(suffix) {
            return None;
        }
        let id = &path[prefix.len()..path.len() - suffix.len()];
        (!id.is_empty() && !id.contains('/')).then_some(id)
    }
}

use lingonberry_core::{
    build_runtime_storage_backend, quarantine_annotation_json, quarantine_dismissal_json,
    quarantine_permanent_rejection_json, runtime_state_dir, QuarantineStore,
    OPERATOR_DISMISSED_REASON_CODE, OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
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
        Some("quarantine-permanently-reject") => handle_quarantine_permanently_reject(&args),
        Some("quarantine-permanent-rejections") => handle_quarantine_permanent_rejections(&args),
        Some("quarantine-promote") => guard_then_run_promotion(args),
        Some("serve-http") => {
            let backend = build_runtime_storage_backend();
            let addr = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:8787");
            legacy::serve_public_http(addr, &backend)
        }
        Some("serve-admin-http") => {
            let backend = build_runtime_storage_backend();
            let addr = args.get(1).map(String::as_str).unwrap_or("127.0.0.1:8788");
            legacy::serve_admin_http(addr, &backend)
        }
        _ => legacy::run_existing(args),
    };

    if let Err(error) = result {
        eprintln!("{}", error);
        process::exit(legacy::exit_code(&error));
    }
}

fn guard_then_run_promotion(args: Vec<String>) -> Result<(), String> {
    let quarantine_id = args
        .get(1)
        .ok_or_else(|| "usage: lingonberry quarantine-promote <quarantine-id>".to_string())?;
    if QuarantineStore::new(runtime_state_dir())
        .get_permanent_rejection(quarantine_id)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err(format!(
            "LB_QUARANTINE_PERMANENTLY_REJECTED: permanently rejected quarantine record cannot be promoted: {quarantine_id}"
        ));
    }
    legacy::run_existing(args)
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
    println!(
        "{}",
        to_canonical_json(&quarantine_annotation_json(&annotation))
    );
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
    if QuarantineStore::new(runtime_state_dir())
        .get_permanent_rejection(quarantine_id)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err(format!(
            "LB_QUARANTINE_PERMANENTLY_REJECTED: permanently rejected quarantine record cannot be dismissed: {quarantine_id}"
        ));
    }
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
    println!(
        "{}",
        to_canonical_json(&quarantine_dismissal_json(&dismissal))
    );
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

fn handle_quarantine_permanently_reject(args: &[String]) -> Result<(), String> {
    let quarantine_id = args.get(1).ok_or_else(|| {
        "usage: lingonberry quarantine-permanently-reject <quarantine-id> <operator> <note>"
            .to_string()
    })?;
    let operator = args.get(2).ok_or_else(|| {
        "usage: lingonberry quarantine-permanently-reject <quarantine-id> <operator> <note>"
            .to_string()
    })?;
    let note = args.get(3).ok_or_else(|| {
        "usage: lingonberry quarantine-permanently-reject <quarantine-id> <operator> <note>"
            .to_string()
    })?;
    let event = QuarantineStore::new(runtime_state_dir())
        .permanently_reject(
            quarantine_id,
            operator,
            OPERATOR_PERMANENTLY_REJECTED_REASON_CODE,
            note,
        )
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        to_canonical_json(&quarantine_permanent_rejection_json(&event))
    );
    Ok(())
}

fn handle_quarantine_permanent_rejections(args: &[String]) -> Result<(), String> {
    let quarantine_id = args.get(1).map(String::as_str);
    let events = QuarantineStore::new(runtime_state_dir())
        .list_permanent_rejections(quarantine_id)
        .map_err(|error| error.to_string())?;
    let output = JsonValue::Object(BTreeMap::from([
        (
            "count".to_string(),
            JsonValue::Number(events.len().to_string()),
        ),
        (
            "permanentRejections".to_string(),
            JsonValue::Array(
                events
                    .iter()
                    .map(quarantine_permanent_rejection_json)
                    .collect(),
            ),
        ),
    ]));
    println!("{}", to_canonical_json(&output));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::admin_auth::{
        AdminCredential, AdminCredentials, AdminRole, ADMIN_OBSERVER_TOKEN_ENV,
        ADMIN_OPERATOR_TOKEN_ENV, ADMIN_REVIEWER_TOKEN_ENV,
    };
    use super::legacy::{
        annotation_route_id, classify_admin_http_access, is_admin_path,
        is_quarantine_metrics_route, is_quarantine_status_route, permanent_rejection_route_id,
        promotion_route_id, AdminHttpAccess,
    };
    use std::collections::BTreeMap;

    fn credentials() -> AdminCredentials {
        AdminCredentials {
            credentials: vec![
                AdminCredential {
                    role: AdminRole::Observer,
                    token: "observe".to_string(),
                    source_env: ADMIN_OBSERVER_TOKEN_ENV,
                },
                AdminCredential {
                    role: AdminRole::Reviewer,
                    token: "review".to_string(),
                    source_env: ADMIN_REVIEWER_TOKEN_ENV,
                },
                AdminCredential {
                    role: AdminRole::Operator,
                    token: "operate".to_string(),
                    source_env: ADMIN_OPERATOR_TOKEN_ENV,
                },
            ],
            used_legacy_operator_fallback: false,
        }
    }

    fn headers(token: Option<&str>) -> BTreeMap<String, String> {
        token
            .map(|token| BTreeMap::from([("authorization".to_string(), format!("Bearer {token}"))]))
            .unwrap_or_default()
    }

    #[test]
    fn public_admin_boundary_is_explicit() {
        assert!(is_admin_path("/metrics"));
        assert!(is_admin_path("/v1/quarantine-status"));
        assert!(is_admin_path("/v1/quarantine"));
        assert!(is_admin_path("/v1/quarantine/lb:q:123"));
        assert!(is_admin_path("/v1/quarantine/lb:q:123/permanent-rejection"));
        assert!(!is_admin_path("/v1/ready"));
        assert!(!is_admin_path("/v1/capabilities"));
        assert!(!is_admin_path("/v1/objects"));
    }

    #[test]
    fn admin_http_access_distinguishes_401_and_403() {
        let credentials = credentials();
        assert_eq!(
            classify_admin_http_access(
                "GET",
                "/v1/quarantine-status",
                &headers(None),
                &credentials,
            ),
            AdminHttpAccess::Unauthorized
        );
        assert_eq!(
            classify_admin_http_access(
                "GET",
                "/v1/quarantine-status",
                &headers(Some("invalid")),
                &credentials,
            ),
            AdminHttpAccess::Unauthorized
        );
        assert_eq!(
            classify_admin_http_access(
                "POST",
                "/v1/quarantine/lb:q:1/promote",
                &headers(Some("observe")),
                &credentials,
            ),
            AdminHttpAccess::Forbidden(AdminRole::Observer)
        );
        assert_eq!(
            classify_admin_http_access("GET", "/v1/ready", &headers(Some("operate")), &credentials,),
            AdminHttpAccess::NotFound
        );
    }

    #[test]
    fn role_permissions_are_enforced_at_http_boundary() {
        let credentials = credentials();
        assert_eq!(
            classify_admin_http_access("GET", "/metrics", &headers(Some("observe")), &credentials,),
            AdminHttpAccess::Authorized(AdminRole::Observer)
        );
        assert_eq!(
            classify_admin_http_access(
                "POST",
                "/v1/quarantine/lb:q:1/annotations",
                &headers(Some("review")),
                &credentials,
            ),
            AdminHttpAccess::Authorized(AdminRole::Reviewer)
        );
        assert_eq!(
            classify_admin_http_access(
                "POST",
                "/v1/quarantine/lb:q:1/promote",
                &headers(Some("review")),
                &credentials,
            ),
            AdminHttpAccess::Forbidden(AdminRole::Reviewer)
        );
        assert_eq!(
            classify_admin_http_access(
                "POST",
                "/v1/quarantine/lb:q:1/permanent-rejection",
                &headers(Some("operate")),
                &credentials,
            ),
            AdminHttpAccess::Authorized(AdminRole::Operator)
        );
    }

    #[test]
    fn quarantine_status_and_metrics_routes_are_exact() {
        assert!(is_quarantine_status_route("GET", "/v1/quarantine-status"));
        assert!(!is_quarantine_status_route("POST", "/v1/quarantine-status"));
        assert!(is_quarantine_metrics_route("GET", "/metrics"));
        assert!(!is_quarantine_metrics_route("POST", "/metrics"));
    }

    #[test]
    fn quarantine_subresource_routes_are_exact() {
        assert_eq!(
            annotation_route_id("GET", "/v1/quarantine/lb:q:123/annotations"),
            Some("lb:q:123")
        );
        assert_eq!(
            permanent_rejection_route_id("POST", "/v1/quarantine/lb:q:123/permanent-rejection"),
            Some("lb:q:123")
        );
        assert_eq!(
            promotion_route_id("POST", "/v1/quarantine/lb:q:123/promote"),
            Some("lb:q:123")
        );
        assert_eq!(
            permanent_rejection_route_id("DELETE", "/v1/quarantine/lb:q:123/permanent-rejection"),
            None
        );
    }
}
