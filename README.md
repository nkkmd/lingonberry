# Lingonberry

Lingonberry is a Rust workspace for publishing, validating, storing, retrieving, querying, indexing, and operating canonical knowledge objects. Canonical storage is the source of truth; indexes are derived, verifiable, and rebuildable. The workspace also includes persistent quarantine, verified backup and replacement workflows, and proof-bound retention cleanup.

## v0.5.0

v0.5.0 completes the normal single-node knowledge-object lifecycle:

```text
publish
→ validate
→ store
→ retrieve
→ query
→ restart
→ retrieve / query
→ rebuild / consistency verification
→ checkpoint catch-up
```

Key additions:

- versioned publish, object-retrieval, and basic-query result contracts;
- shared CLI and HTTP ingestion orchestration;
- deterministic duplicate and conflict classification across live publish, retry, archive import, and quarantine promotion;
- stable machine codes, HTTP status mappings, and CLI exit mappings;
- deterministic index generations using canonical-ID and record-content digests;
- machine-readable rebuild and consistency reports;
- atomic index checkpoints and checkpoint-driven catch-up;
- fail-closed handling for corrupt, unsupported, stale, partial, and ambiguous index state;
- real-binary smoke coverage for restart, recovery, duplicate, conflict, defer, validation rejection, and ambiguity rejection.

The Rust workspace packages are versioned as `0.5.0`. The `v0.5.0` tag and GitHub Release were published from commit `bf8176da0d992152fb116ca0c45177904d1aa61c` after successful main-branch CI.

## Safety boundaries

Lingonberry treats ambiguous or contradictory state as an error. In particular:

- validation failures do not enter canonical storage;
- conflicts do not overwrite existing canonical records;
- canonical storage remains authoritative over derived index state;
- a canonical-storage commit is not rewritten as a storage failure when only index processing fails;
- corrupt, unsupported, partial, stale, or ambiguous index state is not reported as success;
- inconsistent verification results cannot replace an existing checkpoint;
- cleanup never rewrites archive segments or immutable evidence ledgers;
- active, incomplete, orphan, corrupt, legacy-root, unverified, or insufficiently aged subjects are not cleanup-eligible;
- rollback is available only before irreversible cleanup processing begins;
- same-host locking is not a distributed lock;
- secure erase semantics are not promised.

## Workspace

```text
packages/protocol     canonical protocol model
packages/identity     identity primitives
packages/validation   validation rules
packages/core         ingestion contracts and quarantine / replacement / cleanup logic
packages/indexer      deterministic index lifecycle, checkpoints, verification, and catch-up
packages/relay        CLI and HTTP relay surfaces
packages/storage      File and SQLite storage backends
```

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables
cargo test --workspace
```

JavaScript contract tests are also run by `.github/workflows/ci.yml`.

## Documentation

- [Current implementation status](docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
- [Roadmap to v1.0](docs/roadmap/ROADMAP_TO_V1_0.md)
- [v0.5.0 roadmap](docs/roadmap/RELEASE_0_5_0_ROADMAP.md)
- [v0.5.0 release checklist](docs/roadmap/RELEASE_0_5_0_CHECKLIST.md)
- [v0.5.0 release notes](docs/roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)
- [Index lifecycle contract](packages/indexer/INDEX_LIFECYCLE.md)
- [Index catch-up contract](packages/indexer/INDEX_CATCH_UP.md)
- [Operations index](docs/operations/README.md)
- [v0.4.0 cleanup retention policy](docs/operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md)
- [v0.4.0 cleanup operations runbook](docs/operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
- [Changelog](CHANGELOG.md)

## Release history

- v0.5.0: versioned normal-object lifecycle, deterministic index verification, checkpoints, catch-up, and restart/recovery smoke coverage
- v0.4.0: deterministic retention cleanup, proof-bound authorization, and path-level recovery
- v0.3.0: verified replacement-generation transaction and recovery
- v0.2.0: persistent quarantine lifecycle, backup/restore, maintenance, and RBAC
- v0.1.0: initial protocol, schema, fixtures, and carrier contracts

## License

See the package metadata and repository license files for applicable terms.