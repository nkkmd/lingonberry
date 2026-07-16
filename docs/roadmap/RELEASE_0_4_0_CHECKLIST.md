# Lingonberry v0.4.0 Release Checklist

## Scope

v0.4.0 adds verified quarantine replacement retention evaluation, cleanup preview/proof artifacts, durable completion evidence, and operator-controlled cleanup transaction recovery.

## Safety invariants

- [x] Active generations are categorically excluded.
- [x] Incomplete, orphan, corrupt, legacy-root, and unverified subjects are excluded.
- [x] Eligibility uses durable completion evidence rather than filesystem timestamps.
- [x] Exact subject selection is required; wildcard and implicit-all selection are rejected.
- [x] Preview/proof artifacts bind the complete eligible subject set and current state.
- [x] Apply-time state is revalidated before mutation.
- [x] Symbolic links and unsupported entry types fail closed.
- [x] Cleanup uses a dedicated versioned journal and sealed inventory.
- [x] Progress is recorded in deterministic managed-path order.
- [x] Rollback is available only before the irreversible boundary.
- [x] Post-boundary interruption is represented as recovery-required or partially-deleted.
- [x] Operator request and a separate irreversible-action acknowledgement are required.
- [x] No scheduled or unattended cleanup is enabled.
- [x] Terminal cleanup transaction workspaces are retained in v0.4.0.
- [x] Metrics prohibit path, transaction ID, generation ID, digest, record ID, and free-form error labels.
- [x] Audit records are append-only and secret-free.

## Validation

- [ ] Rust formatting passes on main.
- [ ] Library, binary, and test clippy pass with warnings denied.
- [ ] Workspace tests pass.
- [ ] JavaScript tests pass.
- [ ] Cleanup preview/proof determinism tests pass.
- [ ] Artifact tamper and partial-pair tests pass.
- [ ] State-bound stale proof tests pass.
- [ ] Tomb inventory and recovery tests pass.
- [ ] Completion and rollback evidence-retention tests pass.
- [ ] Legacy-root compatibility tests pass.
- [ ] Generation-layout compatibility tests pass.
- [ ] End-to-end smoke procedure has been reviewed.

## Publication

- [ ] Package versions are `0.4.0`.
- [ ] `Cargo.lock` is synchronized.
- [ ] `CHANGELOG.md` includes the final v0.4.0 entry.
- [ ] Release notes match the committed implementation.
- [ ] Release-readiness PR is merged.
- [ ] Main-branch CI passes after merge.
- [ ] Annotated tag `v0.4.0` points to the reviewed main commit.
- [ ] GitHub Release title and notes match the tag and committed release note.

## Deferred work

Terminal cleanup transaction workspace retention remains outside v0.4.0 and is tracked separately. No automatic workspace retention mechanism may be inferred from generation cleanup eligibility.
