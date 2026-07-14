# Quarantine Replacement Transaction and Recovery

**Status: implementation contract for QL-5C3C** | **Last updated: 2026-07-14**

This document defines the safety boundary and transaction model for applying a verified policy-v2 replacement plan. It extends the read-only preview and proof contract in `QUARANTINE_REPLACEMENT_PREVIEW.md`; it does not broaden the approved replacement semantics.

## 1. Safety boundary

QL-5C3C may apply only a plan that passes the QL-5C3B replacement-proof verifier.

The transaction must never:

- overwrite an existing ledger in place;
- modify immutable evidence ledgers;
- rewrite or delete archive segments;
- perform retention deletion;
- deduplicate records or collapse events;
- resolve conflicts;
- migrate schemas;
- move records across archive boundaries;
- reinterpret policy-v1 behavior.

All failures are fail-closed.

## 2. Required inputs

```text
verified backup v2 directory
verified replacement proof directory
new or empty transaction directory
current runtime state
```

The transaction must bind the following values into its journal:

```text
backup manifest digest
segment manifest digest
replacement plan digest
replacement proof digest
policy version
plan version
proof version
runtime fingerprint
```

A mismatched or unverifiable input aborts before staging.

## 3. Proposed CLI

```bash
lingonberry-quarantine-maintenance replacement-apply \
  <verified-backup-v2-dir> \
  <verified-proof-dir> \
  <transaction-dir>

lingonberry-quarantine-maintenance replacement-status \
  <transaction-dir>

lingonberry-quarantine-maintenance replacement-recover \
  <transaction-dir> --resume

lingonberry-quarantine-maintenance replacement-recover \
  <transaction-dir> --rollback
```

The existing policy-v1 commands and QL-5C3B preview/proof commands remain unchanged.

## 4. Pre-apply gates

Before writing staged output, the implementation must:

1. acquire the existing same-host operation lock;
2. verify archive segments;
3. verify the supplied backup and require `lingonberry-quarantine-backup/v2`;
4. run the QL-5C3B replacement-proof verifier;
5. require exact plan/proof digest agreement;
6. recompute the current runtime fingerprint and compare it with the plan;
7. reject stale index, stale proof, unknown managed ledgers, duplicate terminal keys, unsupported versions, and semantic verification failures;
8. require a new or empty transaction directory;
9. durably create the initial transaction journal;
10. abort without publishing any ledger on failure.

The runtime fingerprint must be recomputed immediately before publication. Any change aborts publication.

## 5. Transaction state machine

Normative states:

```text
prepared
writing
staged
verified
publishing
committed
rolled-back
recovery-required
```

Normal path:

```text
prepared
→ writing
→ staged
→ verified
→ publishing
→ committed
```

Recovery path:

```text
prepared | writing | staged | verified | publishing
→ recovery-required
→ resumed or rolled-back
```

`committed` and `rolled-back` are terminal. Unknown, skipped, duplicated, or contradictory transitions are corruption.

Every transition must be written and fsynced before the next externally observable mutation.

## 6. Transaction journal

The journal must be versioned and append-safe or atomically replaceable. It must contain enough information to classify the filesystem state without guessing.

Minimum fields:

```text
journal version
transaction ID
state
created-at and updated-at timestamps
policy / plan / proof versions
plan and proof digests
backup and segment-manifest digests
pre-transaction runtime fingerprint
expected staged generation digest
old active generation references
staged file paths and digests
publication progress
recovery classification
last durable transition
```

Timestamps are operational metadata and must not alter the replacement-plan digest.

## 7. Staging

Staging occurs only inside the transaction directory and on the same filesystem required for atomic rename.

Rules:

- construct complete staged ledgers; never patch active ledgers;
- copy immutable evidence ledgers byte-for-byte when the publication model requires a complete generation;
- apply only approved `canonical-json-representation` replacements;
- preserve logical ordinal, replacement key, terminal state, logical order, parsed JSON value, and one-to-one provenance;
- fsync each staged file;
- fsync the staging directory;
- record staged file digests in the journal;
- never publish incomplete staged output.

