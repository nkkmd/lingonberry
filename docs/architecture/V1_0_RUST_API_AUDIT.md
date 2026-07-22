# v1.0 Rust Public API Audit

**Status: active audit** | **Target: v1.0.0** | **Parent issue: #112**

## 1. Purpose

This document records the evidence and review process used to classify Lingonberry's Rust exported surface for the v1.x compatibility commitment.

A Rust item being `pub` does not automatically make it a supported third-party API. The audit distinguishes:

- stable v1 entry points
- behavior-stable contracts
- operator contracts
- workspace-internal boundaries
- implementation details
- deprecated candidates

## 2. Mechanical inventory

The canonical mechanical inventory is generated with:

```bash
scripts/generate-rust-public-api.sh
```

The generator records the candidate commit, Rust compiler version, and `cargo-public-api` version alongside one exported-surface file for each workspace crate.

Generated output is evidence, not the normative compatibility policy. Every exported item is covered either by an explicit item classification or by a reviewed namespace/classification rule in this document.

## 3. Workspace coverage

The audit covers all library crates in the workspace:

- `lingonberry-protocol`
- `lingonberry-identity`
- `lingonberry-validation`
- `lingonberry-core`
- `lingonberry-indexer`
- `lingonberry-relay`
- `lingonberry-storage`

## 4. Initial discrepancy review

A source-level review confirms that the v0.9 inventory is not a complete mechanical export list.

Examples in `lingonberry-protocol` that require explicit disposition include:

- `ReadJsonFile`
- `read_json_file`
- `detect_shape`
- `MAX_JSON_INPUT_BYTES`
- `MAX_JSON_NESTING_DEPTH`

These items are publicly exported in source but were not all enumerated as v1 frozen candidates in `V0_9_RUST_API_INVENTORY.md`.

Initial disposition:

| Item | Proposed class | Rationale |
|---|---|---|
| `MAX_JSON_INPUT_BYTES` | behavior-stable resource boundary | Public rejection boundary; value changes require compatibility and security review. |
| `MAX_JSON_NESTING_DEPTH` | behavior-stable resource boundary | Public parser acceptance boundary; value changes require compatibility and security review. |
| `ReadJsonFile` | workspace-internal / unstable helper | File-loading representation is not a protocol contract. |
| `read_json_file` | workspace-internal / unstable helper | Convenience I/O helper used by workspace CLIs; public protocol consumers should use parser and validator entry points. |
| `detect_shape` | workspace-internal / unstable helper | Heuristic convenience function used by workspace code, not a normative wire discriminator. |

No item is removed or renamed during this audit merely because it appears accidental. Any source change requires consumer search, fixture review, and compatibility disposition.

## 5. Required classification record

For every exported item, the final audit records directly or through a namespace rule:

1. crate and fully qualified item
2. item kind and signature
3. current documentation references
4. known internal and external consumers
5. test or fixture coverage
6. stability class
7. compatibility semantics
8. deprecation status
9. earliest permitted removal version
10. notes for the normative v1 compatibility policy

Compiler-generated trait implementations and blanket implementations inherit the classification of the owning public type. They are not promoted to independent Lingonberry compatibility promises unless explicitly documented.

## 6. Public documentation review

The audit searches the root README, protocol specifications, operator documentation, examples, fixtures, and conformance clients for direct Rust API references.

A workspace-internal or unstable item must not be presented as a recommended third-party entry point.

Current consumer findings:

- `read_json_file` is used by workspace command implementations; no external conformance dependency has been identified.
- `detect_shape` is used by workspace protocol/relay command paths; no normative protocol specification depends on it as a discriminator.
- parser resource limits are referenced by security documentation and boundary tests and therefore are compatibility- and security-relevant behavior.

## 7. Error compatibility review

The audit treats versioned machine-readable error codes as compatibility mechanisms.

The following are not stable by default:

- prose wording
- debug formatting
- internal error type layout
- helper-module paths

Ordering becomes compatibility-relevant only where a documented API, fixture, or external conformance consumer depends on it. Such dependencies must be recorded explicitly rather than inferred from current implementation order.

Current disposition:

- protocol acceptance/rejection categories and versioned codes are behavior-stable
- vector order of free-form validation prose is not stable unless a fixture explicitly asserts it
- operator exit status and machine-readable diagnostic ordering are operator contracts where documented
- blanket `Display`, `Debug`, conversion, marker-trait, and auto-trait implementations are not independently frozen

## 8. Deprecation policy inputs

Before v1.0.0 publication:

- every deprecated candidate must be identified
- replacement guidance must be documented
- the earliest removal release must be named
- no stable v1 item may be removed during v1.x
- security corrections that alter behavior require explicit release-blocker review and compatibility notes

Current policy proposal:

- no existing exported item is removed for v1.0.0 solely to reduce surface area
- unstable/workspace-internal exports may be deprecated during v1.x but are not removed before v2.0 unless a security emergency requires explicit disposition
- stable v1 entry points and behavior-stable contracts are not removed or incompatibly changed during v1.x

## 9. Mechanical evidence record

The first successful complete inventory was generated by GitHub Actions workflow `Rust public API audit`, run `29926699269`, against the PR head commit recorded by the artifact as:

```text
ff3ca598c983276664ee34b871ac184eb6536e7e
```

Generation environment:

```text
cargo-public-api 0.52.0
rustc 1.99.0-nightly (0e29c21d9 2026-07-21)
host: x86_64-unknown-linux-gnu
LLVM 22.1.8
```

Artifact:

```text
name: rust-public-api-ff3ca598c983276664ee34b871ac184eb6536e7e
artifact id: 8532290377
sha256: baecd8243eecb55ed89d5f5b0c28561113eb721359ed454ddfe2eeb4b2956548
retention: through 2026-08-21
```

