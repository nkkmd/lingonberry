# Storage node local quickstart

**Status: v1.0.0 pre-release local-evaluation guide** | **Last updated: 2026-07-23**

English is normative for this document.

## 1. Purpose and boundary

This guide provides a minimal local-development path from a repository checkout to a verified `lingonberry-storage` command environment.

It is not the production installation procedure. For an installed reference node, service-account setup, protected environment files, checked-in systemd units, backup, upgrade, rollback, and release evidence, use [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md).

The current storage binary is an operator and storage command binary. It is not a long-running daemon:

- `lingonberry-storage run` prints a runtime snapshot and exits;
- `lingonberry-storage ready` evaluates startup readiness and exits;
- `lingonberry-relay` is the long-running HTTP relay process;
- the checked-in storage systemd unit is a oneshot readiness gate.

## 2. Requirements

Use a supported Linux development environment with:

- Git;
- a current stable Rust toolchain;
- Cargo;
- the build dependencies required by the Rust workspace.

Confirm the tools are available:

```bash
 git --version
 rustc --version
 cargo --version
```

The formal reference platform is defined in [`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md).

## 3. Clone the repository

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

For release qualification, do not assume that `main` is the release candidate. Use the exact revision named by the qualification documentation. This quickstart uses the current checkout only for local evaluation.

## 4. Build and test the storage package

```bash
cargo build -p lingonberry-storage
cargo test -p lingonberry-storage
```

For a release-mode local binary:

```bash
cargo build --release -p lingonberry-storage
```

The development examples below use `cargo run`. An installed-node procedure must use reviewed release binaries rather than an implicit rebuild from an arbitrary checkout.

## 5. Create isolated local directories

Use a disposable directory tree so the quickstart cannot modify an existing node:

```bash
export LB_STORAGE_ROOT="$(mktemp -d)"
export LINGONBERRY_STORAGE_STATE_DIR="$LB_STORAGE_ROOT/state"
export LINGONBERRY_STORAGE_DATA_DIR="$LB_STORAGE_ROOT/data"
export LINGONBERRY_STORAGE_BACKUP_DIR="$LB_STORAGE_ROOT/backups"
export LINGONBERRY_STORAGE_TEMP_DIR="$LB_STORAGE_ROOT/tmp"

mkdir -p \
  "$LINGONBERRY_STORAGE_STATE_DIR" \
  "$LINGONBERRY_STORAGE_DATA_DIR" \
  "$LINGONBERRY_STORAGE_BACKUP_DIR" \
  "$LINGONBERRY_STORAGE_TEMP_DIR"
```

The storage-specific variables are preferred for this guide because they make every directory explicit. Configuration precedence is:

```text
defaults
→ config file
→ environment variables
→ CLI options
```

See [`STORAGE_NODE_RUNTIME.md`](./STORAGE_NODE_RUNTIME.md) for the full contract.

## 6. Inspect the command surface

```bash
cargo run -p lingonberry-storage -- capabilities
```

The output is canonical JSON and should identify the storage service and implemented operations.

## 7. Inspect effective configuration

```bash
cargo run -p lingonberry-storage -- config
```

Confirm that the resolved values point under `$LB_STORAGE_ROOT`:

```text
stateDir
dataDir
backupDir
tempDir
rawLogPath
catalogPath
```

`rawLogPath` and `catalogPath` are derived from `dataDir`:

```text
<dataDir>/relay-wire-log.jsonl
<dataDir>/canonical-catalog.sqlite3
```

You can also pass one-off CLI overrides before the command:

```bash
cargo run -p lingonberry-storage -- \
  --data-dir "$LB_STORAGE_ROOT/alternate-data" \
  config
```

CLI overrides have higher precedence than environment variables.

## 8. Understand `status` and `run`

```bash
cargo run -p lingonberry-storage -- status
cargo run -p lingonberry-storage -- run
```

Both commands print snapshots and exit. Neither command starts a resident service, listens on a network port, or proves strict storage integrity.

Do not keep a terminal open expecting `run` to remain active.

## 9. Run diagnostics

### 9.1 Process-level health

```bash
cargo run -p lingonberry-storage -- health
```

`health` confirms that the command process can run. It does not inspect storage readiness.

### 9.2 Read-only doctor report

```bash
cargo run -p lingonberry-storage -- doctor
```

On a newly created empty workspace, warnings are expected for items such as an empty storage format, missing log or catalog files, or an empty backup inventory. Warnings do not make `doctor` fail.

Failed checks must not be ignored. They indicate conditions such as corrupt or unsupported storage, invalid file types, symlink rejection, inconsistent index state, or failed capacity checks.

### 9.3 Startup readiness

```bash
cargo run -p lingonberry-storage -- ready
```

`ready` fails only when the doctor report contains a failed check. A warning-only empty local workspace may therefore be ready.

This is the command used by the checked-in oneshot unit:

```text
deploy/systemd/lingonberry-storage-ready.service
```

### 9.4 Strict verification

```bash
cargo run -p lingonberry-storage -- verify
```

`verify` treats warnings as non-zero. A pristine empty workspace may not pass strict verification until the storage state and required operational artifacts have been initialized.

Use this distinction deliberately:

```text
doctor = warnings allowed
ready  = failed checks rejected
verify = warnings and failed checks rejected
```

## 10. Exercise read-only storage commands

```bash
cargo run -p lingonberry-storage -- list
cargo run -p lingonberry-storage -- replay
```

On an empty workspace, these commands should return canonical JSON representing an empty result rather than starting a service.

`retrieve` requires a canonical ID:

```bash
cargo run -p lingonberry-storage -- retrieve 'lb:obj:example'
```

A missing object is an expected not-found failure, not proof of runtime corruption.

## 11. Optional append smoke test

`append` is mutating and requires a valid publish-request JSON file. Use a reviewed fixture or create an isolated test request that conforms to the current protocol contract.

Example with a checked-in fixture when appropriate for the current checkout:

```bash
cargo run -p lingonberry-storage -- \
  append fixtures/http-publish-request/minimal-request.json
```

Then inspect the result:

```bash
cargo run -p lingonberry-storage -- list
cargo run -p lingonberry-storage -- replay
cargo run -p lingonberry-storage -- doctor
```

Do not run `append`, restore, migration, index rebuild, or drill commands against an existing production directory from this quickstart.

## 12. Metrics snapshot

```bash
cargo run -p lingonberry-storage -- metrics
```

This emits a bounded-cardinality command snapshot. It is not a continuously served metrics endpoint.

## 13. Cleanup

After local evaluation, remove only the disposable directory created by this guide:

```bash
printf 'quickstart root: %s\n' "$LB_STORAGE_ROOT"
rm -rf -- "$LB_STORAGE_ROOT"
unset LB_STORAGE_ROOT
unset LINGONBERRY_STORAGE_STATE_DIR
unset LINGONBERRY_STORAGE_DATA_DIR
unset LINGONBERRY_STORAGE_BACKUP_DIR
unset LINGONBERRY_STORAGE_TEMP_DIR
```

Before running `rm -rf`, inspect the printed path and confirm that it is the temporary quickstart root. Never substitute an installed-node path such as `/var/lib/lingonberry` or `/var/backups/lingonberry`.

## 14. Moving to an installed node

For a reference installed node, stop using `cargo run` and follow the operator runbook. The installed model uses:

- release-mode binaries under `/usr/local/bin`;
- service account `lingonberry:lingonberry`;
- protected environment file `/etc/lingonberry/storage.env`;
- durable paths under `/var/lib/lingonberry` and `/var/backups/lingonberry`;
- checked-in systemd unit templates;
- explicit backup, migration, verification, and rollback procedures.

Do not expose `lingonberry-storage` directly to the network. Public HTTP service belongs to the relay and its reviewed reverse-proxy boundary.

## 15. Release boundary

This guide describes the current v1.0.0 pre-release implementation. It does not indicate that v1.0.0 has been published, that the formal 72-hour soak has completed, or that privileged reference-host qualification has completed.

The designated pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Documentation and evidence commits after that candidate do not redefine it.

## References

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Storage Node Runtime Contract](./STORAGE_NODE_RUNTIME.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
