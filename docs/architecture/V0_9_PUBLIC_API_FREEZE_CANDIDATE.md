# v0.9 Public API Freeze Candidate

**Status: draft freeze candidate** | **Target release: v0.9.0** | **Last updated: 2026-07-22**

## 1. Purpose

This document classifies the externally observable Rust, protocol, command, HTTP, storage, and operational surfaces that must be reviewed before Lingonberry v1.0.0.

The objective is not to promise stability for every currently `pub` Rust item. The objective is to distinguish intentional public contracts from implementation details and to prevent accidental public surface from becoming a v1.x compatibility burden.

## 2. Stability classes

### Frozen candidate

A surface intended to become a v1.0 compatibility commitment. Breaking changes require release-blocker review during v0.9.0.

### Supported internal boundary

A cross-crate boundary used by the workspace and covered by tests, but not promised as a stable third-party Rust API. Refactoring is permitted when externally observable behavior remains compatible.

### Implementation detail

A surface that should be private, crate-private, test-only, or explicitly unstable before v1.0.

### Operator contract

A CLI, HTTP, filesystem, diagnostic, exit-code, configuration, or runbook behavior relied on by operators rather than Rust callers.

## 3. Freeze candidates

### 3.1 Protocol contract

The following are frozen candidates:

- canonical Knowledge Object envelope and required fields
- canonical serialization and deterministic object-key ordering
- Knowledge Object identifier syntax
- identity-key derivation rule version
- digest and signature payload definition
- publish-request envelope
- protocol and schema version axes
- supported object types
- relation, lineage, provenance, raw-reference, attachment, label, and metadata validation semantics
- duplicate and conflict classification
- validation failure codes exposed through public APIs
- valid, invalid, boundary, legacy, digest, and signature conformance fixtures

The Rust representation used to implement these behaviors is not automatically frozen merely because an item is declared `pub`.

### 3.2 Public read/write behavior

The following externally observable behavior is a frozen candidate:

- publish request acceptance and rejection semantics
- validation-before-storage ordering
- identity and signature verification ordering
- canonical storage result
- duplicate-safe success behavior
- conflict rejection without overwrite
- object retrieval by canonical identifier
- basic query and index behavior
- restart persistence
- storage-authoritative index rebuild and verification

### 3.3 Operator CLI

The v0.8 operator command surface is a frozen candidate:

- `serve`
- `config`
- `health`
- `ready`
- `status`
- `doctor`
- `verify`
- `metrics`
- `backup create`
- `backup verify`
- `restore plan`
- `restore apply`
- `index verify`
- `index rebuild`

For these commands, command names, required arguments, machine-readable output, diagnostic codes, severity, exit status, and fail-closed behavior are compatibility-relevant.

### 3.4 HTTP and administration contracts

The following are compatibility-relevant where already documented and tested:

- public publish and retrieval endpoints
- health and readiness semantics
- bounded-cardinality metrics
- quarantine administration behavior
- role and authorization ordering
- stable machine-readable error responses

Endpoint implementation structure and internal handler composition remain implementation details.

### 3.5 Storage and recovery contracts

The following are frozen candidates:

- storage format manifest and version semantics
- unknown-newer-format rejection
- migration inspect, plan, verified-backup, apply, verify, commit, resume, and rollback semantics
- canonical storage as the semantic source of truth
- index as derived and rebuildable state
- generation pointer publication rules
- journal, proof, inventory, archive, and evidence integrity rules
- backup verification requirements
- isolated restore target requirements
- replacement and cleanup proof binding
- contradictory-state and partial-state rejection

Internal Rust type names, helper modules, and file-local algorithms are not frozen unless explicitly referenced by a public specification.

## 4. Rust crate classification

### `lingonberry-protocol`

Intended stable behavior:

- parse rejection and canonical serialization semantics
- Knowledge Object validation and finalization behavior
- identity-key derivation
- publish-request canonical payload and signature verification behavior
- protocol, schema, capability, and identity-rule version constants

