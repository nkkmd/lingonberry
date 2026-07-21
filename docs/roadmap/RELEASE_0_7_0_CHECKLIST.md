# Lingonberry v0.7.0 release checklist

**Status: published** | **Last updated: 2026-07-21** | **Release target: `b364ac0c19e9dcec10c25db22a850c9d096b0f9b`**

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

- [x] Merge release candidate PR #100.
- [x] Confirm release validation.
- [x] Publish annotated tag `v0.7.0` at `b364ac0c19e9dcec10c25db22a850c9d096b0f9b`.
- [x] Publish GitHub Release `Lingonberry v0.7.0`.
- [x] Close parent Issue #99 as completed.
- [x] Synchronize root and index documentation through PR #101.