## 8. Staged verification

Before publication, verify:

```text
exact managed-ledger membership
immutable-ledger byte identity
replacement-plan conformance
source/replacement value equivalence
logical order equivalence
terminal-state equivalence
status-count equivalence
Prometheus-metric equivalence
promotion-eligibility equivalence
single-operation idempotency equivalence
batch-operation idempotency equivalence
reader-result equivalence
corruption-detection equivalence
complete one-to-one provenance
duplicate terminal-key absence
```

The journal may enter `verified` only after every required check succeeds.

## 9. Publication model

Publication must use fsync and atomic rename. The implementation must not claim that several independent renames are collectively atomic.

Preferred design:

```text
generation directory or equivalent indirection
→ fsynced complete staged generation
→ one atomic namespace switch
```

If current reader compatibility requires per-ledger path replacement, the implementation must define a journaled publication sequence and recovery invariant proving that a mixed generation cannot be accepted as healthy.

Before the first publication rename:

1. verify journal integrity;
2. verify staged digests;
3. recompute runtime fingerprint;
4. verify the QL-5C3B proof again or verify its journal-bound digest and validity;
5. fsync all staged content and parent directories.

After publication:

1. fsync affected parent directories;
2. rebuild and verify the derived index;
3. verify runtime semantics against the contract;
4. write and fsync `committed`.

## 10. Backup and rollback

Verified backup v2 is mandatory. The journal binds rollback to the exact backup manifest and segment-manifest digests used before apply.

Rollback must:

- acquire the same-host operation lock;
- verify the journal and bound backup;
- restore only the exact pre-transaction generation;
- use staged restoration plus fsync and atomic rename;
- preserve archive segments and immutable evidence;
- verify restored runtime fingerprint and index;
- be idempotent;
- write and fsync `rolled-back` only after verification.

A missing or mismatched backup makes rollback fail closed.

## 11. Resume and recovery

Recovery must inspect the journal and filesystem state and classify the transaction as:

```text
committed
rolled-back
resumable
rollback-only
recovery-required
corrupt
```

Resume may repeat only idempotent unfinished steps. It must not infer success from absent temporary files or partial publication.

Ambiguous, contradictory, or unverifiable state is `corrupt` and must not be automatically repaired.

## 12. Idempotency and concurrency

- apply, resume, and rollback require the existing same-host operation lock;
- the lock is not a distributed lock;
- applying an already committed transaction performs no second rewrite;
- repeated resume and rollback calls are idempotent;
- a changed runtime fingerprint outside the journaled transaction aborts;
- policy-v1 behavior remains compatible.

## 13. Stable error-code families

```text
LB_QUARANTINE_REPLACEMENT_TRANSACTION
LB_QUARANTINE_REPLACEMENT_JOURNAL
LB_QUARANTINE_REPLACEMENT_STAGING
LB_QUARANTINE_REPLACEMENT_PUBLICATION
LB_QUARANTINE_REPLACEMENT_RECOVERY
LB_QUARANTINE_REPLACEMENT_ROLLBACK
```

Existing replacement backup, changed, conflict, corrupt, policy, proof, and semantics families remain applicable.

## 14. Required tests

```text
valid verified apply
proof tampering rejection before staging
backup mismatch rejection
stale runtime fingerprint rejection
runtime change before publication rejection
immutable-ledger byte identity
approved canonical replacement only
staged semantic verification failure
fsync failure injection at each durable boundary
rename failure injection at each publication step
crash after each journal transition
resume from every resumable state
rollback from every rollback-capable state
repeated apply/resume/rollback idempotency
contradictory journal rejection
missing staged file rejection
mixed-generation rejection
post-commit index rebuild and verification
post-commit semantic-equivalence verification
policy-v1 regression
```

## 15. Non-goals

- retention deletion;
- deduplication;
- event collapse;
- schema migration;
- conflict resolution;
- archive mutation;
- immutable evidence mutation;
- distributed locking;
- remote storage;
- cryptographic signing;
- general-purpose data migration.
