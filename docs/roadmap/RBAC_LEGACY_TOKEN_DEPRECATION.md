# Legacy Admin Token Deprecation

**Status: active deprecation** | **Last updated: 2026-07-12**

`LINGONBERRY_ADMIN_TOKEN` is deprecated. It remains an operator fallback only when `LINGONBERRY_ADMIN_OPERATOR_TOKEN` is absent.

## Stable contract

```text
deprecation code: LB_ADMIN_LEGACY_TOKEN_DEPRECATED
removal target: next-major-release
replacement: LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

No token value, token digest, or credential fingerprint is emitted or stored by the diagnostic path.

## Diagnostic

Run in the same environment as the admin listener:

```bash
lingonberry-admin-auth-config
```

Healthy explicit-role output has:

```json
{
  "actionRequired": false,
  "legacyOperatorFallbackActive": false,
  "operatorConfiguredExplicitly": true,
  "secretsIncluded": false
}
```

Legacy fallback output has:

```json
{
  "actionRequired": true,
  "deprecationCode": "LB_ADMIN_LEGACY_TOKEN_DEPRECATED",
  "legacyOperatorFallbackActive": true,
  "migrationAction": "set LINGONBERRY_ADMIN_OPERATOR_TOKEN and remove LINGONBERRY_ADMIN_TOKEN",
  "removalTarget": "next-major-release",
  "secretsIncluded": false
}
```

## Migration checklist

1. Generate a new independent operator secret.
2. Set `LINGONBERRY_ADMIN_OPERATOR_TOKEN`.
3. Keep observer and reviewer secrets distinct.
4. Restart the admin listener.
5. Run `lingonberry-admin-auth-config` in the service environment.
6. Confirm `legacyOperatorFallbackActive` is `false`.
7. Smoke-test observer, reviewer, and operator routes.
8. Remove `LINGONBERRY_ADMIN_TOKEN` from the environment file.
9. Restart and rerun the diagnostic.

## Removal conditions

The compatibility fallback may be removed only when all conditions are satisfied:

1. role-scoped credentials have shipped for at least one release;
2. supported deployment templates use explicit role tokens;
3. supported deployments can demonstrate `legacyOperatorFallbackActive: false`;
4. release notes announce the breaking change;
5. removal occurs in a major release.

## Non-goals

- remote telemetry
- secret fingerprinting
- automatic token rotation
- immediate compatibility removal
