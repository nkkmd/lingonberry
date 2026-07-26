# v0.9.0 Public API Freeze Candidate Record

**Status: historical freeze record**  
**Original target: v0.9.0**  
**Current role: evidence for the v1.0 compatibility audit**

## 1. Purpose

This document records the public-surface classification used during the v0.9.0 freeze review. It is retained as historical audit evidence.

It is not the current v1.0 compatibility declaration, does not redefine the v1.0 release candidate, and does not make every Rust item marked `pub` a supported third-party API.

For current interpretation, use the checked-in protocol, schema, runtime, conformance, operator, storage, and compatibility documents. Later audit documents take precedence where this record and a current contract differ.

## 2. Historical decision model

The v0.9.0 review used four classes.

### Frozen candidate

An externally observable surface considered for a v1.0 compatibility commitment.

### Behavior-frozen or supported internal boundary

A workspace boundary whose externally observable behavior was intended to remain compatible even when Rust types, modules, or helper functions changed.

### Implementation detail

A representation, helper, module path, parser decomposition, temporary-file layout, or internal algorithm that remained refactorable.

### Operator contract

A CLI, HTTP, filesystem, diagnostic, exit-code, configuration, backup, restore, or runbook behavior relied on by operators rather than Rust callers.

These classes were review inputs. A candidate became an actual compatibility commitment only when supported by a checked-in specification, implementation, fixture, test, or later audit disposition.

## 3. Surfaces considered for freezing

### 3.1 Protocol behavior

The review treated the following as compatibility-sensitive:

- the canonical Knowledge Object envelope and required fields;
- deterministic canonical serialization;
- Knowledge Object identifier syntax;
- identity-rule versioning and identity-key derivation;
- digest and signature payload definitions;
- publish-request validation;
- protocol, schema, capability, archive, and identity version axes;
- supported object types;
- relation, lineage, provenance, raw-reference, attachment, label, and metadata validation;
- duplicate and conflict classification;
- machine-readable validation failures; and
- conformance fixtures for valid, invalid, boundary, legacy, digest, and signature cases.

The Rust representation implementing these behaviors was not automatically frozen.

### 3.2 Public read and write behavior

The review considered these observable behaviors compatibility-sensitive:

- validation before canonical storage;
- identity and signature verification ordering;
- accepted, duplicate, deferred, quarantined, and rejected outcomes where implemented;
- duplicate-safe success without mutation;
- conflict rejection without overwrite;
- retrieval by canonical identifier;
- documented query and index behavior;
- restart persistence; and
- index verification and rebuild from canonical storage.

### 3.3 Operator CLI

The review covered the documented operator command surface, including commands for serving, configuration inspection, health, readiness, status, diagnostics, verification, metrics, backup, restore, and index maintenance.

Compatibility-sensitive parts included:

- command and subcommand names;
- required arguments;
- machine-readable output;
- diagnostic and error codes;
- severity;
- exit status; and
- fail-closed behavior.

A command named in this historical record is not current merely because it appears here. Current operator documentation and executable help are authoritative.

### 3.4 HTTP and administration behavior

Where documented and tested, the review treated these as compatibility-sensitive:

- publish and retrieval endpoints;
- health and readiness semantics;
- bounded-cardinality metrics;
- quarantine administration;
- authentication and authorization ordering; and
- stable machine-readable error responses.

Handler composition and runtime wiring remained implementation details.

### 3.5 Storage and recovery behavior

The review treated these as compatibility-sensitive:

- storage-format manifest and version semantics;
- unknown-newer-format rejection;
- migration inspection, planning, backup, apply, verification, commit, resume, and rollback semantics;
- canonical storage as the semantic source of truth;
- derived-state rebuildability;
- generation publication rules;
- journal, proof, inventory, archive, and evidence integrity;
- backup verification;
- isolated restore targets;
- replacement and cleanup proof binding; and
- rejection of corrupt, contradictory, unknown, or partial state.

Internal type names and file-local algorithms remained refactorable unless encoded into a versioned external format.

## 4. Rust API interpretation

The v0.9.0 review covered these workspace crates:

- `lingonberry-protocol`;
- `lingonberry-identity`;
- `lingonberry-validation`;
- `lingonberry-core`;
- `lingonberry-indexer`;
- `lingonberry-relay`; and
- `lingonberry-storage`.

The intended freeze boundary was behavioral first:

- canonical parsing, validation, finalization, identity, and signature outcomes;
- append, duplicate, conflict, quarantine, retrieval, and query semantics;
- deterministic indexing and rebuild behavior;
- public HTTP and operator behavior;
- durable storage, migration, backup, restore, and recovery outcomes.

The following were not stable by default:

- internal module paths;
- helper names;
- parser representation types;
- transaction and staging helpers;
- handler decomposition;
- debug formatting;
- temporary-file or subprocess layout; and
- unversioned error prose.

The detailed exported-item inventory is maintained separately in [`V0_9_RUST_API_INVENTORY.md`](./V0_9_RUST_API_INVENTORY.md). The later v1.0 audit and compatibility policy determine the current disposition.

## 5. Compatibility review rules retained from v0.9.0

The following rules remain useful review principles, but current normative documents control their application:

1. Changes to canonical bytes, identifier derivation, signature payloads, accepted input, durable formats, endpoints, commands, diagnostic codes, or exit statuses require compatibility review.
2. Newly accepted data requires security and conformance review.
3. Newly rejected previously valid data requires fixture-impact analysis and an explicit migration or compatibility disposition.
4. Machine-readable codes, not incidental prose, are the compatibility mechanism unless a document explicitly freezes prose.
5. Rust visibility alone does not create a supported external API.
6. Security corrections may change behavior but require compatibility and migration notes where relevant.
7. Unknown, corrupt, contradictory, and partial state remains fail closed unless a more specific contract defines a safe recovery path.

## 6. Historical audit outputs

The v0.9.0 freeze process required:

- an exported-item inventory for each library crate;
- identification of exports used by conformance consumers;
- identification of accidental public fields, helpers, and re-exports;
- classification of supported Rust entry points;
- mapping of supported behavior to tests or fixtures;
- mapping of operator contracts to documentation and acceptance coverage;
- mapping of durable artifacts to version axes; and
- explicit disposition of intentional breaking corrections.

Completion status must be read from the later audit records, not inferred from this historical checklist.

## 7. Non-goals

This record does not promise:

- stability of every Rust `pub` item;
- stable internal module paths;
- stable debug output or unversioned error prose;
- multi-node replication, consensus, or distributed ordering;
- Kubernetes, remote-backup, vector-search, or AI-integration contracts;
- compatibility for undocumented manual mutation of durable state; or
- publication of v1.0.0.

## 8. Current release boundary

The fixed v1.0 release candidate remains:

`f9543019f2c219aea3b085ff90f2da201b268a48`

This documentation normalization does not redefine that candidate.

The following release gates remain separate and incomplete until evidenced elsewhere:

- the formal 72-hour soak;
- privileged reference-host qualification;
- version preparation;
- the release pull request;
- the v1.0.0 tag; and
- the GitHub Release.

## 9. Precedence

When this historical record conflicts with a current artifact, use the more specific current source in this order:

1. versioned schemas and conformance fixtures;
2. checked-in protocol and storage contracts;
3. runtime behavior covered by tests;
4. the v1.0 Rust API audit and compatibility policy;
5. operator documentation; and
6. this historical v0.9.0 record.
