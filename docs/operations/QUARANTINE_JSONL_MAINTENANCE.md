# Quarantine JSONL Index, Segments, and Rotation

**Status: implemented through QL-5B** | **Last updated: 2026-07-12**

This runbook covers the verified read-only index, non-destructive maintenance planning, archive-aware ordered reads, and byte-preserving ledger rotation.

Record-rewriting compaction, retention deletion, compression, and remote archives remain prohibited.

## Managed ledgers

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
```

## Derived index

```text
<LINGONBERRY_STATE_DIR>/quarantine-ledger-index.json
```

Build and verify it with:

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
```

The index records presence, byte length, non-empty JSONL line count, first and last record offsets, and an integrity digest. Build acquires the shared operation lock and rejects malformed JSON, partial trailing lines, and source mutation.

Rotation requires a fresh index. Rebuild it after every active-ledger mutation.

## Segment layout

```text
<LINGONBERRY_STATE_DIR>/quarantine-segments.json
<LINGONBERRY_STATE_DIR>/quarantine-segments/
```

Example immutable segment:

```text
quarantine.00000000000000000001.jsonl
```

The versioned segment manifest records:

```text
managed ledger name
strictly increasing sequence
segment file name
creation timestamp
byte length
non-empty line count
integrity digest
```

Archive-aware readers consume matching segments in manifest order and then consume the active ledger. Quarantine records, resolutions, annotations, dismissals, and permanent rejections use this ordered reader.

## Verify segments

```bash
lingonberry-quarantine-maintenance verify-segments
```

Verification rejects:

- unsupported manifest versions
- duplicate or out-of-order sequences
- duplicate segment file names
- missing segments
- modified byte length, line count, or digest
- malformed JSONL or partial trailing lines
- path traversal
- archive files not listed in the manifest

An empty or missing manifest is valid only when the archive directory has no unlisted segment files.

## Rotate a ledger

First build a fresh active-ledger index:

```bash
lingonberry-quarantine-maintenance build-index
```

Then rotate one managed ledger:

```bash
lingonberry-quarantine-maintenance rotate quarantine.jsonl
```

The rotation operation:

1. acquires `.quarantine-operation.lock`;
2. verifies the saved QL-5A index against the active ledgers;
3. validates the active ledger and refuses missing or empty input;
4. reads the complete logical stream before rotation;
5. writes the active bytes to a new immutable segment;
6. replaces the active ledger with an empty file;
7. publishes the updated manifest through temporary-file plus atomic rename;
8. reads archived segments plus the active ledger again;
9. compares logical line count and ordered-stream digest;
10. rolls back the active file, new segment, and manifest if equivalence fails.

Successful rotation preserves every source byte. It does not compact or delete lifecycle evidence.

## Repeated rotation

After new records are appended to the active ledger:

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance rotate quarantine.jsonl
lingonberry-quarantine-maintenance verify-segments
```

Sequences are ledger-specific and strictly increasing.

## Maintenance planning

```bash
lingonberry-quarantine-maintenance plan 67108864 100000
```

The planner verifies the active-ledger index and reports byte or line threshold crossings. It does not rotate automatically and does not modify data.

## Backup boundary

The current backup manifest predates archive segments and covers the six active ledger paths. Do not treat it as a complete post-rotation backup of archived evidence.

Until archive-inclusive backup/restore is implemented, preserve the following together using filesystem-level snapshots or an equivalent operator-controlled mechanism:

```text
six active ledgers
quarantine-segments.json
quarantine-segments/
```

This limitation is explicit and must be resolved before automated retention or compaction.

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

## Follow-up: QL-5C

Before record-rewriting compaction or retention deletion, QL-5C must add:

- archive-inclusive backup and restore
- explicit compaction policy per ledger type
- semantic equivalence checks for status, metrics, eligibility, idempotency, and corruption detection
- retained source evidence or a verifiable replacement proof
- interrupted-compaction recovery

No archived segment may be deleted under QL-5B.
