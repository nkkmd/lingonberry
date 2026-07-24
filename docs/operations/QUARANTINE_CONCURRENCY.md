# Quarantine Concurrent Operations

**Status: implemented** | **v1.0 pre-release normative operations contract**

Lingonberry serializes cooperating quarantine mutations and coordinated maintenance operations on one local state directory with a filesystem lock. This contract is intentionally single-host and single-filesystem.

## Lock path and acquisition

```text
<LINGONBERRY_STATE_DIR>/.quarantine-operation.lock
```

Acquisition uses exclusive file creation. When the lock already exists and is not stale, the new operation fails immediately with:

```text
LB_QUARANTINE_BUSY
```

There is no wait queue, lease renewal, fairness guarantee, implicit retry, or backoff policy. Callers must decide whether and when to retry.

The lock implementation creates the state directory when necessary. Failure to create, inspect, write, synchronize, remove, or otherwise access the lock is reported as an I/O failure rather than treated as successful coordination.

## Operation metadata

The lock contains bounded operational metadata only:

```text
operation=<ASCII identifier, at most 64 characters>
pid=<process id>
acquiredAt=<Unix seconds>
```

The operation identifier may contain ASCII alphanumeric characters, `-`, `_`, and `.`. Invalid identifiers fail with `LB_QUARANTINE_LOCK`.

The lock must not contain bearer tokens, request payloads, quarantine identifiers, operator names, annotations, rejection notes, or other lifecycle data. The PID is diagnostic metadata; the implementation does not use it as a live-process ownership proof.

## Covered operations

The common lock is acquired by checked-in mutation and maintenance paths that call `acquire_quarantine_lock`, including:

- quarantine record append;
- promotion resolution append;
- annotation append;
- manual dismissal;
- permanent rejection;
- administrative authentication-failure audit append;
- active-ledger index construction;
- immutable ledger rotation;
- quarantine backup export;
- restore writes in the destination state directory; and
- replacement, compaction, cleanup, or related lifecycle operations where the checked-in implementation explicitly uses the same lock.

The exact operation identifier is implementation-defined but must satisfy the bounded metadata rules.

A document must not infer lock coverage merely because two commands modify related data. Coverage exists only where the checked-in path acquires the common lock for the relevant state directory.

## Read-only paths

Read-only list, get, status, metrics, backup verification, planning, preview, and dry-run paths generally do not acquire the mutation lock unless their implementation explicitly says otherwise.

Consequences:

- a read may observe state before or after one atomic file publication;
- a multi-file read is not automatically a transactional snapshot;
- archive-aware readers verify the segment manifest and files but do not thereby fence a concurrent writer; and
- operators must establish a maintenance window when a stable cross-ledger snapshot is required.

The absence of a lock on a read-only path is not permission to run it concurrently with destructive maintenance when the corresponding runbook requires quiescence.

## Terminal lifecycle races

Promotion, dismissal, and permanent rejection acquire the common lock and re-read the relevant terminal ledgers while holding it. Cooperating processes that use the same state path and lock implementation therefore cannot successfully commit one quarantine record into two incompatible terminal states through those supported paths.

The persistent ledgers remain authoritative. The lock does not repair pre-existing corruption. Duplicate terminal events, malformed JSONL, or conflicting evidence already present in active or archived ledgers must still fail according to the relevant lifecycle reader.

Idempotent retries may return an existing event where the specific lifecycle contract defines that behavior. Idempotency is not supplied by the lock itself.

## Lock lifetime and release

The lock guard removes the file when the guard leaves scope. This is best-effort cleanup: removal errors during guard destruction are not surfaced to the completed operation.

A crash, forced termination, host failure, or process abort can leave the lock file behind. Operators must therefore treat the stale-lock mechanism as crash recovery, not as proof that the prior operation completed or rolled back safely.

## Stale-lock recovery

The stale threshold is:

```text
15 minutes
```

