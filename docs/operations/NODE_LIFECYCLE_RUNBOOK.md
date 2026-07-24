# Node Lifecycle Runbook

**Status: v1.0 pre-release normative operations contract**  
**Last reviewed: 2026-07-24**

## Purpose

This runbook defines the supported lifecycle for a single-node Lingonberry reference deployment: installation, preflight, start, verification, stop, restart, backup, restore, migration, failure recovery, and retirement.

The checked-in systemd units are the reference host integration for v1.0. Container deployment is optional and must preserve the same binary, configuration, filesystem, readiness, and evidence boundaries. This document does not make containers the primary or normative lifecycle.

## 1. Runtime model

The reference node has two different lifecycle components:

| Component | Lifecycle | Responsibility |
|---|---|---|
| `lingonberry-storage ready` | systemd `oneshot` readiness gate | Resolve storage configuration and reject failed storage checks before relay startup. |
| `lingonberry-relay serve-http` | long-running process | Bind the HTTP listener and serve the relay surface. |

`lingonberry-storage run` prints a resolved runtime snapshot and exits. It is not a daemon and has no resident process to stop.

The reference units are:

```text
deploy/systemd/lingonberry-storage-ready.service
deploy/systemd/lingonberry-relay.service
```

The relay unit requires and starts after the storage readiness gate. Both units use the `lingonberry` service account. The relay may write the active state root but must not receive storage-backup-root write access merely because the storage gate has it.

See:

- [Relay and Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)

## 2. Lifecycle invariants

Every lifecycle operation must preserve these invariants:

1. Record the exact application commit or release artifact under operation.
2. Resolve and record `configPath`, `stateDir`, `dataDir`, `backupDir`, and `tempDir` before mutation.
3. Do not run migration, restore, or replacement cleanup while the relay can write the same active storage.
4. Do not treat `health` as storage verification.
5. Do not treat relay HTTP readiness as deep storage verification.
6. Keep verified backups distinct from migration backups, archive exports, qualification evidence, and formal-soak evidence.
7. Never perform an implicit migration during ordinary relay or readiness startup.
8. Preserve unresolved migration, quarantine, incident, and release-blocking evidence.
9. Do not publish environment files, bearer tokens, or unreviewed network identifiers in lifecycle evidence.

## 3. Configuration preflight

Configuration precedence is:

```text
defaults -> configuration file -> environment -> CLI overrides
```

Storage-specific environment variables are:

```text
LINGONBERRY_STORAGE_CONFIG
LINGONBERRY_STORAGE_STATE_DIR
LINGONBERRY_STORAGE_DATA_DIR
LINGONBERRY_STORAGE_BACKUP_DIR
LINGONBERRY_STORAGE_TEMP_DIR
```

Equivalent CLI overrides are:

```text
--config
--state-dir
--data-dir
--backup-dir
--temp-dir
```

Before startup or mutation, inspect the resolved configuration:

```bash
lingonberry-storage config
lingonberry-storage status
```

Then run read-only diagnostics:

```bash
lingonberry-storage doctor
lingonberry-storage ready
```

Use strict verification when warnings must block the operation:

```bash
lingonberry-storage verify
```

Semantics:

- `health` reports process-level health only.
- `doctor` fails on failed checks but permits warnings.
- `ready` fails on failed checks but permits warning-only reports.
- `verify` fails on warnings or failed checks.
- `run` prints the resolved runtime snapshot and exits.

## 4. Initial installation and host preparation

Before enabling the services:

1. Install the intended `lingonberry-storage`, `lingonberry-storage-migrate`, and `lingonberry-relay` binaries.
2. Install the checked-in systemd units without editing repository copies in place.
3. Create the `lingonberry` service account and required directories.
4. Create `/etc/lingonberry/storage.env` and `/etc/lingonberry/relay.env` with restrictive ownership and permissions.
5. Confirm the storage and relay units use the intended environment files.
6. Confirm the relay listen address is private when a reverse proxy is responsible for public TLS termination.
7. Run storage `config`, `doctor`, and `ready` under the same account and environment used by systemd.
8. Validate reverse-proxy configuration separately when used.

Do not place administrator tokens in unit command lines, repository files, shell history, or evidence bundles.

## 5. Start procedure

### 5.1 Reference systemd deployment

Reload units after installation or unit changes:

```bash
sudo systemctl daemon-reload
```

Run the storage gate first:

```bash
sudo systemctl restart lingonberry-storage-ready.service
sudo systemctl status --no-pager lingonberry-storage-ready.service
```

Only after it succeeds, start or restart the relay:

```bash
sudo systemctl restart lingonberry-relay.service
sudo systemctl status --no-pager lingonberry-relay.service
```

The relay unit's `Requires=` and `After=` relationship is the host-level startup ordering contract. It does not turn the storage gate into a resident storage service.

