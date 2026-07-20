# Transition Re-evaluation Coalescing

**Status: normative for v0.6.0** | **Rule: `lb.transition.reevaluation.coalescing.v1`**

## Work subject

Re-evaluation work is scoped to a target Knowledge Object, not to an individual Transition Object.

```text
subject = targetId
requiredGeneration = evidenceGeneration
```

There MUST be at most one current logical intent for a target. Physical queue deliveries MAY be duplicated by at-least-once delivery.

## Evidence generation

Any change that can affect authority or effective-view projection advances the target evidence generation, including:

- target Knowledge Object arrival or replacement of relay-local evidence;
- Transition Object arrival;
- delegation evidence arrival;
- revocation evidence arrival;
- recovery of previously unavailable authority evidence.

A generation identifies a complete target-scoped evidence snapshot. It is not derived from wall-clock ordering.

## Pending coalescing

When work for generation `g1` is pending and newer evidence produces `g2`, the current intent is advanced to `g2` rather than creating an independent transition-scoped job. Repeated arrivals MAY advance the same intent again.

```text
pending g1 + evidence g2 -> pending g2
pending g2 + evidence g3 -> pending g3
```

The worker evaluates the complete target graph for the latest claimed generation.

## Running work and newer evidence

A running worker is not required to be cancelled. Before committing its result it MUST compare the claimed generation with the current generation.

- equal: the result MAY be committed;
- different: the result is stale and MUST NOT update the derived view or checkpoint.

Newer evidence leaves or creates a pending intent for the current generation.

## Completion

Success for generation `gN` means:

1. the full target evidence snapshot for `gN` was evaluated;
2. the snapshot remained current at commit time;
3. the derived result was durably committed;
4. the target checkpoint advanced to `gN`;
5. no newer generation was lost.

A completed older delivery MUST NOT clear pending work for a newer generation.

## Reconciliation

Reconciliation compares each target's current evidence generation with its derived checkpoint. When they differ and no current intent exists, it recreates one target-scoped intent for the current generation.

## Safety rules

- Transition bytes and identities are never rewritten by coalescing.
- Coalescing MUST NOT discard evidence.
- Timestamp or transition ID ordering MUST NOT choose a graph winner.
- A stale worker MUST NOT overwrite a newer view.
- Duplicate physical deliveries MUST NOT create duplicate logical application.
