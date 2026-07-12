# Quarantine JSONL Index and Maintenance Planning

**Status: implemented (QL-5A)** | **Last updated: 2026-07-12**

This runbook defines the safe first phase of long-term JSONL maintenance. It adds verified indexing and non-destructive planning. It does not truncate, rotate, compact, or delete audit ledgers.

## Why destructive maintenance is blocked

Quarantine ledgers are append-only audit evidence. Rotation or compaction is unsafe until:

1. every reader can consume active and archived segments in a defined order;
2. archive manifests preserve provenance and integrity;
3. status, metrics, lifecycle eligibility, and duplicate/corruption detection are proven equivalent before and after maintenance;
4. recovery from an interrupted segment transition is defined.

QL-5A establishes the index and verification substrate needed for that follow-up work.

## Index file

```text
<LINGONBERRY_STATE_DIR>/quarantine-ledger-index.json
```

The versioned index covers the exact managed ledger set:

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
```

For each ledger it records:

```text
presence
byte length
non-empty JSONL line count
first record byte offset
last record byte offset
integrity digest
```

The current digest uses `fnv1a64:<hex>` for accidental corruption and staleness detection. It is not a cryptographic authenticity mechanism.

## Build the index

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-maintenance build-index
```

Index construction:

- acquires `.quarantine-operation.lock`;
- validates every non-empty line with the protocol JSON parser;
- rejects malformed JSON and partial trailing lines;
- re-reads each source to detect mutation during indexing;
- writes the index last through a temporary file and atomic rename.

## Verify the index

```bash
lingonberry-quarantine-maintenance verify-index
```

Verification is read-only and recomputes the full index. It rejects unsupported versions, an incomplete managed-file set, tampering, missing files, changed lengths, changed line offsets, and changed digests.

A successful verification means the index matches the current active ledgers. It does not certify archived data because archive segments are not yet implemented.

## Plan maintenance

```bash
lingonberry-quarantine-maintenance plan 67108864 100000
```

Arguments are:

```text
<byte-threshold> <line-threshold>
```

The command first verifies the index, then reports which present ledgers meet or exceed either threshold. The result always includes:

```json
{
  "destructiveActionsBlocked": true
}
```

The planner never changes ledger contents.

## Operational cadence

A reasonable initial workflow is:

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance plan 67108864 100000
```

Rebuild the index after any ledger mutation before using it for planning.

## Error meanings

```text
LB_QUARANTINE_BUSY          another mutation/index operation holds the lock
LB_QUARANTINE_CORRUPT       malformed JSONL or partial trailing line
LB_QUARANTINE_INDEX_CHANGED source changed during index construction
LB_QUARANTINE_INDEX_STALE   saved index no longer matches active ledgers
LB_QUARANTINE_INDEX_INVALID malformed or unsupported index
```

## Follow-up: QL-5B

QL-5B must introduce archive-aware ordered readers and verified segment transitions before enabling rotation or compaction. Active ledger truncation and retention enforcement remain prohibited until that work is complete.
