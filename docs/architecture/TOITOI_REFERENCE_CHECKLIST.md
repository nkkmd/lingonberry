# Toitoi Reference Checklist

**Status:** active reference checklist  
**Normative status:** non-normative  
**Scope:** evaluating ideas and artifacts from the external Toitoi project for possible use in Lingonberry

## Purpose

This checklist helps maintainers examine Toitoi material without treating an external repository as part of the Lingonberry specification.

Use it to:

- identify design ideas that may be relevant to Lingonberry;
- separate domain-specific Toitoi semantics from protocol-level concerns;
- record why an idea was adopted, adapted, or rejected;
- avoid copying external schemas or terminology without compatibility analysis; and
- preserve the boundary between checked-in Lingonberry contracts and historical design influences.

This document does not define the Toitoi specification, import Toitoi behavior into Lingonberry, or create a compatibility promise between the projects.

## Authority boundary

For Lingonberry behavior, the authority order is:

1. checked-in schemas and conformance fixtures;
2. runtime behavior covered by tests;
3. normative Lingonberry protocol documents;
4. operational and architecture guidance;
5. this checklist and external Toitoi material.

An external document cannot override a Lingonberry schema, identifier rule, canonicalization rule, signature contract, transition rule, storage contract, API contract, or release gate.

The fixed v1.0 release candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Reviewing or revising this checklist does not redefine that candidate. The formal 72-hour soak, privileged reference-host qualification, version preparation, release pull request, tag, and GitHub Release remain separate incomplete gates.

## Core separation rules

A review must preserve these boundaries:

- **protocol and carrier are distinct**: a transport may carry protocol objects, but transport-local behavior must not silently become protocol semantics;
- **canonical evidence and derived state are distinct**: indexes, projections, caches, and effective views are rebuildable outputs, not replacement evidence;
- **identity and provenance are distinct**: identity answers what an object is; provenance records origin and processing history;
- **authenticity and truth are distinct**: a valid signature authenticates a signed statement but does not prove its factual correctness;
- **Lingonberry core and application profiles are distinct**: domain vocabulary and workflow policy belong outside the core unless explicitly standardized;
- **external influence and normative adoption are distinct**: an idea becomes part of Lingonberry only through checked-in contracts, implementation, tests, and review.

## External material to inspect

When relevant, inspect the current Toitoi repository rather than relying on filenames or field lists copied into this checklist. Useful categories include:

- canonical event or object schemas;
- identity-claim and provenance models;
- raw-reference handling;
- lineage and relationship semantics;
- protocol abstraction and carrier adapters;
- indexer and API boundaries;
- terminology and application-profile documents; and
- migration, compatibility, and conformance material.

External paths may move or disappear. Record the repository URL, exact commit, path, and access date in the review evidence. A branch-relative `main` URL alone is not sufficient evidence for a compatibility decision.

## Review worksheet

For every external artifact, record the following.

| Field | Required question |
|---|---|
| Source | Which repository, commit, and path were reviewed? |
| Status | Is the source normative, draft, historical, generated, or implementation-specific? |
| Problem | What concrete Lingonberry problem could the material address? |
| Existing contract | Which current Lingonberry schema, document, runtime path, or fixture already governs the area? |
| Proposed use | Adopt unchanged, adapt, use as background only, or reject? |
| Compatibility | Is the change additive, behavioral, migration-required, semantically breaking, or cryptographically breaking? |
| Identity effect | Does it change canonical bytes, canonical IDs, identity keys, signatures, or signed evidence? |
| Storage effect | Does it change append, duplicate/conflict, replay, quarantine, transition, or effective-view behavior? |
| Carrier effect | Does it introduce transport-specific assumptions into the protocol? |
| Profile effect | Should the concept remain in a Toitoi application profile instead of Lingonberry core? |
| Security effect | What new trust boundary, parser risk, resource limit, key lifecycle, or abuse case appears? |
| Conformance | Which positive and negative fixtures prove the proposed behavior? |
| Migration | How are existing evidence, nodes, clients, and archives handled? |
| Decision evidence | Where is the accepted decision, implementation, and test evidence recorded? |

