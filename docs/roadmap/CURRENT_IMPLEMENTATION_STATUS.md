# 現在の実装状況

**Status: v0.6.0 release candidate pre-merge** | **Last updated: 2026-07-20**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## Release state

```text
released version: 0.5.0
candidate version: 0.6.0
parent issue: #97
release candidate PR: #98
branch: agent/v0.6.0-protocol-contract-foundation
publication state: pre-merge; tag and GitHub Release not published
```

## v0.6.0で実装済み

- append-only Transition Object model
- dedicated signed `POST /v1/transitions`
- duplicate／immutable conflict classification
- missing-target orphan retention
- authority／supersession／multi-parent graph contract
- durable target-scoped reevaluation intent
- reevaluation／restart reconciliation CLI
- deterministic evidence generation
- classified `unsupported`／`corrupt`／`unreadable` markers
- last-known-good effective view
- `GET /v1/effective-objects/{targetId}` projection
- stable public diagnostics
- bounded summary and generation-fixed pagination contract
- diagnostic retention／cursor lease／read guard／heartbeat conformance
- all Rust workspace packages and `Cargo.lock` set to `0.6.0`
- v0.6.0 release note／checklist／root README

## Fixed safety model

- Original Knowledge Objects are never rewritten or deleted by transitions.
- Only authorized transitions affect the effective view.
- Ambiguous authorized heads are not resolved by timestamps or arbitrary ID order.
- Valid signed missing-target transitions remain append-only orphan evidence.
- Canonical target commit precedes asynchronous derived reevaluation.
- Reevaluation is durable, target-scoped, at-least-once, idempotent, and generation-aware.
- Stale workers cannot advance a newer checkpoint.
- Incomplete evidence cannot overwrite the last-known-good semantic view.
- Stale views are never labeled current.
- Public diagnostics exclude storage paths, row IDs, stack traces, and unstable errors.
- Diagnostic truncation and unavailable retained generations are explicit.
- Derived snapshot cleanup never deletes canonical evidence.

## Runtime

```text
POST /v1/objects
POST /v1/transitions
GET  /v1/effective-objects/{targetId}
```

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- serve-http 127.0.0.1:8787
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions
cargo run -p lingonberry-relay --bin lingonberry-reevaluate-transitions -- --reconcile
```

## Validation state

The release-candidate line has passed:

- library Clippy
- binary Clippy
- test-target Clippy compilation
- `cargo test --workspace`
- JavaScript tests
- external conformance suite

Final pre-merge validation must be repeated after README／CHANGELOG／status／PR synchronization.

## Known limitations

- Complete external delegation／revocation registry evaluation is not included.
- Multi-node queue coordination and distributed snapshot locking are not included.
- Durable cursor lease／read-guard storage remains deployment-specific.
- CI formats the checkout with `cargo fmt --all` before Rust validation.
- Test-target Clippy is compile verification and does not deny warnings.

## Remaining before merge

1. Add the v0.6.0 CHANGELOG entry.
2. Synchronize the release checklist and PR #98 body.
3. Confirm final candidate CI green.
4. Move PR #98 to review／merge-ready state.

After merge: confirm main CI, publish annotated tag `v0.6.0`, publish the GitHub Release, and close Issue #97.
