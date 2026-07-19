# Changelog

All notable changes to Lingonberry are documented in this file.

## [0.5.0] - 2026-07-19

### Added

- Versioned publish ingestion, object retrieval, and basic-query contracts with stable machine codes.
- Shared CLI and HTTP ingestion orchestration and deterministic duplicate/conflict classification.
- Deterministic index generations with ID/content digests, rebuild, verification, atomic checkpoints, and catch-up.
- Fail-closed handling for corrupt, unsupported, stale, partial, and ambiguous index state.
- Real-binary smoke coverage for publish, restart, retrieval, query, rebuild, checkpoint recovery, and ambiguity rejection.

### Changed

- All Rust workspace packages are versioned as `0.5.0`.
- Canonical storage is explicitly treated as the source of truth and index state as derived and rebuildable.
- CLI `rebuild-index` and `catch-up-index` emit versioned machine-readable contracts.

### Compatibility and safety

- Validation failures do not enter canonical storage and conflicts do not overwrite existing canonical records.
- Inconsistent verification results cannot replace an existing checkpoint.
- Existing quarantine, backup, replacement, cleanup, archive, and immutable-evidence safety boundaries remain intact.

### Known limitations

- v0.5.0 does not add a separately persisted searchable index database, multi-node consistency, vector search, or AI integration.

## [0.4.0] - 2026-07-17

### Added

- Deterministic quarantine replacement retention evaluation with an explicit retained-generation floor.
- Durable terminal completion evidence bound to replacement journals and generation digests.
- Versioned cleanup plan and proof artifacts with canonical JSON and digest sidecars.
- Read-only state reconstruction and stale-proof verification across pointers, journals, generations, completion evidence, and managed-path inventories.
- Dedicated cleanup transaction journals, sealed inventories, deterministic path-level progress, and resumable recovery classification.
- Operator runbook, machine-readable failure-point inventory, crash matrix, smoke test procedure, release checklist, and release notes.

### Changed

- All Rust workspace packages are versioned as `0.4.0`.
- Cleanup requires exact subject selection, verified durable age evidence, immediate state revalidation, and explicit two-stage operator authorization.
- Terminal cleanup transaction workspaces remain retained as operational evidence in v0.4.0.

### Compatibility

- Legacy-root state remains readable and categorically excluded from implicit cleanup selection.
- Generation-aware layouts continue to use verified active pointers and generation metadata.
- Existing replacement apply, resume, rollback, backup, index, and segment verification behavior remains compatible.

### Security and safety

- Active, incomplete, orphan, corrupt, legacy-root, unverified, and insufficiently aged subjects fail closed.
- Symbolic links, unsupported entry types, partial artifact pairs, stale temporary artifacts, and contradictory state are rejected.
- Metrics remain bounded-cardinality and exclude paths, identifiers, digests, record IDs, and free-form error labels.
- No scheduled or unattended cleanup is enabled.

### Known limitations

- Terminal cleanup transaction workspace retention is deferred to a separately versioned future policy.
- Quarantine coordination remains same-host only and is not a distributed lock.
- Secure erase semantics are not promised.

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
