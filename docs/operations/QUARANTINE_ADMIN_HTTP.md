# Quarantine Admin HTTP Isolation and Authorization

**Status: role-scoped RBAC implemented** | **Last updated: 2026-07-12**

Quarantine administration is served from a dedicated authenticated listener instead of the public relay listener.

## Listener separation

Public relay:

```bash
lingonberry-relay serve-http 127.0.0.1:8787
```

The public listener exposes readiness, capabilities, publish, and object retrieval. Requests for `/metrics`, `/v1/quarantine-status`, `/v1/quarantine`, `/v1/quarantine-resolutions`, and `/v1/quarantine/*` return `404`.

Admin listener:

```bash
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

The listener defaults to loopback and now loads role-scoped credentials at startup.

## Credentials

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

`LINGONBERRY_ADMIN_TOKEN` remains available as an operator fallback when an explicit operator token is absent. Startup emits a warning when this compatibility path is active.

Credential rules:

- at least one usable credential is required;
- configured values must not be empty;
- configured tokens must be pairwise distinct;
- one token resolves to exactly one role;
- bearer comparison uses constant-time byte comparison;
- the expected token or role is never disclosed in an error response.

## Permission matrix

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

Unknown method and route combinations receive no permission.

## Authentication and authorization order

For an admin-listener request:

1. non-admin paths return `404 Not Found`;
2. the bearer credential is resolved to a role;
3. missing or invalid credentials return `401 Unauthorized`;
4. the method and route are checked against the role permission matrix;
5. insufficient permissions return `403 Forbidden`;
6. only authorized requests have their body read and interpreted;
7. the existing route handler is executed.

Reading and parsing a mutation body after authorization prevents unauthorized payloads from reaching validation or mutation code.

## Examples

Observer read:

```bash
curl -sS \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_OBSERVER_TOKEN" \
  http://127.0.0.1:8788/v1/quarantine-status
```

Reviewer annotation:

```bash
curl -sS \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_REVIEWER_TOKEN" \
  -H 'Content-Type: application/json' \
  --data '{"operator":"reviewer-a","note":"manual review complete"}' \
  http://127.0.0.1:8788/v1/quarantine/lb:q:123/annotations
```

Operator promotion:

```bash
curl -sS \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_OPERATOR_TOKEN" \
  -X POST \
  http://127.0.0.1:8788/v1/quarantine/lb:q:123/promote
```

Missing and invalid credentials receive the same bounded response:

```text
401 Unauthorized
```

Authenticated credentials without permission receive:

```text
403 Forbidden
```

Neither response identifies the configured role or credential.

## Authentication and authorization audit

Events are appended to:

```text
<LINGONBERRY_STATE_DIR>/admin-auth-audit.jsonl
```

Authentication failure:

```json
{
  "attemptedAt": "...Z",
  "remoteAddr": "127.0.0.1:12345",
  "method": "GET",
  "path": "/v1/quarantine-status",
  "role": null,
  "outcomeCode": "LB_ADMIN_AUTH_FAILED"
}
```

Authorization failure:

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

The ledger never stores bearer tokens, request bodies, annotation notes, or quarantine payloads. Audit append failures are not silently ignored.

## systemd environment

Template:

```text
deploy/systemd/lingonberry-admin-http.service
```

Create independent credentials with restrictive permissions:

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

The three secrets must be different. Remove `LINGONBERRY_ADMIN_TOKEN` after all clients have migrated to explicit least-privilege credentials.

The service binds only to `127.0.0.1:8788`. Remote access should use a separately authenticated and TLS-terminated administrative channel.

## Remaining migration work

```text
RBAC-1A: role types, credential validation, permission matrix, audit schema — complete
RBAC-1B: HTTP role resolution and 401/403 enforcement — complete
RBAC-1C: legacy token deprecation and removal plan — pending
```

## Non-goals

- user accounts
- browser sessions or CSRF protection
- per-record ACLs
- OAuth/OIDC
- distributed authorization policy
- remote-by-default binding
