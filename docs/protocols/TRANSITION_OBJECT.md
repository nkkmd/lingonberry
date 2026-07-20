# Transition Object Contract

**Status: draft for v0.6.0** | **Schema version: `0.1.0`** | **Identity rule: `lb.transition.identity.v1`** | **Last updated: 2026-07-20**

## 1. Purpose

A transition object records a replacement or withdrawal without mutating the original canonical knowledge object.

Transition objects are append-only protocol objects. They have their own canonical ID, provenance, raw reference, identity, publisher signature, authority classification, and conflict history.

## 2. Required fields

| Field | Contract |
|---|---|
| `id` | `lb:transition:` identifier |
| `schemaVersion` | `0.1.0` |
| `objectType` | `transition` |
| `transitionType` | `replace` or `withdraw` |
| `targetId` | canonical knowledge-object ID affected by the transition |
| `issuedAt` | `lb.timestamp.rfc3339.utc.v1` timestamp |
| `provenance` | origin evidence, using the same source structure as knowledge objects |
| `rawRef` | carrier/source reference |

Optional fields are `replacementId`, `supersedesTransitionId`, `reason`, `identityClaims`, and `meta`.

## 3. Type-specific invariants

### Replace

A `replace` transition MUST include `replacementId`. `replacementId` MUST differ from `targetId`.

### Withdraw

A `withdraw` transition MUST NOT include `replacementId`.

## 4. Explicit supersession

`supersedesTransitionId` identifies one earlier authorized transition that this transition explicitly supersedes.

The referenced transition MUST exist, target the same Knowledge Object, be structurally valid, and be authorized. A transition MUST NOT supersede itself.

Timestamp and identifier ordering do not create implicit supersession. Multiple authorized heads without an explicit supersession chain produce an `ambiguous` effective view.

## 5. Identity

`lb.transition.identity.v1` is:

```text
sha256(canonical-json(identity-basis))
```

The identity basis contains, when present:

```text
objectType
transitionType
targetId
replacementId
supersedesTransitionId
issuedAt
reason
```

The encoded identity key is:

```text
lb:key:lb.transition.identity.v1:sha256:<64-lowercase-hex>
```

Transport, provenance, raw references, metadata, canonical ID, and identity claims are excluded from the identity basis.

## 6. Publish envelope

Transition objects use the existing HTTP publish envelope and `lb.http.publish.signature.v1`. The publisher signs the complete request after removing only `publisher.signature`.

## 7. Append-only behavior

Publishing a transition never rewrites or deletes the target object. A consumer derives an effective view from the transition log.

Exact duplicate transitions are idempotent. Authorized, unauthorized, unknown-authority, ambiguous, and disputed transitions remain retained.

## 8. Validation boundary

Structural validation, identity validation, signature validation, and append-only persistence are protocol concerns.

Authorization and supersession projection are derived classifications. Only one unambiguous authorized head may affect the effective view.

## 9. Conformance

The corpus fixes:

- valid replacement and withdrawal transitions;
- invalid type-specific field combinations;
- transition identity derivation;
- original, delegated, unauthorized, and unknown authority;
- one authorized head;
- parallel authorized heads classified as `ambiguous`;
- explicit supersession producing one effective head.
