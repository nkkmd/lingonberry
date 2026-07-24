# Quarantine Replacement Recovery Runbook

**Status: normative v1.0 pre-release operator procedure** | **Last reviewed: 2026-07-24**

This runbook defines recovery for a quarantine replacement transaction after interruption, an injected failure, or an explicit `recovery-required` result. It applies only to the implemented generation-directory replacement workflow, a verified policy-v2 replacement proof, and a verified archive-inclusive backup v2.

Recovery does not broaden the approved replacement semantics. It must not rewrite immutable evidence, mutate archive segments, perform retention deletion, deduplicate records, collapse events, resolve conflicts, migrate schemas, or move records across archive boundaries.

## 1. Recovery authority and safety boundary

The recovery commands operate on one existing transaction workspace:

```text
<transaction-dir>/
```

The workspace basename is the transaction ID used by the apply command. It must remain unchanged throughout inspection and recovery.

Recovery may use only:

- the transaction journal and its bound inputs;
- the staged and sealed generation artifacts already associated with the transaction;
- the current-generation pointer and publication intent;
- the verified backup v2 and replacement proof bound by the journal;
- the current runtime state directory.

Recovery must not infer success from the existence of a generation directory, a pointer file, a temporary file, or a partially written completion artifact. `committed` and `rolled-back` are the only terminal journal states.

The implementation uses the same host-local operation lock as apply and other mutating quarantine maintenance operations. This lock does not provide distributed locking, multi-host coordination, leader election, or network-filesystem consensus.

## 2. Preserve the incident state first

Before running a recovery mutation:

1. stop the public relay listener, administrator listener, scheduler, maintenance commands, and operator processes that can write the same state directory;
2. prevent another host or container from using the same state directory through an uncoordinated mount;
3. preserve command output, stderr, service logs, and filesystem errors from the interrupted operation;
4. retain the state directory, transaction workspace, verified backup, and verified proof without editing them;
5. record the binary version or commit, host identity, state-directory path, transaction-directory path, and current time;
6. do not remove temporary, publication, generation, pointer, journal, digest, or completion-evidence files.

Do not repair the state by hand before classification. Manual edits can destroy the evidence required to distinguish a resumable transaction from a corrupt one.

## 3. Inspect status without mutation

Run:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-status \
  /var/lib/lingonberry-maintenance/tx-20260714-001
```

The command emits canonical JSON when the transaction can be classified. Relevant classifications include transaction states and recovery-oriented classifications such as:

```text
prepared
writing
staged
verified
resumable-before-switch
resumable-after-switch
recovery-required
committed
rolled-back
```

The exact classification is derived from the journal, bound inputs, staged or sealed generation, publication intent, current-generation pointer, completion evidence, index, and archive state. Operators must use the emitted classification and error code; they must not substitute a classification based only on directory contents.

`replacement-status` itself can fail. A journal, pointer, digest, manifest, generation, or input-binding contradiction is reported as corrupt rather than converted into a successful status object. A status failure is not authorization to run resume or rollback blindly.

Status operations are recorded in the replacement-operation audit ledger. A corrupt status attempt is recorded as a rejected `status-corrupt` event. Audit output is diagnostic evidence; it does not repair or authorize the transaction.

## 4. Decide between resume, rollback, and manual investigation

Use the following decision boundary:

| Observed result | Permitted action |
|---|---|
| `committed` | No resume or rollback. Verify the active generation and retain completion evidence. |
| `rolled-back` | No further mutation. Verify the restored generation and retain rollback evidence. |
| Explicitly resumable classification | Resume is permitted after all bound inputs remain available and unchanged. |
| Explicitly rollback-capable classification | Rollback is permitted after the previous pointer or legacy-root state and bound backup can be verified. |
| `recovery-required` with a valid, classifiable journal and filesystem state | Use the recovery mode supported by the classification; do not assume both modes are valid. |
| Corrupt, contradictory, or unclassifiable state | Preserve evidence and stop. Manual investigation is required. |

`recovery-required` means that the operation did not complete safely and requires recovery handling. It does not, by itself, mean that resume is safe. The journal and filesystem classification determine whether resume, rollback, or neither is allowed.

A committed transaction is terminal. Reverting a committed generation requires a new independently verified replacement transaction. Do not edit or delete the current-generation pointer to simulate rollback.

## 5. Resume an eligible transaction

Run resume only when status identifies a resumable state:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-recover \
  /var/lib/lingonberry-maintenance/tx-20260714-001 \
  --resume
```

Resume acquires the host-local operation lock and re-evaluates the durable transaction state. Depending on the last durable transition, it may re-run only idempotent unfinished steps, including:

- verifying the journal and transaction input binding;
- verifying backup v2 and the bound replacement proof;
- verifying staging artifacts or the sealed generation;
- verifying publication intent and current-generation pointer consistency;
- completing an eligible pointer publication step;
- rebuilding or verifying the derived index;
- verifying archive segments;
- completing the durable `committed` transition and completion evidence when all required checks succeed.

Resume must not repeat a non-idempotent mutation merely because an expected temporary file is absent. It must not republish a different generation or accept an unrelated current pointer.

