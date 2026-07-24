# Quarantine Backup, Verification, and Restore Contract

**Status: implemented v1.0 pre-release contract** | **Last reviewed: 2026-07-24**

This document defines the implemented local backup, verification, and restore behavior for quarantine state. The current export format is `lingonberry-quarantine-backup/v2`; verification and restore also accept the earlier active-ledger-only `v1` format.

## 1. Scope

The backup utility protects quarantine lifecycle evidence stored in the relay state directory. It does not back up the storage-node database, application binaries, configuration files, secrets, TLS material, systemd units, or external monitoring data.

A backup is not considered operationally qualified merely because export completed. Operators must verify the backup and exercise restoration into an isolated destination.

## 2. Managed state included in v2

Every v2 manifest contains entries for these active managed ledgers, whether present or absent:

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
```

When the source has a managed segment manifest, v2 also includes:

```text
quarantine-segments.json
quarantine-segments/<immutable-segment-files>
```

The export excludes derived and coordination files, including:

```text
quarantine-ledger-index.json
.quarantine-operation.lock
```

The derived ledger index must be rebuilt or regenerated after restore when required by the operational workflow. The operation lock must not be copied from the source or backup.

## 3. Manifest and integrity metadata

The manifest is published as:

```text
quarantine-backup-manifest.json
```

Each entry records:

- relative path;
- presence;
- byte length;
- `fnv1a64:<hex>` integrity digest when present.

The FNV-1a digest detects accidental modification and stale copies. It is not cryptographic authentication, tamper-proof signing, or proof of provenance.

The manifest also records its format version, creation time, and source state-directory string. The source path is informational and does not authorize a restore destination.

## 4. Snapshot and concurrency boundary

Export acquires the quarantine operation lock for the source state directory. It verifies the managed segment state before copying, writes each destination through a temporary file and rename, and re-reads each source file to detect mutation during export. The manifest is written last, then the completed backup is verified.

This lock coordinates cooperating processes on one filesystem. It is not distributed locking and does not make a snapshot across multiple nodes.

For a controlled production backup, stop or quiesce every process capable of mutating quarantine state, including the public relay, administrator listener, scheduler, maintenance commands, and operator CLI sessions. The in-process lock is a safety boundary, not a substitute for a documented maintenance window.

## 5. Export

Set the source state directory and export to an empty destination:

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-backup export \
  /srv/backups/lingonberry/quarantine-2026-07-24
```

The destination directory may be created by the command but must contain no entries. Export fails if a managed source path that should be a file is another filesystem object, if segment verification fails, or if a source changes during copying.

New exports always use:

```text
lingonberry-quarantine-backup/v2
```

## 6. Verification

Verify a backup before transfer, after transfer, and immediately before restore:

```bash
lingonberry-quarantine-backup verify \
  /srv/backups/lingonberry/quarantine-2026-07-24
```

Verification accepts v1 and v2. It rejects conditions including:

- missing, malformed, or unsupported manifests;
- duplicate manifest paths;
- missing required active-ledger entries;
- invalid absolute or parent-traversing paths;
- unsupported paths;
- inconsistent presence, byte-length, or digest metadata;
- missing or modified files;
- unlisted archive files;
- invalid segment-manifest or immutable-segment state.

For v2, the same segment verifier used by runtime maintenance is applied to the backup directory.

Verification validates structural and byte-level consistency only. It does not authenticate who created the backup, prove confidentiality, or establish that the source node was healthy.

## 7. Restore preconditions

Before restoring:

1. stop the public relay, administrator listener, scheduler, and all quarantine maintenance writers;
2. verify the backup;
3. choose an isolated destination state directory;
4. ensure the destination contains none of the managed active ledgers, segment manifest, segment archive directory, or derived ledger index;
5. preserve the original state directory until post-restore validation and rollback decisions are complete.

The restore operation acquires its own destination operation lock. An operation-lock file created for that restore is coordination state and must not be supplied by the backup.

Do not restore over a live or partially populated state directory.

## 8. Restore

```bash
lingonberry-quarantine-backup restore \
  /srv/backups/lingonberry/quarantine-2026-07-24 \
  /var/lib/lingonberry/relay-restored
```

For v2, restore:

1. reads and verifies the complete backup before managed writes;
2. acquires the destination operation lock;
3. rejects conflicting managed paths;
4. creates required directories;
5. copies present files through temporary files and atomic rename;
6. verifies restored segment state;
7. removes files written by the restore when final segment verification fails.

The rollback-on-error behavior covers files written by that restore attempt. It is not a transactional filesystem snapshot and does not recover unrelated destination content.

A v1 restore delegates to the legacy active-ledger restore path. It cannot reconstruct archive segments that were never present in the v1 backup.

## 9. Post-restore validation

Keep the restored state isolated from production traffic while validating it:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay-restored \
  lingonberry-quarantine-maintenance verify-segments

LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay-restored \
  lingonberry-relay quarantine-status
```

Then:

- rebuild or validate the derived ledger index as required by the maintenance runbook;
- list and sample active and archived annotations, dismissals, permanent rejections, and promotion resolutions;
- compare status and metrics with the source evidence captured at backup time;
- verify that terminal-state conflicts and idempotency still behave correctly;
- verify filesystem ownership and permissions for the runtime service account;
- start administrative and public listeners only after isolated checks succeed.

A successful restore command alone is not sufficient evidence for cutover.

## 10. Backup evidence

Retain at minimum:

- application commit or release identifier;
- backup format version;
- source node and state-directory identity;
- export and verification timestamps;
- manifest and its externally computed cryptographic digest, when available;
- command output and exit status;
- destination or media identifier;
- restore rehearsal results;
- post-restore status and segment-verification output;
- operator identity and approval record.

An external cryptographic digest of the completed backup artifact improves evidence handling but does not replace encryption or signing.

## 11. Security and retention

Backups may contain submitted payloads, operator notes, rejection reasons, annotations, publisher material, and authentication-failure metadata. Restrict ownership, permissions, access paths, copies, and retention according to the deployment policy.

The implementation does not provide:

- encryption at rest;
- cryptographic signing or authenticated manifests;
- remote upload or replication;
- compression;
- automatic scheduling;
- retention enforcement;
- secure deletion;
- record-level selective restore;
- cross-node consistency;
- storage-node database backup;
- automatic cutover or rollback.

## 12. Qualification boundary

Repository CI and the documentation walkthrough can validate code and frozen commands, but they do not constitute a production backup, privileged reference-host restore rehearsal, disaster-recovery qualification, or formal 72-hour soak evidence.

## 13. Related contracts

- [`QUARANTINE_LEDGER_ROTATION.md`](./QUARANTINE_LEDGER_ROTATION.md)
- [`QUARANTINE_JSONL_MAINTENANCE.md`](./QUARANTINE_JSONL_MAINTENANCE.md)
- [`QUARANTINE_ANNOTATIONS.md`](./QUARANTINE_ANNOTATIONS.md)
- [`QUARANTINE_DISMISSALS.md`](./QUARANTINE_DISMISSALS.md)
- [`QUARANTINE_PERMANENT_REJECTIONS.md`](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md)
