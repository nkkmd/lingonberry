# Transition Object Contract

## Status

Normative protocol contract for the v1.0.0 pre-release documentation set.

- Transition schema version: `0.1.0`
- Transition identity rule: `lb.transition.identity.v1`
- Canonical JSON rule: `lb.canonical.json.v1`
- HTTP transition signature rule: `lb.http.publish.signature.v1`

This document describes the intended protocol contract and records the narrower validation currently performed by the reference relay. It does not change the fixed v1.0.0 release candidate.

## 1. Purpose

A Transition Object records a replacement or withdrawal without mutating the original canonical Knowledge Object.

Transitions are append-only protocol objects. Their immutable content, signed publish request, carrier evidence, authority classification, supersession relationships, and derived effective-view effect are related but distinct concepts.

A stored transition does not by itself prove that it is authorized or that it affects the effective view.

## 2. Object shape

The schema is [`schemas/transition-object.schema.json`](../../schemas/transition-object.schema.json).

Required fields:

| Field | Contract |
|---|---|
| `id` | Bounded ASCII `lb:transition:` identifier |
| `schemaVersion` | Exactly `0.1.0` |
| `objectType` | Exactly `transition` |
| `transitionType` | `replace` or `withdraw` |
| `targetId` | Bounded ASCII `lb:obj:` identifier |
| `issuedAt` | Zoned timestamp; conforming producers emit the UTC form defined by `lb.timestamp.rfc3339.utc.v1` |
| `provenance` | Origin evidence using the Knowledge Object provenance structure |
| `rawRef` | Carrier/source reference using the Knowledge Object raw-reference structure |

Optional fields:

- `replacementId`
- `supersedesTransitionIds`
- `reason`
- `identityClaims`
- `meta`

Unknown top-level fields are rejected by the JSON Schema and the reference relay ingest validator.

## 3. Type-specific invariants

### 3.1 Replace

A `replace` transition:

- MUST contain `replacementId`;
- MUST use a bounded ASCII `lb:obj:` replacement identifier;
- MUST have `replacementId` different from `targetId`.

The last rule is fixed by the external conformance contract. At the current pre-release boundary, the JSON Schema and Rust relay ingest validator require a valid replacement identifier but do not independently reject `replacementId == targetId`. Deployments claiming full conformance must enforce the inequality before treating the transition as structurally valid.

### 3.2 Withdraw

A `withdraw` transition MUST NOT contain `replacementId`.

## 4. Explicit supersession

`supersedesTransitionIds`, when present, is a non-empty array of bounded ASCII `lb:transition:` identifiers.

Structural requirements:

- duplicate parent IDs are invalid;
- a transition MUST NOT supersede itself;
- input array order has no semantic precedence.

Graph requirements are evaluated separately from structural validation. For an authorized transition to supersede an earlier transition, the referenced parent must exist, target the same Knowledge Object, be structurally usable, and be authorized under the applicable authority rules.

A transition resolves a fork atomically only when it explicitly supersedes every relevant authorized head. Partial coverage does not silently select a winner and leaves the effective view ambiguous.

Timestamp order, ingestion order, lexical identifier order, and carrier receipt order do not create implicit supersession.

## 5. Transition identity

`lb.transition.identity.v1` is encoded as:

```text
lb:key:lb.transition.identity.v1:sha256:<64-lowercase-hex>
```

The digest is:

```text
sha256(UTF-8(lb.canonical.json.v1(normalized identity basis)))
```

The identity basis includes only these fields when present:

```text
objectType
transitionType
targetId
replacementId
supersedesTransitionIds
issuedAt
reason
```

Before canonical JSON serialization, a present `supersedesTransitionIds` array is copied and sorted lexically. This normalization applies only to the transition identity basis. It does not rewrite the stored Transition Object and does not change general array-order semantics in `lb.canonical.json.v1`.

Therefore, permutations of the same valid parent set produce the same transition identity. Duplicate parents remain structurally invalid rather than being normalized away.

The following are excluded from the transition identity basis:

- canonical transition `id`;
- provenance;
- raw references;
- transport and receipt metadata;
- `identityClaims`;
- `meta`;
- publisher key and signature.

The external conformance runner derives and checks this identity. The current Rust transition ingest path does not independently derive `lb.transition.identity.v1` or verify a supplied `identityClaims` entry against the transition content. An accepted signature or stored transition is therefore not evidence that an identity claim was validated.

## 6. Publish envelope and signature

Transitions are published to the transition-specific HTTP route using an envelope with exactly:

```json
{
  "transition": {},
  "publisher": {
    "publicKey": "<64 lowercase hex>",
    "signature": "<128 lowercase hex>"
  }
}
```

The signature target is the complete request after removing only `publisher.signature`, serialized with canonical JSON. The Transition Object itself does not contain the publisher signature.

Envelope validation, signature validation, transition structural validation, authority evaluation, and graph projection are separate stages.

## 7. Validation boundaries

The normative contract covers:

- required and allowed fields;
- identifier classes and bounds;
- transition-type invariants;
- timestamp contract;
- provenance and raw-reference structures;
- transition identity derivation;
- signature target construction;
- append-only duplicate and conflict behavior.

Current reference relay limitations include:

- `issuedAt` is checked with the relay's zoned RFC 3339 shape validator, while conforming producers are expected to emit the uppercase-`Z` UTC profile;
- `provenance` and `rawRef` are checked as objects during transition ingestion, but their full referenced schema is not revalidated in that Rust function;
- `identityClaims` and `meta` are accepted by shape without transition-identity verification;
- `replacementId != targetId` is not yet enforced by the Rust ingest validator;
- structural ingest does not establish authority or effective-view applicability.

Consumers must not promote these implementation gaps into alternative protocol rules.

## 8. Append-only storage and duplicates

Publishing a transition never rewrites or deletes the target Knowledge Object.

For the reference relay:

- the canonicalized complete signed request is stored in an append-only transition log;
- an existing transition ID with exactly the same canonical request is idempotent and returns the duplicate result;
- an existing transition ID with different immutable request content is a conflict;
- a structurally and cryptographically accepted orphan transition may be retained before its target is available;
- storage does not imply authorization.

Authorized, unauthorized, unknown-authority, ambiguous, disputed, and orphan evidence may remain retained for audit and later re-evaluation. Only a valid, authorized, unambiguous derived result may affect the effective view.

## 9. Conformance coverage

The external corpus covers:

- valid replacement and withdrawal transitions;
- missing replacement and forbidden replacement combinations;
- duplicate and self-referencing parent rejection;
- transition identity derivation;
- parent-set order equivalence;
- original, delegated, unauthorized, and unknown authority classifications;
- a single authorized head;
- parallel authorized heads classified as ambiguous;
- full-fork supersession;
- partial fork coverage remaining ambiguous;
- missing, cross-target, unauthorized, and cyclic supersession evidence.

Conformance fixtures are executable reference evidence. They do not by themselves prove that every carrier, relay configuration, persistence path, or downstream consumer enforces every rule.

## 10. Release boundary

This normalization is documentation-only. The fixed v1.0.0 candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

The formal 72-hour soak, privileged reference-host qualification, version update, release PR, tag, and GitHub Release remain separate incomplete gates.
