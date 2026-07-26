# Distributed Knowledge Commons Architecture

**Status:** architecture direction for the v1.0.0 pre-release documentation set  
**Implementation boundary:** the checked-in v1.0 candidate is a production-oriented single-node implementation, not a completed distributed commons.

## 1. Purpose

Lingonberry is intended to provide durable, replayable infrastructure for publishing, validating, storing, retrieving, and deriving views over knowledge objects. The long-term direction is a distributed knowledge commons that can support applications such as Toitoi without embedding one application's vocabulary or workflow into the protocol core.

This document separates:

- behavior implemented in the checked-in repository;
- architectural constraints that guide compatible evolution;
- future distributed capabilities that are not v1.0 runtime guarantees.

Protocol contracts, schemas, command behavior, and operational procedures are defined by their dedicated documents and checked-in implementation. This architecture document does not override them.

## 2. Current v1.0 implementation boundary

The pre-release v1.0 implementation provides a single-node Rust workspace with:

- bounded parsing and validation of protocol objects;
- canonical storage using File and SQLite backends;
- duplicate and conflict handling;
- persistent quarantine and replacement workflows;
- append-only Transition Object evidence;
- derived effective views and diagnostics;
- index checkpoints, verification, catch-up, and rebuild;
- backup, isolated restore, storage migration, and operator tooling;
- HTTP and CLI publication and retrieval surfaces.

The current implementation does **not** claim:

- multi-primary replication;
- consensus among independent nodes;
- automatic federation or peer discovery;
- cross-node canonical-identity resolution;
- distributed locking;
- a completed relay subscription network;
- protocol translation from Nostr, AT Protocol, or another external protocol;
- semantic truth adjudication;
- global ordering across independent nodes.

## 3. Architectural invariants

### 3.1 Canonical storage is authoritative

Accepted protocol objects are written to canonical storage. Indexes, effective views, diagnostics, search projections, and caches are derived state.

Derived-state failure must not retroactively turn a durable canonical-storage success into an ingestion failure. Derived state must remain rebuildable from authoritative records and durable evidence.

### 3.2 Append-only evidence

Published Knowledge Objects and Transition Objects are immutable evidence. Replacement, withdrawal, correction, and supersession are represented by additional objects and derived evaluation, not by rewriting the original object.

### 3.3 Fail closed on ambiguity

Unknown versions, unsupported rules, contradictory authority evidence, invalid transition graphs, and multiple authorized heads must not be resolved by timestamps, identifier order, or implementation accident.

When a current result cannot be established safely, the system reports ambiguity, invalidity, unsupported state, or stale last-known-good state as defined by the relevant contract.

### 3.4 Deterministic replay

Parsing, canonicalization, identity-rule evaluation, transition evaluation, and digest generation must be versioned and deterministic for the inputs governed by each rule.

Replayability does not imply that all external data, clocks, policies, or trust decisions are globally identical. Authority inputs and policy versions must be explicit where they affect results.

### 3.5 Provenance is not truth

Provenance records source and processing history. Signatures establish control of a key for a defined byte sequence. Neither proves that a claim is true, complete, current, or endorsed by an operator.

### 3.6 Protocol and carrier remain distinct concerns

The protocol defines object semantics, canonical bytes, identifiers, signatures, transitions, and compatibility rules. A carrier defines how those protocol objects are framed, transmitted, stored, retried, or discovered.

A carrier may add transport metadata, but it must not silently change protocol semantics. Mapping an external protocol into Lingonberry requires an explicit, versioned adapter and provenance record; it is not treated as lossless identity by default.

## 4. Logical layers

```text
Authoring applications
        |
        v
Carrier and ingress surfaces
        |
        v
Parsing, validation, identity, and acceptance
        |
        v
Canonical storage and durable evidence
        |
        +--------------------+
        |                    |
        v                    v
Indexes and search      Effective views
        |                    |
        +----------+---------+
                   v
             Read APIs / UI
```

### 4.1 Authoring applications

Editors, local agents, batch importers, and application profiles construct protocol objects. Application-specific vocabulary and user experience belong here unless promoted through a separately versioned protocol change.

Toitoi is a possible application profile. Inquiry workflows, agricultural context, and Toitoi-specific UI behavior are not core Lingonberry semantics.

### 4.2 Carrier and ingress surfaces

HTTP, CLI, files, archives, and future synchronization mechanisms deliver protocol objects to an implementation. Carrier authentication, framing, retry behavior, and transport-level identifiers are separate from canonical object identity.

### 4.3 Parsing, validation, identity, and acceptance

This layer performs bounded parsing, schema and runtime validation, signature checks where required, identity-rule evaluation, duplicate/conflict classification, and acceptance-policy decisions.

