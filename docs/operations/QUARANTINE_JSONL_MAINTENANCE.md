# Quarantine JSONL Index, Segments, and Rotation

**Status: implemented** | **v1.0 pre-release normative operations contract**

This document defines the supported single-host maintenance contract for quarantine JSONL ledgers, the derived active-ledger index, immutable archive segments, and byte-preserving rotation.

It does not authorize record-rewriting compaction, evidence deletion, retention expiry, compression, remote archive management, or distributed coordination.

## Managed ledgers

The maintenance commands recognize exactly these managed JSONL ledgers:

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
```

A managed ledger may be absent before its first event. Once created, malformed JSONL, a partial trailing line, an unsupported path, or an unexpected duplicate lifecycle event must fail explicitly through the relevant reader or verifier.

## State-directory coordination

Mutating maintenance operations use the state-directory lock:

```text
<LINGONBERRY_STATE_DIR>/.quarantine-operation.lock
```

The lock coordinates participating processes on one filesystem. It is not a distributed lock and does not protect against writers that ignore the Lingonberry lock protocol.

Before rotation, restore, replacement cleanup, or another maintenance operation that changes managed quarantine state, operators must stop or otherwise quiesce:

- the relay process;
- the administrative HTTP listener;
- scheduled promotion or maintenance jobs;
- concurrent operator CLI commands; and
- any external process capable of writing the same files.

The command lock remains a last line of same-host coordination, not a substitute for an operator-controlled maintenance window.

## Derived active-ledger index

The derived index is stored at:

```text
<LINGONBERRY_STATE_DIR>/quarantine-ledger-index.json
```

Build and verify it with:

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
```

The index records, for each active managed ledger:

- whether the file is present;
- byte length;
- non-empty JSONL line count;
- first and last record offsets when applicable; and
- an integrity digest.

Index construction acquires the shared operation lock and rejects malformed JSON, partial trailing lines, unsupported managed paths, and source mutation observed during the build.

The index is derived state. It is not a backup, an authoritative lifecycle ledger, or a replacement for archive verification. Rebuild it after every active-ledger mutation and after restore. Rotation requires a current index and fails with `LB_QUARANTINE_INDEX_STALE` when the saved index no longer matches the active ledgers.

## Archive segment layout

Archive state consists of:

```text
<LINGONBERRY_STATE_DIR>/quarantine-segments.json
<LINGONBERRY_STATE_DIR>/quarantine-segments/
```

The segment manifest version is:

```text
lingonberry-quarantine-segments/v1
```

A segment file is named from its ledger and a ledger-specific, strictly increasing sequence. For example:

```text
quarantine.00000000000000000001.jsonl
```

Each manifest entry records:

- managed ledger name;
- sequence number;
- segment file name;
- creation timestamp;
- byte length;
- non-empty JSONL line count; and
- integrity digest.

Segment files are immutable evidence. Operators must not edit, rename, truncate, replace, or delete a listed segment outside a separately specified and verified lifecycle procedure.

## Ordered logical reads

Archive-aware readers verify the manifest and segment files, read matching segments in manifest order, and then read the active ledger.

This ordered logical stream is used by the implemented readers for quarantine records, resolutions, annotations, dismissals, permanent rejections, and other consumers that call the managed-ledger reader. Rotation must preserve the exact ordered logical line stream for the selected ledger.

The segment integrity digest and logical-stream digest detect accidental mutation and equivalence failures. They are not cryptographic signatures, provenance proofs, or protection against a malicious actor who can rewrite both data and metadata.

## Verify archive segments

Run:

```bash
lingonberry-quarantine-maintenance verify-segments
```

Verification rejects at least:

- unsupported manifest versions;
- duplicate ledger-and-sequence identities;
- non-increasing sequences for one ledger;
- duplicate segment file names;
- invalid ledger names or segment paths;
- missing segment files;
- byte-length, line-count, or digest mismatches;
- malformed JSONL or partial trailing lines;
- path traversal; and
- archive files not listed by the manifest.

A missing manifest is interpreted as an empty manifest only when the archive directory does not contain unlisted segment files. The existence of an untracked archive file is corruption, not recoverable spare evidence.

## Rotation prerequisites

Before rotating one ledger:

1. establish a maintenance window and quiesce writers;
2. verify current archive state;
3. build a fresh active-ledger index;
4. verify the index; and
5. confirm that an archive-inclusive v2 backup exists or create one before the change.

