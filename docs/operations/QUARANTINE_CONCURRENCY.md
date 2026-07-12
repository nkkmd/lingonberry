# Quarantine Concurrent Operations

**Status: implemented** | **Last updated: 2026-07-12**

Lingonberry serializes quarantine mutations and backup writes on one host with a state-directory-wide filesystem lock.

## Lock file

```text
<LINGONBERRY_STATE_DIR>/.quarantine-operation.lock
```

The lock is created with exclusive `create_new` semantics. A second operation fails closed with:

```text
LB_QUARANTINE_BUSY
```

There is no indefinite wait queue or implicit retry.

## Covered operations

The same lock protects:

- quarantine record append
- promotion resolution append
- operator annotation append
- manual dismissal
- permanent rejection
- admin authentication failure audit append
- quarantine backup export
- restore writes in the destination state directory

Read-only list, get, status, metrics, backup verification, and dry-run evaluation do not acquire the lock.

## Terminal lifecycle races

Promotion, dismissal, and permanent rejection re-read terminal ledgers while holding the same lock. Therefore a record cannot be committed into two terminal states by cooperating processes on the same host.

The persistent ledgers remain authoritative. Duplicate terminal events already present in a ledger are still treated as corruption.

## Lock metadata

The lock file contains only bounded operational metadata:

```text
operation=<bounded identifier>
pid=<process id>
acquiredAt=<unix seconds>
```

It does not contain bearer tokens, request payloads, quarantine IDs, operator names, or notes.

## Stale lock recovery

A lock older than 15 minutes is considered stale and may be removed once during acquisition. This supports recovery after an abnormal process exit.

Before relying on stale recovery, confirm that no long-running operation is still active. Normal operations remove the lock when their guard leaves scope.

## Backup interaction

Backup export acquires the same source state-directory lock as ledger mutations. This prevents cooperating relay, scheduler, CLI, and admin processes from changing managed ledgers during export.

Restore verifies the backup first, then locks the destination state directory before checking conflicts and writing files.

## Boundaries

This mechanism is valid only for cooperating processes that see the same local filesystem path and use Lingonberry's lock API. It is not:

- distributed locking
- multi-node consensus
- a network filesystem lease
- protection against manual file edits
- protection against older Lingonberry binaries that do not acquire the lock

Do not place one writable state directory behind multiple hosts or independent containers unless a separate distributed coordination design is introduced.

## Troubleshooting

Inspect the lock:

```bash
cat "$LINGONBERRY_STATE_DIR/.quarantine-operation.lock"
```

Do not delete a fresh lock without confirming the owning process is gone. A persistent `LB_QUARANTINE_BUSY` after a crash will recover automatically once the stale threshold is exceeded.
