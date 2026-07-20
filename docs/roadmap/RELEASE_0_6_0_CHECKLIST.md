# v0.6.0 Release Checklist

## Implementation

- [x] Dedicated signed `POST /v1/transitions`
- [x] Append-only transition storage
- [x] Duplicate and immutable conflict classification
- [x] Missing-target orphan retention
- [x] Durable target-scoped reevaluation intent
- [x] Reevaluation and restart reconciliation CLI
- [x] Deterministic evidence generation
- [x] Last-known-good effective-view behavior
- [x] Stable public diagnostics
- [x] Bounded diagnostic summary and pagination contract
- [x] Diagnostic retention, cursor lease, read guard, and heartbeat contracts
- [x] Existing `rebuild-index` and `catch-up-index` CLI compatibility retained

## Versioning and documentation

- [x] Rust workspace packages set to `0.6.0`
- [x] `Cargo.lock` synchronized to `0.6.0`
- [x] Root `README.md` updated for v0.6.0
- [x] `CHANGELOG.md` v0.6.0 entry added
- [x] Current implementation status synchronized
- [x] v0.6.0 release note added
- [x] v0.6.0 release checklist added
- [x] PR #98 title and body synchronized

## Validation

- [x] Rust source formatted in CI with `cargo fmt --all`
- [x] Library Clippy with warnings denied
- [x] Binary Clippy with warnings denied
- [x] Test targets compiled with Clippy
- [x] `cargo test --workspace`
- [x] JavaScript tests
- [x] External conformance suite
- [x] Final candidate validation green after release-document and compatibility updates

## Pre-merge

- [x] PR #98 marked ready for review
- [x] Final diff and compatibility review completed
- [ ] Merge authorization confirmed

## Publication after merge

- [ ] PR #98 merged to `main`
- [ ] Main branch CI green
- [ ] Annotated tag `v0.6.0`
- [ ] GitHub Release `Lingonberry v0.6.0`
- [ ] Issue #97 closed as completed
