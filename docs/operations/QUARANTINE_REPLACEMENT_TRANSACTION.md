# Quarantine Replacement Transaction and Recovery Contract

**Status: implemented** | **v1.0 pre-release normative operations contract**

This document defines the mutation-capable quarantine replacement transaction. It consumes a verified policy-v2 preview and proof, stages a complete generation, verifies it, publishes it through the current-generation pointer, and records durable recovery state. It does not broaden the replacement semantics approved by the replacement policy.

## Safety boundary

A transaction may apply only the one-to-one canonical representation changes described by a verified replacement plan.

It must never:

- overwrite an active ledger in place;
- modify immutable-evidence ledgers semantically;
- rewrite or delete archive segments;
- perform retention deletion;
- deduplicate, merge, split, or collapse events;
- select a winner for conflicting terminal state;
- migrate schemas or insert defaults;
- move events across archive boundaries; or
- reinterpret policy-v1 compaction behavior.

All failures are fail closed.

## Commands

The maintenance CLI exposes replacement apply, status, resume, and rollback operations. Exact argument syntax is defined by the checked-in CLI help for the reviewed binary.

Conceptually:

```text
replacement-apply <backup-v2> <proof-dir> <transaction-dir>
replacement-status <transaction-dir>
replacement-recover <transaction-dir> --resume
replacement-recover <transaction-dir> --rollback
```

Operators must capture the actual command, binary identity, exit status, stdout, and stderr used for evidence.

## Required inputs

A new transaction requires:

- verified archive-inclusive quarantine backup v2;
- verified replacement plan and proof artifacts;
- current runtime state;
- a new transaction workspace; and
- the same-host quarantine operation lock.

The transaction binds the relevant input versions, plan and proof digests, backup-manifest digest, optional segment-manifest digest, runtime fingerprint, transaction ID, and staged/publication metadata in versioned artifacts.

A mismatch or unverifiable input aborts before a generation is activated.

## Same-host lock

Apply, resume, and rollback use the quarantine operation lock. The lock coordinates writers using the same state directory and lock path on one host.

It is not:

- a distributed lock;
- leader election;
- a network-filesystem consensus mechanism; or
- protection against another process using a different state-directory alias.

Operators must quiesce external writers that do not participate in this lock contract.

## Pre-apply gates

Before staging, transaction preparation must:

1. acquire the operation lock;
2. verify archive segments;
3. verify the supplied backup and require v2;
4. verify the replacement plan and proof;
5. require exact digest and version agreement;
6. compare current runtime fingerprint with the plan input;
7. reject corrupt lifecycle state, duplicate terminal keys, unsupported versions, and failed semantic expectations;
8. require a new transaction workspace; and
9. durably publish the initial transaction journal and input binding.

Preparation must not activate a generation.

## Durable state machine

The implemented transaction uses versioned journal states representing preparation, staging, verification, publication, terminal completion, rollback, and recovery-required conditions.

The normal lifecycle is:

```text
prepared
→ writing
→ staged
→ verified
→ publishing
→ committed
```

An interrupted or failed non-terminal transaction may transition to `recovery-required` where required by the implementation. `committed` and `rolled-back` are terminal states.

Transitions must follow the checked-in state-transition validator. Skipped, backward, unsupported, duplicated, or contradictory transitions are errors. The next externally observable mutation must not occur before the preceding durable transition and required filesystem synchronization succeed.

The journal is replaced through its checked-in durable publication path and accompanied by its integrity digest. The digest detects byte changes; it is not a signature, trusted timestamp, or authorization token.

## Transaction workspace

The transaction workspace contains the durable journal, bound inputs, staging data, generation publication artifacts, publication intent, completion evidence when terminal, and any recovery evidence produced by the implementation.

An existing non-empty or conflicting workspace must not be silently reused as a new transaction. Operators must inspect it through status/recovery procedures.

Transaction IDs and paths must pass the implementation's path-safety checks. Symbolic-link or unexpected-file conditions must fail closed where the corresponding artifact verifier requires regular files.

## Staging

Staging builds the complete managed-ledger set inside the transaction workspace. It does not patch active ledger files.

Staging must:

- retain immutable evidence according to the plan;
- apply only approved canonical representation replacements;
- preserve parsed values, logical order, terminal states, replacement keys, and provenance;
- record present/absent ledger metadata, byte counts, line counts, replacement counts, and digests;
- synchronize staged files and directories; and
- reject missing or unexpected files.

A partial staged set must never be activated.

## Staged verification

Before generation sealing, the staged set is verified against the plan, proof, and exact managed-ledger membership.

Verification covers:

- immutable-evidence retention;
- canonical replacement values;
- logical order and record identity;
- terminal-state and lifecycle behavior;
- status and state-derived metrics;
- promotion eligibility;
- idempotent and conflicting action outcomes;
- ordered reader results;
- corruption behavior;
- one-to-one provenance; and
- duplicate terminal-key absence.

The transaction may enter the verified state only after all required checks succeed.

## Generation sealing

A verified transaction seals a transaction-local `publication/` directory containing:

```text
quarantine-replacement-generation.json
quarantine-replacement-generation.digest
managed ledger files
```

The generation manifest binds the transaction ID and exact ledger metadata. Sealing verifies the staged source again and synchronizes the publication directory.