Passing JSON parsing or schema validation alone does not imply authorization, durable acceptance, signature validity, or effective-view applicability.

### 4.4 Canonical storage and durable evidence

Canonical storage preserves accepted Knowledge Objects, Transition Objects, and the durable records required by checked-in workflows. Quarantine, replacement journals, backup manifests, migration state, and proof ledgers have their own operational contracts.

### 4.5 Derived indexes and effective views

Indexes support lookup and search. Effective views evaluate authorized transition evidence without mutating canonical objects. These projections must expose freshness and verification state and must not present stale or ambiguous output as current.

### 4.6 Read APIs and user interfaces

Read surfaces expose canonical objects, effective results, capabilities, and bounded diagnostics. Public diagnostics must avoid leaking storage paths, internal row identifiers, stack traces, secrets, or unstable implementation errors.

## 5. Identity boundaries

Lingonberry distinguishes several identifiers:

- a protocol object identifier defined by the applicable schema and identifier contract;
- a versioned identity key derived by a specific identity rule;
- signature-key identifiers and authority identities;
- carrier-local identifiers such as a file name, request identifier, or remote record URI;
- storage-internal keys that are not public protocol identity.

These values are not interchangeable. Equality under one identity rule does not automatically establish equality under another rule or across protocol versions.

The v1.0 product release version also does not change protocol, schema, identity-rule, signature-rule, transition-rule, or digest-rule versions automatically.

## 6. Distribution model

A future distributed commons may replicate immutable protocol objects and durable transition evidence among independently operated nodes. Compatible distribution must preserve these constraints:

1. replicated bytes and their rule versions remain inspectable;
2. duplicate and conflict results are deterministic for the same governed inputs;
3. node-local policy and trust inputs are explicit;
4. derived views remain local projections unless a stronger agreement protocol is specified;
5. no node silently rewrites another node's canonical evidence;
6. synchronization failure does not create a false claim of global completeness.

Different nodes may legitimately hold different subsets of evidence or use different local acceptance policies. Therefore, identical canonical evidence can be portable while effective-view completeness and policy conclusions remain node-relative.

## 7. Future carrier and federation work

Potential future work includes:

- archive exchange and offline synchronization;
- peer-to-peer or relay-based object replication;
- capability negotiation for supported schema and rule versions;
- explicit adapters for external protocols;
- signed synchronization manifests and completeness proofs;
- distributed discovery and collection-scoped replication.

Each capability requires a separate contract covering framing, authentication, replay, duplicate/conflict behavior, partial failure, version negotiation, privacy, and evidence retention. Mention here is not a commitment that the capability exists in v1.0.

## 8. Security and privacy boundaries

A distributed knowledge commons increases exposure to untrusted input, replay, resource exhaustion, malicious graphs, signature abuse, privacy leakage, and inconsistent policy.

Implementations must preserve bounded parsing, explicit limits, fail-closed version handling, safe path handling, secret separation, and verifiable durable transitions. Local context or personal data must not be published merely because a protocol field can carry it.

Encryption, access control, deletion policy, legal compliance, and secure erase require separate policy and implementation. Append-only protocol evidence must not be described as guaranteeing erasure.

## 9. Relationship to repository contracts

Read this architecture together with:

- [`docs/protocols/PROTOCOL_CONTRACT.md`](../protocols/PROTOCOL_CONTRACT.md)
- [`docs/protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md`](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [`docs/protocols/CANONICALIZATION.md`](../protocols/CANONICALIZATION.md)
- [`docs/protocols/IDENTITY_AND_PROVENANCE.md`](../protocols/IDENTITY_AND_PROVENANCE.md)
- [`docs/protocols/VERSIONING_AND_COMPATIBILITY.md`](../protocols/VERSIONING_AND_COMPATIBILITY.md)
- [`docs/architecture/DUPLICATE_AND_CONFLICT_CONTRACT.md`](./DUPLICATE_AND_CONFLICT_CONTRACT.md)
- [`docs/operations/RELAY_STORAGE_SEPARATION.md`](../operations/RELAY_STORAGE_SEPARATION.md)
- [`docs/operations/STORAGE_NODE_RUNTIME.md`](../operations/STORAGE_NODE_RUNTIME.md)

Where this directional document conflicts with a checked-in schema, normative protocol contract, runtime behavior, or conformance fixture, the specific checked-in contract governs the current release.

## 10. Release boundary

The fixed v1.0.0 pre-version candidate is `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation and tooling commits after that candidate do not redefine it.

Formal 72-hour soak, privileged reference-host qualification, product version preparation, release PR, tag, GitHub Release, and final publication evidence remain separate and incomplete release gates.