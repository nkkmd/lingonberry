# Quarantine Manual Dismissals

**Status: implemented** | **Last updated: 2026-07-12**

Manual dismissal removes a pending quarantine record from the normal promotion scan without deleting or mutating the original quarantine record.

## Persistent ledger

```text
<LINGONBERRY_STATE_DIR>/quarantine-dismissals.jsonl
```

Each line is one append-only lifecycle event:

```json
{
  "id": "lb:qd:...",
  "quarantineId": "lb:q:...",
  "dismissedAt": "...Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_DISMISSED",
  "note": "duplicate external submission"
}
```

## Lifecycle semantics

- Only an existing pending record can be dismissed.
- A promoted record is rejected with `LB_QUARANTINE_ALREADY_PROMOTED`.
- One active dismissal exists per quarantine record.
- Repeating the same operation is idempotent and returns the original dismissal event.
- Undo/reopen is not implemented.
- The original `quarantine.jsonl` record and all annotation events remain unchanged.
- Duplicate dismissal events already present in the ledger are treated as corruption rather than silently collapsed.

## CLI

Dismiss a pending record:

```bash
cargo run -p lingonberry-relay -- \
  quarantine-dismiss <quarantine-id> <operator> <note>
```

List all dismissal events:

```bash
cargo run -p lingonberry-relay -- quarantine-dismissals
```

List the dismissal for one record:

```bash
cargo run -p lingonberry-relay -- quarantine-dismissals <quarantine-id>
```

The CLI fixes the bounded reason code to `LB_OPERATOR_DISMISSED`; operators provide a required non-empty note.

## Promotion behavior

The default quarantine listing used by batch promotion excludes dismissed records. Therefore scheduled and manually invoked batch promotion do not scan dismissed records.

The dismissal ledger is checked during status and metrics reconstruction. Corrupt JSONL and I/O errors fail explicitly.

## Status and metrics

`quarantine-status` and `GET /v1/quarantine-status` include:

```text
dismissed
latestDismissedAt
```

Prometheus output includes:

```text
lingonberry_quarantine_records{state="dismissed"}
```

`pending` excludes both promoted and dismissed records.

## HTTP scope

No dismissal mutation endpoint is added in this version. Administrative HTTP writes remain deferred until authentication, authorization, and network isolation are designed.

## Non-goals

- physical deletion
- undo/reopen
- distributed locking
- retention, rotation, or compaction
- unauthenticated HTTP dismissal API