Example:

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-maintenance verify-segments
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-backup export /srv/backups/lingonberry/quarantine-pre-rotation
lingonberry-quarantine-backup verify /srv/backups/lingonberry/quarantine-pre-rotation
```

## Rotate one managed ledger

Example:

```bash
lingonberry-quarantine-maintenance rotate quarantine.jsonl
```

The implementation:

1. validates that the requested name is one of the managed ledgers;
2. acquires `.quarantine-operation.lock`;
3. verifies the saved active-ledger index;
4. verifies the current manifest and all archive segments;
5. refuses a missing or empty active ledger;
6. validates the active JSONL and reads the complete logical stream before rotation;
7. allocates the next ledger-specific sequence and refuses a conflicting segment path;
8. writes the active bytes to a temporary archive file and renames it to the immutable segment name;
9. replaces the active ledger with an empty file through a temporary file and rename;
10. writes the updated segment manifest through a temporary file and rename;
11. reads archived segments plus the active ledger again;
12. compares logical line count and ordered-stream digest; and
13. rolls back the active file, new segment, and manifest when publication or equivalence verification fails.

A successful rotation preserves the source bytes in an immutable segment and leaves an empty active file. It does not rewrite records, collapse duplicates, remove lifecycle evidence, or apply retention policy.

## Post-rotation verification

After rotation:

```bash
lingonberry-quarantine-maintenance verify-segments
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-relay quarantine-status
```

Record the rotation report fields, including:

```text
ledger
segmentFile
sequence
bytes
lines
logicalLinesBefore
logicalLinesAfter
semanticDigest
```

Also record the application commit, state-directory identity, command output, backup manifest, segment verification output, rebuilt-index verification output, and post-rotation status or metrics comparison.

Successful command execution alone is not reference-host qualification, a formal soak result, or proof that application-level lifecycle semantics remained correct under production workload.

## Repeated rotation

After new events are appended to an active ledger, rebuild the active index before another rotation:

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance rotate quarantine.jsonl
lingonberry-quarantine-maintenance verify-segments
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
```

Sequences are independent per ledger and must remain strictly increasing within each ledger.

## Maintenance planning

The read-only planner is invoked with byte and line thresholds:

```bash
lingonberry-quarantine-maintenance plan 67108864 100000
```

The planner verifies the active-ledger index and reports threshold crossings. It does not rotate automatically, schedule work, modify data, enforce retention, or prove that the selected thresholds are operationally appropriate.

## Backup and restore boundary

Current exports use the archive-inclusive format:

```text
lingonberry-quarantine-backup/v2
```

A v2 backup includes the six active ledger entries, `quarantine-segments.json` when present, and every listed immutable segment. The derived `quarantine-ledger-index.json` and `.quarantine-operation.lock` are intentionally excluded.

Before deleting a pre-rotation backup or returning the node to service, verify that:

- the v2 backup verifies successfully;
- the segment manifest and all listed segments verify successfully;
- the derived active-ledger index has been rebuilt and verifies;
- lifecycle status and metrics are consistent with the pre-maintenance evidence; and
- the backup has been copied and protected according to the operator's retention and access-control policy.

The verifier and restore path remain backward-compatible with v1 active-ledger-only backups, but a v1 backup cannot reconstruct archive segments that were not included. A v1 backup is therefore insufficient as the sole recovery point after rotation.

See [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md) for the complete export, verification, and restore contract.

## Error meanings

```text
LB_QUARANTINE_BUSY                 another coordinated operation holds the lock
LB_QUARANTINE_CORRUPT              malformed active or archived JSONL
LB_QUARANTINE_INDEX_STALE          saved active-ledger index is stale
LB_QUARANTINE_ROTATION_EMPTY       active ledger is missing or empty
LB_QUARANTINE_ROTATION_CONFLICT    target segment already exists
LB_QUARANTINE_ROTATION_EQUIVALENCE ordered logical stream changed
LB_QUARANTINE_SEGMENT_CORRUPT      manifest or archived evidence is inconsistent
```

Additional filesystem and validation failures may be reported with their specific checked-in error codes. Operators must preserve the complete error output in maintenance evidence rather than reducing it to the summary above.

## Prohibited assumptions and non-goals

The v1.0 contract does not provide:

- automatic rotation;
- automatic retention or archive deletion;
- record-rewriting compaction;
- semantic deduplication;
- compression;
- remote archive upload;
- cryptographic signing or authenticated manifests;
- distributed locking or multi-node consensus;
- concurrent-writer fencing outside the local lock protocol;
- automatic interrupted-operation recovery beyond the implemented rollback paths; or
- proof that a successful maintenance run satisfies formal soak or privileged reference-host qualification.

No archived segment may be deleted merely because rotation, backup export, or backup verification succeeded. Evidence deletion requires a separately implemented, documented, and verified retention or compaction contract.

## Release boundary

This documentation normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence remains unperformed, privileged reference-host qualification remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.