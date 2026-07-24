# Quarantine Replacement Operations Hardening

**Status: normative v1.0 pre-release operations contract** | **Last reviewed: 2026-07-24**

This document defines the implemented observability, audit, failure-injection, recovery-classification, generation-inspection, retention, cleanup, and operator-evidence boundary for quarantine replacement transactions.

Operations hardening must not broaden replacement semantics or evidence-deletion authority.

## 1. Safety boundary

The replacement operations layer must never:

- overwrite active managed ledgers in place;
- modify immutable evidence ledgers;
- rewrite or delete archive segments as part of replacement;
- infer a successful transaction from directory or pointer existence alone;
- classify contradictory state as healthy;
- automatically delete generations or transaction workspaces;
- deduplicate, collapse, or resolve conflicting events;
- migrate schemas;
- expose transaction IDs, generation digests, filesystem paths, record IDs, secrets, or user-controlled values as unbounded metric labels;
- treat host-local locking as distributed coordination.

Status, audit, metrics, retention inspection, and recovery failures are fail-closed.

## 2. Versioned transaction status

`replacement-status <transaction-dir>` returns canonical JSON using:

```text
lingonberry-quarantine-replacement-status/v1
```

The implemented fields are:

```text
version
transactionId
state
classification
activeGeneration
activeGenerationPresent
targetGenerationActive
generationDigest
recoveryRequired
terminal
publicationPhase
```

`transactionId`, `activeGeneration`, and `generationDigest` are explicit diagnostic fields in CLI status output. They must not be reused as Prometheus label values.

`terminal` is true only for:

```text
committed
rolled-back
```

`recoveryRequired` is true only when the journal state is `recovery-required`.

The bounded publication phase is derived from durable state and classification:

```text
prepared
writing
staged
verified
materialized
switched
committed
rolled-back
recovery-required
```

An invalid journal, pointer, manifest, digest, generation, input binding, or publication-intent relationship is an error. The status command does not convert corrupt state into a successful `unknown` report.

## 3. Replacement metrics

`replacement-metrics <transaction-dir>` derives Prometheus text from a successfully classified transaction status. Metric collection is read-only with respect to transaction state.

The implemented metric families are:

```text
lingonberry_quarantine_replacement_transactions{state="..."}
lingonberry_quarantine_replacement_active_generation{layout="legacy|generation",target="active"}
lingonberry_quarantine_replacement_recovery_required
lingonberry_quarantine_replacement_publication_phase{phase="..."}
```

Rules:

- state, layout, target, and phase values come from bounded implementation enums or fixed values;
- transaction IDs are not metric labels;
- generation digests are not metric labels;
- filesystem paths are not metric labels;
- quarantine record IDs are not metric labels;
- free-form errors are not metric labels;
- metric generation must not mutate the journal, pointer, generation, index, or archive state;
- a corrupt status causes metric generation to fail rather than emit a healthy zero snapshot.

The metrics output is transaction-local. It does not aggregate every transaction directory automatically and does not replace external scrape-target configuration.

## 4. Replacement operation audit ledger

Replacement CLI operations append canonical JSON lines to:

```text
<state-dir>/quarantine-replacement-audit.jsonl
```

The event version is:

```text
lingonberry-quarantine-replacement-audit/v1
```

Implemented event types are:

```text
replacement-operation-started
replacement-operation-completed
replacement-operation-rejected
replacement-recovery-required
replacement-generation-switched
replacement-committed
replacement-rolled-back
replacement-status-corrupt
```

Implemented operations are:

```text
apply
resume
rollback
status
```

Implemented outcomes are:

```text
started
success
rejected
failed
```

Each event contains:

```text
version
occurredAt
eventType
operation
outcome
transactionState
classification
boundedErrorCode
```

The ledger intentionally omits transaction IDs, generation digests, full paths, raw ledger records, backup contents, proof contents, environment values, credentials, bearer tokens, and free-form error text.

`classification` is restricted to an implementation allowlist. `boundedErrorCode` must start with `LB_`, contain only uppercase ASCII letters, digits, or underscores, and remain within the configured length bound.

Audit append uses a dedicated host-local quarantine lock, append mode, file sync, and parent-directory sync. This provides same-state-directory local serialization only; it is not a distributed audit service.

An audit append failure is returned to the caller. It is not silently discarded.

## 5. Failure injection contract

Replacement failure injection is disabled by default and requires both:

```text
LINGONBERRY_ENABLE_REPLACEMENT_FAILURE_INJECTION=1
LINGONBERRY_REPLACEMENT_FAILURE_POINT=<registered-point>
```

Injection is one-shot per process. Production launch configuration must not set either variable unintentionally.

The machine-readable registry is:

```text
docs/operations/quarantine-replacement-crash-points.v1.json
```

Its version is:

```text
lingonberry-quarantine-replacement-crash-points/v1
```

Registered boundaries include:

