# Storage node runtime contract

**Status: v1.0.0 pre-release implementation contract** | **Last updated: 2026-07-23**

English is normative for this document.

## 1. Purpose and scope

This document defines the implemented runtime configuration, directory layout, diagnostic commands, and systemd readiness boundary of the `lingonberry-storage` binary.

It does not define a long-running storage daemon. In the current v1.0.0 pre-release design:

- `lingonberry-storage` is an operator and storage command binary;
- `lingonberry-storage run` prints a read-only runtime snapshot and exits;
- `lingonberry-storage ready` is used by the checked-in oneshot systemd readiness gate;
- `lingonberry-relay` is the long-running HTTP relay process.

For the complete single-node procedure, use [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md). For CLI output and exit-code rules, use [`OPERATOR_CLI_CONTRACT.md`](./OPERATOR_CLI_CONTRACT.md).

## 2. Binary and command surface

The binary name is:

```text
lingonberry-storage
```

The implemented top-level commands are:

```text
capabilities
config
status
doctor
verify
health
ready
metrics
backup
restore
index
drill
run
append
retrieve
replay
list
```

Command classes:

- configuration and diagnostics: `capabilities`, `config`, `status`, `doctor`, `verify`, `health`, `ready`, `metrics`, `run`;
- recovery and maintenance: `backup`, `restore`, `index`, `drill`;
- storage data operations: `append`, `retrieve`, `replay`, `list`.

`run` does not remain resident and does not provide an HTTP listener. It prints the resolved runtime state and exits.

## 3. Configuration precedence

Storage configuration is resolved in this order, from lowest to highest precedence:

```text
defaults
→ config file
→ environment variables
→ CLI options
```

The available global CLI overrides are:

```text
--config PATH
--state-dir PATH
--data-dir PATH
--backup-dir PATH
--temp-dir PATH
```

An empty CLI or environment path is rejected.

## 4. Configuration file selection

The configuration path is selected in this order:

1. CLI `--config PATH`;
2. `LINGONBERRY_STORAGE_CONFIG`;
3. `<default-state-dir>/storage-config.json`.

The default state directory is resolved by the shared core runtime helper. For a normal installed node, set the storage-specific directory variables explicitly in `/etc/lingonberry/storage.env` rather than depending on a working-directory-relative default.

If an explicit CLI or environment configuration path does not exist, configuration resolution fails. If only the implicit default path is absent, the built-in defaults remain valid.

## 5. Configuration file format

The configuration file must be a JSON object. The only accepted fields are:

```text
stateDir
dataDir
backupDir
tempDir
```

Unknown fields, non-string values, and empty strings fail closed.

Example:

```json
{
  "stateDir": "/var/lib/lingonberry/storage",
  "dataDir": "/var/lib/lingonberry/storage/data",
  "backupDir": "/var/backups/lingonberry/storage",
  "tempDir": "/var/lib/lingonberry/storage/tmp"
}
```

## 6. Environment variables

The implemented storage variables are:

```text
LINGONBERRY_STORAGE_CONFIG
LINGONBERRY_STORAGE_STATE_DIR
LINGONBERRY_STORAGE_DATA_DIR
LINGONBERRY_STORAGE_BACKUP_DIR
LINGONBERRY_STORAGE_TEMP_DIR
```

`LINGONBERRY_STATE_DIR` may still affect the shared default state root through the core runtime helper, but it is not a substitute for the storage-specific directory variables in the reference installed-node configuration.

Recommended reference-node values:

```bash
LINGONBERRY_STORAGE_STATE_DIR=/var/lib/lingonberry/storage
LINGONBERRY_STORAGE_DATA_DIR=/var/lib/lingonberry/storage/data
LINGONBERRY_STORAGE_BACKUP_DIR=/var/backups/lingonberry/storage
LINGONBERRY_STORAGE_TEMP_DIR=/var/lib/lingonberry/storage/tmp
```

Keep `/etc/lingonberry/storage.env` readable only by the operator or service account as required by the secret-management policy. Directory paths themselves are not secrets, but the protected environment file is the stable deployment interface.

## 7. Derived defaults

After all configuration layers are applied:

- an unassigned `dataDir` defaults to the resolved `stateDir`;
- an unassigned `backupDir` defaults to `<stateDir>/backup`;
- an unassigned `tempDir` defaults to `<stateDir>/tmp`.

A higher-precedence assignment is preserved. For example, changing `stateDir` does not replace an explicitly assigned `dataDir`.

## 8. Runtime layout

The implemented durable paths are derived from `dataDir`:

```text
rawLogPath = <dataDir>/relay-wire-log.jsonl
catalogPath = <dataDir>/canonical-catalog.sqlite3
```

Additional format and migration files may also exist in `dataDir`:

```text
storage-format.manifest
storage-migration.journal
```

Responsibilities:

