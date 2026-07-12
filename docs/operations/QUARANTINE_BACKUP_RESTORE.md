# Quarantine Backup / Export / Restore

**Status: implemented** | **Last updated: 2026-07-12**

This runbook covers verified local backups of all quarantine-related append-only state.

## Managed files

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
```

Missing files are represented as absent in the manifest. Environment variables and bearer tokens are never included.

## Snapshot boundary

Stop mutation traffic and scheduled promotion before export. The exporter copies each managed file, then re-reads the source and verifies that byte length and integrity digest are unchanged. If a source changes during export, the backup fails and no valid manifest is published.

This is same-host snapshot verification, not distributed locking.

## Export

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-backup export /srv/backups/lingonberry/quarantine-2026-07-12
```

The destination directory must be empty. The versioned manifest is written last through a temporary file and atomic rename:

```text
quarantine-backup-manifest.json
```

Each entry records file name, presence, byte length, and an integrity digest. The current digest format is `fnv1a64:<hex>` and is intended for accidental corruption detection, not cryptographic authenticity.

## Verify

```bash
lingonberry-quarantine-backup verify /srv/backups/lingonberry/quarantine-2026-07-12
```

Verification rejects:

- unsupported manifest versions
- missing or modified files
- byte-length or digest mismatches
- incomplete or duplicate managed-file entries
- invalid file names and path traversal
- a file marked absent when it exists

Run verification after copying a backup to other media.

## Restore

Stop the relay and scheduler first. Restore only into a state directory that contains no managed quarantine files.

```bash
lingonberry-quarantine-backup restore \
  /srv/backups/lingonberry/quarantine-2026-07-12 \
  /var/lib/lingonberry/relay-restored
```

The backup is verified before any destination writes. Each restored file is written through a temporary file and atomic rename. Existing managed files cause a conflict instead of being overwritten.

After restore:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay-restored \
  lingonberry-relay quarantine-status

lingonberry-quarantine-backup verify \
  /srv/backups/lingonberry/quarantine-2026-07-12
```

Compare status and metrics with the source node before switching traffic.

## Permissions

Backups can contain request payloads, operator notes, and authentication-failure metadata. Restrict directory ownership and permissions. The backup does not encrypt data and does not manage remote uploads or retention schedules.
