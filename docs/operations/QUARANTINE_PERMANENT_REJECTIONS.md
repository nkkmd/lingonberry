# Quarantine Permanent Rejection Contract

**Status: implemented v1.0 pre-release contract** | **Last reviewed: 2026-07-24**

This document defines the implemented operator-controlled terminal rejection of a quarantine record. A permanent rejection is an explicit administrative decision recorded in an append-only managed ledger. It is distinct from a transient validation or acceptance-policy `Rejected` result.

## 1. Scope and lifecycle semantics

A permanent rejection applies only to an existing pending quarantine record.

The implementation enforces these rules:

- a promoted record cannot be permanently rejected;
- a dismissed record cannot be permanently rejected;
- a record with an existing permanent rejection returns that existing event unchanged;
- one permanent-rejection event is allowed per quarantine record;
- there is no reopen, undo, update, or delete operation;
- the original quarantine record, annotations, and other ledgers are not rewritten;
- a permanent rejection does not physically delete the quarantined payload.

Repeated requests are idempotent only after the existing event is found successfully. A duplicate event already present in the managed ledger is corruption, not a second valid decision.

## 2. Persistent managed ledger

The active ledger name is:

```text
quarantine-rejections.jsonl
```

It is resolved under the configured runtime state directory through the managed-ledger path resolver. The effective active path is normally equivalent to:

```text
<state-dir>/quarantine-rejections.jsonl
```

Each line is one canonical JSON event:

```json
{
  "id": "lb:qr:<seconds>-<nanoseconds>",
  "quarantineId": "lb:q:...",
  "rejectedAt": "<seconds>.<nanoseconds>Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_PERMANENTLY_REJECTED",
  "note": "known prohibited content"
}
```

Required string fields are:

- `id`
- `quarantineId`
- `rejectedAt`
- `operator`
- `reasonCode`
- `note`

The reader processes both active and archived managed-ledger segments. Rotation does not make an earlier rejection disappear, and idempotency remains effective after rotation.

Malformed JSON, a non-object event, a missing required string, or more than one event for the same quarantine ID causes a corruption error. The implementation does not silently skip damaged entries.

## 3. Validation and append behavior

The append operation acquires the local quarantine lock named `quarantine-permanently-reject` before checking state and writing the event.

The implementation rejects:

- an unknown quarantine ID;
- a promoted record;
- a dismissed record;
- an empty operator after trimming;
- an empty note after trimming;
- any reason code other than `LB_OPERATOR_PERMANENTLY_REJECTED`.

`operator` and `note` are trimmed before storage. The reason code is bounded by implementation and is not caller-extensible in v1.

The lock is a local filesystem coordination mechanism. It does not provide distributed locking, multi-node consensus, or replicated terminal-state arbitration.

## 4. CLI surface

Create or retrieve the idempotent permanent rejection:

```bash
lingonberry-relay quarantine-permanently-reject \
  <quarantine-id> \
  <operator> \
  <note>
```

List all permanent rejections or filter by quarantine ID:

```bash
lingonberry-relay quarantine-permanent-rejections
lingonberry-relay quarantine-permanent-rejections <quarantine-id>
```

The CLI fixes the reason code to:

```text
LB_OPERATOR_PERMANENTLY_REJECTED
```

CLI access is a local operator surface. It does not apply the admin HTTP bearer-token RBAC layer by itself; host access and executable permissions remain deployment responsibilities.

## 5. Authenticated administrator HTTP surface

The routes are available only on `serve-admin-http`:

```text
GET  /v1/quarantine/<quarantine-id>/permanent-rejection
POST /v1/quarantine/<quarantine-id>/permanent-rejection
```

The public relay listener returns `404` for these administrative paths.

Authorization follows the administrator HTTP contract:

- observer, reviewer, and operator credentials may read the event;
- only the operator role may create the event;
- missing or invalid credentials return `401`;
- a valid credential without permission returns `403`;
- unauthorized mutation bodies are denied before body parsing.

