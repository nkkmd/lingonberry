# Lingonberry v0.3.0 Release Checklist

**Status: in progress** | **Release target: v0.3.0** | **Last updated: 2026-07-14**

## 1. Release scope

- [x] QL-5C3A replacement policy and semantic-equivalence contract complete
- [x] QL-5C3B policy-v2 replacement preview and proof complete
- [x] QL-5C3C generation-directory rewrite transaction and recovery complete
- [ ] QL-5C3D operations, observability, and release hardening complete

## 2. Safety invariants

- [ ] Existing ledgers are never overwritten in place
- [ ] Immutable evidence ledgers remain byte-identical
- [ ] Archive segments are not rewritten or deleted
- [ ] Mixed or contradictory generations fail closed
- [ ] Pointer-present states never fall back to legacy root ledgers
- [ ] Verified backup v2 remains mandatory for replacement apply
- [ ] QL-5C3B proof verification remains mandatory before apply
- [ ] Runtime fingerprint changes abort publication
- [ ] Committed transactions remain terminal
- [ ] Automatic generation/workspace deletion is absent
- [ ] Retention deletion, deduplication, event collapse, conflict resolution, and schema migration remain absent

## 3. Operations and observability

- [ ] Structured replacement status has a versioned contract
- [ ] Status covers prepared, writing, staged, verified, publishing, committed, rolled-back, recovery-required, and corrupt states
- [ ] Prometheus metrics use bounded labels only
- [ ] Metrics expose legacy vs generation layout without transaction-ID labels
- [ ] Recovery-required and fail-closed outcomes are observable
- [ ] Replacement audit events are append-only
- [ ] Audit events contain no secrets, raw ledger lines, full paths, or unbounded errors
- [ ] CLI help matches the implementation contract and runbooks

## 4. Failure and crash hardening

- [ ] Journal write failure injection
- [ ] Journal fsync failure injection
- [ ] Staged ledger write failure injection
- [ ] Staged ledger fsync failure injection
- [ ] Staging-directory fsync failure injection
- [ ] Generation manifest failure injection
- [ ] Generation materialization failure injection
- [ ] Publication-intent failure injection
- [ ] Pointer temporary-write failure injection
- [ ] Pointer rename failure injection
- [ ] State-directory fsync failure injection
- [ ] Index rebuild failure injection
- [ ] Index verification failure injection
- [ ] Segment verification failure injection
- [ ] Commit-transition failure injection
- [ ] Rollback pointer-restoration failure injection
- [ ] Rolled-back transition failure injection
- [ ] Crash-point matrix is table-driven or machine-readable
- [ ] Post-switch/pre-commit recovery test passes
- [ ] Repeated apply/resume/rollback idempotency tests pass
- [ ] Contradictory pointer state fails closed

## 5. Compatibility and upgrade

- [ ] Legacy root-ledger layout remains readable and writable when no pointer exists
- [ ] First generation publication upgrades active resolution without deleting root ledgers
- [ ] v0.2.0-style state fixture upgrade test passes
- [ ] Policy-v1 compaction preview/proof behavior remains compatible
- [ ] Backup v1 verify/restore compatibility remains intact where documented
- [ ] Public/admin listener isolation regression passes
- [ ] Existing quarantine status and metrics regressions pass

## 6. Operational smoke test

- [ ] Backup v2 export succeeds
- [ ] Backup v2 verification succeeds
- [ ] Replacement preview succeeds
- [ ] Replacement proof verification succeeds
- [ ] Replacement apply reaches committed
- [ ] Replacement status reports committed and active target generation
- [ ] Generation-aware reader and writer operations succeed
- [ ] Ledger index verification succeeds
- [ ] Segment verification succeeds
- [ ] Repeated apply/resume is idempotent
- [ ] Injected post-switch failure resumes successfully
- [ ] Separate pre-commit rollback fixture returns to previous generation
- [ ] Smoke-test commands and expected outputs are recorded without secrets or machine-specific absolute paths

## 7. Generation retention policy

- [ ] Active committed generation classification documented
- [ ] Previous committed generation classification documented
- [ ] Rolled-back generation classification documented
- [ ] Incomplete transaction generation classification documented
- [ ] Orphan unreferenced generation classification documented
- [ ] Unknown/corrupt generation classification documented
- [ ] Any cleanup reporting is read-only
- [ ] No automatic deletion path exists

## 8. Documentation

- [x] Replacement policy document
- [x] Replacement preview/proof document
- [x] Replacement preview runbook
- [x] Replacement transaction contract
- [x] Generation-directory contract
- [x] Recovery runbook
- [x] Operations-hardening contract
- [ ] Current implementation status updated for QL-5C3D
- [ ] Quarantine lifecycle backlog updated for QL-5C3D
- [ ] v0.3.0 roadmap updated to actual transaction states and deliverables
- [ ] v0.3.0 release note finalized
- [ ] README or top-level operator documentation updated if required

## 9. CI and release gates

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --lib -- -D warnings`
- [ ] `cargo clippy --workspace --bins -- -D warnings -A dead-code`
- [ ] `cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables`
- [ ] `cargo test --workspace`
- [ ] JavaScript canonicalization tests
- [ ] JavaScript identity tests
- [ ] JavaScript validation tests
- [ ] No temporary diagnostic workflow remains
- [ ] Main-branch CI passes after merge

## 10. Release preparation

- [ ] Version fields updated to `0.3.0` where applicable
- [ ] `Cargo.lock` reviewed
- [ ] Release note includes migration/compatibility behavior
- [ ] Release note includes generation-directory layout
- [ ] Release note includes recovery and rollback limits
- [ ] Release note explicitly states non-goals and no automatic deletion
- [ ] Tag name selected: `v0.3.0`
- [ ] Release commit identified
- [ ] Final repository status clean
- [ ] GitHub release created only after all mandatory items are complete
