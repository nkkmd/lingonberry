# Transition supersession and effective-view conflict rule

Rule version: `lb.transition.supersession.v1`

## Principle

Multiple authorized transitions targeting the same Knowledge Object are not ordered by timestamp or identifier. They remain append-only evidence. The effective view is updated only when the authorized transition graph has one unambiguous active head.

## Multi-parent supersession

A Transition Object may contain:

```json
{
  "supersedesTransitionIds": [
    "lb:transition:previous-a",
    "lb:transition:previous-b"
  ]
}
```

The array MUST be non-empty and contain unique transition IDs. Every referenced transition MUST:

- exist in the evaluated corpus;
- target the same `targetId`;
- be structurally valid;
- be classified `authorized`;
- not be the transition itself.

A transition resolves a fork atomically only when its parent set contains every currently authorized head. Referencing only part of the current head set leaves the graph `ambiguous`.

## Parent-set semantics

`supersedesTransitionIds` is semantically a set, not an ordered history.

For `lb.transition.identity.v1`, a valid parent array is copied and sorted lexically before canonical JSON serialization. The stored object is not rewritten, general JSON array order remains unchanged, and duplicate IDs are rejected rather than removed.

Permutations of the same valid parent set therefore produce the same transition identity.

## Projection states

- no authorized transition: `active-original`
- one authorized head of type `replace`: `replaced`
- one authorized head of type `withdraw`: `withdrawn`
- two or more authorized heads: `ambiguous`
- cycle, missing parent, cross-target reference, unauthorized parent, duplicate parent, or self-reference: `invalid-transition-graph`

`unauthorized` and `unknown` transitions are retained but do not become graph heads and cannot resolve ambiguity.

## Fail-closed behavior

An `ambiguous` or `invalid-transition-graph` result MUST NOT select a replacement, hide the original object, or mutate canonical storage.

Timestamp, input order, transition ID order, and parent-array order MUST NOT select a winner.

## Conformance boundary

The conformance corpus fixes:

- single authorized head projection;
- parallel authorized heads remaining `ambiguous`;
- linear supersession;
- atomic multi-head fork resolution;
- partial fork coverage remaining `ambiguous`;
- duplicate and self-referencing parent rejection;
- identity equivalence across parent-array permutations.
