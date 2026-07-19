# Lingonberry v0.5.0 Release Checklist

## Release candidate

- [x] All Rust workspace packages are versioned as `0.5.0`.
- [x] `Cargo.lock` is synchronized.
- [x] Rust format, library/binary/test Clippy, and workspace tests pass.
- [x] JavaScript tests pass.
- [x] Publish, retrieval, query, restart, rebuild, catch-up, and ambiguity smoke scenarios pass.
- [x] `CHANGELOG.md` and release notes are synchronized.
- [x] `CURRENT_IMPLEMENTATION_STATUS.md` is synchronized.
- [x] Release roadmap is marked complete through Phase 6.

## Publication gate

- [x] Release hardening PR #94 is merged to `main`.
- [x] README and documentation synchronization PR #95 is merged to `main`.
- [x] Post-merge `main` CI succeeds, as confirmed in the GitHub Actions UI.
- [x] Tag `v0.5.0` is created from the verified main commit.
- [x] GitHub Release `Lingonberry v0.5.0` is published.
- [x] Tag, release URL, commit, and CI confirmation are recorded here.

## Publication record

- Release: https://github.com/nkkmd/lingonberry/releases/tag/v0.5.0
- Tag: `v0.5.0`
- Release target commit: `bf8176da0d992152fb116ca0c45177904d1aa61c`
- Tag/main comparison at publication: identical
- Main CI: successful, confirmed in the GitHub Actions UI
- Published: 2026-07-19

## Safety invariants

- Canonical storage remains the source of truth.
- Validation failures never enter canonical storage.
- Duplicate and conflict classifications remain deterministic.
- Index corruption, unsupported checkpoints, and ambiguous content fail closed.
- Existing checkpoints are not overwritten from inconsistent verification results.
- Archive and immutable evidence data are not rewritten by the v0.5.0 lifecycle work.