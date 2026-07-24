# Quarantine Admin HTTP Isolation and Authorization

**Status: v1.0 pre-release normative operations contract**  
**Last reviewed: 2026-07-24**

## Purpose

This document defines the supported v1.0 administrative HTTP boundary for quarantine inspection, annotation, promotion, permanent rejection, metrics, authentication, authorization, and audit evidence.

The administrative listener is distinct from the public relay listener. It is not a public carrier surface, user-account system, browser session service, or general-purpose management API.

## 1. Listener isolation

Run the public and administrative listeners as separate processes or separately supervised service instances:

```bash
lingonberry-relay serve-http 127.0.0.1:8787
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

The following invariants apply:

- the public listener returns `404` for quarantine administration paths;
- the administrative listener defaults to loopback and must not be published by the public reverse-proxy route set;
- firewall, namespace, container-network, or host routing policy must prevent unintended remote access;
- the built-in listener is a bounded HTTP/1.1 implementation, not a production TLS terminator or hardened edge proxy;
- TLS, external request limits, connection policy, and remote-access controls belong to a separately reviewed administrative access layer;
- exposing the administrative listener beyond a trusted host boundary requires an explicit deployment security review.

## 2. Role credentials

Supported role-token environment variables are:

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

Configured values must be non-empty and pairwise distinct. Bearer-token comparison is constant-time.

`LINGONBERRY_ADMIN_TOKEN` is deprecated. It is accepted only as an operator fallback when `LINGONBERRY_ADMIN_OPERATOR_TOKEN` is absent. The listener emits a startup warning when fallback is active.

At least one supported role token or the legacy fallback must be configured. A deployment is not required to configure all three roles, but absent roles are unavailable.

Tokens must be injected through a deployment-controlled secret path. They must not appear in repository files, image layers, command lines, shell history, logs, evidence bundles, or public diagnostics.

## 3. Secret-free configuration diagnostic

Run the diagnostic under the same environment and service identity as the administrative listener:

```bash
lingonberry-admin-auth-config
```

The command reports bounded configuration state only, including whether each role is configured and whether the legacy fallback is active. It must not emit token values, token-derived digests, fingerprints, or reversible credential material.

When legacy fallback is active, the diagnostic reports:

```text
deprecationCode: LB_ADMIN_LEGACY_TOKEN_DEPRECATED
actionRequired: true
migrationAction: set LINGONBERRY_ADMIN_OPERATOR_TOKEN and remove LINGONBERRY_ADMIN_TOKEN
```

## 4. Permission matrix

### Observer

```text
GET /metrics
GET /v1/quarantine-status
GET /v1/quarantine
GET /v1/quarantine/<id>
GET /v1/quarantine-resolutions
GET /v1/quarantine/<id>/annotations
GET /v1/quarantine/<id>/permanent-rejection
```

### Reviewer

A reviewer has observer permissions and may also call:

```text
POST /v1/quarantine/<id>/annotations
```

### Operator

An operator has observer and reviewer permissions and may also call:

```text
POST /v1/quarantine/<id>/promote
POST /v1/quarantine/promote-batch
POST /v1/quarantine/<id>/permanent-rejection
```

No role grants access to arbitrary public relay routes through the administrative listener. Paths and methods not present in the administrative route set return `404` or `403` according to the authorization sequence below.

## 5. Authentication and authorization order

The implementation evaluates requests in this order:

```text
non-admin path -> 404
missing or invalid bearer credential -> 401
valid credential without required permission -> 403
authorized request -> read body -> parse -> execute
```

Unauthorized mutation bodies are not read or interpreted before denial. Operators must preserve this ordering when adding reverse proxies, middleware, or request logging.

A `401` response proves only that no configured role token matched. A `403` response proves that a valid role lacked the required permission. Neither response should disclose which tokens or roles are configured beyond the authenticated role already presented.

## 6. Quarantine mutation boundaries

Administrative authorization does not bypass protocol validation, acceptance policy, duplicate/conflict classification, permanent-rejection checks, or storage errors.

- annotation records do not mutate the quarantined payload;
- promotion reruns the governing promotion and storage path;
- permanent rejection prevents later promotion of that quarantine record unless a future explicitly specified recovery mechanism is introduced;
- batch promotion is an operator action and may produce per-record mixed outcomes;
- HTTP success for an administrative request is not evidence of formal release qualification, backup validity, or storage durability beyond the returned operation result.

See the quarantine-specific contracts for payload and lifecycle semantics.

## 7. Authentication and authorization audit

Authentication and authorization failures are appended under the resolved runtime state directory:

```text
<runtime-state-dir>/admin-auth-audit.jsonl
```

Current outcome codes include:

```text
LB_ADMIN_AUTH_FAILED  role=null
LB_ADMIN_FORBIDDEN    role=observer|reviewer|operator
```

Each event records bounded metadata such as attempt time, remote address, method, path, role when known, and outcome code.

The audit ledger must not contain bearer tokens, request bodies, annotation notes, quarantine payloads, or token-derived fingerprints. Access to the ledger must be restricted because remote addresses and administrative access patterns are operationally sensitive.

The current ledger covers authentication and authorization failures. It must not be represented as a complete successful-action audit trail; successful quarantine actions have their own operation-specific records where implemented.

Failure to append required authentication or authorization audit evidence is an administrative request failure, not a condition to silently ignore.

## 8. Deployment requirements

For the checked-in systemd integration:

- use `deploy/systemd/lingonberry-admin-http.service` as the reference unit;
- inject role tokens through the unit's protected environment file or an equivalent secret mechanism;
- keep the listener on loopback unless an explicitly reviewed private administrative network is used;
- run with the same resolved state root and storage configuration required for the quarantine records under administration;
- do not grant the public relay process administrator credentials;
- do not route the administrative listener through the public Caddy site block;
- restrict environment-file ownership and permissions;
- restart the administrative listener after credential changes;
- smoke-test every configured role after deployment or rotation.

Example environment-file preparation:

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

Placeholders must be replaced through a secret-management workflow, not committed or retained in evidence.

## 9. Legacy-token migration

1. Generate a new operator secret.
2. Set `LINGONBERRY_ADMIN_OPERATOR_TOKEN` without removing the legacy token yet.
3. Restart the administrative listener.
4. Run `lingonberry-admin-auth-config` under the service environment.
5. Confirm `legacyOperatorFallbackActive` is `false`.
6. Smoke-test every configured role and expected denial boundary.
7. Remove `LINGONBERRY_ADMIN_TOKEN`.
8. Restart and rerun the diagnostic and smoke tests.
9. Record the rotation without recording secret material.

The fallback may be removed only in a major release after role credentials have shipped for at least one release, deployment templates use explicit role tokens, supported deployments report no fallback, and release notes announce removal.

See [RBAC Legacy Token Deprecation](../roadmap/RBAC_LEGACY_TOKEN_DEPRECATION.md).

## 10. Verification checklist

Before treating the administrative surface as operationally available, verify:

1. the public listener returns `404` for administrative paths;
2. the administrative listener is not exposed by the public reverse proxy;
3. missing and invalid tokens return `401`;
4. valid but insufficient roles return `403`;
5. observer reads succeed and mutations fail;
6. reviewer annotations succeed while operator-only mutations fail;
7. operator mutations reach the governing quarantine lifecycle;
8. unauthorized mutation bodies are rejected before parsing;
9. failure audit records are appended without secrets or payloads;
10. legacy fallback is inactive unless explicitly documented as a temporary migration state.

These checks are deployment verification only. They do not constitute formal 72-hour soak evidence or privileged reference-host qualification.

## 11. Non-goals

The v1.0 administrative HTTP surface does not provide:

- user accounts or browser sessions;
- OAuth, OIDC, SSO, or delegated identity;
- per-record ACLs;
- dynamic role management;
- automatic token issuance, rotation, or revocation;
- public remote administration;
- clustered administration or cross-node orchestration;
- a complete successful-action security audit ledger;
- a production reverse proxy or TLS implementation.

## References

- [Secret Management](./SECRET_MANAGEMENT.md)
- [Quarantine Annotations](./QUARANTINE_ANNOTATIONS.md)
- [Quarantine Permanent Rejections](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [Quarantine Promotion Runbook](./QUARANTINE_PROMOTION_RUNBOOK.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
