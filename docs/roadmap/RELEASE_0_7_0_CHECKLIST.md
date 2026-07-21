# Lingonberry v0.7.0 release checklist

**Status: release candidate** | **Last updated: 2026-07-21**

## Storage contract

- [x] Versioned storage-format manifest exists.
- [x] Unknown newer format is rejected without mutation.
- [x] Malformed, contradictory, symlinked, or unsupported storage state fails closed.
- [x] Legacy state is bound to a deterministic source inventory digest.

## Migration transaction

- [x] Read-only inspect and deterministic plan are implemented.
- [x] Verified backup is bound to the plan ID and source digest.
- [x] Apply, verify, and commit are journaled durably.
- [x] Commit cannot bypass verification.
- [x] Resume and rollback are deterministic and idempotent.
- [x] Ordinary startup does not migrate implicitly.

## Compatibility

- [x] v0.4.0-equivalent legacy fixture is retained.
- [x] Existing canonical files are not rewritten by the v1 migration.
- [x] Protocol and public object lifecycle contracts are unchanged.
- [x] Upgrade and downgrade policy is documented.
- [x] Deprecated configuration policy is documented.

## Operator surface

- [x] `lingonberry-storage-migrate inspect`
- [x] `lingonberry-storage-migrate plan`
- [x] `lingonberry-storage-migrate backup`
- [x] `lingonberry-storage-migrate apply`
- [x] `lingonberry-storage-migrate verify`
- [x] `lingonberry-storage-migrate commit`
- [x] `lingonberry-storage-migrate resume`
- [x] `lingonberry-storage-migrate rollback`
- [x] `lingonberry-storage-migrate status`

## Validation gate

- [x] Source formatting
- [x] Library Clippy with warnings denied
- [x] Binary Clippy with warnings denied
- [x] Test-target Clippy compilation
- [x] Rust workspace tests
- [x] JavaScript tests
- [x] External conformance suite

## Publication

- [ ] Merge release candidate PR.
- [ ] Confirm main-branch CI.
- [ ] Publish annotated tag `v0.7.0`.
- [ ] Publish GitHub Release using `RELEASE_0_7_0_RELEASE_NOTE.md`.
- [ ] Close parent Issue #99.
