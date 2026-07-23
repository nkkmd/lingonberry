# Operator CLI Contract

**Status: v1.0.0 pre-release qualification contract**  
**Reference platform: Ubuntu Server 24.04 LTS / x86_64 / systemd / ext4**

## 1. Scope and release boundary

This document defines the operator-facing command, output, mutation, and exit-code contract for the Lingonberry single-node v1.x operational surface.

The latest published release is `v0.9.0`. `v1.0.0` is still under qualification and has not been published. This contract does not indicate that the formal 72-hour soak, version update, `v1.0.0` tag, or GitHub Release is complete.

The designated pre-version qualification candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

The primary operator binaries covered here are:

```text
lingonberry-storage
lingonberry-storage-migrate
lingonberry-relay
```

Quarantine administration, replacement, cleanup, and other proof-bound maintenance surfaces remain governed by their dedicated runbooks and binaries.

## 2. General rules

- Successful machine-readable output is written to standard output.
- Operator-facing errors are written to standard error.
- A non-zero exit code is authoritative even when diagnostic JSON was written first.
- Canonical storage is authoritative. Indexes and effective views are derived and rebuildable.
- Read-only inspection must not be replaced by manual edits to manifests, journals, generation pointers, indexes, proof files, or evidence files.
- Ordinary startup must not perform an implicit storage migration.
- Commands that mutate storage require an explicit operator action and the preconditions documented in the operator or upgrade runbook.

## 3. `lingonberry-storage`

### 3.1 Global configuration options

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

Empty values, unknown configuration fields, non-string path values, missing option values, unknown global options, and an explicitly selected missing configuration file are errors.

### 3.2 Capability and inspection commands

```text
capabilities
config
status
doctor
verify
health
ready
metrics
run
```

Mutation classification:

| Command | Contract |
|---|---|
| `capabilities` | Read-only capability manifest |
| `config` | Read-only effective configuration and path report; secrets excluded |
| `status` | Read-only bounded operator status |
| `doctor` | Read-only diagnostics; warnings may still return success |
| `verify` | Read-only strict diagnostics; warnings or failures return non-zero |
| `health` | Read-only process-level health |
| `ready` | Read-only storage-aware readiness; not-ready returns non-zero |
| `metrics` | Read-only fixed-key, bounded-cardinality metrics |
| `run` | Read-only runtime configuration/status report; it is not a daemon lifecycle command |

### 3.3 Canonical storage commands

```text
append JSON_FILE
retrieve CANONICAL_ID
replay
list
```

- `append` is mutating and validates the publish request and knowledge object before writing.
- `retrieve`, `replay`, and `list` are read-only.
- A missing object is a non-zero not-found result.
- Duplicate-safe append behavior is reported in output rather than treated as an error.

### 3.4 Backup, restore, and drill commands

```text
backup create [ARCHIVE_DIR]
backup verify ARCHIVE_DIR
restore plan ARCHIVE_DIR TARGET_DIR
restore apply ARCHIVE_DIR TARGET_DIR
drill restore ARCHIVE_DIR
```

Mutation classification:

| Command | Contract |
|---|---|
| `backup create` | Mutates only the requested backup destination |
| `backup verify` | Does not mutate active storage; may use and clean an isolated temporary target |
| `restore plan` | Read-only with respect to the requested target |
| `restore apply` | Mutates only an explicit isolated target |
| `drill restore` | Uses isolated temporary state and must clean it after completion or interruption |

Safety requirements:

- symbolic-link archive and target paths are refused;
- restore targets must be isolated from active state and data directories;
- `restore apply` requires a missing or empty target directory;
- restore verification reads every restored record and verifies derived index state;
- restore or drill completion does not authorize switching the active data path.

### 3.5 Index commands

```text
index verify
index rebuild
```

- `index verify` is read-only.
- `index rebuild` mutates derived index state only.
- Neither command is a repair mechanism for corrupt canonical storage.

## 4. `lingonberry-storage-migrate`

The migration surface is separate from ordinary storage operation.

```text
inspect
plan
backup
apply
verify
commit
resume
rollback
status
```

The normal migration sequence is:

```text
inspect
→ plan
→ backup
→ apply
→ verify
→ commit
```

Mutation classification:

| Command | Contract |
|---|---|
| `inspect` | Read-only storage-format inspection |
| `plan` | Creates or validates durable migration planning state as implemented by the migration contract |
| `backup` | Creates and verifies migration backup evidence |
| `apply` | Mutates migration-controlled storage state |
| `verify` | Read-only verification of the applied migration and source binding |
| `commit` | Commits the verified format transition |
| `status` | Read-only durable journal report |
| `resume` | Continues an accepted durable journal state |
| `rollback` | Performs only the rollback accepted by the current journal stage |

