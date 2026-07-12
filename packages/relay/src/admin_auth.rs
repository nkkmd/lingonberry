use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::acquire_quarantine_lock;
use lingonberry_protocol::{to_canonical_json, JsonValue};

pub const ADMIN_TOKEN_ENV: &str = "LINGONBERRY_ADMIN_TOKEN";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuthAuditEvent {
    pub attempted_at: String,
    pub remote_addr: String,
    pub method: String,
    pub path: String,
    pub outcome_code: String,
}

pub fn configured_admin_token() -> Result<String, String> {
    let token = std::env::var(ADMIN_TOKEN_ENV)
        .map_err(|_| format!("{ADMIN_TOKEN_ENV} must be configured for serve-admin-http"))?;
    let token = token.trim();
    if token.is_empty() {
        return Err(format!("{ADMIN_TOKEN_ENV} must not be empty"));
    }
    Ok(token.to_string())
}

pub fn bearer_token(headers: &BTreeMap<String, String>) -> Option<&str> {
    let value = headers.get("authorization")?;
    let token = value.strip_prefix("Bearer ")?;
    (!token.is_empty()).then_some(token)
}

pub fn admin_token_matches(headers: &BTreeMap<String, String>, expected: &str) -> bool {
    let Some(actual) = bearer_token(headers) else {
        return false;
    };
    constant_time_eq(actual.as_bytes(), expected.as_bytes())
}

pub fn admin_auth_audit_path(state_dir: impl AsRef<Path>) -> PathBuf {
    state_dir.as_ref().join("admin-auth-audit.jsonl")
}

pub fn append_admin_auth_failure(
    state_dir: impl AsRef<Path>,
    remote_addr: &str,
    method: &str,
    path: &str,
    outcome_code: &str,
) -> Result<AdminAuthAuditEvent, String> {
    let state_dir = state_dir.as_ref();
    let _lock = acquire_quarantine_lock(state_dir, "admin-auth-audit")
        .map_err(|error| error.to_string())?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    let event = AdminAuthAuditEvent {
        attempted_at: format!("{}.{:09}Z", now.as_secs(), now.subsec_nanos()),
        remote_addr: remote_addr.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        outcome_code: outcome_code.to_string(),
    };
    let path = admin_auth_audit_path(state_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{}", to_canonical_json(&admin_auth_audit_json(&event)))
        .map_err(|error| error.to_string())?;
    Ok(event)
}

pub fn admin_auth_audit_json(event: &AdminAuthAuditEvent) -> JsonValue {
    JsonValue::Object(BTreeMap::from([
        (
            "attemptedAt".to_string(),
            JsonValue::String(event.attempted_at.clone()),
        ),
        (
            "method".to_string(),
            JsonValue::String(event.method.clone()),
        ),
        (
            "outcomeCode".to_string(),
            JsonValue::String(event.outcome_code.clone()),
        ),
        (
            "path".to_string(),
            JsonValue::String(event.path.clone()),
        ),
        (
            "remoteAddr".to_string(),
            JsonValue::String(event.remote_addr.clone()),
        ),
    ]))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut diff = left.len() ^ right.len();
    let max = left.len().max(right.len());
    for index in 0..max {
        let l = left.get(index).copied().unwrap_or(0);
        let r = right.get(index).copied().unwrap_or(0);
        diff |= usize::from(l ^ r);
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "lingonberry-admin-auth-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn parses_and_verifies_bearer_token() {
        let headers = BTreeMap::from([(
            "authorization".to_string(),
            "Bearer secret-token".to_string(),
        )]);
        assert_eq!(bearer_token(&headers), Some("secret-token"));
        assert!(admin_token_matches(&headers, "secret-token"));
        assert!(!admin_token_matches(&headers, "other-token"));
    }

    #[test]
    fn audit_event_omits_tokens_bodies_and_notes() {
        let dir = temp_dir();
        let event = append_admin_auth_failure(
            &dir,
            "127.0.0.1:12345",
            "POST",
            "/v1/quarantine/lb:q:1/annotations",
            "LB_ADMIN_AUTH_FAILED",
        )
        .unwrap();
        assert_eq!(event.outcome_code, "LB_ADMIN_AUTH_FAILED");
        let line = fs::read_to_string(admin_auth_audit_path(&dir)).unwrap();
        assert!(!line.contains("secret-token"));
        assert!(!line.contains("annotation note"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn audit_append_respects_operation_lock() {
        let dir = temp_dir();
        let _guard = acquire_quarantine_lock(&dir, "test-holder").unwrap();
        let error = append_admin_auth_failure(
            &dir,
            "127.0.0.1:1",
            "GET",
            "/v1/quarantine-status",
            "LB_ADMIN_AUTH_FAILED",
        )
        .unwrap_err();
        assert!(error.contains("LB_QUARANTINE_BUSY"));
        let _ = fs::remove_dir_all(dir);
    }
}