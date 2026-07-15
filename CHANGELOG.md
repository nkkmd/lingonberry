# Changelog

All notable changes to Lingonberry are documented in this file.

## [0.3.0] - 2026-07-15

### Added

- Verified, proof-bound quarantine replacement transactions backed by complete quarantine backup v2 and QL-5C3B replacement proofs.
- Generation-directory active-ledger publication with a sealed generation manifest and a single atomic current-generation pointer switch.
- Durable transaction journals with deterministic status classification, idempotent resume, and pre-commit rollback.
- Versioned structured replacement status and bounded-cardinality Prometheus metrics.
- Secret-free append-only replacement audit events for apply, status, resume, rollback, commit, and recovery-required outcomes.
- Deterministic double-opt-in failure injection covering 18 journal, staging, generation, publication, verification, commit, and rollback boundaries.
- Machine-readable crash-point inventory with registry and inventory consistency checks in CI.
- Read-only generation-retention inspection for active, previous, rolled-back, incomplete, orphan, legacy, and corrupt states.
- End-to-end operator smoke coverage for backup, proof, apply, observation, verification, resume, and rollback paths.

### Changed

- All Rust workspace packages are versioned as `0.3.0`.
- Active quarantine-ledger resolution becomes generation-aware after the first successful replacement publication.
- Replacement apply requires a verified complete backup v2, a verified replacement proof, and a stable runtime fingerprint.
- Post-publication processing now rebuilds and verifies the quarantine ledger index and verifies archive segments before commit.
- Replacement operation failures that leave durable `recovery-required` state are distinguished from preflight rejections in audit output.

### Compatibility

- Deployments without a current-generation pointer continue to use the legacy root-ledger layout.
- The first successful generation publication does not delete legacy root ledgers.
- Backup v1 verification and restore compatibility remain available where previously documented; new replacement apply requires backup v2.
- Automated tests cover upgrade behavior from a v0.2.0-style state layout.

### Security and safety

- Existing managed ledgers are never overwritten in place.
- Immutable evidence ledgers remain byte-identical.
- Archive segments are not rewritten or deleted.
- Invalid pointers, mixed generations, corrupt journals, digest mismatches, and contradictory states fail closed.
- Metrics and audit output exclude secrets, filesystem paths, transaction IDs as metric labels, record IDs, and free-form error labels.

### Known limitations

- Generation and transaction-workspace deletion remain manual and policy-free; no automatic cleanup path is provided.
- Retention deletion, deduplication, event collapse, conflict resolution, and schema migration are not included.
- Quarantine coordination remains same-host only and is not a distributed lock.
- Remote backup upload, backup encryption/signing, OAuth/OIDC, and per-record ACLs are not included.

## [0.2.0] - 2026-07-12

### Added

- Persistent quarantine lifecycle with revalidation, single and batch promotion, annotations, dismissal, and permanent rejection.
- Quarantine status reporting, Prometheus metrics, and scheduled revalidation support.
- Same-host filesystem coordination for quarantine mutations and maintenance operations.
- Verified JSONL ledger indexing, archive-aware ordered reads, and byte-preserving rotation.
- Archive-inclusive quarantine backup format v2 with verification and restore.
- Non-destructive compaction preview and semantic proof under policy v1.
- Dedicated quarantine admin HTTP listener with observer, reviewer, and operator roles.
- Authentication and authorization audit events with bounded, secret-free fields.
- Secret-free diagnostics for deprecated `LINGONBERRY_ADMIN_TOKEN` fallback usage.

### Changed

- All Rust workspace packages are versioned as `0.2.0`.
- Rust CI now requires rustfmt, clippy with warnings denied, and workspace tests.
- New quarantine backups use `lingonberry-quarantine-backup/v2`; verification and restore retain v1 compatibility.

### Deprecated

- `LINGONBERRY_ADMIN_TOKEN` is deprecated as an operator fallback. Use `LINGONBERRY_ADMIN_OPERATOR_TOKEN`.

### Security

- Public and administrative HTTP surfaces are isolated.
- Missing or invalid admin credentials return uniform `401` responses.
- Authenticated roles without route permission receive `403` before request bodies are read.
- Bearer tokens, request bodies, operator notes, and quarantine payloads are excluded from auth audit records and diagnostics.

### Known limitations

- Quarantine coordination is same-host only and is not a distributed lock.
- Compaction policy v1 authorizes no record rewriting or deletion.
- Retention deletion, remote backup upload, backup encryption/signing, OAuth/OIDC, and per-record ACLs are not included.
- The deprecated legacy admin-token fallback remains available until a future major release.

## [0.1.0]

Initial source release containing the core protocol model, schemas, fixtures, carrier contracts, and bootstrap implementations.
