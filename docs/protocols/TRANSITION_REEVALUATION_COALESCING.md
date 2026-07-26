# Transition Re-evaluation Coalescing

## Status

Normative pre-release contract for target-scoped transition re-evaluation.

- Rule version: `lb.transition.reevaluation.coalescing.v1`
- Evidence-generation rule: `lb.transition.evidence-generation.v1`
- Queue-intent rule: `lb.transition.reevaluation.queue.v1`
- Product release status: v1.0.0 pre-release

This document defines the logical coalescing and stale-result safety model. It does not claim that every rule below is implemented by the current Rust relay worker runtime.

## Purpose

A target Knowledge Object may receive multiple transitions and later authority evidence. Re-evaluation is therefore scoped to the complete evidence snapshot for one target, not to one arriving Transition Object.

```text
logical subject     = targetId
required generation = current target evidence generation
```

Coalescing prevents repeated arrivals from causing duplicate semantic application while preserving every immutable evidence record.

## Logical intent

At most one current logical re-evaluation intent exists for a target.

A logical intent identifies:

- the target `targetId`;
- the latest required evidence generation;
- a lifecycle state such as pending, running, retryable failure, or succeeded;
- implementation-private lease, attempt, or delivery metadata when needed.

Physical queue records and deliveries may be duplicated under at-least-once delivery. They are not themselves the authoritative logical state.

The current Rust transition ingest path appends queue records containing `ruleVersion`, `targetId`, `triggerTransitionId`, and `status: pending`. That append-only record is a durable trigger. It does not by itself implement a target-keyed single-intent store or a generation-bearing claim.

## Generation source

The required generation is recomputed from the complete target-scoped evidence inventory under `lb.transition.evidence-generation.v1`.

Evidence that may change the generation includes:

- target Knowledge Object arrival or an explicit immutable evidence repair or replacement;
- Transition Object arrival;
- verified delegation evidence arrival;
- verified revocation evidence arrival;
- recovery of evidence previously classified as unsupported, corrupt, or unreadable;
- a classification or immutable digest change represented by an explicit normative record.

Receipt time, queue order, delivery count, worker identity, and transition identifier ordering do not define the generation.

## Pending coalescing

When target work is pending for generation `g1` and current evidence advances to `g2`, the logical intent advances to `g2`.

```text
pending g1 + current evidence g2 -> pending g2
pending g2 + current evidence g3 -> pending g3
```

An implementation may append additional physical trigger records, but it must resolve them to one current target-scoped requirement before semantic application.

Coalescing does not delete or merge transition, delegation, revocation, target, or carrier evidence.

## Claiming work

A worker claim must bind the evaluation attempt to:

- `targetId`;
- a claimed evidence generation;
- the complete evidence snapshot used by that evaluation, or a durable reference that resolves to it;
- implementation-private claim or lease metadata when concurrent workers are possible.

The worker evaluates the complete target graph for the claimed generation. Processing only the transition that caused one physical delivery is insufficient.

## Commit-time stale guard

A running worker need not be cancelled when newer evidence arrives. Before committing a semantic result, it must recompute or atomically read the current target generation and compare it with the claimed generation.

| Condition | Required result |
|---|---|
| claimed generation equals current generation | the result may be durably committed |
| claimed generation differs from current generation | the result is stale and must not update the semantic checkpoint or effective view |

The generation comparison and semantic checkpoint update must be protected against a race in which evidence changes between the comparison and commit. An implementation may use a transaction, compare-and-swap, generation-conditioned write, or equivalent atomic mechanism.

Merely checking `snapshotStillCurrent` in process memory is not a production concurrency guarantee.

## Completion

Successful completion for generation `gN` requires all of the following:

1. the complete target-scoped evidence snapshot for `gN` was evaluated;
2. `gN` remained current through the atomic commit boundary;
3. the derived result was durably committed;
4. the semantic checkpoint advanced to `gN`;
5. pending work for a newer generation was not cleared;
6. duplicate physical deliveries did not create duplicate semantic application.

A stale result remains non-authoritative even when its calculation succeeded. It may be recorded as diagnostic or attempt history, but it must not replace the current semantic checkpoint.

## Failure and retry

A retry must preserve the same target-scoped semantics:

- retrying the same current generation may commit at most one semantic application;
- retrying an older generation after evidence advanced must fail the stale guard;
- an older completed delivery must not mark newer target work succeeded;
- transition evidence must not be copied to make a retry unique;
- delivery count is not part of transition identity or evidence generation.

The current conformance runner fixes two internal cases:

- a stale generation produces zero derived applications, preserves the prior checkpoint, and remains retryable;
- a still-current generation produces one derived application, advances the checkpoint, and succeeds.

These fixtures model required outcomes. They do not demonstrate a production queue, lease, transaction, or reconciliation implementation.

## Reconciliation

A conforming reconciliation process compares, for each target:

- current evidence generation;
- current semantic checkpoint generation;
- current logical intent, if any.

When the evidence generation and checkpoint differ and no current intent exists, reconciliation recreates one pending target-scoped intent for the current generation.

Reconciliation must not infer success solely from the presence of an append-only trigger record. It must use durable semantic checkpoint state and current evidence.

The current Rust relay appends a re-evaluation trigger during transition ingestion. A production reconciliation worker and target-keyed logical intent store are not established by that append path.

## Separation of concerns

Coalescing does not decide:

- whether a Transition Object is structurally valid;
- whether its publisher signature is valid;
- whether authority is authorized, unauthorized, or unknown;
- which authorized transition heads survive supersession;
- how incomplete evidence affects last-known-good reads.

Those decisions are governed by their respective contracts. Coalescing schedules and protects a target-scoped evaluation of those rules.

## Safety requirements

A conforming implementation must satisfy all of the following:

- immutable evidence bytes and identities are not rewritten by coalescing;
- no evidence is discarded because physical work was coalesced;
- timestamps, input order, and identifiers do not select a graph winner;
- stale workers cannot overwrite a newer semantic view;
- duplicate deliveries cannot create duplicate logical application;
- a completed older delivery cannot clear newer pending work;
- a missing queue trigger can be reconstructed from evidence and checkpoint state;
- generation comparison and checkpoint commit are atomic with respect to newer evidence.

## Implementation boundary

Repository coverage currently consists of:

- the transition ingest path, which appends an immutable transition record and a pending queue trigger;
- conformance manifest cases for stale-generation and current-generation retry outcomes;
- the general evidence-generation and effective-view contracts.

The repository does not currently establish a production Rust implementation of:

- one mutable logical intent per target;
- generation-bearing worker claims;
- leases or concurrent-claim exclusion;
- atomic generation-conditioned checkpoint commits;
- target reconciliation;
- durable retry state transitions for the full lifecycle described here.

Consumers must not treat append-only `reevaluation-queue.jsonl` records alone as proof that the complete coalescing contract is enforced.

## Related contracts

- [`TRANSITION_REEVALUATION_QUEUE.md`](./TRANSITION_REEVALUATION_QUEUE.md)
- [`TRANSITION_EVIDENCE_GENERATION.md`](./TRANSITION_EVIDENCE_GENERATION.md)
- [`TRANSITION_AUTHORITY.md`](./TRANSITION_AUTHORITY.md)
- [`TRANSITION_SUPERSESSION.md`](./TRANSITION_SUPERSESSION.md)
- [`LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md`](./LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md)
