from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text()
    if old not in content:
        raise SystemExit(f"expected text not found in {path}: {old[:100]!r}")
    target.write_text(content.replace(old, new, 1))


replace_once(
    "packages/relay/src/main.rs",
    '''use lingonberry_validation::{
    finalize_knowledge_object_full, validate_knowledge_object_full, validate_publish_request_full,
    IdentityValidationStatus, ValidationReport,
};
''',
    '''use lingonberry_validation::{
    evaluate_acceptance, finalize_knowledge_object_full, validate_knowledge_object_full,
    validate_publish_request_full, AcceptanceDecision, AcceptancePolicy, IdentityValidationStatus,
    ValidationReport,
};
''',
)

replace_once(
    "packages/relay/src/main.rs",
    '''    let report = validate_publish_request_full(&loaded.value);
    if !report.is_valid() {
        return Err(format_validation_error(
            "validation failed",
            &report.combined_errors(),
        ));
    }
''',
    '''    let report = validate_publish_request_full(&loaded.value);
    let policy = AcceptancePolicy::from_env()?;
    match evaluate_acceptance(&report, &policy) {
        AcceptanceDecision::Accept => {}
        AcceptanceDecision::Reject { code, errors } => {
            return Err(format!("{}: {}", code, errors.join("; ")))
        }
        AcceptanceDecision::Defer { code, errors } => {
            return Err(format!("{}: {}", code, errors.join("; ")))
        }
    }
''',
)

replace_once(
    "packages/relay/src/main.rs",
    '''    let report = validate_publish_request_full(&value);
    if !report.is_valid() {
        let (status, text, kind) = validation_http_error(&report);
        return Ok((
            status,
            text,
            http_error(kind, &report.combined_errors().join("; ")),
        ));
    }
''',
    '''    let report = validate_publish_request_full(&value);
    let policy = AcceptancePolicy::from_env()?;
    match evaluate_acceptance(&report, &policy) {
        AcceptanceDecision::Accept => {}
        AcceptanceDecision::Reject { code, errors } => {
            let kind = match code {
                "LB_IDENTITY_CLAIM_REQUIRED" => "identity_claim_required",
                "LB_UNSUPPORTED_IDENTITY_RULE" => "unsupported_identity_rule",
                _ => "validation_error",
            };
            let status = if code == "LB_UNSUPPORTED_IDENTITY_RULE" { 422 } else { 400 };
            let text = if status == 422 { "Unprocessable Entity" } else { "Bad Request" };
            return Ok((status, text, http_error(kind, &errors.join("; "))));
        }
        AcceptanceDecision::Defer { errors, .. } => {
            return Ok((
                202,
                "Accepted",
                json_object(vec![
                    ("status", JsonValue::String("deferred".to_string())),
                    ("reason", JsonValue::String(errors.join("; "))),
                    ("stored", JsonValue::Bool(false)),
                ]),
            ));
        }
    }
''',
)

replace_once(
    "packages/core/src/lib.rs",
    '''use lingonberry_validation::{finalize_knowledge_object_full, IdentityValidationStatus};
''',
    '''use lingonberry_validation::{
    evaluate_acceptance, finalize_knowledge_object_full, validate_knowledge_object_full,
    AcceptanceDecision, AcceptancePolicy,
};
''',
)

replace_once(
    "packages/core/src/lib.rs",
    '''        let finalized = finalize_knowledge_object_full(object_value).map_err(|report| {
            let code = if report.identity_status == IdentityValidationStatus::Unsupported {
                "LB_UNSUPPORTED_IDENTITY_RULE"
            } else {
                "LB_ARCHIVE_IMPORT"
            };
            store_error(code, report.combined_errors().join("; "))
        })?;
''',
    '''        let report = validate_knowledge_object_full(object_value);
        let policy = AcceptancePolicy::from_env()
            .map_err(|error| store_error("LB_ACCEPTANCE_POLICY", error))?;
        match evaluate_acceptance(&report, &policy) {
            AcceptanceDecision::Accept => {}
            AcceptanceDecision::Reject { code, errors } => {
                return Err(store_error(code, errors.join("; ")))
            }
            AcceptanceDecision::Defer { code, errors } => {
                return Err(store_error(code, errors.join("; ")))
            }
        }
        let finalized = finalize_knowledge_object_full(object_value)
            .map_err(|report| store_error("LB_ARCHIVE_IMPORT", report.combined_errors().join("; ")))?;
''',
)

Path("docs/operations/ACCEPTANCE_POLICY.md").parent.mkdir(parents=True, exist_ok=True)
Path("docs/operations/ACCEPTANCE_POLICY.md").write_text('''# Acceptance Policy

Lingonberry ingress can be configured with environment variables.

## Identity Claim requirement

```bash
export LINGONBERRY_REQUIRE_IDENTITY_CLAIM=true
```

Default: `false`.

When enabled, publish and archive import reject knowledge objects that do not contain at least one Identity Claim with `LB_IDENTITY_CLAIM_REQUIRED`.

## Unsupported Identity Key rules

```bash
export LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY=reject
```

Allowed values:

- `reject` (default): reject with `LB_UNSUPPORTED_IDENTITY_RULE`; HTTP returns 422.
- `defer`: do not store the object; HTTP returns 202 with `status: deferred`, while CLI and archive import return `LB_IDENTITY_DEFERRED`.

`defer` is intentionally non-persistent in this version. A durable quarantine store can be added separately without weakening the guarantee that unverified objects never enter the canonical catalog.

## Defaults

The default policy preserves existing behavior:

```text
identity claim required: false
unsupported identity rule: reject
```
''')
