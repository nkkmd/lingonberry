# Lingonberry

Lingonberry is a Rust workspace for publishing, validating, storing, retrieving, querying, indexing, and operating canonical knowledge objects. Canonical storage is the source of truth; indexes and effective views are derived, verifiable, and rebuildable. The workspace also includes persistent quarantine, verified backup and replacement workflows, and proof-bound retention cleanup.

## v0.6.0

v0.6.0 adds append-only knowledge transitions and deterministic effective views without mutating the original Knowledge Object:

```text
publish Knowledge Object
→ append signed Transition Objects
→ durably record reevaluation work
→ evaluate authority and transition graph
→ publish current, stale, ambiguous, withdrawn, or unavailable effective view
```

Key additions:

- dedicated signed `POST /v1/transitions` route;
- append-only replace and withdraw Transition Objects;
- idempotent duplicate handling and immutable conflict rejection;
- missing-target transitions retained as orphan evidence;
- durable target-scoped reevaluation intent, coalescing, and restart reconciliation;
- deterministic evidence-generation digests over the complete ordered target evidence set;
- fail-closed authority and graph evaluation;
- last-known-good effective views with separate semantic and observation checkpoints;
- `GET /v1/effective-objects/{targetId}` with explicit `current`, `stale`, and `unavailable` freshness;
- stable protocol-level diagnostic reason codes;
- bounded diagnostic summaries and generation-fixed pagination contracts;
- conformance contracts for diagnostic retention, cursor leases, read guards, and bounded heartbeats.

The Rust workspace packages are versioned as `0.6.0`. Publication of the `v0.6.0` tag and GitHub Release follows successful main-branch CI.

## Safety boundaries

Lingonberry treats ambiguous, incomplete, or contradictory state as an error. In particular:

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
- public diagnostics exclude storage paths, row IDs, stack traces, and unstable implementation errors;
- diagnostic truncation and unavailable retained generations are explicit;
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
packages/storage      File and SQLite storage backends
```

## Runtime

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- serve-http 127.0.0.1:8787
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions -- --reconcile
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
- [v0.6.0 release checklist](docs/roadmap/RELEASE_0_6_0_CHECKLIST.md)
- [v0.6.0 release notes](docs/roadmap/RELEASE_0_6_0_RELEASE_NOTE.md)
- [Transition HTTP API](docs/protocols/HTTP_TRANSITION_API.md)
- [Effective View Read API](docs/protocols/EFFECTIVE_VIEW_READ_API.md)
- [Transition reevaluation queue](docs/protocols/TRANSITION_REEVALUATION_QUEUE.md)
- [Deterministic transition evidence generation](docs/protocols/TRANSITION_EVIDENCE_GENERATION.md)
- [Last-known-good effective view](docs/protocols/LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md)
- [Diagnostic pagination and retention](docs/protocols/EFFECTIVE_VIEW_DIAGNOSTIC_PAGINATION.md)
- [Index lifecycle contract](packages/indexer/INDEX_LIFECYCLE.md)
- [Operations index](docs/operations/README.md)
- [Changelog](CHANGELOG.md)

## Release history

- v0.6.0: append-only transitions, durable reevaluation, deterministic effective views, and bounded diagnostics
- v0.5.0: versioned normal-object lifecycle, deterministic index verification, checkpoints, catch-up, and restart/recovery smoke coverage
- v0.4.0: deterministic retention cleanup, proof-bound authorization, and path-level recovery
- v0.3.0: verified replacement-generation transaction and recovery
- v0.2.0: persistent quarantine lifecycle, backup/restore, maintenance, and RBAC
- v0.1.0: initial protocol, schema, fixtures, and carrier contracts

## License

See the package metadata and repository license files for applicable terms.
