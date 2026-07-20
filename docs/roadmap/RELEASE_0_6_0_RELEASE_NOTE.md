# Lingonberry v0.6.0 Release Note

Release date: 2026-07-20

## Summary

v0.6.0 introduces append-only Transition Objects and a deterministic effective-view layer without mutating original Knowledge Objects. It adds dedicated transition publishing, orphan retention, durable target-scoped reevaluation, evidence-generation digests, last-known-good reads, bounded diagnostics, and generation-fixed pagination contracts.

## Main additions

- `POST /v1/transitions` with signed route-isolated envelopes
- Append-only replace and withdraw transitions
- Missing-target orphan transition retention
- Durable reevaluation intent and restart reconciliation CLI
- Deterministic target evidence generations
- `GET /v1/effective-objects/{targetId}` projection model
- Current, stale, and unavailable effective-view freshness
- Stable protocol-level diagnostic reason codes
- Bounded diagnostic summaries and pagination contracts
- Cursor lease, read guard, heartbeat, and retention conformance tests

## Runtime commands

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- serve-http 127.0.0.1:8787
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions -- --reconcile
```

## Compatibility

The v0.5.0 object publish, retrieval, query, indexing, quarantine, backup, replacement, cleanup, archive, and immutable-evidence paths remain available. Transition processing is additive and never rewrites the original canonical object.

## Safety properties

- Invalid signatures and malformed transitions do not enter transition storage.
- Duplicate immutable content is idempotent; conflicting content for the same transition ID is rejected.
- Missing targets do not erase valid signed transition evidence.
- Incomplete evidence cannot overwrite a complete semantic checkpoint.
- Ambiguous authorized heads are not resolved by timestamps or arbitrary identifier order.
- Canonical evidence remains the source of truth; derived snapshots are rebuildable.

## Known limitations

- Complete external delegation and revocation registry evaluation is not included.
- Multi-node queue coordination and distributed snapshot locking are not included.
- Durable cursor lease and read-guard storage remains deployment-specific; protocol behavior is conformance-defined.
