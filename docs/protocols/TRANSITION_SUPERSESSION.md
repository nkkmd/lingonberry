# Transition supersession and effective-view conflict rule

Rule version: `lb.transition.supersession.v1`

## Principle

Multiple authorized transitions targeting the same Knowledge Object are never ordered by timestamp or identifier. They remain append-only evidence. The effective view changes only when the authorized transition graph has one unambiguous active head.

## Multi-parent supersession

A Transition Object may contain:

```json
{
  "supersedesTransitionIds": [
    "lb:transition:a",
    "lb:transition:b"
  ]
}
```

The array MUST be non-empty, contain unique transition IDs, and MUST NOT contain the enclosing transition ID.

Every referenced transition MUST:

- exist in the evaluated corpus;
- target the same `targetId`;
- be structurally valid;
- be classified `authorized`;
- not be the transition itself.

The array participates in `lb.transition.identity.v1`. Array order is preserved by canonicalization and is therefore identity-significant. Producers SHOULD sort IDs lexically before signing so semantically equivalent parent sets produce one identity.

## Projection states

- no authorized transition: `active-original`
- one authorized head of type `replace`: `replaced`
- one authorized head of type `withdraw`: `withdrawn`
- two or more authorized heads: `ambiguous`
- cycle, missing parent, cross-target reference, unauthorized parent, duplicate parent, or self-reference: `invalid-transition-graph`

`unauthorized` and `unknown` transitions are retained but do not become graph heads and cannot resolve ambiguity.

## Fork resolution

A fork is atomically resolved only when a later authorized transition explicitly supersedes every current authorized head. Superseding only part of a fork leaves the remaining heads active and the result `ambiguous`.

## Fail-closed behavior

An `ambiguous` or `invalid-transition-graph` result MUST NOT select a replacement, hide the original object, or mutate canonical storage.

Timestamp, input order, and transition ID lexical order MUST NOT choose a winner.
