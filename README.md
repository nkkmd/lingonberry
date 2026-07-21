# Lingonberry

Lingonberry is a Rust workspace for publishing, validating, storing, retrieving, querying, indexing, and operating canonical knowledge objects. Canonical storage is the source of truth; indexes and effective views are derived, verifiable, and rebuildable. The workspace also includes persistent quarantine, verified backup and replacement workflows, proof-bound retention cleanup, explicit storage-format migration, and a production-oriented single-node operator surface.

## v0.8.0

v0.8.0 completes the single-node operational-readiness milestone for the formal Linux reference platform:

```text
Ubuntu Server 24.04 LTS / x86_64 / systemd
→ install release-built binaries
→ validate effective configuration
→ start through hardened systemd units
→ diagnose with read-only operator commands
→ create and verify backups
→ restore only into isolated targets
→ verify or rebuild derived indexes
→ run a read / write / cleanup disaster-recovery drill
```

Key additions:

- formal reference platform: Ubuntu Server 24.04 LTS, x86_64, and systemd;
- hardened systemd units, environment-file examples, non-root ownership, and filesystem layout guidance;
- integrated `config`, `health`, `ready`, `status`, read-only `doctor`, strict `verify`, and bounded-cardinality `metrics` commands;
- stable canonical JSON diagnostics and documented exit-code contract;
- verified backup creation and isolated verification;
- non-mutating restore planning and explicit isolated restore application;
- fail-closed rejection of symbolic links, active data directories, non-empty targets, partial archives, corrupt state, and unknown-newer formats;
- deterministic index verification and rebuilding;
- isolated disaster-recovery drill with read verification, duplicate-safe write verification, index verification, and mandatory cleanup;
- v0.7.0 to v0.8.0 systemd upgrade and compatible rollback procedures;
- fresh-runner acceptance using release-built binaries installed into `/usr/local/bin`.

All Rust workspace packages and `Cargo.lock` are versioned as `0.8.0` on the release branch. The annotated `v0.8.0` tag and GitHub Release are created only after review and merge.

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
- backup and restore paths reject symbolic links and unsafe target reuse;
- restore never overwrites active state or active data directories;
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
packages/storage      File and SQLite storage backends, operator diagnostics, recovery, and migration runtime
```

## Runtime

Development invocation:

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- serve-http 127.0.0.1:8787
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions -- --reconcile
```

Production-oriented reference installation uses release-built binaries and systemd. See the [v0.8.0 Operator Runbook](docs/operations/V0_8_OPERATOR_RUNBOOK.md).

Storage operator examples:

```bash
lingonberry-storage config
lingonberry-storage health
lingonberry-storage ready
lingonberry-storage status
lingonberry-storage doctor
lingonberry-storage verify
lingonberry-storage metrics
lingonberry-storage backup create /var/backups/lingonberry/manual-backup
lingonberry-storage backup verify /var/backups/lingonberry/manual-backup
lingonberry-storage restore plan /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
lingonberry-storage restore apply /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
lingonberry-storage index verify
lingonberry-storage index rebuild
lingonberry-storage drill restore /var/backups/lingonberry/manual-backup
```

Storage migration remains separately operator-controlled:

```bash
lingonberry-storage-migrate inspect
lingonberry-storage-migrate plan
lingonberry-storage-migrate apply
lingonberry-storage-migrate status
lingonberry-storage-migrate resume
lingonberry-storage-migrate rollback
```

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -A warnings
cargo test --workspace
```

JavaScript contract tests and the external conformance suite are also run by `.github/workflows/ci.yml`. The dedicated operator acceptance workflow validates Ubuntu Server 24.04 LTS, x86_64, systemd units, release-built installed binaries, restart persistence, backup, isolated restore, index lifecycle, fail-closed fixtures, and the DR drill.

## Documentation

- [Current implementation status](docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
- [Roadmap to v1.0](docs/roadmap/ROADMAP_TO_V1_0.md)
- [v0.8.0 release checklist](docs/roadmap/RELEASE_0_8_0_CHECKLIST.md)
- [v0.8.0 release notes](docs/roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [v0.8.0 Operator Runbook](docs/operations/V0_8_OPERATOR_RUNBOOK.md)
- [v0.8.0 Operator CLI Contract](docs/operations/OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 Upgrade and Rollback](docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md)
- [Supported Platforms](docs/operations/SUPPORTED_PLATFORMS.md)
- [Storage migration and upgrade contract](docs/operations/STORAGE_MIGRATION_AND_UPGRADE.md)
- [Transition HTTP API](docs/protocols/HTTP_TRANSITION_API.md)
- [Effective View Read API](docs/protocols/EFFECTIVE_VIEW_READ_API.md)
- [Index lifecycle contract](packages/indexer/INDEX_LIFECYCLE.md)
- [Operations index](docs/operations/README.md)
- [Changelog](CHANGELOG.md)

## Release history

- v0.8.0: single-node operational readiness, Ubuntu 24.04 reference platform, diagnostics, verified recovery, systemd deployment, and fresh-runner acceptance
- v0.7.0: storage-format manifest, deterministic migration planning, verified backup binding, resume and rollback guarantees
- v0.6.0: append-only transitions, durable reevaluation, deterministic effective views, and bounded diagnostics
- v0.5.0: versioned normal-object lifecycle, deterministic index verification, checkpoints, catch-up, and restart/recovery smoke coverage
- v0.4.0: deterministic retention cleanup, proof-bound authorization, and path-level recovery
- v0.3.0: verified replacement-generation transaction and recovery
- v0.2.0: persistent quarantine lifecycle, backup/restore, maintenance, and RBAC
- v0.1.0: initial protocol, schema, fixtures, and carrier contracts

## License

See the package metadata and repository license files for applicable terms.
