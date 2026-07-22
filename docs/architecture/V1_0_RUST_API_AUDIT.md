# v1.0 Rust Public API Audit

**Status: complete** | **Target: v1.0.0** | **Parent issue: #112**

## Final validation

- standard CI run `1172`: success
- Rust public API audit run `9`: success
- validated commit: `4d6549e67420bfd641b201afc4fffef7e0666625`

## Audit conclusions

- The mechanical exported-surface inventory covers all seven Rust library crates.
- Every exported item is covered by an explicit item classification or a reviewed crate/namespace rule.
- `pub` does not automatically mean supported third-party API.
- Stable and behavior-stable contracts are the documented protocol, identity, validation, storage, index, HTTP, operator, recovery, and machine-readable diagnostic semantics.
- File-loading helpers, heuristic shape detection, helper-module paths, runtime wiring, intermediate implementation structures, blanket implementations, debug formatting, and free-form prose ordering are not stable contracts.
- Parser byte and nesting limits are behavior-stable security boundaries.
- No exported item is removed for v1.0.0 solely to reduce surface area.
- No current external dependency on free-form validation error ordering was identified.
- Stable contract families map to protocol, identity, validation, lifecycle, quarantine, crash-matrix, index, relay, migration, backup, restore, and operator acceptance evidence.
- Any compatibility-relevant source change after the recorded inventory requires regeneration and review.

## Mechanical evidence

- generator: `scripts/generate-rust-public-api.sh`
- first complete workflow run: `29926699269`
- artifact: `rust-public-api-ff3ca598c983276664ee34b871ac184eb6536e7e`
- artifact ID: `8532290377`
- artifact SHA-256: `baecd8243eecb55ed89d5f5b0c28561113eb721359ed454ddfe2eeb4b2956548`
- `cargo-public-api 0.52.0`
- `rustc 1.99.0-nightly (0e29c21d9 2026-07-21)`

## Compatibility classes

### Stable or behavior-stable

- protocol/schema/archive/capability/identity-rule version constants
- parser resource rejection boundaries
- canonical parse, normalize, serialize, validate, finalize, identify, and signature behavior
- identity derivation and claim-version semantics
- validation report and reject/defer/accept categories
- storage append, duplicate, conflict, retrieval, archive, quarantine, replacement, cleanup, and query outcomes
- index checkpoint, catch-up, verify, rebuild, and reconstruction outcomes
- HTTP and operator-visible publish, read, query, auth, health, readiness, diagnostics, metrics, migration, backup, restore, and rollback behavior
- machine-readable codes, documented diagnostic severity, HTTP status semantics, and operator exit status

### Workspace-internal or unstable

- `ReadJsonFile`, `read_json_file`, and heuristic `detect_shape`
- helper-module paths and runtime wiring
- temporary signature-workspace layout
- unencoded intermediate structures and helper algorithms
- compiler-generated blanket implementations
- `Debug` formatting and free-form prose wording/order

## Evidence families

| Contract family | Primary evidence |
|---|---|
| Protocol parser and resource bounds | `packages/protocol/tests/parser_baseline.rs`, `packages/protocol/tests/parser_limits.rs` |
| Canonicalization and protocol conformance | protocol tests, `fixtures/conformance/`, external conformance CI |
| Identity and signatures | identity/signature tests and signature-workspace security regression |
| Validation | validation tests and acceptance-policy fixtures |
| Core lifecycle | lifecycle, ingestion, duplicate/conflict, retrieval, archive, and quarantine tests |
| Replacement durability | replacement/cleanup tests and JavaScript crash-point matrix |
| Index consistency | checkpoint, catch-up, verify, rebuild, and restart tests |
| Relay/operator | relay HTTP tests and reference-platform operator acceptance |
| Storage/migration | format and migration plan/apply/resume/verify/commit/rollback tests |
| Backup/recovery | backup verification, isolated restore, doctor, drill, journal/proof, and rollback tests |

The final v1.0 candidate must rerun these evidence families. This audit classifies the surface but does not substitute historical results for final-candidate qualification.

## Deprecation policy input

- no current item is declared deprecated for v1.0.0
- stable v1 contracts are not removed or incompatibly changed during v1.x
- workspace-internal or unstable exports require replacement guidance before deprecation
- removal does not occur before v2.0 unless an explicitly reviewed security emergency requires it

## Release blockers

v1.0.0 is blocked if an export is unclassified, public documentation recommends an unstable/internal item, a frozen behavior lacks evidence, error ordering is relied upon but undocumented, a deprecated item lacks policy, or compatibility-relevant source changes are not followed by regenerated inventory and review.
