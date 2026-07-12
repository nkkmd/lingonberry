# Changelog

All notable changes to Lingonberry are documented in this file.

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
