# Lingonberry Protocol Evolution Proposal

**Status: architecture proposal**  
**Normative status: non-normative until adopted by an explicit specification change**

## 1. Purpose

This document describes how Lingonberry may evolve after the current v1.0 release candidate. It is not a protocol contract, release checklist, or claim that the proposed capabilities already exist.

The fixed v1.0 candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Documentation commits made after that candidate do not redefine its runtime contents.

## 2. Current baseline

The checked-in repository already defines and tests a production-oriented single-node baseline, including:

- versioned protocol and schema constants;
- deterministic JSON canonicalization;
- a versioned identity-key rule;
- protocol-native wire formats;
- HTTP publish and signature contracts;
- append-only storage and archive replay;
- duplicate and conflict classification;
- quarantine and replacement workflows;
- transition and effective-view semantics;
- conformance fixtures and documentation walkthrough evidence.

The concrete schemas, runtime code, tests, and conformance fixtures are authoritative for that baseline. This proposal cannot override them.

## 3. Evolution principles

Any protocol evolution should preserve these invariants unless a deliberately breaking migration is approved:

1. **Determinism** — the same accepted evidence produces the same canonical and derived results.
2. **Fail-closed ambiguity** — unknown versions, conflicting identities, invalid signatures, and unsupported semantics are not guessed through.
3. **Append-only evidence** — semantic change is represented by additional evidence rather than silent mutation of signed or identity-bearing bytes.
4. **Version separation** — release, protocol, schema, canonicalization, identity, signature, API, storage, and proof versions remain independent axes.
5. **Replayability** — durable evidence remains sufficient to reconstruct supported derived state.
6. **Authority separation** — signatures prove control of a key, not truth, correctness, or universal authorization.
7. **Carrier clarity** — carrier framing and transport policy do not silently alter protocol meaning.

## 4. Proposal requirements

A change proposal should include:

- motivation and non-goals;
- affected version axes;
- normative data and wire changes;
- compatibility classification;
- canonicalization and identity impact;
- signature and authority impact;
- migration and rollback behavior;
- security and resource-exhaustion analysis;
- test vectors and conformance changes;
- operational observability requirements;
- explicit behavior for unknown or unsupported versions.

A proposal is not adopted merely because it appears in this document.

## 5. Compatibility classes

Every proposed change should be classified before implementation.

| Class | Meaning | Required treatment |
|---|---|---|
| Additive compatible | Existing accepted objects retain their meaning and old implementations can safely reject or preserve the addition | New tests and explicit unknown-field behavior |
| Behaviorally compatible | Wire/schema shape is unchanged but deterministic processing is clarified | Golden vectors and replay comparison |
| Migration-required | Existing durable state must be transformed or rebuilt | Versioned migration, backup, rollback, and proof |
| Breaking semantic | Previously accepted evidence may acquire different meaning | New protocol/profile version and explicit coexistence policy |
| Cryptographic breaking | Canonical bytes, identity inputs, signature payload, or key rules change | New rule version; never reinterpret existing signatures in place |

Repository release version changes do not by themselves select any of these classes.

## 6. Candidate evolution areas

The following areas are candidates for future work. None are v1.0 guarantees.

### 6.1 Additional identity rules

Future identity rules may use different cryptographic constructions or semantic inputs. Each rule must have a distinct identifier, deterministic byte definition, collision analysis, and conformance vectors.

Existing identity-bearing evidence must retain the rule version under which it was created. A new rule must not silently rewrite old identifiers.

### 6.2 Signature profiles and key lifecycle

Possible extensions include:

- additional signature algorithms;
- organization or threshold signatures;
- delegated signing;
- key rotation and revocation evidence;
- explicit author, publisher, transformer, and attestor roles.

Each profile needs domain separation, exact signed bytes, verification rules, authority scope, and failure semantics.

### 6.3 Multi-node synchronization

Possible synchronization mechanisms include cursor exchange, content inventories, Merkle structures, partial replication, and archive chunk transfer.

Before adoption, a synchronization proposal must define:

- the unit of replication;
- duplicate and conflict behavior;
- ordering assumptions;
- retention and omission semantics;
- proof of completeness or explicit absence of such a proof;
- recovery after partial transfer;
- behavior under malicious peers.

Synchronization does not imply consensus, global ordering, or automatic conflict resolution.

### 6.4 Application profiles and extensions

Application profiles may define domain vocabulary, validation rules, relation families, and policy requirements without changing the protocol core.

A future extension mechanism must state whether unknown extensions are rejected, preserved opaquely, or ignored. The current strict schemas must not be treated as accepting arbitrary extension fields unless a new schema/profile version explicitly permits them.

### 6.5 Attachments and content-addressed blobs

A future attachment contract may separate small protocol objects from large binary artifacts. Such a contract must cover digest algorithms, media type, size, location hints, authorization, malware handling, retention, and unavailable-content behavior.

A URL alone is not an integrity proof, and a digest alone is not an availability guarantee.

### 6.6 Claim, review, and trust vocabularies

The protocol may support richer statements such as support, contradiction, review, retraction, or replication evidence. These remain claims with provenance; the protocol does not certify truth.

Trust ranking, reputation, and contextual applicability should remain application or local-policy concerns unless a narrowly scoped protocol contract is adopted.

## 7. Conformance and independent implementations

Every adopted semantic or cryptographic change should add machine-readable vectors covering:

- valid and invalid inputs;
- canonical bytes or canonical JSON where applicable;
- identity and signature results;
- unknown-version behavior;
- migration and replay outcomes;
- malformed and resource-exhaustion cases.

Independent implementations are valuable evidence of specification clarity, but the current v1.0 release is not contingent on implementing every future proposal in a second language.

## 8. Security requirements

Evolution must assume hostile input and hostile peers. Proposals should bound object size, nesting, collection counts, verification cost, decompression, archive import, replay amplification, and synchronization work.

Cryptographic integrity does not provide confidentiality, authorization, availability, malware safety, legal compliance, or correctness of assertions.

## 9. Governance direction

A future Lingonberry Proposal process may use identifiers such as `LBP-0001`, with states such as Draft, Experimental, Accepted, Stable, Deprecated, and Rejected.

The process should require reviewable specification text, compatibility impact, security considerations, conformance evidence, and migration guidance. Emergency security fixes may require an expedited path, but must still document the resulting contract.

No proposal identifier or lifecycle is currently a protocol feature merely because it is suggested here.

## 10. Adoption sequence

A conservative sequence for a future change is:

1. write a narrowly scoped proposal;
2. identify affected normative contracts and version axes;
3. add fixtures and negative cases before or with implementation;
4. implement without changing unrelated semantics;
5. demonstrate deterministic replay and backend parity;
6. document migration, rollback, and observability;
7. run compatibility and security review;
8. adopt the change through an explicit versioned release decision.

## 11. Explicit v1.0 non-guarantees

This proposal does not claim that v1.0 provides:

- multi-relay convergence;
- federation or consensus;
- Merkle or set-reconciliation synchronization;
- signed application-profile catalogs;
- arbitrary extension preservation;
- attachment/blob distribution;
- key revocation infrastructure;
- cross-implementation interoperability for every component;
- protocol-level truth or trust scoring.

## 12. Release boundary

The formal 72-hour soak has not been performed. Privileged reference-host qualification remains incomplete. Version preparation, the release PR, tag creation, and GitHub Release publication remain incomplete.

Those release gates are separate from this post-candidate architecture proposal.