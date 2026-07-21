# Lingonberry

Lingonberry is a Rust workspace for publishing, validating, storing, retrieving, querying, indexing, and operating canonical knowledge objects. Canonical storage is the source of truth; indexes and effective views are derived, verifiable, and rebuildable. The workspace also includes persistent quarantine, verified backup and replacement workflows, proof-bound retention cleanup, and explicit storage-format migration.

## v0.7.0

v0.7.0 adds upgrade guarantees for existing single-node installations without changing the canonical meaning of stored Knowledge Objects:

```text
inspect durable storage
→ build deterministic migration plan
→ create and verify plan-bound backup
→ apply target storage format
→ verify durable state
→ commit or deterministically resume / rollback
```

Key additions:

- versioned `storage-format.manifest` v1 with stable layout identifier;
- deterministic read-only inventory and source-state digest;
- explicit `empty`, `legacy_unversioned`, `supported`, `unknown_newer`, and `corrupt` classifications;
- fail-closed rejection of malformed manifests, newer unsupported formats, symlinks, special files, and changed-after-plan state;
- durable migration journal with validated forward and rollback transitions;
- verified backup snapshot bound to migration plan ID and source inventory digest;
- explicit apply, verify, commit, resume, rollback, and status operations;
- dedicated `lingonberry-storage-migrate` operator CLI;
- v0.4.0-equivalent persistent fixture and integration coverage;
- documented upgrade, downgrade, and deprecated-configuration policy.

All Rust workspace packages and `Cargo.lock` are versioned as `0.7.0`. The `v0.7.0` tag and GitHub Release are published.

## Safety boundaries

Lingonberry treats ambiguous, incomplete, unsupported, or contradictory state as an error. In particular:

- validation failures do not enter canonical storage;
- conflicts do not overwrite existing canonical records;
- original Knowledge Objects are never rewritten or deleted by Transition Objects;
- unauthorized or unknown transitions do not affect the effective view;
- multiple authorized heads are not resolved by timestamps or arbitrary identifier order;
- missing-target transitions remain evidence but are not applied until reevaluated;
- canonical storage commits are not rewritten as failures when only derived processing fails;
- stale workers cannot overwrite a newer derived checkpoint;
- incomplete evidence cannot overwrite the last-known-good semantic checkpoint;
- stale effective views are never labeled current;
- ordinary startup never performs implicit storage migration;
- unknown newer storage formats are never mutated;
- non-empty legacy migration does not begin without verified backup evidence bound to the inspected source state;
- target format is not committed before verification succeeds durably;
- public diagnostics exclude storage paths, row IDs, stack traces, and unstable implementation errors;
- cleanup never rewrites archive segments or immutable evidence ledgers;
- same-host locking is not a distributed lock;
- secure erase semantics are not promised.

## Workspace

```text
packages/protocol     canonical protocol model
packages/identity     identity primitives
packages/validation   validation rules
packages/core         ingestion contracts and quarantine / replacement / cleanup logic
packages/indexer      deterministic index lifecycle, checkpoints, verification, and catch-up
packages/relay        CLI, HTTP relay, Transition, effective-view, and reevaluation surfaces
packages/storage      File and SQLite storage backends plus storage migration runtime
```

## Runtime

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- serve-http 127.0.0.1:8787
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions -- --reconcile
```

Storage migration is operator-controlled:

```bash
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- inspect
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- plan
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- apply
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- status
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- resume
cargo run -p lingonberry-storage --bin lingonberry-storage-migrate -- rollback
```

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -A warnings
cargo test --workspace
```

JavaScript contract tests and the external conformance suite are also run by `.github/workflows/ci.yml`.

## Documentation

- [Current implementation status](docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
- [Roadmap to v1.0](docs/roadmap/ROADMAP_TO_V1_0.md)
- [v0.7.0 release checklist](docs/roadmap/RELEASE_0_7_0_CHECKLIST.md)
- [v0.7.0 release notes](docs/roadmap/RELEASE_0_7_0_RELEASE_NOTE.md)
- [Storage migration and upgrade contract](docs/operations/STORAGE_MIGRATION_AND_UPGRADE.md)
- [Transition HTTP API](docs/protocols/HTTP_TRANSITION_API.md)
- [Effective View Read API](docs/protocols/EFFECTIVE_VIEW_READ_API.md)
- [Transition reevaluation queue](docs/protocols/TRANSITION_REEVALUATION_QUEUE.md)
- [Deterministic transition evidence generation](docs/protocols/TRANSITION_EVIDENCE_GENERATION.md)
- [Last-known-good effective view](docs/protocols/LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md)
- [Index lifecycle contract](packages/indexer/INDEX_LIFECYCLE.md)
- [Operations index](docs/operations/README.md)
- [Changelog](CHANGELOG.md)

## Release history

- v0.7.0: storage-format manifest, deterministic migration planning, verified backup binding, resume and rollback guarantees
- v0.6.0: append-only transitions, durable reevaluation, deterministic effective views, and bounded diagnostics
- v0.5.0: versioned normal-object lifecycle, deterministic index verification, checkpoints, catch-up, and restart/recovery smoke coverage
- v0.4.0: deterministic retention cleanup, proof-bound authorization, and path-level recovery
- v0.3.0: verified replacement-generation transaction and recovery
- v0.2.0: persistent quarantine lifecycle, backup/restore, maintenance, and RBAC
- v0.1.0: initial protocol, schema, fixtures, and carrier contracts

## License

See the package metadata and repository license files for applicable terms.
