use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lingonberry_core::acquire_quarantine_lock;
use lingonberry_protocol::{to_canonical_json, JsonValue};

pub const ADMIN_TOKEN_ENV: &str = "LINGONBERRY_ADMIN_TOKEN";
pub const ADMIN_OBSERVER_TOKEN_ENV: &str = "LINGONBERRY_ADMIN_OBSERVER_TOKEN";
pub const ADMIN_REVIEWER_TOKEN_ENV: &str = "LINGONBERRY_ADMIN_REVIEWER_TOKEN";
pub const ADMIN_OPERATOR_TOKEN_ENV: &str = "LINGONBERRY_ADMIN_OPERATOR_TOKEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdminRole {
    Observer,
    Reviewer,
    Operator,
}

impl AdminRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observer => "observer",
            Self::Reviewer => "reviewer",
            Self::Operator => "operator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminCredential {
    pub role: AdminRole,
    pub token: String,
    pub source_env: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminCredentials {
    pub credentials: Vec<AdminCredential>,
    pub used_legacy_operator_fallback: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminPermission {
    Observe,
    Annotate,
    Operate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuthAuditEvent {
    pub attempted_at: String,
    pub remote_addr: String,
    pub method: String,
    pub path: String,
    pub role: Option<String>,
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

pub fn configured_admin_credentials() -> Result<AdminCredentials, String> {
    configured_admin_credentials_from(|name| std::env::var(name).ok())
}

fn configured_admin_credentials_from(
    mut read: impl FnMut(&str) -> Option<String>,
) -> Result<AdminCredentials, String> {
    let mut credentials = Vec::new();
    for (role, name) in [
        (AdminRole::Observer, ADMIN_OBSERVER_TOKEN_ENV),
        (AdminRole::Reviewer, ADMIN_REVIEWER_TOKEN_ENV),
        (AdminRole::Operator, ADMIN_OPERATOR_TOKEN_ENV),
    ] {
        if let Some(value) = read(name) {
            let token = value.trim();
            if token.is_empty() {
                return Err(format!("{name} must not be empty when configured"));
            }
            credentials.push(AdminCredential {
                role,
                token: token.to_string(),
                source_env: name,
            });
        }
    }

    let mut used_legacy_operator_fallback = false;
    if !credentials
        .iter()
        .any(|credential| credential.role == AdminRole::Operator)
    {
        if let Some(value) = read(ADMIN_TOKEN_ENV) {
            let token = value.trim();
            if token.is_empty() {
                return Err(format!("{ADMIN_TOKEN_ENV} must not be empty when configured"));
            }
            credentials.push(AdminCredential {
                role: AdminRole::Operator,
                token: token.to_string(),
                source_env: ADMIN_TOKEN_ENV,
            });
            used_legacy_operator_fallback = true;
        }
    }

    if credentials.is_empty() {
        return Err(format!(
            "configure at least one of {ADMIN_OBSERVER_TOKEN_ENV}, {ADMIN_REVIEWER_TOKEN_ENV}, {ADMIN_OPERATOR_TOKEN_ENV}, or legacy {ADMIN_TOKEN_ENV}"
        ));
    }

    let mut unique = BTreeSet::new();
    for credential in &credentials {
        if !unique.insert(credential.token.as_str()) {
            return Err("configured admin role tokens must be pairwise distinct".to_string());
        }
    }

    Ok(AdminCredentials {
        credentials,
        used_legacy_operator_fallback,
    })
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

pub fn resolve_admin_role(
    headers: &BTreeMap<String, String>,
    credentials: &AdminCredentials,
) -> Option<AdminRole> {
    let actual = bearer_token(headers)?;
    let mut matched = None;
    for credential in &credentials.credentials {
        if constant_time_eq(actual.as_bytes(), credential.token.as_bytes()) {
            matched = Some(credential.role);
        }
    }
    matched
}

pub fn required_admin_permission(method: &str, path: &str) -> Option<AdminPermission> {
    if method == "GET"
        && (path == "/metrics"
            || path == "/v1/quarantine-status"
            || path == "/v1/quarantine"
            || path == "/v1/quarantine-resolutions"
            || is_record_read_path(path)
            || is_suffix_path(path, "/annotations")
            || is_suffix_path(path, "/permanent-rejection"))
    {
        return Some(AdminPermission::Observe);
    }
    if method == "POST" && is_suffix_path(path, "/annotations") {
        return Some(AdminPermission::Annotate);
    }
    if method == "POST"
        && (path == "/v1/quarantine/promote-batch"
            || is_suffix_path(path, "/promote")
            || is_suffix_path(path, "/permanent-rejection"))
    {
        return Some(AdminPermission::Operate);
    }
    None
}

pub fn admin_role_allows(role: AdminRole, permission: AdminPermission) -> bool {
    match permission {
        AdminPermission::Observe => true,
        AdminPermission::Annotate => matches!(role, AdminRole::Reviewer | AdminRole::Operator),
        AdminPermission::Operate => role == AdminRole::Operator,
    }
}

pub fn admin_request_allowed(role: AdminRole, method: &str, path: &str) -> bool {
    required_admin_permission(method, path)
        .map(|permission| admin_role_allows(role, permission))
        .unwrap_or(false)
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
    append_admin_auth_event(
        state_dir,
        remote_addr,
        method,
        path,
        None,
        outcome_code,
    )
}

pub fn append_admin_authorization_failure(
    state_dir: impl AsRef<Path>,
    remote_addr: &str,
    method: &str,
    path: &str,
    role: AdminRole,
) -> Result<AdminAuthAuditEvent, String> {
    append_admin_auth_event(
        state_dir,
        remote_addr,
        method,
        path,
        Some(role),
        "LB_ADMIN_FORBIDDEN",
    )
}

fn append_admin_auth_event(
    state_dir: impl AsRef<Path>,
    remote_addr: &str,
    method: &str,
    path: &str,
    role: Option<AdminRole>,
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
        role: role.map(|role| role.as_str().to_string()),
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
        (
            "role".to_string(),
            event
                .role
                .as_ref()
                .map(|role| JsonValue::String(role.clone()))
                .unwrap_or(JsonValue::Null),
        ),
    ]))
}

fn is_record_read_path(path: &str) -> bool {
    let Some(id) = path.strip_prefix("/v1/quarantine/") else {
        return false;
    };
    !id.is_empty() && !id.contains('/')
}

fn is_suffix_path(path: &str, suffix: &str) -> bool {
    let Some(rest) = path.strip_prefix("/v1/quarantine/") else {
        return false;
    };
    let Some(id) = rest.strip_suffix(suffix) else {
        return false;
    };
    !id.is_empty() && !id.contains('/')
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
    fn loads_role_tokens_and_legacy_operator_fallback() {
        let values = BTreeMap::from([
            (ADMIN_OBSERVER_TOKEN_ENV, "observe".to_string()),
            (ADMIN_REVIEWER_TOKEN_ENV, "review".to_string()),
            (ADMIN_TOKEN_ENV, "legacy-operate".to_string()),
        ]);
        let credentials = configured_admin_credentials_from(|name| values.get(name).cloned())
            .unwrap();
        assert!(credentials.used_legacy_operator_fallback);
        assert_eq!(credentials.credentials.len(), 3);
        assert!(credentials
            .credentials
            .iter()
            .any(|credential| credential.role == AdminRole::Operator));
    }

    #[test]
    fn rejects_empty_duplicate_and_missing_tokens() {
        let empty = BTreeMap::from([(ADMIN_OBSERVER_TOKEN_ENV, "  ".to_string())]);
        assert!(configured_admin_credentials_from(|name| empty.get(name).cloned()).is_err());

        let duplicate = BTreeMap::from([
            (ADMIN_OBSERVER_TOKEN_ENV, "same".to_string()),
            (ADMIN_OPERATOR_TOKEN_ENV, "same".to_string()),
        ]);
        assert!(configured_admin_credentials_from(|name| duplicate.get(name).cloned()).is_err());
        assert!(configured_admin_credentials_from(|_| None).is_err());
    }

    #[test]
    fn resolves_roles_and_preserves_single_token_compatibility() {
        let credentials = AdminCredentials {
            credentials: vec![
                AdminCredential {
                    role: AdminRole::Observer,
                    token: "observe".to_string(),
                    source_env: ADMIN_OBSERVER_TOKEN_ENV,
                },
                AdminCredential {
                    role: AdminRole::Operator,
                    token: "operate".to_string(),
                    source_env: ADMIN_OPERATOR_TOKEN_ENV,
                },
            ],
            used_legacy_operator_fallback: false,
        };
        let headers = BTreeMap::from([(
            "authorization".to_string(),
            "Bearer observe".to_string(),
        )]);
        assert_eq!(resolve_admin_role(&headers, &credentials), Some(AdminRole::Observer));
        assert!(admin_token_matches(&headers, "observe"));
        assert!(!admin_token_matches(&headers, "operate"));
    }

    #[test]
    fn permission_matrix_is_least_privilege() {
        assert!(admin_request_allowed(
            AdminRole::Observer,
            "GET",
            "/v1/quarantine-status"
        ));
        assert!(!admin_request_allowed(
            AdminRole::Observer,
            "POST",
            "/v1/quarantine/q1/annotations"
        ));
        assert!(admin_request_allowed(
            AdminRole::Reviewer,
            "POST",
            "/v1/quarantine/q1/annotations"
        ));
        assert!(!admin_request_allowed(
            AdminRole::Reviewer,
            "POST",
            "/v1/quarantine/q1/promote"
        ));
        assert!(admin_request_allowed(
            AdminRole::Operator,
            "POST",
            "/v1/quarantine/q1/promote"
        ));
        assert!(!admin_request_allowed(
            AdminRole::Operator,
            "DELETE",
            "/v1/quarantine/q1"
        ));
    }

    #[test]
    fn audit_event_omits_tokens_bodies_and_notes_and_records_role() {
        let dir = temp_dir();
        let event = append_admin_authorization_failure(
            &dir,
            "127.0.0.1:12345",
            "POST",
            "/v1/quarantine/lb:q:1/promote",
            AdminRole::Reviewer,
        )
        .unwrap();
        assert_eq!(event.role.as_deref(), Some("reviewer"));
        assert_eq!(event.outcome_code, "LB_ADMIN_FORBIDDEN");
        let line = fs::read_to_string(admin_auth_audit_path(&dir)).unwrap();
        assert!(line.contains("reviewer"));
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