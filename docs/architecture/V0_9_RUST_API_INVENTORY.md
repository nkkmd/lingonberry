# v0.9.0 Rust API Inventory Record

**Status: historical audit input** | **Release context: v0.9.0** | **Reviewed for v1 documentation: 2026-07-26**

## 1. Purpose and authority

This document preserves the manual Rust API classification used during the v0.9.0 freeze review. It records which exported surfaces were considered possible compatibility commitments before the mechanical v1.0 public-API audit was completed.

This document is not the current v1.0 compatibility declaration and does not independently freeze a Rust item. Current authority is, in descending order:

1. checked-in schemas and versioned protocol or storage contracts;
2. conformance fixtures and externally exercised behavior;
3. runtime and acceptance tests;
4. [`V1_0_RUST_API_AUDIT.md`](./V1_0_RUST_API_AUDIT.md);
5. [`V1_COMPATIBILITY_POLICY.md`](./V1_COMPATIBILITY_POLICY.md);
6. this historical record.

Where this record differs from current code or later audit evidence, the later and more concrete source controls.

The fixed v1.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. This documentation update does not redefine that candidate.

## 2. Historical classification model

The v0.9.0 review used four classes:

- **Frozen candidate**: an exported item or named surface considered for direct v1 compatibility support.
- **Behavior-frozen**: externally observable behavior considered stable even when the implementing Rust representation remained changeable.
- **Workspace-internal**: a `pub` surface used across crates but not promised to third-party Rust consumers.
- **Implementation detail**: a refactorable helper, representation, module path, or runtime mechanism.

These were review labels, not attributes inferred from Rust visibility. A `pub` item was not automatically stable, and a private implementation could still implement a stable wire, storage, operator, or security contract.

## 3. Historical crate review

### 3.1 `lingonberry-protocol`

Candidate or behavior-stable areas included:

- protocol, schema, archive, capability, identity-rule, and carrier-kind version constants;
- canonical parse, normalization, serialization, validation, finalization, identifier, and signature behavior;
- canonical object-key ordering and duplicate-key rejection;
- malformed-input, resource-bound, identity, digest, and signature outcomes;
- supported knowledge-type and capability-manifest behavior.

Representation types such as parser values and errors were reviewed cautiously. Recursive-descent parsing, helper decomposition, temporary signature workspace layout, and temporary filenames were treated as implementation details unless encoded into a published contract.

### 3.2 `lingonberry-identity`

Candidate or behavior-stable areas included:

- identity-claim validation;
- consistency with canonical identity-key derivation;
- issuer, verification-evidence, digest, and signature binding semantics.

Fixture builders, test signing helpers, command assembly, and temporary-workspace mechanics were workspace-internal or implementation details.

### 3.3 `lingonberry-validation`

Candidate or behavior-stable areas included:

- validation-report semantics;
- reject, defer, and accept categories;
- explicit acceptance-policy selection;
- full validation and finalization behavior;
- the rule that unvalidated or incomplete evidence does not enter canonical storage;
- rejection rather than silent normalization of unknown or schema-inconsistent input.

Individual rule-module layout and report assembly were not stable unless a published machine-readable ordering contract required them.

### 3.4 `lingonberry-core`

Candidate or behavior-stable areas included:

- append, duplicate, conflict, retrieval, listing, subscription, and replay outcomes;
- quarantine, promotion, replacement, cleanup, and archive outcomes;
- durable error categories and evidence boundaries;
- canonical storage as semantic authority;
- conflict rejection without overwrite;
- the prohibition on using quarantine promotion to bypass validation;
- preservation of immutable evidence during archive and lifecycle operations.

Concrete backend layout, runtime-path helpers, transaction helpers, and module decomposition were workspace-internal unless exposed through a documented durable or operator contract.

### 3.5 `lingonberry-indexer`

Candidate or behavior-stable areas included:

- checkpoint and catch-up outcomes;
- deterministic verification and rebuild;
- reconstruction from canonical storage;
- preservation of the last-known-good checkpoint on incomplete work;
- deterministic restart recovery;
- the rule that the index is derived state rather than semantic authority.

Batch sizing, cursor representation, and derived-table implementation were internal.

### 3.6 `lingonberry-storage`

Candidate or behavior-stable areas included:

- storage-format manifest and version semantics;
- migration inspect, plan, apply, verify, resume, commit, rollback, and recovery outcomes;
- backup binding and verification evidence;
- rejection of unknown-newer, corrupt, contradictory, or partial state;
- no implicit migration during ordinary startup;
- verified-backup requirements for non-empty migration;
- publication only after durable verification;
- isolated restore-target requirements.

Unpublished migration workspace files, helper decomposition, and checksum implementation were internal except where a digest algorithm or file format was explicitly versioned.

### 3.7 `lingonberry-relay`

Candidate or behavior-stable areas included:

- publish, retrieve, query, transition, health, readiness, status, doctor, verify, and metrics behavior;
- authorization ordering;
- stable machine-readable diagnostic codes and operator exit status;
- bounded handling of untrusted request material;
- read-only diagnostics that do not repair state;
- effective views that do not rewrite original evidence.

Binary decomposition, handler function names, runtime wiring, and metrics implementation were internal unless a metric name or bounded-cardinality label was published as an operator contract.

## 4. Review rules retained as historical rationale

The v0.9.0 review applied the following principles:

1. Removing, renaming, or changing a candidate surface required compatibility review.
2. Changing behavior-frozen semantics required fixtures or regression evidence.
3. Workspace-internal items could be refactored and were not to be recommended as third-party APIs.
4. Security hardening that newly rejected previously accepted input required compatibility and migration analysis.
5. Wire, storage, identity, signature, authorization, diagnostic-code, and exit-status changes could not be dismissed as internal refactors.
6. Free-form prose, debug formatting, helper paths, and runtime wiring were not stable unless explicitly declared otherwise.

## 5. Disposition of the original open work

The original record listed mechanical export enumeration, external-consumer analysis, error-ordering review, deprecation classification, and v1 approval as unfinished work. Those items were superseded by the completed v1 audit and its generated evidence.

The later audit concluded that:

- all seven Rust library crates were mechanically inventoried;
- each export was classified directly or by a reviewed namespace rule;
- stable support is primarily behavioral and contract-based rather than visibility-based;
- no current external dependency on free-form validation error ordering was identified;
- compatibility-relevant source changes require regenerated inventory and review;
- no item was removed solely to reduce the v1 surface.

This record therefore has no remaining checklist and creates no independent release gate.

## 6. Release boundary

This historical normalization does not complete or replace v1.0 qualification. The following remain incomplete:

- the formal 72-hour soak;
- privileged reference-host qualification;
- version preparation;
- the release pull request;
- the `v1.0.0` tag;
- the GitHub Release.

Use the v1 audit, compatibility policy, current schemas, tests, conformance fixtures, operator contracts, and release qualification evidence for present decisions.