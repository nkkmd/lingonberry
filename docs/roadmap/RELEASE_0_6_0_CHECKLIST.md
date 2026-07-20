# v0.6.0 Release Checklist

## Implementation

- [x] Dedicated signed `POST /v1/transitions`
- [x] Append-only transition storage
- [x] Duplicate and immutable conflict classification
- [x] Missing-target orphan retention
- [x] Durable reevaluation intent
- [x] Target-scoped queue coalescing contract
- [x] Deterministic evidence generation
- [x] Last-known-good effective-view behavior
- [x] Stable public diagnostics
- [x] Bounded diagnostic summary and pagination contract
- [x] Diagnostic retention, cursor lease, read guard, and heartbeat contracts
- [x] Reevaluation and reconciliation CLI

## Versioning and documentation

- [x] Rust workspace packages set to `0.6.0`
- [x] `Cargo.lock` synchronized to `0.6.0`
- [x] v0.6.0 release note added
- [x] v0.6.0 release checklist added
- [ ] CHANGELOG v0.6.0 entry merged
- [ ] Current implementation status synchronized

## Validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --lib -- -D warnings`
- [ ] `cargo clippy --workspace --bins -- -D warnings -A dead-code`
- [ ] `cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables`
- [ ] `cargo test --workspace`
- [ ] JavaScript tests
- [ ] External conformance suite
- [ ] PR CI green on final candidate commit

## Publication

- [ ] PR #98 ready for review
- [ ] PR #98 merged to `main`
- [ ] Main branch CI green
- [ ] Annotated tag `v0.6.0`
- [ ] GitHub Release `Lingonberry v0.6.0`
- [ ] Issue #97 closed as completed