During acquisition, the implementation may remove one stale lock and retry acquisition once. Staleness is determined from a parseable `acquiredAt` value when present; otherwise filesystem modification time is used.

This is time-based recovery only. It does not verify that:

- the recorded PID is absent;
- the previous process is no longer running;
- the previous operation was bounded to less than 15 minutes;
- filesystem clocks are correct; or
- the protected data is internally consistent.

Before relying on automatic stale removal or manually deleting a lock, confirm that no process is still operating on the state directory and inspect the relevant ledgers, segment manifest, temporary files, backup state, or replacement transaction state.

A legitimate operation that runs longer than the stale threshold can be disrupted by another cooperating process attempting acquisition. Long-running maintenance must therefore be performed with all competing writers quiesced; the stale threshold is not a renewable lease.

## Backup, restore, and rotation interaction

Backup export locks the source state directory while it verifies and copies managed state. Restore verifies the backup before acquiring the destination lock, then checks destination conflicts and writes restored files.

Ledger rotation acquires the lock, requires a fresh active-ledger index, publishes an immutable segment and updated manifest, and verifies ordered-stream equivalence before returning success.

These operations coordinate only with other paths using the same lock. They do not prevent:

- manual edits;
- writes by an older or modified Lingonberry binary that omits the lock;
- writes through a different path alias or mount that does not share exclusive-create semantics as expected; or
- remote-host races on an unsupported shared filesystem.

## Containers and filesystem identity

Independent containers coordinate only when they resolve `LINGONBERRY_STATE_DIR` to the same underlying filesystem directory and the filesystem correctly implements exclusive create and deletion semantics.

Do not run multiple writable relay, scheduler, administrator, or maintenance instances against one state directory merely because they share a volume. The v1.0 contract provides no container-orchestrator fencing, leader election, distributed lease, or split-brain recovery.

Symlink, bind-mount, network-filesystem, and path-alias arrangements require operator validation. The implementation does not canonicalize all deployment paths into a global lock identity.

## Operational procedure for `LB_QUARANTINE_BUSY`

1. Do not delete the lock immediately.
2. Read the bounded metadata:

   ```bash
   cat "$LINGONBERRY_STATE_DIR/.quarantine-operation.lock"
   ```

3. Confirm whether the recorded operation and process may still be active.
4. Check service, scheduler, administrator listener, and operator-command activity.
5. If a crash occurred, inspect operation-specific recovery evidence before retrying.
6. Allow stale recovery only after confirming no legitimate long-running operation remains.
7. After acquisition succeeds, run the verification required by the operation-specific runbook.

A persistent lock older than 15 minutes may be removed automatically during the next acquisition attempt. That automatic action does not itself constitute recovery evidence.

## Evidence requirements

For maintenance or qualification evidence, record:

- application commit and binary identity;
- resolved state-directory path and filesystem or mount identity;
- lock metadata observed before the operation, when applicable;
- command, start and completion timestamps, and complete output;
- any stale-lock removal decision and the basis for determining the previous owner was inactive;
- operation-specific verification output; and
- post-operation status, metrics, backup, segment, or transaction checks.

Ordinary CI success or one successful local concurrency test does not establish distributed safety, production filesystem semantics, formal soak completion, or privileged reference-host qualification.

## Non-goals and prohibited assumptions

The v1.0 lock is not:

- a distributed lock;
- multi-node consensus;
- leader election;
- a renewable lease;
- a network-filesystem lease;
- a database transaction spanning all quarantine files;
- protection against manual file edits;
- protection against non-cooperating or older binaries;
- proof that a stale owner is dead;
- automatic interrupted-operation recovery; or
- authorization to share one writable state directory across hosts.

One writable state directory must not be placed behind multiple hosts or independent active writers unless a separate, implemented, documented, and qualified distributed coordination design is introduced.

## Release boundary

This documentation normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence remains unperformed, privileged reference-host qualification/rehearsal remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.