# Quarantine Replacement Operations Hardening

**Status: implementation contract for QL-5C3D** | **Last updated: 2026-07-14**

This document defines the operations, observability, failure-injection, retention-inspection, and release-hardening boundary for the generation-directory replacement transaction implemented by QL-5C3C.

## 1. Safety boundary

QL-5C3D must not broaden replacement semantics or evidence-deletion authority.

It must never:

- overwrite managed ledgers in place;
- modify immutable evidence ledgers;
- rewrite or delete archive segments;
- automatically delete generations or transaction workspaces;
- perform retention deletion;
- deduplicate or collapse events;
- resolve conflicts;
- migrate schemas;
- expose secrets, filesystem paths, transaction IDs, record IDs, or user-controlled values as unbounded metric labels;
- treat same-host locking as distributed coordination;
- classify ambiguous or corrupt state as healthy.

All operations and observability failures are fail-closed.

## 2. Structured transaction status

The canonical status representation must be versioned and machine-readable.

Minimum fields:

```text
version
state
classification
activeGenerationPresent
targetGenerationActive
recoveryRequired
terminal
publicationPhase
indexVerified
segmentsVerified
lastOperationOutcome
```

Transaction IDs, generation digests, and paths may be present in explicit CLI status output when required for operator diagnosis, but they must not be reused as Prometheus label values.

Required classifications include:

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
corrupt
```

Unknown journal state, invalid pointer, unrelated pointer, missing generation, digest mismatch, or contradictory publication intent is an error rather than an `unknown` success state.

## 3. Prometheus metrics

Metrics must use a bounded label set.

Proposed families:

```text
lingonberry_quarantine_replacement_transactions{state="..."}
lingonberry_quarantine_replacement_active_generation{layout="legacy|generation"}
lingonberry_quarantine_replacement_recovery_required
lingonberry_quarantine_replacement_last_operation{operation="apply|resume|rollback|status",outcome="success|rejected|failed"}
lingonberry_quarantine_replacement_publication_phase{phase="none|prepared|materialized|switched|committed"}
```

Rules:

- no transaction ID labels;
- no generation digest labels;
- no filesystem path labels;
- no quarantine record ID labels;
- no free-form error labels;
- stable error families may be represented only through a bounded allowlist if needed;
- corrupt state must not disappear from metrics because a reader returned early;
- metric collection must not mutate transaction state.

## 4. Secret-free audit events

Replacement operations must emit append-only audit events without secrets.

Required event types:

```text
replacement-apply-started
replacement-apply-rejected
replacement-staging-verified
replacement-publication-prepared
replacement-generation-switched
replacement-committed
replacement-resume-started
replacement-resume-completed
replacement-rollback-started
replacement-rolled-back
replacement-recovery-required
replacement-status-corrupt
```

Minimum fields:

```text
version
eventType
operation
outcome
transactionState
classification
boundedErrorCode
timestamp
```

Audit events must not contain:

- bearer tokens, credentials, or authorization headers;
- full filesystem paths;
- raw ledger lines or object payloads;
- unbounded error messages;
- environment-variable contents;
- backup or proof contents.

Transaction IDs may be represented only if the existing audit threat model accepts them as non-secret operational identifiers. Otherwise use a bounded presence flag or stable digest prefix policy documented separately.

## 5. Deterministic failure injection

Failure injection must be test-only or explicitly opt-in and impossible to activate accidentally in production.

Durable boundaries requiring injection coverage:

```text
journal write
journal fsync
staged ledger write
staged ledger fsync
staging directory fsync
generation manifest write
generation directory materialization
publication intent write
current-generation pointer temporary write
current-generation pointer rename
state directory fsync
index rebuild
index verification
segment verification
committed transition
rollback pointer restoration
rolled-back transition
```

Each injection point must have a stable identifier and deterministic one-shot behavior.

Tests must verify:

- active readers never accept a partial generation;
- pre-switch failure preserves the old active generation;
- post-switch/pre-commit failure is classified as resumable-after-switch or recovery-required;
- resume completes only idempotent unfinished steps;
- rollback restores only the exact previous pointer and is unavailable after commit;
- immutable evidence remains byte-identical;
- archive segments remain unchanged;
- repeated recovery does not create a second generation or duplicate journal transition.

## 6. Crash-point matrix

A machine-readable or table-driven crash matrix must cover at least:

| Crash point | Expected pointer | Expected state | Allowed action |
|---|---|---|---|
| before staging | previous | prepared / writing | resume or rollback |
| after staged write | previous | writing / staged | resume or rollback |
| after staged verification | previous | verified | resume or rollback |
| after generation materialization | previous | publishing | resume or rollback |
| after atomic switch | target | publishing / recovery-required | resume or rollback |
| after index rebuild | target | publishing | resume |
| after committed journal | target | committed | status only / new transaction |

Contradictory pointer states are corrupt and must not be automatically repaired.

## 7. Generation retention and cleanup policy

QL-5C3D may specify and inspect retention state but must not automatically delete evidence.

Classification categories:

```text
active-committed-generation
previous-committed-generation
rolled-back-generation
incomplete-transaction-generation
orphan-unreferenced-generation
legacy-root-layout
unknown-or-corrupt
```

A read-only cleanup-candidate report may include:

```text
classification
referencedByPointer
referencedByJournal
terminalTransactionState
verificationStatus
manualReviewRequired
```

It must not delete, rename, truncate, or rewrite any artifact.

Any future deletion feature requires a separate approved policy, explicit operator confirmation, backup verification, and a dedicated issue.

## 8. CLI contract

Existing commands remain stable:

```text
replacement-apply
replacement-status
replacement-recover --resume
replacement-recover --rollback
```

QL-5C3D may add read-only commands such as:

```text
replacement-metrics <transaction-dir>
replacement-inspect-generations
replacement-smoke-check <transaction-dir>
```

Final command names must match CLI help, runbooks, tests, and release notes.

No command may silently repair a pointer, journal, generation, or backup binding.

## 9. Operational smoke test

The recorded smoke test must exercise:

```text
legacy root-ledger state
→ verified backup v2 export and verify
→ policy-v2 replacement preview and proof verify
→ replacement apply
→ committed status
→ generation-aware read/write
→ index verify
→ segment verify
→ repeated apply/resume idempotency
→ injected post-switch failure in a separate fixture
→ successful resume
→ pre-commit rollback in a separate fixture
```

The smoke test must record commands, expected bounded outputs, and evidence-retention requirements without embedding secrets or environment-specific absolute paths.

## 10. Release gates

QL-5C3D is complete only when:

- structured status is versioned and tested;
- metrics use bounded cardinality and are tested;
- audit output is secret-free and append-only;
- failure-injection coverage spans every durable boundary approved for v0.3.0;
- crash-state classification and recovery tests pass;
- generation retention policy is explicit and non-destructive;
- CLI help and operator documentation agree;
- legacy root-layout upgrade behavior is tested;
- the v0.3.0 release checklist is complete;
- the v0.3.0 release note reflects final behavior;
- all Rust and JavaScript release gates pass.

## 11. Stable error families

Existing transaction error families remain authoritative:

```text
LB_QUARANTINE_REPLACEMENT_TRANSACTION
LB_QUARANTINE_REPLACEMENT_JOURNAL
LB_QUARANTINE_REPLACEMENT_STAGING
LB_QUARANTINE_REPLACEMENT_PUBLICATION
LB_QUARANTINE_REPLACEMENT_RECOVERY
LB_QUARANTINE_REPLACEMENT_ROLLBACK
```

Operations hardening may add bounded families for observability or inspection, but callers must not branch on free-form messages.