### 5.2 Manual diagnostic startup

Manual startup is for controlled diagnosis and development, not a substitute for the reference service contract.

```bash
lingonberry-storage config
lingonberry-storage ready
lingonberry-relay serve-http 127.0.0.1:8787
```

Run the relay in a supervised terminal and stop it with `SIGTERM` or an equivalent normal termination request.

### 5.3 Container deployment

A container deployment is supported only when it preserves these boundaries:

- separate storage readiness and relay execution steps;
- the same configuration precedence and directory layout;
- persistent active storage outside ephemeral container layers;
- explicit backup and restore locations;
- relay startup blocked by a failed storage readiness check;
- ordinary process restart does not perform migration;
- secrets are injected without appearing in images or command lines;
- lifecycle evidence records the image digest and application commit.

## 6. Post-start verification

Verify in this order:

1. Storage gate state:

   ```bash
   systemctl is-active lingonberry-storage-ready.service
   ```

2. Resolved storage configuration:

   ```bash
   lingonberry-storage config
   ```

3. Storage diagnostics:

   ```bash
   lingonberry-storage ready
   lingonberry-storage doctor
   ```

4. Strict storage verification when required by the operation:

   ```bash
   lingonberry-storage verify
   ```

5. Relay process state:

   ```bash
   systemctl is-active lingonberry-relay.service
   ```

6. Relay CLI and public HTTP readiness:

   ```bash
   lingonberry-relay ready
   curl -fsS http://127.0.0.1:8787/v1/ready
   curl -fsS http://127.0.0.1:8787/v1/capabilities
   ```

7. When public reverse proxying is enabled, repeat the HTTP checks through the public endpoint.
8. When the operation changes storage state, perform a controlled object read or other operation-specific verification defined by the governing runbook.

`GET /v1/ready` proves that the HTTP listener can receive and route the request. It is not a replacement for storage `doctor`, `ready`, or `verify`.

## 7. Stop procedure

### 7.1 Normal relay stop

Stop the network-facing process first:

```bash
sudo systemctl stop lingonberry-relay.service
```

Confirm it is inactive and that no process still holds the relay listener or writes the active storage.

### 7.2 Storage gate stop

```bash
sudo systemctl stop lingonberry-storage-ready.service
```

Stopping this unit clears its active systemd state. It does not terminate a resident storage daemon because none exists.

### 7.3 Maintenance stop boundary

Before migration, restore, destructive cleanup, or retirement:

1. stop the relay;
2. confirm the relay is inactive;
3. confirm no other process can mutate the same `dataDir`;
4. record the resolved directories;
5. create or identify the required verified backup;
6. perform the maintenance operation;
7. rerun storage readiness and verification before restarting the relay.

## 8. Restart procedure

A relay-only restart is appropriate for relay binary, listener, or non-storage relay configuration changes.

A full lifecycle restart is required after:

- storage configuration changes;
- restore;
- storage-format migration;
- active storage replacement;
- binary changes that affect storage diagnostics or compatibility;
- host permission changes affecting storage paths.