A successful resume returns canonical JSON. The required terminal result is:

```text
state: committed
classification: committed
activeGeneration: <transaction-id>
```

Repeating resume against a correctly committed transaction must not perform a second rewrite. The command may return the existing terminal result after verification.

## 6. Roll back an eligible transaction

Run rollback only when the transaction has not reached `committed` and status identifies rollback as valid:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-recover \
  /var/lib/lingonberry-maintenance/tx-20260714-001 \
  --rollback
```

Rollback acquires the host-local operation lock and must restore the exact previous activation state recorded by the transaction:

- when a previous generation pointer existed, restore that exact pointer;
- when no pointer existed before apply, remove the transaction publication pointer and reactivate the legacy root-ledger layout;
- verify the restored pointer or legacy-root state;
- rebuild and verify the derived index;
- verify archive segments;
- record `rolled-back` only after restoration verification succeeds.

Rollback must not restore from an arbitrary backup, choose a convenient generation, edit ledger files in place, or delete the failed generation as part of proving rollback. Retention and cleanup are separate operations.

A missing or mismatched previous pointer, bound backup, journal digest, or input binding causes rollback to fail closed.

## 7. Audit behavior

`replacement-apply`, `replacement-status`, and `replacement-recover` append operation events to the quarantine replacement audit ledger.

For resume and rollback:

- an operation-started event is appended before the operation;
- `committed` produces a committed success event;
- `rolled-back` produces a rolled-back success event;
- a returned `recovery-required` state is recorded as a failed recovery-required event;
- a rejected operation records its stable error code;
- an audit append failure is surfaced with the operation error rather than silently ignored.

The audit ledger records operation outcome and classification. It is not a substitute for the transaction journal, current pointer, completion evidence, backup, proof, or filesystem artifacts.

## 8. Fail-closed conditions

Stop and preserve evidence when any of the following occurs:

- status cannot classify the transaction;
- journal, journal digest, or transition validation fails;
- backup or proof no longer matches the journal binding;
- required bound inputs are missing;
- runtime fingerprint is stale at a publication boundary;
- staging or publication directories contain missing or unexpected files;
- a staged or materialized ledger digest differs from its manifest;
- immutable evidence differs byte-for-byte;
- the generation manifest or generation digest is invalid;
- publication intent conflicts with the current pointer;
- the current pointer identifies an unrelated transaction or generation digest;
- the pointer exists but its generation directory is missing or invalid;
- archive segment or segment-manifest verification fails;
- completion evidence conflicts with the journal state;
- the derived index cannot be rebuilt or verified;
- a state transition is skipped, duplicated, or contradictory;
- the filesystem cannot provide the required local rename or durability behavior.

Do not:

- copy staged ledgers over active paths;
- modify journal JSON or digest files;
- edit the current-generation pointer;
- delete a generation, transaction workspace, or temporary artifact to make status pass;
- regenerate a proof against changed runtime state and reuse the old transaction;
- treat an audit success line as proof that the active generation is valid.

## 9. Evidence bundle

Retain at least:

```text
transaction journal and digest
transaction input binding
replacement-operation audit events
staging manifest and staged ledgers
sealed generation manifest and digest
publication preparation pointer
publication intent and digest
completion evidence and digest, when present
verified backup v2 and manifest
verified replacement plan, proof, and digests
current-generation pointer
previous-pointer evidence
materialized transaction generation directory
ledger index
segment manifest and referenced immutable segments
command stdout and stderr
service and filesystem logs
binary version or commit identity
host, state-directory, and transaction-directory identity
```

Do not include administrator bearer tokens, signing secrets, TLS private keys, or unrelated payload data in the incident bundle.

## 10. Post-recovery verification

After a successful resume or rollback, keep writers quiescent and run:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance verify-index

LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance verify-segments

LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-status \
  /var/lib/lingonberry-maintenance/tx-20260714-001
```

For a committed transaction, status must identify the transaction generation as active. For a rolled-back transaction, status must identify `rolled-back` and the active state must match the exact pre-transaction pointer or legacy-root layout.

Also verify the public and administrator runtime paths that consume quarantine state before restoring traffic. A successful maintenance command does not by itself prove that an independently running process is using the intended state directory.

## 11. Cleanup boundary

Recovery completion does not authorize deletion.

Do not delete:

- the previous generation;
- the transaction generation;
- the transaction workspace;
- the verified backup or proof;
- completion, rollback, or audit evidence;
- legacy root ledgers;
- orphan-looking directories.

Cleanup requires the separate retention policy, generation inspection, cleanup plan/proof, retention floor, and deletion acknowledgement defined by the replacement cleanup runbook. A pointer switch, `committed` state, or elapsed wall-clock time alone is insufficient cleanup authorization.

## 12. Non-goals

This runbook does not provide:

- automatic repair of corrupt or contradictory state;
- distributed recovery coordination;
- remote backup retrieval;
- cryptographic authentication of FNV-1a digests;
- operator identity binding beyond local execution and caller-provided evidence;
- retention authorization or secure deletion;
- rollback of a committed generation;
- schema migration, deduplication, event collapse, or conflict resolution;
- reference-host qualification, formal soak completion, or release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
