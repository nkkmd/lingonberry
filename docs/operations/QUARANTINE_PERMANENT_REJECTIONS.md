# Quarantine Permanent Rejections

**Status: implemented** | **Last updated: 2026-07-12**

Permanent rejection is an explicit operator decision that moves a pending quarantine record into the terminal `permanently-rejected` state. It is not created automatically from a transient validation `Rejected` outcome.

## Fixed semantics

- only pending records are eligible
- promoted and dismissed records are rejected as conflicts
- one event per quarantine record
- repeated requests are idempotent
- no reopen or undo
- no physical deletion
- original quarantine records and annotations remain unchanged

## Persistent ledger

```text
<LINGONBERRY_STATE_DIR>/quarantine-rejections.jsonl
```

```json
{
  "id": "lb:qr:...",
  "quarantineId": "lb:q:...",
  "rejectedAt": "...Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_PERMANENTLY_REJECTED",
  "note": "known prohibited content"
}
```

Duplicate events already present in the ledger are treated as corruption.

## CLI

```bash
lingonberry-relay quarantine-permanently-reject \
  <quarantine-id> <operator> <note>

lingonberry-relay quarantine-permanent-rejections [quarantine-id]
```

The CLI fixes the bounded reason code to `LB_OPERATOR_PERMANENTLY_REJECTED`.

## Authenticated admin HTTP

```text
POST /v1/quarantine/<quarantine-id>/permanent-rejection
GET  /v1/quarantine/<quarantine-id>/permanent-rejection
```

Example:

```bash
curl -sS \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"operator":"operator-name","note":"known prohibited content"}' \
  http://127.0.0.1:8788/v1/quarantine/lb:q:123/permanent-rejection
```

These routes are available only through `serve-admin-http`. The public listener returns `404` for the same paths.

## Promotion behavior

Permanently rejected records are excluded from the default quarantine list and batch promotion scan. Direct CLI and authenticated HTTP promotion attempts are rejected.

## Status and metrics

Status fields:

```text
permanentlyRejected
latestPermanentlyRejectedAt
```

Prometheus gauge:

```text
lingonberry_quarantine_records{state="permanently_rejected"}
```

The batch report's transient `rejected` count remains a separate concept and does not create persistent rejection events.
