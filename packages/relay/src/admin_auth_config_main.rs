mod admin_auth;

use std::collections::BTreeMap;
use std::process;

use admin_auth::{configured_admin_credentials, AdminRole};
use lingonberry_protocol::{to_canonical_json, JsonValue};

const LEGACY_DEPRECATION_CODE: &str = "LB_ADMIN_LEGACY_TOKEN_DEPRECATED";
const LEGACY_REMOVAL_TARGET: &str = "next-major-release";

fn main() {
    match configuration_report() {
        Ok(report) => println!("{}", to_canonical_json(&report)),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

fn configuration_report() -> Result<JsonValue, String> {
    let credentials = configured_admin_credentials()?;
    let observer_configured = credentials
        .credentials
        .iter()
        .any(|credential| credential.role == AdminRole::Observer);
    let reviewer_configured = credentials
        .credentials
        .iter()
        .any(|credential| credential.role == AdminRole::Reviewer);
    let operator_configured = credentials.credentials.iter().any(|credential| {
        credential.role == AdminRole::Operator
            && credential.source_env == admin_auth::ADMIN_OPERATOR_TOKEN_ENV
    });
    let legacy_active = credentials.used_legacy_operator_fallback;

    Ok(JsonValue::Object(BTreeMap::from([
        (
            "actionRequired".to_string(),
            JsonValue::Bool(legacy_active),
        ),
        (
            "configuredCredentialCount".to_string(),
            JsonValue::Number(credentials.credentials.len().to_string()),
        ),
        (
            "deprecationCode".to_string(),
            if legacy_active {
                JsonValue::String(LEGACY_DEPRECATION_CODE.to_string())
            } else {
                JsonValue::Null
            },
        ),
        (
            "legacyOperatorFallbackActive".to_string(),
            JsonValue::Bool(legacy_active),
        ),
        (
            "migrationAction".to_string(),
            JsonValue::String(if legacy_active {
                format!(
                    "set {} and remove {}",
                    admin_auth::ADMIN_OPERATOR_TOKEN_ENV,
                    admin_auth::ADMIN_TOKEN_ENV
                )
            } else {
                "none".to_string()
            }),
        ),
        (
            "observerConfigured".to_string(),
            JsonValue::Bool(observer_configured),
        ),
        (
            "operatorConfiguredExplicitly".to_string(),
            JsonValue::Bool(operator_configured),
        ),
        (
            "removalTarget".to_string(),
            JsonValue::String(LEGACY_REMOVAL_TARGET.to_string()),
        ),
        (
            "reviewerConfigured".to_string(),
            JsonValue::Bool(reviewer_configured),
        ),
        (
            "secretsIncluded".to_string(),
            JsonValue::Bool(false),
        ),
    ])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_schema_never_contains_secret_fields() {
        let source = include_str!("admin_auth_config_main.rs");
        let token_value = ["token", "Value"].concat();
        let token_digest = ["token", "Digest"].concat();
        assert!(!source.contains(&token_value));
        assert!(!source.contains(&token_digest));
        assert!(source.contains("secretsIncluded"));
    }

    #[test]
    fn deprecation_contract_is_stable() {
        assert_eq!(LEGACY_DEPRECATION_CODE, "LB_ADMIN_LEGACY_TOKEN_DEPRECATED");
        assert_eq!(LEGACY_REMOVAL_TARGET, "next-major-release");
    }
}