```text
journal.write
journal.fsync
staging.ledger-write
staging.ledger-fsync
staging.directory-fsync
generation.manifest-write
generation.manifest-fsync
publication.generation-materialize-rename
publication.intent-write
publication.pointer-temporary-write
publication.pointer-rename
publication.state-directory-fsync
publication.index-rebuild
publication.index-verification
publication.segment-verification
publication.commit-transition
rollback.pointer-restore
rollback.rolled-back-transition
```

The registry binds each point to its durable boundary, reader-visible target state, expected journal state, expected classification, allowed recovery actions, and test name.

Failure injection is a test and rehearsal mechanism. It is not a production recovery control and does not itself prove reference-host qualification.

## 6. Crash and recovery invariants

Tests and rehearsals must preserve these invariants:

- before pointer switch, readers continue to resolve the previous generation or legacy root layout;
- after pointer switch, readers may resolve the target generation even if the transaction has not reached `committed`;
- post-switch failures are classified as resumable or rollback-capable according to the durable state;
- resume repeats only idempotent unfinished steps;
- rollback restores only the exact previous pointer or the recorded legacy-root absence;
- rollback is unavailable after `committed`;
- immutable evidence remains byte-identical;
- archive segments remain unchanged;
- repeated recovery does not create a second generation or duplicate a journal transition;
- contradictory pointer, intent, journal, or digest relationships fail closed.

The crash-point registry is authoritative for the expected classification and allowed action at each registered injected boundary.

## 7. Generation inspection

`replacement-inspect-generations [transaction-dir ...]` is a read-only inspection surface. It classifies the active layout and explicitly supplied transaction directories for retention review.

Relevant classifications include:

```text
active-committed-generation
previous-committed-generation
rolled-back-generation
incomplete-transaction-generation
orphan-unreferenced-generation
legacy-root-layout
unknown-or-corrupt
```

Inspection evidence may include bounded classification, pointer and journal references, terminal state, verification status, durable age evidence, and manual-review requirements.

Inspection must not:

- delete, rename, truncate, or rewrite a generation;
- repair a pointer or journal;
- convert orphan or corrupt state into an eligible cleanup subject;
- infer deletion authority from age alone.

## 8. Retention and cleanup separation

The implemented retention policy evaluates exact generation IDs only. A retention decision report is non-destructive classification evidence.

A cleanup operation requires additional independently verified layers:

1. exact generation inspection;
2. retention-policy evaluation;
3. complete cleanup plan and proof binding;
4. apply-time state and inventory revalidation;
5. a dedicated cleanup transaction journal;
6. reversible tomb preparation;
7. explicit destructive-action acknowledgement;
8. irreversible deletion progress evidence;
9. committed, rolled-back, recovery-required, or partially-deleted terminal handling.

Eligibility does not authorize automatic deletion. Replacement transaction workspaces and cleanup transaction workspaces remain outside the current retention-policy subject model.

## 9. CLI contract

Implemented replacement observability and recovery commands include:

```text
replacement-apply <backup-v2-dir> <proof-dir> <transaction-dir>
replacement-status <transaction-dir>
replacement-metrics <transaction-dir>
replacement-inspect-generations [transaction-dir ...]
replacement-recover <transaction-dir> --resume|--rollback
```

Proof and maintenance commands used by the wider workflow include:

```text
replacement-preview <backup-v2-dir> <output-dir>
verify-replacement-proof <proof-dir>
verify-index
verify-segments
```

CLI help, tests, and operator runbooks must use the checked-in command names and argument ordering. No command may silently repair a pointer, journal, generation, proof, backup binding, or cleanup state.

## 10. Operator smoke and rehearsal evidence

An operator smoke or rehearsal should exercise, in isolated state directories:

```text
legacy root-ledger state
verified backup v2 export and verification
policy-v2 replacement preview and proof verification
replacement apply
committed status
generation-aware read and write
index verification
segment verification
idempotent status or recovery behavior
injected pre-switch failure
injected post-switch failure
successful resume
eligible pre-commit rollback
retention inspection without deletion
```

Evidence should retain:

- exact binary or commit identity;
- command lines with secrets removed;
- canonical status output;
- bounded metrics output;
- relevant audit lines;
- failure-injection point and expected classification;
- pointer, journal, manifest, and digest evidence;
- index and segment verification results;
- state-directory identity;
- operator and host identity recorded outside secret-bearing configuration.

Ordinary smoke tests, CI, and documentation walkthroughs are not the formal 72-hour soak and are not privileged reference-host qualification.

## 11. Stable error handling

Callers must branch on stable error codes, not free-form messages.

Replacement-related error families include transaction, journal, staging, publication, recovery, rollback, audit, retention-policy, cleanup-preview, cleanup-transaction, tomb, and cleanup-execution families defined by the implementation.

Audit events accept only bounded `LB_...` codes. Metrics do not expose error messages as labels.

## 12. Non-goals

Operations hardening does not provide:

- distributed locking or consensus;
- automatic corruption repair;
- automatic generation or workspace deletion;
- secure erase;
- remote backup retrieval;
- centralized audit replication;
- cryptographic authentication of FNV-1a digests;
- operator identity binding to local CLI execution;
- formal soak completion;
- privileged reference-host qualification;
- release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary CI or walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
