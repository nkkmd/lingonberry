# Operator CLI Contract

**Status: v0.8.0 release contract** | **Reference platform: Ubuntu Server 24.04 LTS x86_64**

## Purpose

This document defines the operator-facing command, exit-code, and output contract for the v0.8.0 single-node release. The contract covers `lingonberry-storage` and explicitly routes storage migration and quarantine maintenance to their existing dedicated surfaces.

## Global configuration options

Global options must appear before the command:

```text
--config PATH
--state-dir PATH
--data-dir PATH
--backup-dir PATH
--temp-dir PATH
```

Effective precedence is:

```text
defaults < config file < environment < CLI
```

Supported environment variables:

```text
LINGONBERRY_STORAGE_CONFIG
LINGONBERRY_STORAGE_STATE_DIR
LINGONBERRY_STORAGE_DATA_DIR
LINGONBERRY_STORAGE_BACKUP_DIR
LINGONBERRY_STORAGE_TEMP_DIR
```

Empty values, unknown configuration fields, non-string path values, and an explicitly selected missing configuration file are errors.

## Command groups

### Inspection and observability

```text
config
status
doctor
verify
health
ready
metrics
```

- `config` prints the effective paths and precedence without secret material.
- `status` and `doctor` are read-only and may report warnings.
- `verify` is strict and fails when the doctor reports a failed check.
- `health` reports process-level availability.
- `ready` evaluates storage-aware readiness and returns a failure exit code when not ready.
- `metrics` uses fixed keys and bounded-cardinality labels.

### Canonical storage access

```text
append
retrieve
replay
list
run
```

These commands operate on canonical storage through the existing storage backend contract.

### Backup and restore

```text
backup create [ARCHIVE_DIR]
backup verify ARCHIVE_DIR
restore plan ARCHIVE_DIR TARGET_DIR
restore apply ARCHIVE_DIR TARGET_DIR
drill restore ARCHIVE_DIR
```

Safety requirements:

- symbolic-link archive and target paths are refused;
- a restore target must be isolated from active state and data directories;
- `restore apply` requires a missing or empty target directory;
- `restore plan` does not mutate the requested target;
- every created or verified archive is restored into a temporary isolated target;
- isolated verification reads every restored record, verifies the index, re-imports the archive to prove duplicate-safe write behavior, and removes the temporary target;
- interruption during isolated verification must not leave the temporary target behind.

### Index lifecycle

```text
index verify
index rebuild
```

Canonical storage is authoritative. The index is derived state and must be reproducibly verifiable and rebuildable.

## Explicit routing to existing operator surfaces

The v0.8.0 integrated storage command does not duplicate established proof-bound maintenance workflows.

### Storage migration

Use:

```text
lingonberry-storage-migrate inspect
lingonberry-storage-migrate plan
lingonberry-storage-migrate apply
lingonberry-storage-migrate status
lingonberry-storage-migrate resume
lingonberry-storage-migrate rollback
```

Migration remains explicit. Normal startup never performs implicit migration.

### Quarantine, replacement, and cleanup

Use the existing quarantine admin HTTP/RBAC surface and the proof-bound runbooks linked from the Operations index. Replacement and cleanup remain separate operator-authorized workflows because they require preview, proof, durable transaction state, completion evidence, retention evaluation, and a separate irreversible acknowledgement.

## Exit-code contract

```text
0  command completed and its required invariant is satisfied
1  operational or validation failure
2  invocation or configuration error
```

Commands that report degraded or warning state may still return `0` unless their command contract is strict. `verify`, failed readiness, failed index verification, failed archive verification, and failed restore/drill invariants return non-zero.

The command writes machine-readable success output to standard output and operator-facing errors to standard error.

## JSON output contract

Machine-readable commands emit one canonical JSON object per invocation.

Required conventions:

- field names use lower camel case;
- `status` is a stable, bounded string;
- paths are emitted as strings;
- counts are non-negative JSON numbers;
- booleans describe verified invariants, not requested intent;
- errors must not include secret configuration values;
- new optional fields may be added in a compatible release;
- existing fields must not change meaning within the v0.8.x line.

Examples of stable status values include:

```text
ok
warning
failed
verified
planned
restored
passed
consistent
inconsistent
```

## Human-readable output policy

v0.8.0 treats canonical JSON as the authoritative automation contract. Human-readable guidance belongs in errors, the runbook, and systemd journal messages. A future presentation layer may render the canonical JSON, but it must not replace or reinterpret the machine-readable contract.
