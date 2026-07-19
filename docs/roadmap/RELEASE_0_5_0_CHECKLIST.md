# Lingonberry v0.5.0 Release Checklist

## Release candidate

- [x] All Rust workspace packages are versioned as `0.5.0`.
- [x] `Cargo.lock` is synchronized.
- [x] Rust format, library/binary/test Clippy, and workspace tests pass.
- [x] JavaScript tests pass.
- [x] Publish, retrieval, query, restart, rebuild, catch-up, and ambiguity smoke scenarios pass.
- [x] `CHANGELOG.md` and release notes are synchronized.
- [x] `CURRENT_IMPLEMENTATION_STATUS.md` is synchronized.
- [x] Release roadmap is marked complete through Phase 6 candidate preparation.

## Publication gate

- [ ] Release hardening PR is merged to `main`.
- [ ] Post-merge `main` CI succeeds.
- [ ] Annotated tag `v0.5.0` is created from the verified main commit.
- [ ] GitHub Release `Lingonberry v0.5.0` is published.
- [ ] Tag, release URL, commit, and CI run are recorded here.

## Safety invariants

- Canonical storage remains the source of truth.
- Validation failures never enter canonical storage.
- Duplicate and conflict classifications remain deterministic.
- Index corruption, unsupported checkpoints, and ambiguous content fail closed.
- Existing checkpoints are not overwritten from inconsistent verification results.
- Archive and immutable evidence data are not rewritten by the v0.5.0 lifecycle work.
