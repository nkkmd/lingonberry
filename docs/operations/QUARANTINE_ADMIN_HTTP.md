# Quarantine Admin HTTP Isolation

**Status: implemented** | **Last updated: 2026-07-12**

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

The admin listener defaults to loopback. Startup fails when `LINGONBERRY_ADMIN_TOKEN` is missing or empty.

## Authentication

Send the token as a bearer credential:

```bash
curl -sS \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_TOKEN" \
  http://127.0.0.1:8788/v1/quarantine-status
```

Missing and invalid credentials receive the same response:

```text
401 Unauthorized
```

The initial authorization model has one admin token and one effective role. Fine-grained RBAC is not implemented.

## Protected routes

```text
GET  /v1/quarantine
GET  /v1/quarantine/<quarantine-id>
POST /v1/quarantine/<quarantine-id>/promote
POST /v1/quarantine/promote-batch
GET  /v1/quarantine-resolutions
GET  /v1/quarantine-status
GET  /metrics
POST /v1/quarantine/<quarantine-id>/annotations
GET  /v1/quarantine/<quarantine-id>/annotations
```

Manual dismissal remains CLI-only in this version.

## Authentication audit

Failed authentication attempts are appended to:

```text
<LINGONBERRY_STATE_DIR>/admin-auth-audit.jsonl
```

Each event contains only bounded operational metadata:

```json
{
  "attemptedAt": "...Z",
  "remoteAddr": "127.0.0.1:12345",
  "method": "GET",
  "path": "/v1/quarantine-status",
  "outcomeCode": "LB_ADMIN_AUTH_FAILED"
}
```

The ledger never stores bearer tokens, request bodies, annotation notes, or quarantine payloads. Audit append failures are not silently ignored.

## systemd

Template:

```text
deploy/systemd/lingonberry-admin-http.service
```

Create the environment file with restrictive permissions:

```bash
sudo install -d -m 0750 /etc/lingonberry
sudo sh -c 'printf "%s\n" "LINGONBERRY_ADMIN_TOKEN=<long-random-secret>" > /etc/lingonberry/admin-http.env'
sudo chmod 0600 /etc/lingonberry/admin-http.env
sudo chown root:root /etc/lingonberry/admin-http.env
```

Install and start:

```bash
sudo install -m 0644 deploy/systemd/lingonberry-admin-http.service \
  /etc/systemd/system/lingonberry-admin-http.service
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-admin-http.service
```

The template binds only to `127.0.0.1:8788`. Remote access should be provided through a separately authenticated and TLS-terminated administrative channel, not by changing the default listener to a public address.

## Non-goals

- TLS termination
- multiple roles or per-route permissions
- distributed rate limiting
- browser session or CSRF protection
- remote-by-default binding
