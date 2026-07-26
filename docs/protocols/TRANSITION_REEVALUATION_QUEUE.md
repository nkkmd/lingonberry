# Durable Transition Re-evaluation Queue

**Status: v1.0.0 pre-release contract** | **Rule version: `lb.transition.reevaluation.queue.v1`** | **Last reviewed: 2026-07-26**

## 1. Purpose

Transition re-evaluation computes derived authority and effective-view state after canonical evidence changes. It is asynchronous work and is not part of the canonical-object mutation model.

This contract defines the durability and failure-isolation requirements for that work. It distinguishes:

- the normative queue obligation;
- the executable conformance model;
- the narrower behavior currently implemented by the Rust relay.

## 2. Canonical commit boundary

A conforming target-publication path follows this logical sequence:

```text
validate and accept target Knowledge Object
→ durably commit canonical target state
→ durably preserve a discoverable re-evaluation obligation
→ return target-publication success
→ process derived state asynchronously
```

The canonical target record and stored Transition Objects are source evidence. Authority classifications, graph projections, effective views, and re-evaluation checkpoints are derived state.

A derived-state failure MUST NOT retroactively convert a completed canonical target commit into a failed commit. Conversely, a successful publication response MUST NOT permit the re-evaluation obligation to disappear permanently.

The mechanism does not have to be one physical transaction, but after publication success there MUST be a restart-safe path that either finds the durable intent or reconstructs it deterministically.

## 3. Durable intent

For every canonical evidence change that can alter transition authority or projection, the relay MUST preserve target-scoped re-evaluation work in durable form.

Acceptable mechanisms include:

- an outbox record committed atomically with canonical evidence;
- a durable queue entry followed by a separately durable recovery marker;
- deterministic reconciliation from canonical evidence generation and the last committed derived checkpoint;
- another design with equivalent crash-recovery properties.

An in-memory-only task, channel, timer, or process-local flag is insufficient.

If canonical evidence is committed but the direct queue append fails, publication may remain successful only when the obligation is independently recoverable. The machine-readable state SHOULD distinguish this condition as `pending-recovery` or an equivalent degraded status.

## 4. Logical work subject

The logical work subject is the affected `targetId`, not one Transition Object and not one physical queue line.

Multiple physical deliveries MAY represent the same target-scoped obligation. They MUST NOT cause duplicate canonical evidence or duplicate semantic application.

Coalescing across evidence generations is governed by [`TRANSITION_REEVALUATION_COALESCING.md`](./TRANSITION_REEVALUATION_COALESCING.md).

## 5. Lifecycle model

A complete durable queue implementation represents, explicitly or equivalently, these states:

```text
pending
running
succeeded
retryable-failed
```

A direct-enqueue failure after canonical commit additionally requires a discoverable recovery condition such as:

```text
pending-recovery
```

Claiming work MUST be restart-safe. A worker crash while work is `running` MUST eventually make the target claimable again without deleting source evidence or advancing the derived checkpoint.

Implementations may use leases, visibility timeouts, compare-and-swap state, or append-only attempt records. Process memory MUST NOT be the sole ownership record.

## 6. At-least-once processing and idempotency

Physical delivery is at-least-once. Repeated processing of the same target against the same complete evidence generation MUST yield the same semantic result.

At minimum, repeated delivery MUST NOT:

- create additional Transition Object copies;
- mutate signed transition bytes or identities;
- apply a replacement or withdrawal more than once;
- regress the last-known-good effective view;
- clear a newer pending generation;
- claim success without a durable derived result.

A worker may append operational attempt records more than once. Operational duplication is acceptable; semantic duplication is not.

## 7. Evidence snapshot and stale work

Before committing a derived result, a complete implementation MUST verify that the evaluated evidence generation is still current for the target.

If target, transition, delegation, revocation, or other authority evidence changed while processing was in progress, the worker result is stale. A stale result:

- MUST NOT replace the current effective view;
- MUST NOT advance the semantic checkpoint;
- MUST leave or recreate work for the current evidence generation;
- MAY record a retryable operational outcome.

Generation construction is defined by [`TRANSITION_EVIDENCE_GENERATION.md`](./TRANSITION_EVIDENCE_GENERATION.md).

## 8. Derived checkpoint rule

A semantic checkpoint advances only after all of the following are true:

