# Orphan Transition Retention

**Status: draft for v0.6.0** | **Rule version: `lb.transition.orphan.v1`** | **Last updated: 2026-07-20**

## 1. Purpose

A structurally valid, correctly signed Transition Object may arrive before its target Knowledge Object. A conforming relay retains that transition append-only as orphan evidence rather than rejecting it solely because `targetId` is not yet locally available.

This rule preserves signed evidence under out-of-order distributed ingestion while keeping effective-view behavior fail closed.

## 2. Classification

When the target Knowledge Object is unavailable, the relay records derived state equivalent to:

```json
{
  "targetStatus": "missing",
  "authority": {
    "classification": "unknown",
    "basis": "target-unavailable"
  },
  "applyToEffectiveView": false
}
```

The transition remains structurally valid and retained. `targetStatus`, authority classification, and effective-view applicability are derived state and are not inserted into or used to rewrite the stored Transition Object.

## 3. Storage requirements

A relay MUST retain:

- the exact signed publish request bytes or equivalent immutable wire evidence;
- the validated Transition Object bytes;
- the transition identity;
- the transition publisher identity;
- the fact that target resolution was unavailable at evaluation time.

A relay MUST NOT:

- mutate or delete another Knowledge Object;
- invent a target publisher;
- classify the transition as authorized or unauthorized without target authority evidence;
- apply replacement or withdrawal semantics;
- rewrite the stored transition when the target later arrives.

## 4. HTTP result

A newly retained orphan transition returns the normal durable-storage success class:

```text
HTTP 201
code: LB_TRANSITION_STORED
```

The response additionally reports:

```json
{
  "targetStatus": "missing",
  "authority": {
    "classification": "unknown",
    "basis": "target-unavailable"
  },
  "effectiveView": {
    "applied": false
  }
}
```

Target absence is not represented as `404`, `409`, or storage failure after the signed transition has otherwise passed required validation.

## 5. Re-evaluation

When the target Knowledge Object becomes available, the relay re-evaluates derived state using the stored transition publisher, target publisher evidence, verified delegations, revocations, and the original transition `issuedAt`.

Re-evaluation may change:

- `targetStatus` from `missing` to `available`;
- authority from `unknown` to `authorized` or `unauthorized`;
- effective-view projection.

Re-evaluation MUST NOT change:

- Transition Object bytes;
- transition identity;
- publisher signature evidence;
- original receipt metadata.

If required evidence remains incomplete, authority remains `unknown` and the transition remains excluded from the effective view.

## 6. Duplicate and conflict behavior

An exact duplicate orphan transition is idempotent and returns `LB_TRANSITION_DUPLICATE`.

The same transition ID with different signed bytes is `LB_TRANSITION_CONFLICT`, regardless of target availability. Later target arrival MUST NOT convert a prior duplicate or conflict classification.

## 7. Supersession boundary

An orphan transition cannot become an authorized graph head or satisfy `supersedesTransitionIds` while its authority is `unknown`.

A transition that references an orphan parent remains excluded from effective-view resolution until the referenced parent and its authority can be resolved. Implementations MUST NOT treat target absence as proof that a parent is invalid or unauthorized.

## 8. Conformance

The conformance corpus fixes:

- retention of a valid signed transition whose target is missing;
- authority `unknown` with basis `target-unavailable`;
- effective-view exclusion while orphaned;
- deterministic re-evaluation after target arrival;
- preservation of transition identity and stored bytes across re-evaluation.
