# Transition supersession and effective-view conflict rule

Rule version: `lb.transition.supersession.v1`

## Principle

Multiple authorized transitions targeting the same Knowledge Object are not ordered by timestamp or identifier. They remain append-only evidence. The effective view is updated only when the authorized transition graph has one unambiguous active head.

## Supersession field

A Transition Object may contain:

```json
{"supersedesTransitionId":"lb:transition:previous"}
```

The referenced transition MUST:

- exist in the evaluated corpus;
- target the same `targetId`;
- be structurally valid;
- be classified `authorized`;
- not be the transition itself.

The field participates in `lb.transition.identity.v1`.

## Projection states

- no authorized transition: `active-original`
- one authorized head of type `replace`: `replaced`
- one authorized head of type `withdraw`: `withdrawn`
- two or more authorized heads: `ambiguous`
- cycle, missing superseded transition, cross-target reference, or unauthorized supersession edge: `invalid-transition-graph`

`unauthorized` and `unknown` transitions are retained but do not become graph heads and cannot resolve ambiguity.

## Fail-closed behavior

An `ambiguous` or `invalid-transition-graph` result MUST NOT select a replacement, hide the original object, or mutate canonical storage. A later authorized transition may resolve ambiguity only by explicitly superseding every currently authorized head. Because the schema carries one `supersedesTransitionId`, resolving a fork requires a sequence of authorized merge transitions or a future schema revision with a multi-parent field.

## Determinism

Input order, `issuedAt`, and transition ID lexical order MUST NOT affect the result.
