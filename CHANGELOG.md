# Changelog

All notable changes to Lingonberry are documented in this file. Detailed operational contracts and release notes are retained under `docs/`.

## [0.7.0] - 2026-07-21

### Added

- Versioned storage-format manifest and deterministic storage inspection.
- Source-bound migration plans, durable journals, and fail-closed state classification.
- Verified migration snapshots bound to the plan ID and source inventory digest.
- Explicit migration apply, verify, commit, resume, and rollback execution.
- Dedicated `lingonberry-storage-migrate` operator CLI.
- v0.4.0-equivalent persistent fixture and migration integration coverage.

### Changed

- All Rust workspace packages and `Cargo.lock` are versioned as `0.7.0`.
- Existing unversioned single-node durable state is treated as legacy storage requiring verified backup before migration.

### Compatibility and safety

- Ordinary startup does not perform implicit migration.
- Unknown newer formats, malformed manifests, unsupported layouts, symlinks, special files, missing backup evidence, and changed-after-plan state fail closed.
- Storage format v1 migration does not rewrite canonical durable files; it introduces a verified format manifest.
- Protocol and public object-lifecycle contracts remain unchanged.

### Known limitations

- Automatic downgrade is not supported; downgrade requires restoration of a compatible verified backup.
- Multi-node migration coordination and distributed locking are not included.

## [0.6.0] - 2026-07-20

### Added

- Signed append-only Transition Objects through dedicated `POST /v1/transitions` handling.
- Replace and withdraw transitions with duplicate, immutable-conflict, orphan-retention, authority, supersession, and ambiguous-head contracts.
- Durable target-scoped reevaluation, generation coalescing, restart reconciliation, and a dedicated reevaluation CLI.
- Deterministic target evidence generations, including classified unsupported, corrupt, and unreadable markers.
- Last-known-good effective views with separate semantic and observation checkpoints.
- Stable bounded diagnostics and generation-fixed pagination, retention, cursor-lease, read-guard, and heartbeat contracts.

### Changed

- All Rust workspace packages and `Cargo.lock` are versioned as `0.6.0`.
- Original Knowledge Objects remain immutable; replacement and withdrawal are represented as derived effective views.

### Compatibility and safety

- v0.5.0 publish, retrieval, query, indexing, quarantine, backup, cleanup, archive, and immutable-evidence paths remain available.
- Unauthorized, unknown, incomplete, stale, corrupt, or ambiguous transition state fails closed.
- Missing-target transitions remain evidence but do not affect an effective view until reevaluated.
- Incomplete observations cannot overwrite the last-known-good semantic checkpoint.

### Known limitations

- Complete external delegation and revocation evaluation is not included.
- Multi-node queue coordination and distributed snapshot locking are not included.
- Durable cursor lease and read-guard storage remains deployment-specific.

## [0.5.0] - 2026-07-19

- Added versioned publish, retrieval, and basic-query contracts with stable machine codes.
- Added deterministic duplicate/conflict classification and shared CLI／HTTP ingestion orchestration.
- Added deterministic index generations, verification, atomic checkpoints, catch-up, and restart/recovery smoke coverage.
- Canonical storage remains authoritative over derived index state.

## [0.4.0] - 2026-07-17

- Added deterministic quarantine replacement retention evaluation and a retained-generation floor.
- Added durable terminal completion evidence, proof-bound cleanup authorization, and resumable cleanup journals.
- Added path-level recovery, operator runbooks, crash matrices, and release smoke coverage.

## [0.3.0] - 2026-07-15

- Added verified backup-bound replacement transactions and generation-directory publication.
- Added deterministic transaction journals, resume, rollback, structured status, metrics, and failure injection.
- Preserved immutable evidence ledgers and archive segments under fail-closed recovery semantics.

## [0.2.0] - 2026-07-12

- Added persistent quarantine lifecycle, revalidation, promotion, dismissal, and permanent rejection.
- Added verified indexing, archive-aware reads, backup/restore v2, maintenance, and RBAC admin surfaces.
- Added authentication and authorization audit events with bounded secret-free diagnostics.

## [0.1.0]

- Initial protocol model, schemas, fixtures, carrier contracts, and bootstrap implementations.
