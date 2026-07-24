# Quarantine Replacement Cleanup Runbook

**Status: implemented** | **v1.0 pre-release normative operations runbook**

This runbook governs explicit cleanup of non-active quarantine replacement generations and their managed transaction artifacts. Cleanup is exact-subject, proof-bound, same-host locked, operator-triggered, and double opt-in. It is separate from representation replacement and is never scheduled or inferred automatically.

## Safety model

Cleanup has two distinct authorization stages:

1. **Tomb preparation** requires `operator_requested: true` and `irreversible_delete_confirmed: false`.
2. **Irreversible deletion** requires both `operator_requested: true` and `irreversible_delete_confirmed: true`.

Rollback requires an operator request without irreversible-delete confirmation and is available only before deletion has crossed the runbook's irreversible boundary.

A digest file, retention decision, cleanup proof, or completion-evidence record is not operator authorization by itself.

## Preconditions

Before creating a cleanup transaction:

- identify the exact state directory and application build;
- verify the current-generation pointer and active generation;
- inspect replacement generations and transaction workspaces;
- verify replacement completion evidence for each selected terminal transaction;
- bind an explicit evaluation time;
- evaluate the versioned retention policy;
- select exact generation IDs without globs, wildcards, path separators, or duplicate subjects;
- build and verify cleanup plan/proof artifacts;
- confirm the active generation is not selected;
- confirm the minimum previous-committed-generation floor remains satisfied;
- confirm each selected subject is old enough according to durable completion evidence;
- preserve a verified backup and evidence bundle; and
- stop unattended replacement or cleanup invocations.

Any mismatch is a stop condition. Do not attempt automatic repair or broaden the selection.

## Retention policy

The implemented policy identity is:

```text
lingonberry-quarantine-replacement-retention-policy/v1
```

The policy requires:

- at least one previous committed generation to remain;
- an explicit minimum age in seconds;
- explicit enablement for previous committed generations and/or rolled-back generations; and
- a non-empty list of exact selected generation IDs.

The evaluator rejects or marks ineligible subjects for reasons including:

- subject not found;
- active generation;
- classification disabled by policy;
- non-terminal transaction;
- orphan requiring manual review;
- unknown or corrupt state;
- legacy root layout;
- terminal-state mismatch;
- generation not verified;
- durable-age evidence missing;
- minimum age not satisfied; and
- minimum retention floor.

An eligible retention decision is a prerequisite, not deletion authorization.

## Cleanup preview and proof

Build cleanup preview artifacts only from exact selected subjects and verified current state. The preview binds:

- retention decisions;
- state identity;
- runtime fingerprint;
- explicit evaluation time;
- selected subject inputs;
- exact managed-path set; and
- cleanup proof digest.

Verify both artifact integrity and semantic agreement with current state. Preview is read-only and does not itself move or delete files.

The cleanup transaction journal must bind the verified cleanup proof and runtime fingerprint before tomb preparation.

## Transaction preparation

Create the cleanup transaction workspace and journal according to the checked-in command contract. Before moving any path, the execution path acquires the state-directory quarantine operation lock and revalidates:

- cleanup plan/proof artifact pair;
- retention decisions;
- state identity;
- runtime fingerprint;
- evaluation time;
- selected subject inputs; and
- transaction journal binding.

The transaction journal must be in `prepared` state and must match the proof digest and runtime fingerprint. The implementation advances it to `revalidated` before moving approved paths.

## Tomb preparation

Tomb preparation moves only the exact approved managed-path set into the transaction-local tomb area. It must not follow symbolic links or expand the selected subject set.

The operation produces and verifies a sealed inventory and digest. A successful preparation leaves the transaction in the tomb-sealed lifecycle with rollback still available.

Expected evidence includes:

```text
cleanup transaction journal and digest
cleanup plan/proof and digests
tomb inventory and digest
moved-path state
append-only cleanup audit events
```

Do not interpret a missing source path alone as proof that the move succeeded. Verify journal, tomb inventory, and actual filesystem entries together.

## Irreversible deletion

Deletion is a separate invocation and requires explicit irreversible-delete confirmation.

The implementation:

- acquires the same-host quarantine operation lock;
- resumes deletion from the durable transaction state;
- processes managed paths in deterministic order;
- records durable per-path progress; and
- reaches `committed` only when all approved tomb entries are processed.

