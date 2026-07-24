# Quarantine Dismissal Contract

**Status: implemented v1.0 pre-release contract** | **Last reviewed: 2026-07-24**

This document defines the implemented operator-controlled dismissal of a quarantine record. Dismissal is an append-only terminal lifecycle decision that excludes a record from normal promotion processing without deleting or rewriting the original quarantine record.

## 1. Lifecycle semantics

A dismissal applies only to an existing pending quarantine record.

The implementation enforces these rules:

- a promoted record cannot be dismissed;
- a permanently rejected record cannot be dismissed;
- a record with an existing dismissal returns that existing event unchanged;
- one dismissal event is allowed per quarantine record;
- there is no reopen, undo, update, or delete operation;
- the original quarantine record, annotations, and other ledgers are not rewritten;
- dismissal does not physically delete the quarantined payload.

Repeated requests are idempotent only after the existing event is found successfully. More than one dismissal event for the same quarantine ID in the managed ledger is corruption, not a second valid decision.

## 2. Persistent managed ledger

The active ledger name is:

```text
quarantine-dismissals.jsonl
```

It is resolved under the configured runtime state directory through the managed-ledger path resolver. The effective active path is normally equivalent to:

```text
<state-dir>/quarantine-dismissals.jsonl
```

Each line is one canonical JSON event:

```json
{
  "id": "lb:qd:<seconds>-<nanoseconds>",
  "quarantineId": "lb:q:...",
  "dismissedAt": "<seconds>.<nanoseconds>Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_DISMISSED",
  "note": "duplicate external submission"
}
```

Required string fields are:

- `id`
- `quarantineId`
- `dismissedAt`
- `operator`
- `reasonCode`
- `note`

The reader processes both active and archived managed-ledger segments. Rotation does not make an earlier dismissal disappear, and idempotency remains effective after rotation.

Malformed JSON, a non-object event, a missing required string, or more than one event for the same quarantine ID causes a corruption error. Damaged entries are not silently skipped.

## 3. Validation and append behavior

The append operation acquires the local quarantine lock named `quarantine-dismiss` before checking lifecycle state and writing the event.

The implementation rejects:

- an unknown quarantine ID;
- a promoted record with `LB_QUARANTINE_ALREADY_PROMOTED`;
- a permanently rejected record with `LB_QUARANTINE_PERMANENTLY_REJECTED`;
- an empty operator after trimming;
- an empty note after trimming;
- any reason code other than `LB_OPERATOR_DISMISSED`.

`operator` and `note` are trimmed before storage. The reason code is bounded by implementation and is not caller-extensible in v1.

The lock is a local filesystem coordination mechanism. It does not provide distributed locking, multi-node consensus, or replicated terminal-state arbitration.

## 4. CLI surface

Create or retrieve the idempotent dismissal:

```bash
lingonberry-relay quarantine-dismiss \
  <quarantine-id> \
  <operator> \
  <note>
```

List all dismissals or filter by quarantine ID:

```bash
lingonberry-relay quarantine-dismissals
lingonberry-relay quarantine-dismissals <quarantine-id>
```

The CLI fixes the reason code to:

```text
LB_OPERATOR_DISMISSED
```

CLI access is a local operator surface. It does not apply administrator HTTP bearer-token RBAC by itself; host access and executable permissions remain deployment responsibilities.

## 5. Authenticated administrator HTTP surface

The implemented administrator HTTP contract exposes dismissal read and mutation operations only on `serve-admin-http`. The corresponding administrative path must not be exposed by the public relay listener.

Authorization follows the administrator HTTP contract:

- observer, reviewer, and operator credentials may read lifecycle evidence;
- only the operator role may create a dismissal;
- missing or invalid credentials return `401`;
- a valid credential without permission returns `403`;
- unauthorized mutation bodies are denied before body parsing.

The request metadata fields `operator` and `note` are caller-supplied. `operator` is not cryptographically derived from, or automatically bound to, the bearer credential, OS account, or a human identity.

Deployments must use the exact routes implemented by the checked-in administrator HTTP server and must verify them against the frozen candidate before qualification. This document does not authorize exposing administrative routes through the public reverse proxy.

## 6. Promotion and queue behavior

A dismissed record is terminal for the implemented dismissal lifecycle:

- it is excluded from the default pending quarantine list;
- it is excluded from the batch-promotion scan;
- scheduled and manually invoked batch promotion do not scan it;
- direct promotion attempts are rejected;
- the original quarantine payload remains available through administrative evidence paths.

Dismissal is distinct from permanent rejection and from a transient validation or acceptance-policy rejection. One lifecycle decision must not be inferred from another.

## 7. Status and metrics

Quarantine status exposes dismissal state through fields including:

```text
dismissed
latestDismissedAt
```

The Prometheus state gauge includes:

```text
lingonberry_quarantine_records{state="dismissed"}
```

The `pending` state excludes promoted, dismissed, and permanently rejected records. Metrics and status counts are derived operational views; they do not replace the append-only dismissal event as the authoritative decision record.

## 8. Audit and evidence boundaries

The dismissal event records successful decision metadata and remains separate from:

- administrator authentication and authorization failure events;
- operator annotations;
- promotion resolutions;
- permanent-rejection events;
- source validation and acceptance-policy reports.

The administrator-auth audit ledger records authentication and authorization failures, not a complete ledger of successful dismissal actions. The dismissal event itself is therefore required evidence for the successful terminal decision.

Operators must not place bearer tokens, credentials, private keys, unnecessary personal data, or unbounded sensitive payloads in `operator` or `note`.

## 9. Rotation, backup, and recovery

A complete backup or migration of dismissal state must preserve:

- the active `quarantine-dismissals.jsonl` file;
- archived managed-ledger segments containing dismissal events;
- the managed-ledger index and metadata needed to resolve those segments;
- the quarantine records referenced by the events.

Copying only the currently active JSONL file can lose earlier terminal decisions after rotation. A volume snapshot is not considered a verified backup until restore and ledger-read verification succeed.

## 10. Operational verification

Before enabling administrative mutation in a deployment, verify at minimum:

1. the administrator listener is not exposed through the public relay route;
2. observer and reviewer credentials cannot create dismissals;
3. an operator can dismiss a pending record;
4. a repeated request returns the original event without appending a duplicate;
5. promoted and permanently rejected records produce lifecycle conflicts;
6. direct and batch promotion of the dismissed record are blocked;
7. the event remains readable after managed-ledger rotation;
8. backup and restore preserve the event and terminal behavior.

Passing repository CI or the documentation walkthrough does not by itself constitute privileged reference-host qualification, backup/restore qualification, or formal soak evidence.

## 11. Non-goals for v1

The implementation does not guarantee:

- reopening or undoing a dismissal;
- deleting or editing dismissal events;
- caller-defined dismissal reason codes;
- automatic dismissal from validation failures;
- cryptographic binding between `operator` and the bearer credential;
- dynamic user or role administration;
- distributed locking or multi-node consensus;
- automatic cross-node replication of the dismissal ledger;
- automatic retention, compaction, or secure deletion of dismissed payloads.

## 12. Related contracts

- [`QUARANTINE_ADMIN_HTTP.md`](./QUARANTINE_ADMIN_HTTP.md)
- [`QUARANTINE_ANNOTATIONS.md`](./QUARANTINE_ANNOTATIONS.md)
- [`QUARANTINE_PERMANENT_REJECTIONS.md`](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)
- [`QUARANTINE_LEDGER_ROTATION.md`](./QUARANTINE_LEDGER_ROTATION.md)