- `stateDir`: storage runtime state and quarantine-generation root;
- `dataDir`: canonical durable storage, wire log, catalog, format manifest, and migration journal;
- `backupDir`: backup inventory and verified recovery artifacts;
- `tempDir`: bounded operational workspace for restore, rebuild, and maintenance work.

Do not place the active canonical storage on a network filesystem for the reference platform. See [`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md).

## 9. Configuration and runtime output

`config`, `status`, and `run` emit canonical JSON describing the effective runtime configuration and derived layout. Fields include the resolved equivalents of:

```text
configPath
stateDir
dataDir
backupDir
tempDir
rawLogPath
catalogPath
```

`status` and `run` are snapshots. They do not prove that all filesystem, index, backup, or disk-capacity checks pass.

## 10. Diagnostic semantics

### 10.1 `health`

`health` reports process-level command health. It does not inspect storage readiness and does not replace `ready`.

### 10.2 `doctor`

`doctor` is read-only and evaluates the effective configuration and operational storage state. Implemented checks include:

- required paths are non-empty;
- state, data, backup, and temporary directories;
- symlink and file-type rejection;
- storage format classification;
- migration-journal presence and validity boundary;
- raw log and catalog file types;
- quarantine generation pointer;
- index consistency;
- backup inventory;
- operational workspace;
- disk capacity.

Warnings do not fail `doctor`; failed checks do.

### 10.3 `verify`

`verify` runs the same diagnostic report in strict mode. Both warnings and failed checks cause a non-zero result. Use it before an upgrade, after restore, and before declaring an operator procedure complete.

### 10.4 `ready`

`ready` runs the doctor checks and reports:

```text
status: ready | not_ready
ready: true | false
diagnosticStatus: ok | warning | failed
```

Readiness fails only when the diagnostic report contains a failed check. A warning-only report can be ready, while `verify` remains non-zero. This distinction is deliberate.

### 10.5 `metrics`

`metrics` emits bounded-cardinality diagnostic counts, including readiness and doctor-check totals. It is a command snapshot, not a continuously exposed metrics endpoint.

## 11. systemd readiness gate

The checked-in unit is:

```text
deploy/systemd/lingonberry-storage-ready.service
```

Its implemented model is:

- `Type=oneshot`;
- service account `lingonberry:lingonberry`;
- optional environment file `/etc/lingonberry/storage.env`;
- `ExecStart=/usr/local/bin/lingonberry-storage ready`;
- `RemainAfterExit=yes`;
- hardened filesystem and privilege settings;
- write access limited to `/var/lib/lingonberry` and `/var/backups/lingonberry`.

This unit is a startup gate, not a storage daemon. The relay unit requires and starts after this gate. A failed readiness check prevents the reference relay service from starting normally.

The checked-in files under `deploy/systemd/` are the service-template source of truth. See [`SYSTEMD_UNIT_TEMPLATES.md`](./SYSTEMD_UNIT_TEMPLATES.md).

## 12. Backup, restore, and migration boundaries

- ordinary backups use the storage recovery command surface documented in the operator runbook;
- storage-format migration uses `lingonberry-storage-migrate` and [`STORAGE_MIGRATION_AND_UPGRADE.md`](./STORAGE_MIGRATION_AND_UPGRADE.md);
- pre-commit migration rollback is not equivalent to restoring a release backup;
- release rollback after committed or incompatible storage change follows [`V1_0_UPGRADE_AND_ROLLBACK.md`](./V1_0_UPGRADE_AND_ROLLBACK.md);
- restore completion requires application-level read, index, readiness, and persistence checks; a successful file copy alone is insufficient.

## 13. Fail-closed rules

Normal operation must stop for operator review when:

- an explicit configuration file is missing;
- configuration contains an unknown field, wrong type, or empty path;
- a required path is an unsupported file type or symlink;
- storage format is corrupt or newer than supported;
- the index is inconsistent;
- readiness reports a failed diagnostic check;
- a migration journal is present and its state has not been inspected;
- disk or workspace checks report a failure.

Do not bypass these conditions by deleting manifests or journals, weakening systemd hardening, or silently changing directory roots.

## 14. Release boundary

This document describes the current v1.0.0 pre-release implementation. It does not indicate that v1.0.0 has been published or that formal qualification is complete.

The designated pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal reference-host qualification, the 72-hour soak, version update, tag, and GitHub Release remain separate release gates.

## 15. Related documents

- [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md)
- [`OPERATOR_CLI_CONTRACT.md`](./OPERATOR_CLI_CONTRACT.md)
- [`STORAGE_MIGRATION_AND_UPGRADE.md`](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [`V1_0_UPGRADE_AND_ROLLBACK.md`](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [`SYSTEMD_UNIT_TEMPLATES.md`](./SYSTEMD_UNIT_TEMPLATES.md)
- [`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md)
- [`SECRET_MANAGEMENT.md`](./SECRET_MANAGEMENT.md)
