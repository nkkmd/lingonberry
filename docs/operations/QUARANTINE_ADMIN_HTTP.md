# Quarantine Admin HTTP Isolation and Authorization

**Status: authentication implemented; RBAC foundation implemented but not yet wired** | **Last updated: 2026-07-12**

Quarantine administration is served from a dedicated authenticated listener instead of the public relay listener.

## Listener separation

Public relay:

```bash
lingonberry-relay serve-http 127.0.0.1:8787
```

The public listener exposes readiness, capabilities, publish, and object retrieval. Requests for `/metrics`, `/v1/quarantine-status`, `/v1/quarantine`, `/v1/quarantine-resolutions`, and `/v1/quarantine/*` return `404`.

Admin listener:

```bash
export LINGONBERRY_ADMIN_TOKEN='replace-with-a-long-random-secret'
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

The active listener still uses the legacy single operator-equivalent token. RBAC-1A adds the credential and permission model; RBAC-1B will connect it to HTTP request handling.

## RBAC credential model

The staged role-token variables are:

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

`LINGONBERRY_ADMIN_TOKEN` is retained as an operator fallback during migration.

Credential loading rules:

- at least one usable credential is required;
- configured values must not be empty;
- all configured role tokens must be pairwise distinct;
- the legacy token is used only when an explicit operator token is absent;
- bearer comparison uses constant-time byte comparison;
- one token resolves to exactly one role.

Do not deploy the role-specific variables as an access-control change until RBAC-1B is merged. The current listener does not yet consume the new credential set.

## Permission matrix

### Observer

Read-only access:

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

Observer permissions plus:

```text
POST /v1/quarantine/<id>/annotations
```

### Operator

Reviewer permissions plus:

```text
POST /v1/quarantine/<id>/promote
POST /v1/quarantine/promote-batch
POST /v1/quarantine/<id>/permanent-rejection
```

Unknown methods and routes have no permission assignment and are denied by the matrix.

## Active authentication behavior

Send the current token as a bearer credential:

```bash
curl -sS \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_TOKEN" \
  http://127.0.0.1:8788/v1/quarantine-status
```

Missing and invalid credentials receive the same response:

```text
401 Unauthorized
```

RBAC-1B will add authenticated-but-forbidden `403 Forbidden` responses without disclosing which credential or role was expected.

## Authentication and authorization audit

Events are appended to:

```text
<LINGONBERRY_STATE_DIR>/admin-auth-audit.jsonl
```

The role-aware schema is:

```json
{
  "attemptedAt": "...Z",
  "remoteAddr": "127.0.0.1:12345",
  "method": "POST",
  "path": "/v1/quarantine/lb:q:1/promote",
  "role": "reviewer",
  "outcomeCode": "LB_ADMIN_FORBIDDEN"
}
```

Authentication failures use `role: null` and `LB_ADMIN_AUTH_FAILED`. The ledger never stores bearer tokens, request bodies, annotation notes, or quarantine payloads. Audit append failures are not silently ignored.

## systemd

Template:

```text
deploy/systemd/lingonberry-admin-http.service
```

Current production environment file:

```bash
sudo install -d -m 0750 /etc/lingonberry
sudo sh -c 'printf "%s\n" "LINGONBERRY_ADMIN_TOKEN=<long-random-secret>" > /etc/lingonberry/admin-http.env'
sudo chmod 0600 /etc/lingonberry/admin-http.env
sudo chown root:root /etc/lingonberry/admin-http.env
```

After RBAC-1B, migrate to independent role credentials and remove the legacy token after confirming all clients use the intended least-privilege role.

The service binds only to `127.0.0.1:8788`. Remote access should use a separately authenticated and TLS-terminated administrative channel.

## Staged rollout

```text
RBAC-1A: role types, credential validation, permission matrix, audit schema
RBAC-1B: HTTP role resolution, 401/403 enforcement, integration tests
RBAC-1C: legacy token deprecation and removal plan
```

## Non-goals

- user accounts
- browser sessions or CSRF protection
- per-record ACLs
- OAuth/OIDC
- distributed authorization policy
- remote-by-default binding
