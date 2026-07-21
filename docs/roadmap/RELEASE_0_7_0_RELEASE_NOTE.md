# Lingonberry v0.7.0 release note

**Status: published** | **Date: 2026-07-21** | **Release target: `b364ac0c19e9dcec10c25db22a850c9d096b0f9b`**

## Summary

v0.7.0 introduces explicit, fail-closed storage-format migration and upgrade guarantees for existing single-node installations.

## Added

- Versioned `storage-format.manifest` with storage format v1 and layout `single-node-canonical-v1`.
- Deterministic read-only inspection of empty, legacy, supported, unknown-newer, and corrupt storage states.
- Source-inventory-bound migration plans and durable migration journals.
- Verified migration snapshots bound to the plan and source inventory digest.
- Explicit `prepare`, `backup`, `apply`, `verify`, `commit`, `resume`, and `rollback` execution.
- Dedicated `lingonberry-storage-migrate` operator CLI.
- v0.4.0-equivalent persistent fixture and integration coverage.

## Safety

- Unknown newer formats, malformed manifests, symlinks, unsupported entries, changed-after-plan state, and missing backup evidence fail closed.
- Ordinary startup never performs implicit migration.
- The v1 migration does not rewrite canonical durable data; it adds a verified format manifest.
- Migration cannot commit before durable verification.
- Interrupted migrations resume from journal evidence or roll back by removing the uncommitted manifest.
- Committed migrations cannot be rolled back in place; downgrade requires restoration of a compatible verified backup.

## Operator sequence

```text
lingonberry-storage-migrate inspect
lingonberry-storage-migrate plan
lingonberry-storage-migrate backup
lingonberry-storage-migrate apply
lingonberry-storage-migrate verify
lingonberry-storage-migrate commit
```

Use `resume` after interruption and `rollback` before commit when recovery policy requires returning to the legacy state.

## Validation

The published release passed:

- `cargo fmt --all -- --check`
- library and binary Clippy with warnings denied
- test-target Clippy compilation
- `cargo test --workspace`
- JavaScript tests
- external conformance suite

## Compatibility

Protocol and public object-lifecycle contracts remain unchanged. Existing v0.4.0-v0.6.0 durable layouts without a storage manifest are treated as legacy unversioned state and require a verified backup before migration.

## Publication

- Tag: `v0.7.0`
- Release target: `b364ac0c19e9dcec10c25db22a850c9d096b0f9b`
- Parent issue: #99, closed as completed
- Release candidate PR: #100, merged
- Post-release documentation PR: #101, merged
