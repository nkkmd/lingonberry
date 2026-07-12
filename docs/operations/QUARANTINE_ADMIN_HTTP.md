# Quarantine Admin HTTP Isolation and Authorization

**Status: role-scoped RBAC implemented; legacy token deprecated** | **Last updated: 2026-07-12**

Quarantine administration is served from a dedicated authenticated listener instead of the public relay listener.

## Listener separation

```bash
lingonberry-relay serve-http 127.0.0.1:8787
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

The public listener returns `404` for quarantine administration paths. The admin listener defaults to loopback.

## Role credentials

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

Credential values must be non-empty and pairwise distinct. Bearer comparison is constant-time.

`LINGONBERRY_ADMIN_TOKEN` is deprecated. It is accepted only as an operator fallback when an explicit operator token is absent. The admin listener emits a startup warning when fallback is active.

## Secret-free configuration diagnostic

Run in the same environment as the admin listener:

```bash
lingonberry-admin-auth-config
```

The command reports only bounded configuration state:

```json
{
  "actionRequired": false,
  "configuredCredentialCount": 3,
  "deprecationCode": null,
  "legacyOperatorFallbackActive": false,
  "migrationAction": "none",
  "observerConfigured": true,
  "operatorConfiguredExplicitly": true,
  "removalTarget": "next-major-release",
  "reviewerConfigured": true,
  "secretsIncluded": false
}
```

When legacy fallback is active:

```text
deprecationCode: LB_ADMIN_LEGACY_TOKEN_DEPRECATED
actionRequired: true
migrationAction: set LINGONBERRY_ADMIN_OPERATOR_TOKEN and remove LINGONBERRY_ADMIN_TOKEN
```

No token value, token-derived digest, or credential fingerprint is emitted.

## Permission matrix

Observer:

```text
GET /metrics
GET /v1/quarantine-status
GET /v1/quarantine
GET /v1/quarantine/<id>
GET /v1/quarantine-resolutions
GET /v1/quarantine/<id>/annotations
GET /v1/quarantine/<id>/permanent-rejection
```

Reviewer adds:

```text
POST /v1/quarantine/<id>/annotations
```

Operator adds:

```text
POST /v1/quarantine/<id>/promote
POST /v1/quarantine/promote-batch
POST /v1/quarantine/<id>/permanent-rejection
```

## Authorization order

```text
non-admin path -> 404
missing or invalid credential -> 401
valid credential without permission -> 403
authorized request -> read body -> parse -> execute
```

Unauthorized mutation bodies are not read or interpreted before denial.

## Audit

Events are appended to:

```text
<LINGONBERRY_STATE_DIR>/admin-auth-audit.jsonl
```

```text
LB_ADMIN_AUTH_FAILED  role=null
LB_ADMIN_FORBIDDEN    role=observer|reviewer|operator
```

The audit ledger never contains bearer tokens, request bodies, annotation notes, or quarantine payloads.

## Migration

1. Set `LINGONBERRY_ADMIN_OPERATOR_TOKEN` to a new secret.
2. Restart the admin listener.
3. Run `lingonberry-admin-auth-config`.
4. Confirm `legacyOperatorFallbackActive` is `false`.
5. Smoke-test each role.
6. Remove `LINGONBERRY_ADMIN_TOKEN`.
7. Restart and rerun the diagnostic.

The fallback may be removed only in a major release, after role credentials have shipped for at least one release, deployment templates use explicit role tokens, supported deployments report no fallback, and release notes announce removal.

See `docs/roadmap/RBAC_LEGACY_TOKEN_DEPRECATION.md`.

## systemd environment

```bash
sudo install -d -m 0750 /etc/lingonberry
sudo sh -c 'cat > /etc/lingonberry/admin-http.env <<EOF
LINGONBERRY_ADMIN_OBSERVER_TOKEN=<observer-secret>
LINGONBERRY_ADMIN_REVIEWER_TOKEN=<reviewer-secret>
LINGONBERRY_ADMIN_OPERATOR_TOKEN=<operator-secret>
EOF'
sudo chmod 0600 /etc/lingonberry/admin-http.env
sudo chown root:root /etc/lingonberry/admin-http.env
```

## Non-goals

- user accounts
- browser sessions
- OAuth/OIDC
- per-record ACLs
- remote telemetry
- automatic token rotation
