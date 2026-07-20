# v0.6.0 Changelog Entry

- Append-only signed Transition Objects through dedicated `POST /v1/transitions` handling.
- Replace and withdraw transitions with duplicate, immutable-conflict, orphan-retention, authority, supersession, and ambiguous-head contracts.
- Durable target-scoped reevaluation, generation coalescing, restart reconciliation, and a dedicated reevaluation CLI.
- Deterministic target evidence generations over ordered canonical evidence, including classified unsupported, corrupt, and unreadable markers.
- Last-known-good effective views with separate semantic and observation checkpoints.
- Stable bounded diagnostics and generation-fixed pagination, retention, cursor-lease, read-guard, and heartbeat contracts.
- All Rust workspace packages are versioned as `0.6.0`.
