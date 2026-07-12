# Quarantine Backup / Export / Restore

**Status: implemented through QL-5C1** | **Last updated: 2026-07-12**

This runbook covers verified local backups of active quarantine ledgers and immutable archive segments.

## Backup format

New exports use:

```text
lingonberry-quarantine-backup/v2
```

V2 includes:

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
quarantine-segments.json        when present
quarantine-segments/*           every listed immutable segment
```

The derived `quarantine-ledger-index.json` and `.quarantine-operation.lock` are excluded.

Verification and restore remain backward-compatible with v1 active-ledger-only backups.

## Snapshot boundary

Export acquires the shared state-directory operation lock, verifies the segment manifest and archive directory, copies every source through a temporary file and atomic rename, then re-reads every source to detect mutation. The backup manifest is published last.

This is same-host coordination, not distributed locking.

## Export

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-backup export /srv/backups/lingonberry/quarantine-2026-07-12
```

The destination directory must be empty. The manifest is:

```text
quarantine-backup-manifest.json
```

Each entry records relative path, presence, byte length, and `fnv1a64:<hex>` integrity digest. The digest detects accidental corruption and staleness; it is not cryptographic authentication.

## Verify

```bash
lingonberry-quarantine-backup verify /srv/backups/lingonberry/quarantine-2026-07-12
```

Verification accepts v1 and v2 and rejects:

- unsupported or malformed versions
- missing, modified, or duplicate entries
- byte-length or digest mismatches
- path traversal or unsupported paths
- missing or tampered segment manifests and segments
- archive files not listed by the backup manifest
- segment files not listed by `quarantine-segments.json`
- malformed JSONL or partial trailing lines

V2 verification runs the same archive-segment verifier used by the runtime.

## Restore

Stop the relay and scheduler first. Restore into a destination without active ledgers, segment manifest, segment directory, derived index, or operation lock.

```bash
lingonberry-quarantine-backup restore \
  /srv/backups/lingonberry/quarantine-2026-07-12 \
  /var/lib/lingonberry/relay-restored
```

Restore behavior:

1. verify the complete backup before writes;
2. acquire the destination operation lock;
3. reject conflicting state;
4. restore active files, segment manifest, and segments through temporary files and atomic rename;
5. run segment verification against the restored destination;
6. remove files written by this restore if final verification fails.

After restore:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay-restored \
  lingonberry-quarantine-maintenance verify-segments

LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay-restored \
  lingonberry-relay quarantine-status
```

Compare status and metrics with the source node before switching traffic.

## Compatibility

- `export` always creates v2.
- `verify` accepts v1 and v2.
- `restore` accepts v1 and v2.
- A v1 restore contains active ledgers only and cannot reconstruct archive segments that were never included in that backup.

## Permissions and non-goals

Backups can contain request payloads, operator notes, rejection reasons, and authentication-failure metadata. Restrict ownership and permissions.

The backup system does not provide encryption, cryptographic signing, remote upload, compression, retention scheduling, or record compaction.
