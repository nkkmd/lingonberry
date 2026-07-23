# Relay and storage responsibility separation

**Status: v1.0.0 pre-release implementation contract** | **Last updated: 2026-07-23**

English is normative for this document.

## 1. Purpose

This document defines the implemented separation between `lingonberry-relay` and `lingonberry-storage` for the v1.0.0 pre-release reference node.

The separation is primarily a separation of binaries, responsibilities, configuration surfaces, and lifecycle operations. It is not a claim that both binaries run as independent long-lived services.

In the current design:

- `lingonberry-relay` is the long-running HTTP relay process;
- `lingonberry-storage` is an operator and storage command binary;
- `lingonberry-storage ready` is executed by a oneshot systemd readiness gate;
- the relay service starts only after the storage readiness gate succeeds.

For the complete installed-node procedure, use [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md). For storage configuration and diagnostics, use [`STORAGE_NODE_RUNTIME.md`](./STORAGE_NODE_RUNTIME.md).

## 2. Binary responsibilities

### 2.1 `lingonberry-relay`

The relay binary owns the long-running network-facing process. Its responsibilities include:

- binding the configured relay listen address;
- serving the implemented HTTP surface;
- accepting and validating relay operations;
- using the configured durable state required by the relay command surface;
- exposing relay process health and readiness behavior;
- terminating cleanly on `SIGTERM` under systemd.

The checked-in reference unit starts it with:

```text
/usr/local/bin/lingonberry-relay serve-http ${LINGONBERRY_RELAY_LISTEN}
```

### 2.2 `lingonberry-storage`

The storage binary owns operator-facing storage inspection, diagnostics, maintenance, recovery, and direct storage commands. Its responsibilities include:

- resolving storage configuration;
- reporting capabilities and runtime layout;
- running `health`, `doctor`, `verify`, `ready`, and `metrics`;
- backup, restore, index, and drill operations;
- append, retrieve, replay, and list operations;
- enforcing storage-format and migration safety boundaries.

`lingonberry-storage run` prints a runtime snapshot and exits. It does not start a resident storage daemon and does not provide a network listener.

### 2.3 Migration binary

Storage-format migration is additionally separated into:

```text
lingonberry-storage-migrate
```

Migration is explicit and operator-driven. It is not performed implicitly by starting the relay or by running the storage readiness gate. See [`STORAGE_MIGRATION_AND_UPGRADE.md`](./STORAGE_MIGRATION_AND_UPGRADE.md).

## 3. Reference systemd topology

The checked-in systemd source of truth is:

```text
deploy/systemd/lingonberry-storage-ready.service
deploy/systemd/lingonberry-relay.service
```

The startup topology is:

```text
local filesystem
→ lingonberry-storage-ready.service
→ lingonberry-relay.service
```

The storage readiness unit is:

- `Type=oneshot`;
- executed as `lingonberry:lingonberry`;
- configured through `/etc/lingonberry/storage.env`;
- started with `/usr/local/bin/lingonberry-storage ready`;
- retained as active after successful completion with `RemainAfterExit=yes`.

The relay unit is:

- `Type=simple`;
- executed as `lingonberry:lingonberry`;
- configured through `/etc/lingonberry/relay.env`;
- started with `lingonberry-relay serve-http`;
- restarted on failure;
- stopped with `SIGTERM` and a bounded stop timeout.

The relay unit declares both:

```ini
Requires=lingonberry-storage-ready.service
After=network-online.target lingonberry-storage-ready.service
```

A failed storage readiness check therefore prevents the reference relay service from starting normally.

## 4. Separation does not mean unrelated storage

The two binaries have separate command and configuration surfaces, but the reference node is one operational system.

The reference deployment intentionally uses:

- the same Unix service account and group;
- coordinated filesystem ownership;
- a storage readiness dependency before relay startup;
- the same release qualification and rollback boundary.

Do not interpret binary separation as permission to point the relay and storage commands at unrelated durable state while treating them as one node.

The operator must verify that the effective relay and storage configuration describe the intended node and that the storage command surface inspects the same durable state the relay is expected to use.

## 5. Configuration ownership

### 5.1 Storage configuration

Storage configuration is provided through the storage-specific interface:

```text
/etc/lingonberry/storage.env
LINGONBERRY_STORAGE_CONFIG
LINGONBERRY_STORAGE_STATE_DIR
LINGONBERRY_STORAGE_DATA_DIR
LINGONBERRY_STORAGE_BACKUP_DIR
LINGONBERRY_STORAGE_TEMP_DIR
```

The reference paths are under:

```text
/var/lib/lingonberry/storage
/var/backups/lingonberry/storage
```

### 5.2 Relay configuration

Relay process configuration is provided through:

```text
/etc/lingonberry/relay.env
```

The checked-in unit requires at least the implemented listen-address variable used by its `ExecStart` command:

```text
LINGONBERRY_RELAY_LISTEN
```

Do not place storage migration decisions, backup destinations, or restore state into the relay unit command line.

### 5.3 Shared generic defaults

Shared runtime helpers may still recognize generic defaults such as `LINGONBERRY_STATE_DIR`, but the reference installed-node configuration should use the documented storage-specific variables and reviewed relay environment file.

Do not depend on the systemd working directory or an implicit relative `.lingonberry` directory for a production node.

## 6. Filesystem ownership and write boundaries

Both reference units use:

```text
User=lingonberry
Group=lingonberry
```

The storage readiness unit is permitted to write under:

```text
/var/lib/lingonberry
/var/backups/lingonberry
```

The relay unit is permitted to write under:

```text
/var/lib/lingonberry
```

This difference is deliberate:

- normal relay service operation does not own backup creation or backup retention;
- backup and restore operations are operator-controlled storage responsibilities;
- expanding relay write access to backup paths weakens the separation contract.

Do not resolve a permission failure by broadly weakening `ProtectSystem`, changing service ownership ad hoc, or adding unrestricted write paths. Correct the configured directories and reviewed ownership instead.

## 7. Lifecycle operations

### 7.1 Startup

For the installed reference node:

```bash
sudo systemctl start lingonberry-storage-ready.service
sudo systemctl start lingonberry-relay.service
```

Starting the relay also pulls in the required storage readiness unit through systemd dependency resolution.

### 7.2 Status

Inspect each responsibility separately:

```bash
sudo systemctl status lingonberry-storage-ready.service
sudo systemctl status lingonberry-relay.service
sudo journalctl -u lingonberry-storage-ready.service
sudo journalctl -u lingonberry-relay.service
```

A successful storage unit means the readiness command completed successfully. It does not mean a storage daemon is running.

### 7.3 Re-running readiness

Because the storage gate uses `RemainAfterExit=yes`, rerun it explicitly after a relevant storage, configuration, restore, migration, or binary change:

```bash
sudo systemctl restart lingonberry-storage-ready.service
```

Then restart or start the relay only after the gate succeeds.

### 7.4 Relay restart

A relay-only restart does not execute a migration and does not replace backup or restore verification:

```bash
sudo systemctl restart lingonberry-relay.service
```

Use the v1 operator and upgrade runbooks for changes that affect binaries, storage format, configuration, or durable data.

### 7.5 Stop behavior

Stopping the relay terminates the long-running network process. Stopping the oneshot storage unit only changes systemd's recorded active state; there is no resident storage process to terminate.

## 8. Development topology

For local development, the commands may be invoked independently:

```bash
cargo run -p lingonberry-storage -- capabilities
cargo run -p lingonberry-storage -- config
cargo run -p lingonberry-storage -- ready
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

Use disposable, explicit directories for development. Do not point an ad hoc `cargo run` process at the installed reference node's `/var/lib/lingonberry` state.

The storage command exits after producing its result. Only the relay `serve-http` command remains resident in this example.

## 9. Upgrade and replacement boundary

Binary separation permits the operator to inspect or stage each binary independently, but release replacement remains coordinated.

Before changing either binary:

- record both binary versions and checksums;
- save both environment files and deployed systemd units;
- create and verify the required storage backup;
- stop the relay before mutating durable storage;
- run explicit migration only when required;
- rerun storage `verify` and the readiness gate;
- restart the relay and complete application-level checks.

Do not claim a successful node upgrade because only one binary starts. The release evidence must cover the storage and relay responsibilities together.

See [`V1_0_UPGRADE_AND_ROLLBACK.md`](./V1_0_UPGRADE_AND_ROLLBACK.md).

## 10. Failure boundaries

Stop and investigate when:

- the storage readiness unit fails;
- the relay starts without the checked-in dependency contract;
- relay and storage commands resolve unintended or inconsistent durable paths;
- either service runs under an unexpected user or group;
- a required path is a symlink or unsupported file type;
- a migration journal is present and has not been inspected;
- storage format is unsupported or corrupt;
- a permission workaround would require weakening the checked-in hardening contract.

Do not bypass the readiness gate with `--no-block`, manual background execution, a copied unit without `Requires=`, or deletion of storage evidence files.

## 11. Non-goals

This contract does not define:

- a remote storage service protocol between relay and storage;
- an independently scalable storage daemon;
- multi-node placement or replication;
- container orchestration;
- network-filesystem support for active canonical storage;
- independent release compatibility for arbitrary relay and storage versions.

Those capabilities require separate protocol, compatibility, and qualification work.

## 12. Verification checklist

Before accepting the reference topology, verify:

- both checked-in units pass `systemd-analyze verify`;
- the storage unit is `Type=oneshot` and runs `lingonberry-storage ready`;
- the relay unit is `Type=simple` and runs `lingonberry-relay serve-http`;
- the relay unit requires and starts after the storage readiness gate;
- both units use `lingonberry:lingonberry`;
- relay write access does not include the backup root;
- storage readiness succeeds with the reviewed storage environment;
- relay startup, health, persistence, and restart checks succeed;
- the evidence records the exact binaries, configuration, units, and candidate revision.

## 13. Release boundary

This document describes the current v1.0.0 pre-release implementation. It does not indicate that v1.0.0 has been published or that formal qualification is complete.

The designated pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Privileged reference-host qualification and the formal 72-hour soak remain pending.

## References

- [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md)
- [`V1_0_UPGRADE_AND_ROLLBACK.md`](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [`STORAGE_NODE_RUNTIME.md`](./STORAGE_NODE_RUNTIME.md)
- [`STORAGE_MIGRATION_AND_UPGRADE.md`](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [`SYSTEMD_UNIT_TEMPLATES.md`](./SYSTEMD_UNIT_TEMPLATES.md)
- [`OPERATOR_CLI_CONTRACT.md`](./OPERATOR_CLI_CONTRACT.md)
- [`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md)
