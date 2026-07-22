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

The generator records both the Rust compiler version and `cargo-public-api` version alongside one exported-surface file for each workspace crate.

Generated output is evidence, not the normative compatibility policy. Every exported item still requires explicit classification.

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

A source-level review already confirms that the v0.9 inventory is not a complete mechanical export list.

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
| `ReadJsonFile` | unstable helper candidate | File-loading representation is not a protocol contract. |
| `read_json_file` | unstable helper candidate | Convenience I/O helper; public protocol consumers should use parser and validator entry points. |
| `detect_shape` | unstable helper candidate | Heuristic convenience function, not a normative wire discriminator. |

No item is removed or renamed during this audit merely because it appears accidental. Any source change requires consumer search, fixture review, and compatibility disposition.

## 5. Required classification record

For every exported item, the final audit must record:

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

## 6. Public documentation review

The audit must search the root README, protocol specifications, operator documentation, examples, fixtures, and conformance clients for direct Rust API references.

A workspace-internal or unstable item must not be presented as a recommended third-party entry point.

## 7. Error compatibility review

The audit treats versioned machine-readable error codes as compatibility mechanisms.

The following are not stable by default:

- prose wording
- debug formatting
- internal error type layout
- helper-module paths

Ordering becomes compatibility-relevant only where a documented API, fixture, or external conformance consumer depends on it. Such dependencies must be recorded explicitly rather than inferred from current implementation order.

## 8. Deprecation policy inputs

Before v1.0.0 publication:

- every deprecated candidate must be identified
- replacement guidance must be documented
- the earliest removal release must be named
- no stable v1 item may be removed during v1.x
- security corrections that alter behavior require explicit release-blocker review and compatibility notes

## 9. Completion checklist

- [ ] Generate the exported-surface inventory for all crates
- [ ] Record toolchain provenance
- [ ] Classify every exported item
- [ ] Compare the generated output with the v0.9 inventory
- [ ] Search public documentation for Rust API references
- [ ] Search fixtures and conformance consumers for error-order dependencies
- [ ] Identify deprecated candidates
- [ ] Map stable entry points to tests or fixtures
- [ ] Resolve every unclassified or contradictory item
- [ ] Incorporate approved results into the normative v1 compatibility policy

## 10. Release-blocker rules

The v1.0 release is blocked when:

- an exported item remains unclassified
- public documentation recommends an item classified as internal or unstable
- a frozen behavior lacks test, fixture, or acceptance coverage
- error ordering is relied upon but undocumented
- a deprecated item lacks replacement and removal policy
- generated inventory and normative classification contradict each other
