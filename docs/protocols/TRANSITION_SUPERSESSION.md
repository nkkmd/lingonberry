# Transition Supersession and Effective-view Conflict Rule

**Status: normative v1.0.0 pre-release contract** | **Rule version: `lb.transition.supersession.v1`**

## 1. Scope

This document defines how a consumer projects structurally valid Transition Objects for one target Knowledge Object after authority classification has completed.

Supersession does not change canonical Knowledge Objects, Transition Objects, signatures, identities, or stored bytes. It produces derived effective-view state only.

The fixed v1.0.0 release candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation changes do not redefine that candidate.

## 2. Input boundary

The graph input is target-scoped:

```text
targetId
+ retained Transition Objects for that target
+ authority classification for each transition
```

Only transitions classified `authorized` participate in the graph.

Transitions classified `unauthorized` or `unknown` remain retained evidence but:

- do not become graph vertices;
- do not become active heads;
- do not supersede another transition;
- cannot resolve ambiguity.

Structural validation, signature verification, target resolution, and authority classification occur before this rule.

## 3. Graph model

For each authorized transition:

- the transition ID is a vertex;
- every `supersedesTransitionIds` entry is a directed edge from the newer transition to the referenced parent;
- a head is an authorized transition that is not referenced as a parent by another authorized transition in the same valid graph.

Timestamp, input order, transition ID order, and physical storage order are not precedence rules.

## 4. Parent-set requirements

`supersedesTransitionIds`, when present, is semantically a set.

It MUST:

- be an array;
- be non-empty;
- contain unique valid Transition IDs;
- not contain the transition's own ID.

Every referenced parent MUST:

- be present in the evaluated authorized corpus;
- target the same `targetId` as the child;
- be structurally valid;
- be classified `authorized`.

A missing, unauthorized, unknown, cross-target, malformed, duplicate, or self-referencing parent makes the graph `invalid-transition-graph`.

Because only authorized transitions are inserted into the executable conformance graph, a reference to a retained `unknown` or `unauthorized` transition is observed as an unavailable parent and therefore fails closed.

## 5. Parent-set identity semantics

For `lb.transition.identity.v1`, a valid parent array is copied and sorted lexically in the transition identity basis before canonical JSON serialization.

Consequences:

- permutations of the same valid parent set produce the same transition identity;
- stored transition bytes are not rewritten;
- general JSON array order is not normalized;
- duplicate parent IDs are rejected, not silently removed.

Identity normalization does not validate graph existence, authority, target equality, or acyclicity.

## 6. Cycle validation

The authorized transition graph MUST be acyclic.

Any direct or indirect cycle yields:

```text
invalid-transition-graph
```

No partial branch of a cyclic graph may be projected as effective state.

## 7. Projection states

After parent and cycle validation:

| Authorized heads | Projection |
|---|---|
| none | `active-original` |
| one `replace` head | `replaced` |
| one `withdraw` head | `withdrawn` |
| two or more heads | `ambiguous` |

For a single `replace` head, the derived result includes its transition ID and `replacementId`.

For a single `withdraw` head, the derived result includes its transition ID.

For `ambiguous`, the executable conformance result exposes lexically sorted `headTransitionIds` for deterministic diagnostics. Sorting the diagnostic list does not select a winner.

## 8. Fork resolution

A transition resolves an authorized fork only when, after applying all of its parent edges, the authorized graph has exactly one remaining head.

A merge transition normally names every current authorized head as a parent:

```json
{
  "supersedesTransitionIds": [
    "lb:transition:head-a",
    "lb:transition:head-b"
  ]
}
```

If only part of the current head set is superseded, at least two heads remain and the result stays `ambiguous`.

The executable conformance implementation does not use a separate "atomic merge" flag. Atomic fork resolution is the graph consequence of superseding the complete authorized head set and leaving one head.

## 9. Fail-closed behavior

`ambiguous` and `invalid-transition-graph` MUST NOT:

- select a replacement;
- hide or withdraw the original object;
- mutate canonical storage;
- rewrite Transition Objects;
- advance a semantic effective-view checkpoint as if projection succeeded.

A consumer retains the last known good derived view according to `LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md` until a current evidence generation produces an unambiguous valid result.

## 10. Determinism

For the same target-scoped authorized corpus, conforming consumers MUST produce the same:

- graph validity classification;
- head set;
- projection classification;
- effective transition ID and replacement ID when applicable;
- diagnostic head ID ordering.

No wall-clock ordering or storage traversal order may influence projection.

## 11. Executable conformance boundary

`conformance/run.mjs` currently provides the executable reference behavior for this rule.

The manifest covers:

- one authorized head;
- parallel authorized heads;
- explicit linear supersession;
- complete fork merge;
- partial fork merge;
- cycle detection;
- missing parent;
- cross-target parent;
- unauthorized parent.

Transition Object fixtures separately cover duplicate and self-referencing parent rejection and parent-array identity equivalence.

## 12. Current production implementation boundary

The repository currently does not contain an independent production Rust supersession graph evaluator equivalent to the executable conformance implementation.

The Rust relay can retain transitions and calculate effective-view responses through its current runtime path, but this document MUST NOT be read as evidence that every graph-validation and projection guarantee above is independently implemented and enforced in production Rust.

Before v1.0.0 publication, qualification evidence must distinguish:

- conformance-runner coverage;
- production runtime enforcement;
- fixture-only guarantees;
- unresolved implementation gaps.

## 13. Non-claims

This rule does not define:

- authorization or delegation semantics;
- Transition Object structural validity;
- signature verification;
- canonical object replacement in storage;
- deletion or garbage collection;
- queue claim, retry, or coalescing behavior;
- conflict resolution by operator preference.

## 14. Related contracts

- [`TRANSITION_OBJECT.md`](./TRANSITION_OBJECT.md)
- [`TRANSITION_AUTHORITY.md`](./TRANSITION_AUTHORITY.md)
- [`TRANSITION_EVIDENCE_GENERATION.md`](./TRANSITION_EVIDENCE_GENERATION.md)
- [`LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md`](./LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md)
- [`TRANSITION_REEVALUATION_QUEUE.md`](./TRANSITION_REEVALUATION_QUEUE.md)
- [`TRANSITION_REEVALUATION_COALESCING.md`](./TRANSITION_REEVALUATION_COALESCING.md)