Sealing does not make the generation active.

If generation durability fails, the implementation removes incomplete publication output where possible and records `recovery-required` when the durable journal permits that transition.

## Publication intent and activation

Before switching the active generation, the transaction records and verifies publication intent and revalidates current state according to the implementation.

Activation uses the versioned current-generation pointer. The reader-visible local switch is the checked-in pointer publication operation, not a claim that several independent ledger renames are collectively atomic.

The pointer binds:

```text
version
transactionId
generationDigest
```

After a valid pointer is present, readers and writers resolve managed ledgers through the referenced generation. They must not silently fall back to root-level legacy ledgers when the pointer or referenced generation is invalid.

A local atomic rename does not provide cross-host atomicity, distributed consensus, or replicated-storage guarantees.

## Commit boundary

A transaction is committed only after the publication path, active generation, journal state, and required verification steps succeed.

The implementation must not infer commit solely from:

- a generation directory existing;
- the pointer file existing;
- temporary artifacts disappearing; or
- one publication step succeeding.

The durable journal and verified filesystem state together determine the result.

Committed transactions are terminal. A committed generation may be superseded only by another verified replacement transaction; it is not rolled back by editing the pointer manually.

## Completion evidence

Terminal replacement completion evidence is a separate versioned artifact bound to the journal terminal state and generation digest where applicable.

Completion evidence supports retention evaluation but does not authorize cleanup by itself. Missing, partial, conflicting, future-dated, or mismatched evidence fails closed for retention purposes.

## Status and interruption classification

`replacement-status` reads the durable journal and relevant artifacts to classify transaction state. Operators must use status before retrying an interrupted command.

Possible operational outcomes include:

- a normal non-terminal state that can be resumed;
- committed;
- rolled back;
- recovery required; or
- corrupt/contradictory state requiring manual investigation.

Do not infer success from absent temporary files or partial publication.

## Resume

Resume acquires the operation lock, verifies durable journal and filesystem state, and repeats only idempotent unfinished steps permitted for the current state.

Resume must not:

- start a second transaction;
- accept changed input bindings;
- skip required verification;
- overwrite conflicting final artifacts; or
- convert contradictory state into success.

Repeated resume invocations are safe only to the extent explicitly provided by the state machine and idempotent publication functions.

## Rollback

Rollback is available only for states where the checked-in transaction contract permits it. It acquires the operation lock and restores the exact previously active pointer/generation reference recorded by the transaction.

When no previous pointer existed, rollback may reactivate the legacy root layout by restoring the recorded absence of a pointer. It must not reconstruct history from guesses.

Rollback must preserve archive segments and immutable evidence, verify the restored active state, and record `rolled-back` only after successful durable completion.

Rollback is not advertised after `committed`. A committed transaction requires a new forward replacement transaction or a separately reviewed recovery procedure.

A missing, mismatched, or unverifiable bound input causes rollback to fail closed where that input is required by the implementation.

## Recovery-required and corruption

State is recovery-required when the implementation cannot safely complete the current step automatically but retains sufficient durable evidence for an operator to inspect and resume or roll back.

Contradictory journal state, digest mismatch, mixed generation data, unknown files, path-safety violations, or ambiguous pointer/publication state must not be automatically repaired.

For such incidents:

1. stop further writers and replacement commands;
2. preserve the transaction workspace and runtime state;
3. capture status, directory listings, digests, and errors without editing artifacts;
4. verify the backup, segments, journal, generation, pointer, and completion evidence independently; and
5. follow the recovery runbook or escalate for manual review.

## Derived indexes and readers

Managed-ledger generation activation and derived index maintenance are separate concerns. Any derived index required by the runtime must be rebuilt or verified according to its own contract before the deployment is declared healthy.

A successful pointer switch alone is not evidence that all external consumers, caches, replicas, or indexes have converged.

## Operational evidence

Retain:

- application commit and binary identity;
- exact commands and operator identity;
- verified backup v2 and segment evidence;
- replacement plan/proof and digests;
- transaction inputs and journal history;
- staging manifests and verification reports;
- generation manifest and digest;
- publication intent and pointer evidence;
- completion evidence;
- pre/post status and state-derived metrics;
- stdout, stderr, exit status, and timestamps; and
- all resume, rollback, or recovery decisions.

Evidence completeness does not establish formal soak or privileged reference-host qualification unless those procedures were separately executed and recorded.

## Non-goals

The replacement transaction does not provide:

- retention deletion or automatic cleanup;
- conflict repair or deduplication;
- schema migration;
- archive-segment mutation;
- distributed locking or leader election;
- multi-host atomic publication;
- remote storage orchestration;
- cryptographic signing; or
- general-purpose data migration.

## Related documents

- [`QUARANTINE_REPLACEMENT_POLICY.md`](./QUARANTINE_REPLACEMENT_POLICY.md)
- [`QUARANTINE_REPLACEMENT_PREVIEW.md`](./QUARANTINE_REPLACEMENT_PREVIEW.md)
- [`QUARANTINE_REPLACEMENT_GENERATION.md`](./QUARANTINE_REPLACEMENT_GENERATION.md)
- [`QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [`QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md`](./QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md)
- [`QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