Migration output is currently a stable operator-oriented `key=value` line, not canonical JSON. Automation must evaluate both the command-specific fields and the exit code. Do not parse debug formatting beyond fields explicitly documented by the migration and upgrade contract.

## 5. `lingonberry-relay`

The release binary is `lingonberry-relay`. Its Cargo entrypoint routes `publish` and `serve-http` to the HTTP/publish surface and routes other commands through the established classified/index command chain.

Operator-relevant command families include:

```text
publish JSON_FILE
serve-http [ADDR]
capabilities
ready
get CANONICAL_ID
raw CANONICAL_ID
list
subscribe [TYPE]
replay
rebuild-index
catch-up-index
export-archive ARCHIVE_DIR
import-archive ARCHIVE_DIR
```

Additional graph and quarantine commands remain documented by their dedicated contracts and runbooks.

- `publish`, `import-archive`, `rebuild-index`, and `catch-up-index` are mutating.
- `serve-http` is long-running and must normally be managed through the checked-in systemd service contract.
- `ready`, `capabilities`, `get`, `raw`, `list`, `subscribe`, `replay`, and archive export are inspection/read surfaces except for temporary output files explicitly requested by the operator.
- Quarantine promotion, replacement, cleanup, and irreversible decisions require their dedicated authorization and evidence procedures.

## 6. Exit-code contract

Lingonberry uses command-specific non-zero codes. Automation must not collapse all failures into a documented `1` or `2` model.

### 6.1 Shared meanings

| Code | Meaning |
|---:|---|
| `0` | Command completed and its required invariant is satisfied |
| `1` | Unclassified operational failure |
| `64` | Invocation or usage error |
| `65` | Validation, corrupt-state, unknown-newer, or refused-safety condition, depending on the binary |
| `66` | Requested object, record, journal, or path was not found |
| `69` | Storage doctor, strict verify, or readiness invariant failed |
| `70` | Classified internal/proof/index failure, commonly represented by an `LB_*` code |
| `78` | Configuration or service-bind failure |

### 6.2 Binary-specific notes

`lingonberry-storage` may return `64`, `65`, `66`, `69`, `70`, `78`, or `1`.

`lingonberry-storage-migrate` currently returns:

```text
64  usage error
65  refused, corrupt, or unknown-newer state
66  required state or journal not found
1   other operational failure
```

`lingonberry-relay` command paths may return `64`, `65`, `66`, `70`, `78`, or `1`. Index rebuild and catch-up commands explicitly use `70` when their consistency invariant fails.

The exact non-zero code and standard-error message together form the failure classification. Scripts should match documented codes first and use message text only for additional diagnostics.

## 7. Output contract

### 7.1 Canonical JSON surfaces

`lingonberry-storage` successful command output is one canonical JSON object per invocation. Relay commands intended for automation also emit canonical JSON unless a dedicated command contract states otherwise.

Required conventions:

- field names use lower camel case;
- status values are stable, bounded strings;
- paths are strings or `null` where absence is meaningful;
- counts are non-negative JSON numbers;
- booleans describe verified outcomes, not requested intent;
- secrets must not appear in output or errors;
- compatible releases may add optional fields;
- existing fields must not change meaning within the v1 compatibility contract.

Examples of status values used by operator surfaces include:

```text
ok
warning
failed
ready
not_ready
verified
planned
restored
passed
consistent
inconsistent
deferred
promoted
rejected
```

### 7.2 Migration text surface

`lingonberry-storage-migrate` emits one `key=value` status line on success. This is intentionally distinct from canonical JSON and must not be described as JSON output.

### 7.3 Standard error

Errors and invariant failures are written to standard error. Secret configuration values must not be included. Some strict inspection commands print diagnostic JSON to standard output before returning a non-zero exit code; callers must always inspect the exit status.

## 8. Compatibility and change control

A change to any of the following is operator-facing and requires compatibility review, documentation update, and tests:

- command name or argument order;
- read-only versus mutating classification;
- exit-code mapping;
- output format or required field meaning;
- configuration precedence or environment variable name;
- implicit migration behavior;
- safety refusal condition.

New optional JSON fields may be added compatibly. Removing fields, changing field meaning, changing a successful command to mutate active state, or reassigning an established exit code requires an explicit compatibility decision.

## 9. Related documents

- [v1.0 Single-Node Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Systemd Service Contract](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Operations Index](./README.md)
