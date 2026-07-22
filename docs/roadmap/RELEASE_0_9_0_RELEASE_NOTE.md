# Lingonberry v0.9.0 Release Notes

**Status: release candidate** | **Version: v0.9.0** | **Prepared: 2026-07-22**

## Overview

v0.9.0 is the release-candidate hardening release before Lingonberry v1.0. It intentionally avoids new product features and instead freezes candidate public contracts, closes security findings, bounds parser resource use, hardens signature-verification temporary workspaces, expands regression coverage, and records release evidence.

The formal deployment scope remains a single node on Ubuntu Server 24.04 LTS, x86_64, with systemd. The protocol and durable data contracts remain platform-independent.

## Highlights

### Bounded protocol parsing

- Added a 1 MiB maximum JSON input size at the protocol-library boundary.
- Added a shared maximum nesting depth of 128 for arrays and objects.
- Oversized and excessively nested input now fails closed with a stable `JsonError` instead of entering unbounded parsing or recursion.
- Added regression coverage for exact boundaries, over-limit input, maximum accepted depth, excessive depth, and mixed nesting.
- Retained baseline coverage for malformed input, deterministic parsing, canonical object ordering, canonical round trips, and moderate nesting compatibility.

### Signature-verification workspace hardening

- Replaced timestamp-only workspace creation with exclusive directory creation using process ID, timestamp, and a process-local atomic counter.
- Set owner-only `0o700` workspace permissions on Unix reference platforms.
- Create verification artifacts with create-new semantics so existing files are never followed or overwritten.
- Added RAII cleanup for ordinary success and error return paths.
- Replaced host-path and I/O-detail leakage with generic verification-workspace errors.
- Added tests for cleanup, permissions, artifact collisions, concurrent isolation, and concurrent cleanup.

### Public-contract freeze candidate

- Inventoried exported Rust surfaces across protocol, identity, validation, core, indexer, storage, and relay crates.
- Classified exports as freeze candidates, behavior-frozen surfaces, workspace-internal surfaces, or implementation details.
- Recorded protocol, public API, storage, migration, backup, replacement, cleanup, and diagnostic compatibility boundaries.
- Kept protocol and schema versions at `0.1.0`; v0.9.0 introduces no wire-format breaking change.

### Release evidence and validation

- Closed both v0.9.0 security findings with source and regression-test evidence.
- Passed Rust formatting, library Clippy, binary Clippy, test-target Clippy, and all workspace tests.
- Passed JavaScript tests and the external protocol conformance suite.
- Passed five consecutive bounded-soak iterations covering parser boundaries, signature workspace contracts, and the quarantine replacement crash matrix.
- Versioned all Rust workspace packages and `Cargo.lock` as `0.9.0`.

## Compatibility and upgrade

v0.9.0 does not introduce an implicit storage migration. Existing v0.8.0 installation, service, backup, restore, index, disaster-recovery, upgrade, and rollback procedures remain authoritative.

Before upgrading:

1. create and verify a backup;
2. stop the relay;
3. preserve the v0.8.0 binaries;
4. install v0.9.0 binaries atomically;
5. run `doctor`, `verify`, and readiness checks before normal startup.

The new protocol parser limits are intentional availability boundaries. Inputs larger than 1 MiB or nested beyond depth 128 are rejected. Operators or integrations that previously depended on larger protocol-library inputs must split or reject those inputs rather than bypassing the limit.

## Security disposition

- Open Critical findings: 0
- Open High findings: 0
- Open release-blocking Medium findings: 0
- Closed v0.9.0 findings: 2

The security findings and residual-risk record are maintained in `docs/security/V0_9_SECURITY_FINDINGS.md`.

## Operational boundaries

- Canonical storage remains authoritative; indexes remain derived, verifiable, and rebuildable.
- Ordinary startup does not perform implicit migration or destructive repair.
- Unknown, corrupt, contradictory, or unsupported durable state fails closed.
- Replacement and cleanup remain explicit proof-bound operations.
- Same-host locks are not distributed locks.
- Multi-node coordination and replication remain outside the v0.9.0 scope.

## Validation evidence

Release preparation includes:

- post-hardening standard CI run 1141;
- parser and signature hardening implementation commit `fe23c523f358cfa62aea396ec7481778a0915c2c`;
- signature workspace regression-test commit `1083ab0348881aabba924f102151c5d4ed3da292`;
- v0.9.0 version and bounded-soak commit `e5b308e54c5ed888dd3b162c37e70fb6bfd48c42`;
- release-preparation workflow run `29898586767`.

The canonical evidence ledger is `docs/roadmap/V0_9_RELEASE_EVIDENCE.md`.

## Known limitations and residual risks

- A process crash, `SIGKILL`, kernel termination, or host power loss can prevent Rust `Drop` cleanup and leave a signature verification workspace in the OS temporary directory.
- The five-iteration CI soak is bounded and does not replace long-running production telemetry, disk-pressure injection, or power-loss testing.
- Long-duration reference-host soak and resource telemetry remain part of the v1.0 stable release gate.
- Multi-node deployment, distributed locking, and replication are not included.
