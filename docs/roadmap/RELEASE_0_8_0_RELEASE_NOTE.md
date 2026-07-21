# Lingonberry v0.8.0 Release Notes

**Status: release candidate** | **Target: v0.8.0** | **Date: 2026-07-22**

## Overview

v0.8.0 establishes the first formally validated single-node operator environment for Lingonberry. The release focuses on operational readiness: installation, configuration, diagnostics, backup, isolated restore, index recovery, disaster-recovery drills, systemd deployment, upgrade, and rollback.

The formal reference platform is Ubuntu Server 24.04 LTS on x86_64 with systemd. Other systemd-based Linux distributions remain best-effort support. Storage formats, protocol contracts, and public data models are not Ubuntu-specific.

## Highlights

### Operator diagnostics

- Added `config`, `health`, `ready`, `status`, `doctor`, `verify`, and bounded-cardinality `metrics` surfaces.
- Added stable machine-readable diagnostic codes and `ok` / `warning` / `failed` severity.
- Added read-only checks for configuration, storage format, migration journals, raw log, catalog, generation pointer, index consistency, backup inventory, maintenance workspaces, and disk capacity.
- Added fail-closed handling for symlinks, special files, corrupt state, contradictory generation metadata, and unknown-newer storage formats.
- Fixed configuration precedence as `defaults < config file < environment < CLI`.

### Backup, restore, and disaster recovery

- Added verified `backup create` and `backup verify` operations.
- Added non-mutating `restore plan` and isolated `restore apply`.
- Restore refuses active, non-empty, or symbolic-link targets.
- Restored records are read back and the derived index is rebuilt and verified before success is reported.
- Added `index verify` and `index rebuild`.
- Added an isolated restore drill with read verification, duplicate-safe write-path verification, index verification, and mandatory cleanup.
- Added failure-injection coverage proving interrupted isolated restore does not leave partial state.

### Linux operations

- Added hardened systemd units for the storage readiness gate and relay service.
- Added environment-file examples, non-root service ownership, and filesystem layout guidance.
- Added an Ubuntu 24.04 operator runbook and supported-platform contract.
- Added v0.7.0 to v0.8.0 upgrade and rollback procedures.
- Added the operator CLI, exit-code, and canonical JSON output contract.

### Automated acceptance

The Ubuntu 24.04 fresh-runner workflow:

- builds release binaries;
- installs them into `/usr/local/bin`;
- verifies systemd units;
- uses installed binaries rather than `cargo run`;
- verifies persistence across separate process invocations;
- exercises backup, restore, index, and DR paths;
- checks partial archive, active target, and non-empty target failures.

## Compatibility and upgrade

v0.8.0 does not introduce an implicit storage migration. Operators must continue to use the explicit migration workflow when storage inspection indicates migration is required.

Before upgrading:

1. create and verify a backup;
2. stop the relay;
3. preserve the v0.7.0 binaries;
4. install v0.8.0 binaries atomically;
5. run `doctor` and `verify` before normal service startup.

Automatic downgrade of a committed incompatible storage format is prohibited. Rollback uses either binary-only rollback when the storage format remains compatible or an isolated restore from a compatible verified backup.

See `docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md` for the canonical procedure.

## Operational boundaries

Quarantine inspection remains on the existing admin HTTP/RBAC surface. Replacement and cleanup remain explicit, proof-bound operations governed by their existing runbooks and core verifiers. v0.8.0 does not add directory-name-based discovery or implicit repair of replacement or cleanup evidence.

Operators must not manually edit pointers, journals, manifests, proof files, inventory files, completion evidence, or cleanup evidence.

## Validation evidence

Release-candidate validation includes:

- Rust formatting, Clippy, and workspace tests;
- JavaScript tests and the external conformance suite;
- Ubuntu Server 24.04 LTS x86_64 fresh-runner acceptance;
- release binary installation and systemd unit verification;
- installed-binary operator acceptance;
- persistence, fail-closed restore fixtures, and disaster-recovery verification.

## Known limitations

- Other Linux distributions are not part of the formal release gate.
- Quarantine replacement and cleanup evidence are verified through their explicit operation-specific APIs and runbooks rather than automatically discovered by the general storage doctor.
- Multi-node deployment remains outside the v0.8.0 single-node operational-readiness scope.
