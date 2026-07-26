# Transition Authority Classification

**Status: normative protocol contract for the v1.0.0 pre-release implementation**  
**Rule version: `lb.transition.authority.v1`**  
**Protocol version: `0.1.0`**  
**Last verified: 2026-07-26**

## 1. Scope

This document defines how a consumer classifies whether an already structurally valid and correctly signed Transition Object is authorized to affect a target Knowledge Object.

Authority classification is derived state. It is separate from:

- Transition Object schema and identity validation;
- publish-envelope signature verification;
- target availability;
- append-only evidence retention;
- supersession and conflict resolution;
- effective-view projection.

A valid signature proves control of the transition publisher key. It does not by itself prove authority over the target.

## 2. Classification and effect

A structurally valid, correctly signed transition is retained as evidence regardless of its authority classification. Authority determines semantic effect, not evidence retention.

| Classification | Basis used by the reference classifier | Retain transition | Apply to effective view |
|---|---|---:|---:|
| `authorized` | `original-publisher` or `delegated-publisher` | yes | yes |
| `unauthorized` | `no-applicable-authority` | yes | no |
| `unknown` | `target-unavailable`, `target-publisher-unknown`, or `authority-evidence-incomplete` | yes | no |

A relay or consumer MUST NOT delete, overwrite, or mutate stored transition evidence solely because the transition is `unauthorized` or `unknown`.

Only `authorized` transitions are eligible for later supersession and effective-view processing. Eligibility does not guarantee that a transition becomes the unique effective head; conflict resolution is governed separately by `lb.transition.supersession.v1`.

## 3. Inputs

The reference classification function consumes derived inputs rather than extracting all evidence directly from stored protocol objects:

- `targetAvailable`;
- `transitionPublisherKey`;
- `targetPublisherKey`, when known;
- zero or more delegation records;
- transition `issuedAt`.

The transition publisher key is the public key whose signature has already been verified for the transition publish envelope. The target publisher key is publisher evidence associated with the original target record by the consuming implementation.

The protocol schema does not define a standalone delegation or revocation object in the current `0.1.0` contract. Delegation records supplied to this classifier are therefore verified external or implementation-managed authority evidence. Implementations MUST NOT treat an unverified delegation record as authority.

## 4. Reference classification algorithm

For `lb.transition.authority.v1`, the conformance reference classifier applies the following order:

1. If `targetAvailable` is exactly `false`, return `unknown` with basis `target-unavailable`.
2. If `targetPublisherKey` is absent or null, return `unknown` with basis `target-publisher-unknown`.
3. If `transitionPublisherKey` equals `targetPublisherKey`, return `authorized` with basis `original-publisher`.
4. Examine each supplied delegation record.
5. A delegation can authorize the transition only when all of the following hold:
   - `verified` is exactly `true`;
   - `issuerKey` equals `targetPublisherKey`;
   - `delegateKey` equals `transitionPublisherKey`;
   - `scopes` is an array containing `transition`;
   - `validFrom` and `validUntil` satisfy the producer UTC timestamp classifier;
   - `issuedAt` is not before `validFrom` and not after `validUntil`;
   - when `revokedAt` is present, it satisfies the producer UTC timestamp classifier and is later than `issuedAt`.
6. If an applicable delegation exists, return `authorized` with basis `delegated-publisher`.
7. If any examined delegation has incomplete verification or invalid authority timestamps, and no applicable delegation was found, return `unknown` with basis `authority-evidence-incomplete`.
8. Otherwise return `unauthorized` with basis `no-applicable-authority`.

`target-unavailable` and `target-publisher-unknown` are intentionally distinct. The first means the referenced target is not locally resolvable. The second means the target is available but sufficient publisher evidence is absent.

## 5. Timestamp comparison boundary

The conformance reference classifier requires delegation timestamps to match the UTC producer form defined by `lb.timestamp.rfc3339.utc.v1` and compares `issuedAt`, `validFrom`, `validUntil`, and `revokedAt` as strings.

For timestamps in the required fixed-width UTC form, lexicographic ordering is used as the reference behavior. The classifier does not:

- convert offsets;
- parse timestamps into a calendar or epoch type;
- reconcile different fractional-second precisions;
- validate a trusted clock source.

Consumers MUST NOT supply explicit-offset or otherwise noncanonical timestamps to this comparison path and claim conformance with the reference result. A future rule that performs instant-based timestamp comparison requires a new rule version.

## 6. Fail-closed behavior

`unauthorized` and `unknown` transitions MUST NOT replace or withdraw a target in derived projections.

Missing or unsupported authority evidence MUST produce `unknown`, not an optimistic `authorized` result. Evidence that is complete and shows no applicable authority produces `unauthorized`.

Authority classification does not establish:

- that the target itself is trustworthy;
- that the transition is the latest transition;
- that no competing authorized transition exists;
- that the publisher key belongs to a particular legal or human identity;
- that delegation or revocation evidence is globally complete.

## 7. Re-evaluation

An `unknown` classification MAY be re-evaluated when missing target, publisher, delegation, or revocation evidence becomes available.

Re-evaluation changes derived classification only. It MUST preserve:

- the original transition bytes;
- transition identity;
- verified publish-envelope evidence;
- original receipt and storage metadata.

Target-arrival behavior is specified by `lb.transition.orphan.v1` in [ORPHAN_TRANSITIONS.md](./ORPHAN_TRANSITIONS.md). Queueing and retry behavior are specified separately by the transition re-evaluation queue and coalescing contracts.

## 8. Implementation boundary

The v1.0.0 pre-release repository provides an executable JavaScript reference classifier in `conformance/run.mjs` and authority cases in `conformance/manifest.v1.json`.

The current Rust protocol library does not expose a production authority-classification API, a delegation store, or a revocation resolver. Therefore:

- passing the external conformance cases demonstrates agreement with the reference contract;
- it does not demonstrate end-to-end authority enforcement in every carrier, relay, storage, or indexer path;
- operators and integrators MUST verify separately where target publisher evidence and delegation evidence originate and where effective-view filtering is enforced.

This document MUST NOT be interpreted as claiming a repository-wide production authorization service that is not present in the fixed candidate.

## 9. Conformance coverage

The external conformance manifest includes cases for:

- original publisher authorization;
- delegated publisher authorization;
- unauthorized publisher classification;
- unknown target-publisher classification;
- orphan target retention and target-arrival re-evaluation;
- downstream supersession filtering of unauthorized transitions.

The manifest is executable specification evidence. It is not a substitute for privileged-host qualification, formal soak evidence, or deployment-specific authority-source review.

## 10. Release boundary

The fixed v1.0.0 release candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

This documentation update does not redefine that candidate. Formal 72-hour soak, privileged reference-host qualification, version update, release PR, tag, and GitHub Release remain incomplete.