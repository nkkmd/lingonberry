# Durable Transition Re-evaluation Queue

**Status: draft for v0.6.0** | **Rule version: `lb.transition.reevaluation.queue.v1`** | **Last updated: 2026-07-20**

## 1. Purpose

When a target Knowledge Object arrives, orphan Transition Objects may become eligible for authority classification and effective-view projection. This work is derived-state processing and MUST NOT be placed on the critical path of canonical target publication.

## 2. Commit boundary

A conforming relay performs the following logical sequence:

```text
validate and accept target Knowledge Object
→ durably commit canonical target record
→ durably record re-evaluation intent
→ return target publish success
→ process re-evaluation asynchronously
```

The target record is canonical state. Transition authority and effective-view projection are derived state.

A failure to execute or complete re-evaluation MUST NOT rewrite a successful target commit into a publish failure. Conversely, returning publish success MUST NOT permit the re-evaluation obligation to be lost permanently.

## 3. Durable intent

For every committed target that can affect stored orphan transitions, the relay MUST make re-evaluation discoverable after process or host restart.

This requirement may be satisfied by:

- an outbox record committed atomically with the target record;
- a durable queue entry written immediately after the target commit plus a restart-safe recovery marker;
- deterministic reconciliation that compares canonical targets, orphan transitions, and derived checkpoints and recreates missing work.

An in-memory-only task, timer, or channel is insufficient.

If the target is committed but direct enqueue fails, the target remains stored and the relay MUST expose the re-evaluation state as `pending-recovery` or an equivalent machine-readable degraded state until reconciliation recreates the work.

## 4. Job lifecycle

A re-evaluation work item has at least these states:

```text
pending
running
succeeded
retryable-failed
```

Claiming a job MUST be restart-safe. A worker crash while a job is `running` MUST eventually make the work claimable again.

The queue MAY use leases, visibility timeouts, or an append-only attempt log. It MUST NOT depend on process memory as the only record of ownership.

## 5. Idempotency

Processing is at-least-once. A worker MUST therefore be idempotent.

Reprocessing the same logical work against the same canonical evidence MUST produce the same:

- authority classification;
- authority basis;
- transition graph projection;
- effective-view generation or checkpoint;
- completion classification.

Repeated delivery MUST NOT duplicate Transition Objects, mutate signed transition bytes, or apply a replacement or withdrawal more than once.

## 6. Evidence snapshot and stale work

Before committing a derived result, a worker MUST verify that the evidence snapshot used for evaluation is still current. At minimum, the snapshot identifies the relevant target record and transition corpus generation.

If new transition, delegation, revocation, or target evidence arrived during processing, the worker MUST classify its result as stale and schedule or preserve newer work. A stale result MUST NOT overwrite a newer derived checkpoint.

## 7. Checkpoint rule

A derived checkpoint advances only after:

1. all required evidence was read successfully;
2. authority classification completed without unsupported or contradictory state;
3. graph projection completed deterministically;
4. the evidence snapshot was confirmed current;
5. the derived result was durably committed.

Queue acknowledgement occurs only after the checkpoint or equivalent derived result is durable.

A retryable failure leaves the previous consistent checkpoint unchanged.

## 8. Reconciliation

A relay MUST support deterministic reconciliation that finds at least:

- committed targets with orphan transitions but no discoverable work;
- abandoned `running` claims;
- queue completion records without the corresponding derived checkpoint;
- stale checkpoints whose evidence generation no longer matches canonical state.

Reconciliation recreates or reopens work. It MUST NOT modify canonical Knowledge Objects or stored Transition Objects.

## 9. Publish response

A successful target publish MAY report derived processing separately:

```json
{
  "status": "stored",
  "code": "LB_OBJECT_STORED",
  "reevaluation": {
    "status": "pending"
  }
}
```

`pending` means the target is durably stored while transition-derived state may still reflect the prior consistent checkpoint. It MUST NOT be represented as if re-evaluation had already completed.

## 10. Failure isolation

The following failures do not invalidate the canonical target commit:

- queue worker crash;
- temporary authority-evidence lookup failure;
- graph projection retryable failure;
- checkpoint storage failure after target commit;
- temporary queue unavailability when durable recovery intent remains discoverable.

Corruption, unsupported versions, and contradictory evidence remain fail-closed. They MUST NOT produce a new effective-view checkpoint.

## 11. Conformance

The conformance corpus fixes:

- target commit followed by durable pending work;
- target commit surviving direct enqueue failure;
- restart reconciliation recreating missing work;
- repeated delivery remaining idempotent;
- failed processing preserving the previous checkpoint;
- successful processing advancing the checkpoint only after a current-snapshot check.
