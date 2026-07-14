# Lingonberry v0.3.0 Release Checklist

**Status: in progress** | **Release target: v0.3.0** | **Last updated: 2026-07-14**

## 1. Release scope

- [x] QL-5C3A replacement policy and semantic-equivalence contract complete
- [x] QL-5C3B policy-v2 replacement preview and proof complete
- [x] QL-5C3C generation-directory rewrite transaction and recovery complete
- [ ] QL-5C3D operations, observability, and release hardening complete

## 2. Safety invariants

- [x] Existing ledgers are never overwritten in place
- [x] Immutable evidence ledgers remain byte-identical
- [x] Archive segments are not rewritten or deleted
- [x] Mixed or contradictory generations fail closed
- [x] Pointer-present states never fall back to legacy root ledgers
- [x] Verified backup v2 remains mandatory for replacement apply
- [x] QL-5C3B proof verification remains mandatory before apply
- [x] Runtime fingerprint changes abort publication
- [x] Committed transactions remain terminal
- [x] Automatic generation/workspace deletion is absent
- [x] Retention deletion, deduplication, event collapse, conflict resolution, and schema migration remain absent

## 3. Operations and observability

- [x] Structured replacement status has a versioned contract
- [x] Status covers prepared, writing, staged, verified, publishing, committed, rolled-back, recovery-required, and corrupt states
- [x] Prometheus metrics use bounded labels only
- [x] Metrics expose legacy vs generation layout without transaction-ID labels
- [x] Recovery-required and fail-closed outcomes are observable
- [x] Replacement audit events are append-only
- [x] Audit events contain no secrets, raw ledger lines, full paths, or unbounded errors
- [x] CLI help matches the implementation contract and runbooks

## 4. Failure and crash hardening

- [x] Journal write failure injection
- [x] Journal fsync failure injection
- [ ] Staged ledger write failure injection
- [ ] Staged ledger fsync failure injection
- [ ] Staging-directory fsync failure injection
- [ ] Generation manifest failure injection
- [ ] Generation materialization failure injection
- [ ] Publication-intent failure injection
- [x] Pointer temporary-write failure injection
- [x] Pointer rename failure injection
- [ ] State-directory fsync failure injection
- [x] Index rebuild failure injection
- [ ] Index verification failure injection
- [ ] Segment verification failure injection
- [x] Commit-transition failure injection
- [x] Rollback pointer-restoration failure injection
- [x] Rolled-back transition failure injection
- [x] Crash-point matrix is table-driven or machine-readable
- [x] Crash-point registry and inventory consistency are enforced by CI
- [x] Post-switch/pre-commit recovery test passes
- [x] Repeated apply/resume/rollback idempotency tests pass
- [x] Contradictory pointer state fails closed

## 5. Compatibility and upgrade

- [x] Legacy root-ledger layout remains readable and writable when no pointer exists
- [x] First generation publication upgrades active resolution without deleting root ledgers
- [x] v0.2.0-style state fixture upgrade test passes
- [x] Policy-v1 compaction preview/proof behavior remains compatible
- [x] Backup v1 verify/restore compatibility remains intact where documented
- [x] Public/admin listener isolation regression passes
- [x] Existing quarantine status and metrics regressions pass

## 6. Operational smoke test

- [x] Backup v2 export succeeds
- [x] Backup v2 verification succeeds
- [x] Replacement preview succeeds
- [x] Replacement proof verification succeeds
- [x] Replacement apply reaches committed
- [x] Replacement status reports committed and active target generation
- [x] Generation-aware reader operations succeed
- [x] Ledger index verification succeeds
- [x] Segment verification succeeds
- [x] Repeated apply/resume is idempotent
- [x] Injected post-switch failure resumes successfully
- [x] Separate pre-commit rollback fixture returns to previous generation
- [x] Smoke-test commands and expected outputs are recorded without secrets or machine-specific absolute paths

## 7. Generation retention policy

- [x] Active committed generation classification documented
- [x] Previous committed generation classification documented
- [x] Rolled-back generation classification documented
- [x] Incomplete transaction generation classification documented
- [x] Orphan unreferenced generation classification documented
- [x] Unknown/corrupt generation classification documented
- [x] Any cleanup reporting is read-only
- [x] No automatic deletion path exists

## 8. Documentation

- [x] Replacement policy document
- [x] Replacement preview/proof document
- [x] Replacement preview runbook
- [x] Replacement transaction contract
- [x] Generation-directory contract
- [x] Recovery runbook
- [x] Operations-hardening contract
- [x] Current implementation status updated for QL-5C3D
- [x] Quarantine lifecycle backlog updated for QL-5C3D
- [x] v0.3.0 roadmap updated to actual transaction states and deliverables
- [x] v0.3.0 release note finalized
- [ ] README or top-level operator documentation updated if required

## 9. CI and release gates

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --lib -- -D warnings`
- [x] `cargo clippy --workspace --bins -- -D warnings -A dead-code`
- [x] `cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables`
- [x] `cargo test --workspace`
- [x] JavaScript canonicalization tests
- [x] JavaScript identity tests
- [x] JavaScript validation tests
- [x] Crash-point registry/inventory contract test
- [x] No temporary diagnostic workflow remains
- [ ] Main-branch CI passes after merge

## 10. Release preparation

- [x] Version fields updated to `0.3.0` where applicable
- [x] `Cargo.lock` reviewed through a successful full workspace CI run
- [x] Release note includes migration/compatibility behavior
- [x] Release note includes generation-directory layout
- [x] Release note includes recovery and rollback limits
- [x] Release note explicitly states non-goals and no automatic deletion
- [x] Tag name selected: `v0.3.0`
- [ ] Release commit identified
- [ ] Final repository status clean
- [ ] GitHub release created only after all mandatory items are complete
