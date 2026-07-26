# Protocol Documentation Index

**Status:** normative navigation index for the v1.0.0 pre-release documentation set  
**Protocol version:** `0.1.0`

## 1. Purpose

This directory documents Lingonberry protocol objects, wire representation, canonicalization, identity, signatures, transitions, effective views, diagnostics, and compatibility.

The documents describe the checked-in implementation and explicitly identified pre-release requirements. A design goal or broader model is not an implemented runtime guarantee unless the relevant document ties it to checked-in code, schemas, tests, or conformance fixtures.

Product release version, protocol version, schema versions, and independently versioned rules are separate namespaces. Publishing Lingonberry `v1.0.0` does not by itself change protocol version `0.1.0`, Knowledge Object schema version `0.1.0`, or any identity, signature, transition, digest, or diagnostic rule version.

## 2. Core protocol contracts

Start with these documents:

- [Protocol Contract](./PROTOCOL_CONTRACT.md) — top-level checked-in protocol and implementation boundaries.
- [Protocol-Native Wire Format](./PROTOCOL_NATIVE_WIRE_FORMAT.md) — Knowledge Object and HTTP publish-request JSON representation.
- [Protocol Identifiers](./PROTOCOL_IDENTIFIERS.md) — identifier syntax, limits, comparison, and enforcement differences.
- [Canonicalization](./CANONICALIZATION.md) — deterministic JSON serialization and canonical byte rules.
- [Identity and Provenance](./IDENTITY_AND_PROVENANCE.md) — identity-key derivation, identity claims, provenance, and trust boundaries.
- [HTTP Publish Signature](./HTTP_PUBLISH_SIGNATURE.md) — versioned publish-request signature input and verification contract.
- [Versioning and Compatibility](./VERSIONING_AND_COMPATIBILITY.md) — protocol, schema, rule, carrier, and compatibility boundaries.
- [Timestamp Semantics](./TIMESTAMP_SEMANTICS.md) — timestamp validation, preservation, comparison, and authority rules.

## 3. Transition contracts

Transition Objects represent append-only replacement, withdrawal, and related effective-view evidence. They do not mutate canonical Knowledge Objects.

- [Transition Object](./TRANSITION_OBJECT.md) — object schema and semantic fields.
- [HTTP Transition API](./HTTP_TRANSITION_API.md) — checked-in transition ingestion endpoint and persistence boundaries.
- [Transition Authority](./TRANSITION_AUTHORITY.md) — authority model and currently implemented enforcement subset.
- [Transition Evidence Generation](./TRANSITION_EVIDENCE_GENERATION.md) — evidence production and verification inputs.
- [Transition Supersession](./TRANSITION_SUPERSESSION.md) — parent relationships, graph evaluation, and fail-closed handling.
- [Transition Reevaluation Queue](./TRANSITION_REEVALUATION_QUEUE.md) — durable reevaluation intent and queue processing.
- [Transition Reevaluation Coalescing](./TRANSITION_REEVALUATION_COALESCING.md) — duplicate-work reduction and ordering boundaries.
- [Orphan Transitions](./ORPHAN_TRANSITIONS.md) — missing-target handling and later reevaluation.

## 4. Effective-view read and diagnostic contracts

Derived effective-view state is separate from canonical protocol-object storage.

- [Effective View Read API](./EFFECTIVE_VIEW_READ_API.md)
- [Last-Known-Good Effective View](./LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md)
- [Effective View Diagnostics](./EFFECTIVE_VIEW_DIAGNOSTICS.md)
- [Diagnostic Pagination](./EFFECTIVE_VIEW_DIAGNOSTIC_PAGINATION.md)
- [Diagnostic Cursor Lease](./EFFECTIVE_VIEW_DIAGNOSTIC_CURSOR_LEASE.md)
- [Diagnostic Read Guard](./EFFECTIVE_VIEW_DIAGNOSTIC_READ_GUARD.md)
- [Diagnostic Read-Guard Heartbeat](./EFFECTIVE_VIEW_DIAGNOSTIC_READ_GUARD_HEARTBEAT.md)
- [Diagnostic Retention](./EFFECTIVE_VIEW_DIAGNOSTIC_RETENTION.md)

## 5. Digest and index contracts

- [Index Generation Digest](./INDEX_GENERATION_DIGEST.md) — versioned digest inputs used to identify generated index state.

A digest identifies the bytes or semantic basis defined by its rule. It does not by itself prove storage durability, authority, freshness, completeness, or absence of conflicting evidence.

## 6. Schemas, fixtures, and implementation sources

The normative documentation must be read together with the checked-in artifacts it references:

- [`schemas/knowledge-object.schema.json`](../../schemas/knowledge-object.schema.json)
- [`schemas/http-publish-request.schema.json`](../../schemas/http-publish-request.schema.json)
- [`schemas/transition-object.schema.json`](../../schemas/transition-object.schema.json)
- [`fixtures/`](../../fixtures/)
- [`conformance/`](../../conformance/)
- [`packages/protocol/`](../../packages/protocol/)
- [`packages/relay/`](../../packages/relay/)
- [`packages/core/`](../../packages/core/)
- [`packages/indexer/`](../../packages/indexer/)
- [`packages/storage/`](../../packages/storage/)

A JSON Schema establishes only the constraints expressed by that schema. Runtime validators may be stronger or weaker in explicitly documented areas. Passing parsing or schema validation does not imply signature verification, authority, duplicate/conflict acceptance, durable storage, index publication, or effective-view authorization.

## 7. Conformance and operational boundaries

Protocol conformance evidence is rooted in [`conformance/`](../../conformance/). Operational installation, backup, restore, quarantine, upgrade, rollback, and reference-host procedures are documented under [`docs/operations/`](../operations/); they are not protocol semantics.

The fixed v1.0.0 release candidate is `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation and tooling commits after that candidate do not redefine it. Formal 72-hour soak, privileged reference-host qualification, product version update, release tag, and GitHub Release are separate release gates and are not completed by normal documentation checks or walkthrough rehearsals.
