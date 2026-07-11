use std::env;

use crate::{IdentityValidationStatus, ValidationReport};

pub const REQUIRE_IDENTITY_CLAIM_ENV: &str = "LINGONBERRY_REQUIRE_IDENTITY_CLAIM";
pub const UNSUPPORTED_IDENTITY_POLICY_ENV: &str = "LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedIdentityPolicy {
    Reject,
    Defer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptancePolicy {
    pub require_identity_claim: bool,
    pub unsupported_identity_policy: UnsupportedIdentityPolicy,
}

impl Default for AcceptancePolicy {
    fn default() -> Self {
        Self {
            require_identity_claim: false,
            unsupported_identity_policy: UnsupportedIdentityPolicy::Reject,
        }
    }
}

impl AcceptancePolicy {
    pub fn from_env() -> Result<Self, String> {
        let require_identity_claim = match env::var(REQUIRE_IDENTITY_CLAIM_ENV) {
            Ok(value) => parse_bool(REQUIRE_IDENTITY_CLAIM_ENV, &value)?,
            Err(env::VarError::NotPresent) => false,
            Err(error) => {
                return Err(format!(
                    "failed to read {REQUIRE_IDENTITY_CLAIM_ENV}: {error}"
                ))
            }
        };

        let unsupported_identity_policy = match env::var(UNSUPPORTED_IDENTITY_POLICY_ENV) {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "reject" => UnsupportedIdentityPolicy::Reject,
                "defer" => UnsupportedIdentityPolicy::Defer,
                other => {
                    return Err(format!(
                        "{UNSUPPORTED_IDENTITY_POLICY_ENV} must be reject or defer, got {other}"
                    ))
                }
            },
            Err(env::VarError::NotPresent) => UnsupportedIdentityPolicy::Reject,
            Err(error) => {
                return Err(format!(
                    "failed to read {UNSUPPORTED_IDENTITY_POLICY_ENV}: {error}"
                ))
            }
        };

        Ok(Self {
            require_identity_claim,
            unsupported_identity_policy,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptanceDecision {
    Accept,
    Reject {
        code: &'static str,
        errors: Vec<String>,
    },
    Defer {
        code: &'static str,
        errors: Vec<String>,
    },
}

pub fn evaluate_acceptance(
    report: &ValidationReport,
    policy: &AcceptancePolicy,
) -> AcceptanceDecision {
    let mut hard_errors = report.schema_errors.clone();
    hard_errors.extend(report.identity_errors.clone());
    if !hard_errors.is_empty() {
        return AcceptanceDecision::Reject {
            code: "LB_VALIDATION_FAILED",
            errors: hard_errors,
        };
    }

    if report.identity_status == IdentityValidationStatus::NotPresent
        && policy.require_identity_claim
    {
        return AcceptanceDecision::Reject {
            code: "LB_IDENTITY_CLAIM_REQUIRED",
            errors: vec![
                "at least one identity claim is required by the acceptance policy".to_string(),
            ],
        };
    }

    if report.identity_status == IdentityValidationStatus::Unsupported {
        return match policy.unsupported_identity_policy {
            UnsupportedIdentityPolicy::Reject => AcceptanceDecision::Reject {
                code: "LB_UNSUPPORTED_IDENTITY_RULE",
                errors: report.unsupported_identity_rules.clone(),
            },
            UnsupportedIdentityPolicy::Defer => AcceptanceDecision::Defer {
                code: "LB_IDENTITY_DEFERRED",
                errors: report.unsupported_identity_rules.clone(),
            },
        };
    }

    AcceptanceDecision::Accept
}

fn parse_bool(name: &str, value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        other => Err(format!(
            "{name} must be a boolean (true/false, 1/0, yes/no, on/off), got {other}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(status: IdentityValidationStatus) -> ValidationReport {
        ValidationReport {
            schema_errors: Vec::new(),
            identity_errors: Vec::new(),
            unsupported_identity_rules: if status == IdentityValidationStatus::Unsupported {
                vec!["unsupported rule".to_string()]
            } else {
                Vec::new()
            },
            identity_status: status,
        }
    }

    #[test]
    fn default_policy_accepts_objects_without_claims() {
        assert_eq!(
            evaluate_acceptance(
                &report(IdentityValidationStatus::NotPresent),
                &AcceptancePolicy::default()
            ),
            AcceptanceDecision::Accept
        );
    }

    #[test]
    fn strict_policy_requires_an_identity_claim() {
        let policy = AcceptancePolicy {
            require_identity_claim: true,
            ..AcceptancePolicy::default()
        };
        assert!(matches!(
            evaluate_acceptance(&report(IdentityValidationStatus::NotPresent), &policy),
            AcceptanceDecision::Reject {
                code: "LB_IDENTITY_CLAIM_REQUIRED",
                ..
            }
        ));
    }

    #[test]
    fn unsupported_rules_can_be_deferred() {
        let policy = AcceptancePolicy {
            require_identity_claim: false,
            unsupported_identity_policy: UnsupportedIdentityPolicy::Defer,
        };
        assert!(matches!(
            evaluate_acceptance(&report(IdentityValidationStatus::Unsupported), &policy),
            AcceptanceDecision::Defer {
                code: "LB_IDENTITY_DEFERRED",
                ..
            }
        ));
    }
}