## Topic-specific checks

### Object model

Determine whether the external model describes:

- a protocol-core object;
- a domain-specific object such as an inquiry, observation, claim, or evidence item;
- a profile-level vocabulary;
- a carrier envelope; or
- derived index state.

Do not add domain types to the Lingonberry core merely because they are central to Toitoi. Prefer a profile or extension when the semantics are domain-specific.

### Identity and canonicalization

Check whether the proposal changes:

- canonical byte generation;
- canonical ID derivation;
- semantic identity keys;
- duplicate/conflict classification;
- Unicode, number, timestamp, array, or optional-field handling;
- identity claims; or
- domain-separated signature payloads.

Any such change requires explicit versioning, migration analysis, fixtures, and compatibility review. Existing signed or identity-bearing evidence must not be silently reinterpreted.

### Provenance and raw references

Clarify:

- which actor produced, transformed, published, or attested the object;
- whether a raw reference identifies content, location, or both;
- whether referenced material is required for verification or only audit context;
- how unavailable or mutable external locations are handled; and
- whether privacy-sensitive or secret material could be exposed.

Do not treat provenance metadata as proof of truth or authorization.

### Revision, supersession, and removal

Separate:

- creation of a new revision;
- an explicit supersession transition;
- retraction or dispute;
- presentation-level hiding;
- node-local storage suppression;
- legal or policy removal; and
- cryptographic erasure.

An append-only model does not by itself define which object is effective. Use the checked-in transition and effective-view contracts for Lingonberry behavior.

### Carrier and relay behavior

Check whether a proposed relay responsibility is:

- transport delivery;
- admission validation;
- durable storage;
- duplicate/conflict classification;
- query projection;
- replication; or
- application policy.

Do not infer multi-node convergence, consensus, global ordering, federation, or cross-carrier identity equivalence from the existence of a relay or adapter. Those capabilities require separate contracts.

### Indexer and API behavior

Verify that:

- indexes remain derived and rebuildable;
- query convenience does not become hidden protocol authority;
- pagination and ordering are deterministic where promised;
- effective-view reads preserve fail-closed behavior; and
- API fields do not invent semantic guarantees absent from the underlying evidence.

### Vocabulary and application profiles

Keep Toitoi-specific concepts, agricultural context, workflow states, and relation vocabularies in the Toitoi profile unless there is a demonstrated cross-domain requirement and an accepted Lingonberry proposal.

A profile may constrain or interpret core objects, but it must not silently change canonicalization, identity, signature verification, storage semantics, or transition authority.

## Possible outcomes

A review must end with one of these outcomes:

- **Adopt:** the idea is already compatible and is incorporated through a concrete Lingonberry contract and tests.
- **Adapt:** the idea is useful, but Lingonberry-specific changes, versioning, or migration are required.
- **Profile-only:** the idea belongs in the Toitoi application profile or another domain profile.
- **Background only:** the material explains historical motivation but creates no implementation requirement.
- **Reject:** the idea conflicts with current invariants, adds unjustified complexity, or lacks a safe migration path.
- **Defer:** the idea is plausible but lacks implementation, conformance, security, or operational evidence.

“Referenced” is not an adoption status.

## v1.0 boundary

For v1.0, this checklist does not promise:

- compatibility with Toitoi schemas or events;
- automatic import of Toitoi data;
- a stable Toitoi application profile;
- Nostr or any other external carrier adapter;
- multi-node synchronization or convergence;
- profile registries or signed profile catalogs;
- cross-project identity equivalence; or
- migration of external signed evidence into Lingonberry identity v1.

Those capabilities require separate accepted contracts and release qualification.

## Maintenance rules

- Keep this document free of copied field inventories that can drift from either repository.
- Pin external evidence to exact commits when making a decision.
- Link accepted behavior to Lingonberry schemas, fixtures, implementation, and tests.
- Mark historical influence as historical; do not present it as current authority.
- Re-run compatibility and security review when either project changes the relevant contract.
- Do not change the fixed v1.0 candidate through documentation-only maintenance.