Inventory size, measured as rustdoc-public output lines:

| Crate | Lines |
|---|---:|
| protocol | 186 |
| identity | 7 |
| validation | 235 |
| core | 4,036 |
| indexer | 1,014 |
| relay | 210 |
| storage | 700 |

Large counts include public fields, enum variants, inherent methods, derived trait implementations, blanket implementations, and auto traits. The counts must not be interpreted as the number of independently supported Lingonberry APIs.

## 10. Crate-level classification rules

These rules classify every mechanically exported item unless an explicit item-level exception appears in this document or the normative v1 compatibility policy.

### `lingonberry-protocol`

Stable entry points and behavior-stable contracts:

- protocol/schema/archive/capability/identity-rule version constants
- parser input and nesting bounds as rejection boundaries
- `JsonValue`, `JsonError`, and `FinalizedKnowledgeObject` where required by supported entry points
- canonical parsing, normalization, serialization, validation, finalization, identity derivation, object-ID recognition, supported-type discovery, capability construction, and signature verification behavior

Workspace-internal or unstable helpers:

- file-loading convenience representation and helper
- heuristic shape detection
- temporary signature-workspace implementation details
- error prose beyond documented machine-readable semantics

### `lingonberry-identity`

Behavior-stable:

- identity/canonicalization rule version constants
- versioned identity-key derivation
- identity-key basis semantics
- identity-claim version validation

Implementation-specific cryptographic invocation and test helpers remain internal.

### `lingonberry-validation`

Stable or behavior-stable:

- acceptance decision categories
- acceptance policy configuration contract
- validation report semantics
- full validation/finalization entry points
- reject/defer/accept distinctions and versioned codes

Validator decomposition and unversioned prose ordering remain internal.

### `lingonberry-core`

Behavior-stable or stable where explicitly documented:

- `StorageBackend` behavior
- append, duplicate, conflict, retrieval, list, subscribe, replay, archive, quarantine, promotion, replacement, cleanup, and query outcomes
- durable evidence and authorization boundaries
- public result categories and machine-readable error codes

Workspace-internal:

- public modules exposing transaction composition, staging, publication, ledger, compaction, lock, preview-builder, or other implementation decomposition unless a specification explicitly names the path
- concrete backend internals and helper structures not encoded into a public durable format
- compiler-generated implementations inherited from an owning type

### `lingonberry-indexer`

Behavior-stable:

- checkpoint, catch-up, lifecycle, verification, rebuild, and canonical-storage reconstruction outcomes

Workspace-internal:

- batching, cursor representation, derived table/segment implementation, and helper-module paths

### `lingonberry-relay`

Behavior-stable/operator-contract:

- publish, retrieve, query, transition, effective-view, authentication, health, readiness, diagnostics, metrics, and machine-readable HTTP response semantics where documented

Workspace-internal:

- HTTP adapter structs and helper functions not documented as third-party Rust entry points
- runtime wiring and handler decomposition
- compiler-generated implementations inherited from response/report types

### `lingonberry-storage`

Behavior-stable/operator-contract:

- storage-format inspection
- migration plan/apply/resume/verify/commit/rollback outcomes
- backup, restore, drill, doctor, and index command behavior
- unknown-newer, corrupt, contradictory, partial, symlink, active-target, and unverified-backup rejection rules
- journal, archive, inventory, proof, and evidence semantics encoded into durable formats

Workspace-internal:

- `recovery` handler functions as Rust APIs; their operator-visible command behavior remains stable
- intermediate workspace layout not documented as a durable format
- helper types and algorithms not serialized into a public artifact

## 11. Coverage mapping requirements

The normative compatibility policy must map stable categories to the following evidence families:

| Contract family | Required evidence |
|---|---|
| Protocol and canonicalization | protocol unit tests, parser boundary tests, canonical fixtures, external conformance |
| Identity and signatures | identity tests, signature fixtures, workspace security regression |
| Validation | validation unit/integration tests and acceptance-policy fixtures |
| Core storage behavior | lifecycle, duplicate/conflict, quarantine, archive, replacement/cleanup tests |
| Indexer | catch-up, checkpoint, verify, rebuild, restart consistency tests |
| Relay/operator behavior | HTTP contract tests and reference-platform operator acceptance |
| Storage/recovery | migration, backup, isolated restore, doctor, journal/proof, and rollback tests |

## 12. Completion checklist

- [x] Generate the exported-surface inventory for all crates
- [x] Record toolchain provenance
- [x] Compare the generated output with the v0.9 inventory and identify omissions
- [x] Establish crate-level rules covering all mechanical exports
- [x] Search initial public/documentation consumers for identified protocol helpers
- [x] Establish error and compiler-generated implementation classification rules
- [x] Establish deprecation/removal policy proposal
- [ ] Complete targeted search for every explicitly stable entry point
- [ ] Complete fixture and conformance search for error-order dependencies
- [ ] Attach exact test/fixture references to each stable contract family
- [ ] Resolve any item-level exception to the crate-level rules
- [ ] Incorporate approved results into the normative v1 compatibility policy

## 13. Release-blocker rules

The v1.0 release is blocked when:

- an exported item is not covered by an explicit classification or reviewed namespace rule
- public documentation recommends an item classified as internal or unstable
- a frozen behavior lacks test, fixture, or acceptance coverage
- error ordering is relied upon but undocumented
- a deprecated item lacks replacement and removal policy
- generated inventory and normative classification contradict each other
- a compatibility-relevant item changes after the recorded inventory without a regenerated artifact and review