Full restart sequence:

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl restart lingonberry-storage-ready.service
sudo systemctl restart lingonberry-relay.service
```

Then execute the post-start verification sequence. A successful relay restart does not prove restore, migration, or storage consistency.

## 9. Backup lifecycle

Use the implemented storage recovery interface rather than constructing an ad hoc bundle from assumed filenames:

```bash
lingonberry-storage backup ...
```

Before backup:

- record the exact binary or commit;
- inspect resolved storage directories;
- confirm backup-root capacity and permissions;
- determine whether relay writes must be stopped for the required consistency level;
- distinguish a verified operational backup from an archive export or migration backup.

After backup:

- retain the generated manifest and verification output;
- record source storage identity and destination;
- verify the backup using the implemented command contract;
- protect the backup from modification;
- keep credentials and environment files outside the bundle;
- record the retention and disposal decision.

Do not assume that files named `manifest.json`, `replay-metadata.json`, or `resolved-config.json` are universal active-storage backup members. The implemented backup manifest is authoritative for that backup.

See [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md) and [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md).

## 10. Restore lifecycle

Use the implemented restore interface:

```bash
lingonberry-storage restore ...
```

Restore is a maintenance operation, not an online relay action.

Required sequence:

1. identify and verify the intended backup;
2. record the pre-restore state and exact target directories;
3. stop the relay and exclude concurrent writers;
4. use an isolated temporary workspace;
5. execute restore according to the recovery command contract;
6. inspect restore output and manifest verification;
7. run `config`, `doctor`, `ready`, and strict `verify` where required;
8. run `replay`, `list`, `retrieve`, or the operation-specific integrity checks needed for the restored state;
9. rerun the storage systemd gate;
10. restart the relay;
11. verify CLI and HTTP readiness;
12. retain the restore evidence and rollback decision.

Do not overwrite the last known good source or backup merely to make the restored node start. If verification fails, keep the relay stopped and investigate or roll back using the governing recovery procedure.

## 11. Storage-format migration lifecycle

Migration uses a separate binary:

```text
lingonberry-storage-migrate
```

Supported stages are governed by [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md). Ordinary relay or readiness startup must never silently advance migration state.

High-level sequence:

1. stop relay writes;
2. inspect the source and migration status;
3. create and verify the required backup;
4. create the migration plan;
5. apply or resume the migration;
6. verify migration output;
7. commit only after verification;
8. rerun storage diagnostics and the readiness gate;
9. restart the relay and verify service behavior.

The migration primitive's pre-commit rollback and release rollback from a verified backup are different operations. A committed migration cannot be undone by pretending the migration primitive restores backup files.

## 12. Failure handling

### 12.1 Storage gate failure

Keep the relay stopped. Inspect:

```bash
lingonberry-storage config
lingonberry-storage doctor
lingonberry-storage verify
journalctl -u lingonberry-storage-ready.service
```

Prioritize directory type, ownership, permissions, symlinks, storage format, migration journal, raw log, catalog, generation pointer, backup inventory, workspace, and disk-capacity findings reported by the doctor.

### 12.2 Relay startup or bind failure

Inspect:

```bash
lingonberry-relay ready
journalctl -u lingonberry-relay.service
```

Check listen configuration, environment loading, port conflicts, service account access, and reverse-proxy upstream configuration. Do not modify canonical storage merely to solve a listener bind failure.

### 12.3 Storage verification failure after restart

Stop the relay again. Preserve:

- resolved configuration output;
- doctor and verify output;
- relevant journals;
- backup or migration manifests;
- exact binary and commit identity;
- the first observed failure time.

Do not reclassify a failed rehearsal as successful qualification or formal soak evidence.

### 12.4 Disk pressure

Disk-pressure qualification on the privileged reference host remains a separate release requirement. Do not infer it from ordinary CI, local disposable-directory tests, or warning-free doctor output.

## 13. Observability during lifecycle operations

Use only implemented signals:

- systemd unit state;
- journald text for the relay and storage gate;
- storage CLI JSON from `config`, `status`, `doctor`, `verify`, `ready`, and `metrics`;
- relay CLI readiness;
- `GET /v1/ready` and `GET /v1/capabilities`;
- operation-specific backup, restore, migration, quarantine, qualification, or soak evidence.

Storage `metrics` is a point-in-time bounded-cardinality doctor summary. It is not a cumulative Prometheus registry. The repository does not guarantee universal `requestId`, `durationMs`, startup-event names, or generic publish/append/replay counter families in journald.

See [Observability](./OBSERVABILITY.md).

## 14. Retirement lifecycle

Retirement is not equivalent to deleting `tempDir` or exporting an archive.

Required sequence:

1. identify the node, exact application version, operator, and retirement reason;
2. stop external traffic and the relay;
3. confirm no remaining writer can mutate active storage;
4. resolve and record all storage directories;
5. create and verify the required final backup or archive according to its own contract;
6. preserve migration, quarantine, audit, incident, qualification, and release-blocking evidence required by policy;
7. record backup location, retention period, restore owner, and disposal authority;
8. remove or revoke administrator credentials and host access;
9. disable the systemd units and reverse-proxy route;
10. dispose of temporary and derived state only after confirming it is not required evidence;
11. perform physical deletion only under the applicable operator and retention policy;
12. record the completed retirement decision and verification.

Do not assume active canonical storage can always be reconstructed from an archive export. Do not delete the only verified backup while testing retirement replay.

## 15. Lifecycle evidence record

Record at least:

```text
Operation:
Node identifier:
Application commit or release artifact:
Binary or image digest:
Started at:
Completed at:
Operator:
Config path:
State directory:
Data directory:
Backup directory:
Temporary directory:
Relay stopped and writer exclusion verified:
Pre-operation backup and verification:
Storage doctor result:
Storage ready result:
Storage verify result:
Relay CLI ready result:
HTTP ready result:
Operation-specific manifest or evidence:
Rollback decision:
Open findings:
Evidence classification:
Secret and identifier review:
Final decision:
```

For release work, additionally record whether the evidence is local development, rehearsal, independent inspection, privileged reference-host qualification, or formal soak. These classifications are not interchangeable.

## 16. Non-goals

This runbook does not define:

- a resident networked storage service;
- multi-node replication or failover;
- container orchestration;
- online migration with concurrent writers;
- automatic backup scheduling or retention deletion;
- a universal archive-to-active-storage restoration guarantee;
- automatic secret rotation;
- completion of the formal 72-hour soak;
- completion of privileged reference-host qualification.

## Related documents

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Relay and Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Observability](./OBSERVABILITY.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
