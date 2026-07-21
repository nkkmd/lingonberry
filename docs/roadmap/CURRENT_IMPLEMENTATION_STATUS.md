# 現在の実装状況

**Status: v0.7.0 merge-ready release candidate** | **Last updated: 2026-07-21**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## Release state

```text
released version: 0.6.0
candidate version: 0.7.0
parent issue: #99
release candidate PR: #100
branch: agent/v0.7.0-storage-migration-foundation
publication state: ready for review; merge, tag, and GitHub Release not published
```

## v0.7.0で実装済み

- versioned `storage-format.manifest`
- storage format v1／`single-node-canonical-v1` layout contract
- deterministic read-only storage inspection
- explicit `empty`／`legacy_unversioned`／`supported`／`unknown_newer`／`corrupt` classification
- deterministic durable inventory and source digest binding
- migration plan and durable migration journal
- verified migration snapshot bound to plan ID and source digest
- explicit apply／verify／commit orchestration
- deterministic resume／rollback
- dedicated `lingonberry-storage-migrate` operator CLI
- v0.4.0-equivalent persistent fixture
- upgrade／downgrade／deprecated configuration policy
- all Rust workspace packages and `Cargo.lock` set to `0.7.0`
- release note and release checklist

## Fixed safety model

- Ordinary startup never performs implicit migration.
- Unknown newer formats are rejected before mutation.
- Malformed manifests, unsupported layouts, symlinks, special files, and changed-after-plan state fail closed.
- Non-empty legacy storage cannot enter migration without a verified backup.
- Migration cannot reach `committed` before durable verification.
- The v1 migration does not rewrite canonical durable files; it introduces a verified format manifest.
- Interrupted migration is resumed from durable journal evidence or rolled back before commit.
- A committed migration is not downgraded in place; downgrade requires restoration of a compatible verified backup.
- Protocol and public object lifecycle contracts remain unchanged.

## Operator runtime

```bash
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- inspect
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- plan
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- backup
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- apply
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- verify
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- commit
```

Recovery commands:

```bash
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- status
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- resume
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- rollback
```

## Validation state

The release candidate has passed:

- `cargo fmt --all -- --check`
- library Clippy with warnings denied
- binary Clippy with warnings denied
- test-target Clippy compilation
- `cargo test --workspace`
- JavaScript tests
- external conformance suite
- legacy migration／verified backup／commit／resume／rollback integration coverage

## Known limitations

- Automatic downgrade is not supported.
- The storage format v1 migration is format-manifest introduction; future data-rewriting migrations require separate version-specific steps.
- Complete external delegation／revocation registry evaluation remains outside v0.7.0.
- Multi-node migration coordination and distributed locking remain outside v1.0 scope.
- Durable cursor lease／read-guard storage remains deployment-specific.

## Remaining before publication

1. Merge PR #100.
2. Confirm main-branch CI.
3. Publish annotated tag `v0.7.0` and GitHub Release.
4. Close Issue #99 as completed.