1. the complete required target-scoped evidence was read successfully;
2. unsupported, corrupt, or contradictory evidence was handled fail-closed;
3. authority classification completed;
4. graph projection completed deterministically;
5. the evaluated generation remained current at commit time;
6. the derived result was durably committed;
7. queue completion did not erase newer work.

Queue acknowledgement or its logical equivalent occurs only after the corresponding derived commit is durable.

A retryable failure leaves the previous consistent semantic checkpoint unchanged. An operational attempt record marked `retryable-failed` is not itself a replacement semantic checkpoint.

## 9. Reconciliation

A complete implementation MUST support deterministic reconciliation capable of finding at least:

- canonical evidence whose current generation is newer than its derived checkpoint;
- canonical evidence with no discoverable queue intent;
- abandoned running claims;
- success or acknowledgement records without the corresponding durable derived commit;
- pending-recovery obligations after direct enqueue failure.

Reconciliation recreates or reopens target-scoped work. It MUST NOT modify canonical Knowledge Objects, stored Transition Objects, signatures, identities, or provenance.

## 10. Publish-response semantics

A successful canonical target publication may expose derived processing separately:

```json
{
  "status": "stored",
  "code": "LB_OBJECT_STORED",
  "reevaluation": {
    "status": "pending"
  }
}
```

`pending` means canonical evidence is durable while the effective view may still be the previous consistent view. It does not mean re-evaluation completed.

`pending-recovery`, when used, means canonical evidence is durable and the implementation has retained enough independent state for reconciliation to recreate queue work.

## 11. Current Rust relay behavior

The Rust relay currently provides a partial executable implementation:

- transition publication appends a canonical JSON line to `transitions/reevaluation-queue.jsonl` after storing the Transition Object;
- each intent includes `ruleVersion`, `targetId`, `triggerTransitionId`, and `status: pending`;
- `process_reevaluation_queue` scans the complete queue file on each invocation;
- queue rows are deduplicated in memory by `targetId` using an ordered set;
- the worker calls the effective-view read path once for each unique target;
- HTTP status `200` is recorded as `succeeded`; other statuses are recorded as `retryable-failed`;
- the worker appends an operational record to `transitions/reevaluation-checkpoints.jsonl` containing target, status, observed generation, HTTP status, and completion time;
- the checkpoint file is flushed with `sync_all` after each append;
- `reconcile_reevaluation_queue` currently calls the same full-scan processing function.

These behaviors provide append durability, restart re-reading, target-level duplicate suppression within one worker invocation, and durable operational outcome records.

They do **not** currently implement the complete lifecycle model described above. In particular, the Rust relay does not currently provide:

- a durable claim or lease state;
- acknowledgement or removal of completed queue rows;
- filtering that prevents every historical intent from being processed again on later runs;
- an independent outbox or recovery marker for a failed queue append;
- a target-generation compare-and-swap guard at derived commit time;
- deterministic reconciliation by comparing canonical evidence generation with a semantic checkpoint;
- proof that the appended `succeeded` record and the effective-view semantic commit are one atomic operation.

The `generation` stored by the worker is extracted from the effective-view response when present and otherwise recorded as `unknown`. The append-only checkpoint record is operational evidence; it MUST NOT be interpreted as proof that all normative stale-work and atomic-checkpoint guarantees are implemented.

## 12. Conformance model

The external conformance runner fixes three core queue cases:

- committed target plus durable intent produces publication success with re-evaluation pending;
- direct enqueue failure does not undo the target commit when durable recovery intent exists, and reconciliation recreates work;
- repeated delivery creates no transition copies and advances the checkpoint only when the result commit succeeded and the evaluated snapshot remained current.

The current queue fixtures model the required outcomes. They are not an independent production queue, lease manager, transaction coordinator, or reconciliation engine.

## 13. Failure isolation

The following retryable failures do not invalidate an already durable canonical commit:

- worker crash;
- temporary authority-evidence lookup failure;
- retryable graph-projection failure;
- derived checkpoint storage failure after canonical commit;
- temporary queue unavailability when an independent durable recovery obligation exists.

Corruption, unsupported versions, contradictory evidence, incomplete authority evidence, and stale-generation results remain fail-closed. They MUST NOT produce a new semantic effective-view checkpoint.

## 14. Release boundary

This document describes the v1.0.0 pre-release contract and the current implementation boundary. Its normalization does not redefine the fixed release candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence, privileged reference-host qualification, version update, release PR, tag, and GitHub Release remain separate release gates.