Do not add new subjects after tomb sealing. Do not delete paths outside the sealed inventory.

If deletion fails while the journal is in `deleting`, the implementation attempts to record `recovery-required` and then `partially-deleted`. Preserve all artifacts and escalate for manual recovery.

## Rollback

Rollback is permitted only while the transaction and tomb contract still allow restoration and before irreversible deletion has started.

Rollback:

- requires an operator request without delete confirmation;
- acquires the state-directory operation lock;
- verifies transaction and tomb state; and
- restores the exact tombed managed paths according to the sealed inventory.

Expected terminal state:

```text
rolled-back
```

After final deletion begins, rollback must not be advertised. The supported path is resume or manual recovery.

## Recovery classifications

Use the durable journal state, sealed inventory, path-level progress, and actual filesystem state together.

- `prepared`: verify proof/runtime binding before revalidation.
- `revalidated`: inspect before tomb movement or resume the supported preparation path.
- `renaming-to-tomb`: compare every source and tomb entry; do not infer completion from absence.
- `tomb-sealed`: rollback remains possible or a separately confirmed delete may proceed.
- `deleting`: resume from the durable path frontier.
- `recovery-required`: preserve artifacts and establish a consistent resume plan.
- `partially-deleted`: preserve all remaining evidence and escalate for manual recovery.
- `committed`: deletion completed; retain the workspace and evidence.
- `rolled-back`: restoration completed; retain the workspace and evidence.

Contradictory journal, inventory, progress, or filesystem state must fail closed.

## Concurrency boundary

Preparation, deletion, and rollback acquire the same-host quarantine operation lock. The lock coordinates only cooperating processes using the same resolved local state-directory path.

The cleanup system does not provide:

- distributed locking;
- cross-host atomicity;
- network-filesystem leases;
- protection against manual file edits;
- protection against binaries that bypass the lock; or
- safe shared writable state across independent containers or hosts.

## Evidence preservation

Retain:

- application commit and binary identity;
- state-directory identity and runtime fingerprint;
- active-generation pointer verification;
- replacement completion evidence and digests;
- retention policy and decision report;
- cleanup plan/proof and digests;
- cleanup transaction journal and digest;
- sealed tomb inventory and digest;
- durable path-level deletion progress;
- terminal state;
- append-only replacement/cleanup audit records;
- operator requests and irreversible confirmation; and
- incident notes and recovery decisions.

Terminal cleanup workspaces are evidence. Do not remove them merely because the transaction is `committed` or `rolled-back` unless a separate verified retention policy explicitly governs those workspaces.

## Observability

Metrics may use only bounded labels such as:

```text
state
operation
phase
outcome
error_family
```

Do not place paths, transaction IDs, generation IDs, digests, record IDs, or free-form errors in metric labels.

Status and metrics are diagnostic views. They do not replace verification of the journal, proof, tomb inventory, or filesystem state.

## Incident procedure

1. Stop further cleanup and replacement operator invocations.
2. Preserve the cleanup transaction workspace byte-for-byte.
3. Record the journal state and bounded error family.
4. Verify journal, plan/proof, and tomb-inventory digest pairs.
5. Compare actual source/tomb entries with the sealed inventory without following symbolic links.
6. Identify the last durable path-progress frontier.
7. Decide between supported resume, supported rollback, or manual recovery.
8. Record every operator decision and avoid unverified ad hoc deletion.

## Explicit non-goals

The v1.0 cleanup workflow does not provide:

- scheduled or unattended cleanup;
- wildcard or policy-wide implicit selection;
- automatic orphan deletion;
- deletion of the active generation;
- automatic repair of contradictory state;
- rollback after irreversible deletion starts;
- automatic removal of terminal transaction workspaces;
- cryptographic authorization through digest files;
- distributed cleanup coordination; or
- proof of formal soak or privileged reference-host qualification.

## Related documents

- [`QUARANTINE_REPLACEMENT_POLICY.md`](./QUARANTINE_REPLACEMENT_POLICY.md)
- [`QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md`](./QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md)
- [`QUARANTINE_CONCURRENCY.md`](./QUARANTINE_CONCURRENCY.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)

## Release boundary

This normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence remains unperformed, privileged reference-host qualification/rehearsal remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.