Audit concern:

- the crate currently exposes parser representation types and helpers directly
- public data fields may allow callers to depend on implementation representation
- error text should not be treated as stable where a versioned error code exists or should exist

Freeze action:

- retain behavioral compatibility
- document which functions are supported entry points
- classify raw parser representation and incidental helpers as unstable unless required by conformance consumers

### `lingonberry-identity`

Intended stable behavior:

- identity claims and identity-key consistency
- digest/signature input binding
- deterministic verification outcome

Freeze action:

- preserve verification semantics and version identifiers
- avoid freezing implementation-specific cryptographic process invocation or temporary-file layout

### `lingonberry-validation`

Intended stable behavior:

- validation level ordering
- deterministic error classification
- rejection of unknown, malformed, inconsistent, and unsupported objects

Freeze action:

- stabilize machine-readable codes and ordering requirements
- keep internal validator composition refactorable

### `lingonberry-core`

Intended stable behavior:

- append, duplicate, conflict, quarantine, promotion, replacement, cleanup, retrieval, and query semantics
- durable evidence and authorization boundaries

Freeze action:

- treat operation results and durable artifacts as compatibility-relevant
- treat transaction helper modules and staging internals as supported internal boundaries

### `lingonberry-indexer`

Intended stable behavior:

- deterministic indexing
- checkpoint and catch-up semantics
- verification and rebuild from canonical storage

Freeze action:

- freeze observable query and consistency behavior
- keep index representation and segment implementation internal unless part of storage format v1

### `lingonberry-relay`

Intended stable behavior:

- public HTTP and operator behavior
- authorization order
- health, readiness, diagnostics, and metrics contracts

Freeze action:

- endpoint and diagnostic contracts are compatibility-relevant
- runtime wiring, handler modules, and command dispatch internals remain implementation details

### `lingonberry-storage`

Intended stable behavior:

- storage format inspection
- migration and recovery classification
- backup and restore verification
- unknown/corrupt/contradictory-state rejection

Freeze action:

- freeze durable format and recovery outcomes
- keep parser/helper type layout internal unless encoded into an external file format

## 5. Compatibility rules

During v0.9.0:

1. A change to protocol acceptance, canonical bytes, identifier derivation, signature payload, durable format, public endpoint, command, diagnostic code, or exit status requires compatibility review.
2. A change that newly accepts previously invalid data requires security and conformance review.
3. A change that newly rejects previously valid data requires fixture impact analysis and explicit disposition.
4. Error prose is not stable unless a document explicitly declares it stable; machine-readable codes are the compatibility mechanism.
5. Internal Rust module paths and helper names are not stable by default.
6. Critical and high security fixes may change behavior, but must include migration or compatibility notes where relevant.
7. Unknown, corrupt, contradictory, and partial state remains fail closed.

## 6. Required audit outputs

Before v0.9.0 release candidate completion:

- [ ] enumerate exported items in each library crate
- [ ] identify exports used by external conformance clients
- [ ] identify accidental public fields, helpers, and re-exports
- [ ] document supported Rust entry points
- [ ] map each public entry point to tests or fixtures
- [ ] map each operator contract to documentation and acceptance coverage
- [ ] map each durable artifact to a version axis and compatibility policy
- [ ] record any intentional breaking correction as a release blocker decision

## 7. Non-goals

This freeze candidate does not promise:

- stability of every Rust `pub` item
- stable internal module paths
- stable debug formatting or unversioned error prose
- distributed or multi-node behavior
- Kubernetes, remote backup, vector search, or AI integration contracts
- compatibility for undocumented manual mutation of durable state

## 8. Exit criteria

This document becomes the v1.0 public API compatibility declaration only after:

- the exported-item audit is complete
- accidental public surface has been reduced or explicitly classified
- protocol, API, CLI, and storage candidates agree with their specifications and fixtures
- release-candidate tests exercise each frozen behavior
- no unresolved critical or high severity finding affects a frozen surface