Example operator request:

```bash
curl -sS \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_OPERATOR_TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"operator":"operator-name","note":"known prohibited content"}' \
  http://127.0.0.1:8788/v1/quarantine/lb:q:123/permanent-rejection
```

The request object requires non-empty string fields `operator` and `note`. The HTTP handler supplies the fixed bounded reason code.

The `operator` field is caller-supplied administrative metadata. It is not cryptographically derived from, or automatically bound to, the bearer credential, OS account, or a human identity.

## 6. Promotion and queue behavior

A permanently rejected record is terminal for the implemented quarantine lifecycle:

- it is excluded from the default pending quarantine list;
- it is excluded from the batch-promotion scan;
- direct CLI promotion is rejected;
- direct authenticated HTTP promotion returns a conflict;
- the original quarantine payload remains available through administrative evidence paths.

A transient validation or acceptance-policy rejection during a promotion attempt remains a separate batch outcome. It does not automatically create a permanent-rejection event.

## 7. Status and metrics

Quarantine status exposes permanent-rejection state through fields including:

```text
permanentlyRejected
latestPermanentlyRejectedAt
```

The Prometheus state gauge includes:

```text
lingonberry_quarantine_records{state="permanently_rejected"}
```

Metrics and status counts are derived operational views. They do not replace the append-only event as the authoritative decision record.

## 8. Audit and evidence boundaries

The permanent-rejection event records the decision metadata and remains separate from:

- admin authentication and authorization failure events;
- operator annotations;
- promotion resolutions;
- dismissals;
- source validation and acceptance-policy reports.

The admin-auth audit ledger records authentication and authorization failures, not a complete ledger of successful permanent-rejection actions. The permanent-rejection event itself is therefore required evidence for the successful terminal decision.

Operators must not place bearer tokens, credentials, private keys, unnecessary personal data, or unbounded sensitive payloads in `operator` or `note`.

## 9. Rotation, backup, and recovery

A complete backup or migration of permanent-rejection state must preserve:

- the active `quarantine-rejections.jsonl` file;
- archived managed-ledger segments containing rejection events;
- the managed-ledger index and metadata needed to resolve those segments;
- the quarantine records referenced by the events.

Copying only the currently active JSONL file can lose earlier terminal decisions after rotation. A volume snapshot is not considered a verified backup until restore and ledger-read verification succeed.

## 10. Operational verification

Before enabling administrative mutation in a deployment, verify at minimum:

1. the administrator listener is not exposed through the public relay route;
2. observer and reviewer credentials receive `403` for `POST`;
3. an operator credential can create a rejection for a pending record;
4. a repeated request returns the original event without appending a duplicate;
5. promoted and dismissed records produce conflicts;
6. direct promotion of the rejected record is blocked;
7. the event remains readable after managed-ledger rotation;
8. backup and restore preserve the event and terminal behavior.

Passing repository CI or the documentation walkthrough does not by itself constitute privileged reference-host qualification, backup/restore qualification, or formal soak evidence.

## 11. Non-goals for v1

The implementation does not guarantee:

- reopening or undoing a permanent rejection;
- deleting or editing rejection events;
- caller-defined rejection reason codes;
- automatic permanent rejection from validation failures;
- cryptographic binding between `operator` and the bearer credential;
- dynamic user or role administration;
- distributed locking or multi-node consensus;
- automatic cross-node replication of the rejection ledger;
- automatic retention, compaction, or secure deletion of rejected payloads.

## 12. Related contracts

- [`QUARANTINE_ADMIN_HTTP.md`](./QUARANTINE_ADMIN_HTTP.md)
- [`QUARANTINE_ANNOTATIONS.md`](./QUARANTINE_ANNOTATIONS.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)
- [`QUARANTINE_LEDGER_ROTATION.md`](./QUARANTINE_LEDGER_ROTATION.md)
- [`QUARANTINE_DISMISSAL.md`](./QUARANTINE_DISMISSAL.md)
