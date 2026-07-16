# Lingonberry

Lingonberry is a Rust workspace for publishing, validating, storing, indexing, and operating canonical knowledge objects. It includes a persistent quarantine lifecycle and a verified replacement-generation workflow designed around fail-closed state transitions and durable evidence.

## v0.4.0 release candidate

v0.4.0 adds an operator-controlled lifecycle for verified cleanup of inactive quarantine replacement generations:

- deterministic retention-policy evaluation with a retained-generation floor;
- durable terminal completion evidence bound to replacement journals and generation digests;
- canonical cleanup plan/proof artifacts with digest sidecars;
- read-only reconstruction and stale-proof verification across pointers, journals, generations, evidence, and managed paths;
- dedicated cleanup transactions with same-filesystem tomb preparation, sealed inventories, path-level durable progress, resume, rollback before the irreversible boundary, and explicit partial-deletion classification;
- double opt-in authorization with no scheduled or unattended cleanup;
- retained terminal cleanup workspaces for audit and recovery evidence.

The `v0.4.0` tag and GitHub Release are not published until the merged main-branch CI result is confirmed.

## Safety boundaries

Lingonberry treats ambiguous or contradictory state as an error. In particular:

- active, incomplete, orphan, corrupt, legacy-root, unverified, or insufficiently aged subjects are not cleanup-eligible;
- wildcard and implicit-all cleanup selection are rejected;
- filesystem timestamps are not authoritative retention evidence;
- symbolic links, unsupported entry types, stale proofs, partial artifact pairs, and digest mismatches fail closed;
- cleanup never rewrites archive segments or immutable evidence ledgers;
- rollback is available only before irreversible processing begins;
- same-host locking is not a distributed lock;
- secure erase semantics are not promised.

## Workspace

```text
packages/protocol     canonical protocol model
packages/identity     identity primitives
packages/validation   validation rules
packages/core         quarantine, replacement, retention, and cleanup logic
packages/indexer      verified indexing
packages/relay        relay and operator binaries
packages/storage      storage runtime
```

## Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables
cargo test --workspace
```

JavaScript contract tests are also run by `.github/workflows/ci.yml`.

## Documentation

- [Current implementation status](docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
- [v0.4.0 roadmap](docs/roadmap/RELEASE_0_4_0_ROADMAP.md)
- [v0.4.0 release checklist](docs/roadmap/RELEASE_0_4_0_CHECKLIST.md)
- [v0.4.0 release notes](docs/roadmap/RELEASE_0_4_0_RELEASE_NOTE.md)
- [Retention policy](docs/operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md)
- [Cleanup operations runbook](docs/operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
- [Operations index](docs/operations/README.md)
- [Changelog](CHANGELOG.md)

## Release history

- v0.3.0: verified replacement-generation transaction and recovery
- v0.2.0: persistent quarantine lifecycle, backup/restore, maintenance, and RBAC
- v0.1.0: initial protocol, schema, fixtures, and carrier contracts

## License

See the package metadata and repository license files for applicable terms.
