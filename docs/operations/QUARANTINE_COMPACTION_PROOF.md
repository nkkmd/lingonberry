# Quarantine Compaction Preview and Semantic Proof

**Status: implemented through QL-5C2** | **v1.0 pre-release normative operations contract**

This document defines the non-mutating quarantine compaction preview and its versioned proof artifact. The preview demonstrates the policy-v1 result for the current logical managed-ledger streams. It does not authorize or perform runtime-state replacement, truncation, deletion, deduplication, retention, or compaction.

## Contract versions

```text
lingonberry-quarantine-compaction-policy/v1
lingonberry-quarantine-compaction-proof/v1
```

A proof with any other proof or policy version is unsupported.

## Managed-ledger policy

### Immutable evidence

```text
quarantine.jsonl
quarantine-annotations.jsonl
admin-auth-audit.jsonl
```

Every valid logical line is retained. Policy v1 defines no removal rule for these ledgers.

### Terminal single-event ledgers

```text
quarantine-resolutions.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
```

Each quarantine ID may appear at most once in each terminal ledger. A duplicate terminal event is corruption and causes preview generation to fail. It is not a removable duplicate or a compaction opportunity.

Under policy v1, every valid line in every managed ledger is retained and the permitted removable-line count is exactly zero.

## Required source state

The preview reads each managed ledger through the archive-aware ordered reader:

1. immutable segments listed for that ledger in manifest order; then
2. the active ledger.

Before scanning, the command verifies the segment manifest and all listed immutable segments. Missing, modified, malformed, duplicated, out-of-order, path-traversing, or unlisted segment state causes failure.

The preview does not acquire `.quarantine-operation.lock`. It compares a runtime fingerprint before and after scanning, but that comparison is change detection rather than transactional isolation. Operators producing operational evidence must quiesce relay ingestion, scheduler activity, administrator mutations, rotation, restore, replacement, and other writers for the complete preview window.

## Backup prerequisite

Create and verify an archive-inclusive backup v2 immediately before preview:

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay

lingonberry-quarantine-backup export /srv/backups/lingonberry/pre-compaction
lingonberry-quarantine-backup verify /srv/backups/lingonberry/pre-compaction
```

A v1 backup is rejected because it does not include archived segments.

The preview records a digest of the supplied backup manifest. It does not prove that the supplied backup was taken from the exact runtime bytes subsequently scanned. Operators must establish that relationship through quiescence, timestamps, host identity, state-directory identity, backup evidence, and procedural control.

## Create preview and proof

```bash
lingonberry-quarantine-maintenance compaction-preview \
  /srv/backups/lingonberry/pre-compaction \
  /srv/backups/lingonberry/compaction-proof
```

The output directory must be empty. The command creates:

```text
quarantine-compaction-proof.json
quarantine-compaction-proof.digest
```

Files are written through temporary files and renamed into place. Preview creation finishes by verifying the generated proof directory.

## Proof content

The proof records:

- proof and policy versions;
- generation timestamp;
- digest of the supplied backup manifest;
- digest of the runtime segment manifest when present;
- one entry for each exact managed-ledger name;
- ledger classification;
- logical line count;
- logical byte count;
- ordered logical-stream digest;
- retained and removable line counts;
- unique key count;
- blocked-removal reason;
- promoted, dismissed, and permanently rejected terminal-ledger line counts;
- `mutationAllowed: false`; and
- `rewritePerformed: false`.

Logical bytes are reconstructed from the ordered logical lines used by the reader. They are not a claim that active files and archived segment files form one contiguous physical file or that filesystem metadata is preserved.

For immutable-evidence ledgers, `uniqueKeys` counts present `id` values where the parsed object supplies one. It is not a proof that every immutable-evidence record has an ID or that IDs are globally unique across all ledger types.

## Runtime-change detection

The runtime fingerprint covers:

- the six active managed-ledger paths;
- `quarantine-segments.json`; and
- every directory entry currently present under `quarantine-segments/`.

The command computes file digests before and after scanning. An observed difference returns `LB_QUARANTINE_COMPACTION_CHANGED`.

This does not provide a renewable lease, multi-file snapshot, distributed lock, or protection against a change that occurs and is reverted between the two fingerprints. Quiescence remains required for evidence-quality operation.

## Verify proof

```bash
lingonberry-quarantine-maintenance verify-compaction-proof \
  /srv/backups/lingonberry/compaction-proof
```

Verification rejects:

- a missing or unreadable proof or digest file;
- proof digest mismatch;
- malformed proof JSON;
- unsupported proof or policy versions;
- a managed-ledger set other than the exact six expected ledgers;
- duplicate or missing managed-ledger entries;
- a retained-line count different from the ledger line count;
- any non-zero removable-line count;
- `mutationAllowed: true`; or
- `rewritePerformed: true`.

Proof verification validates the proof directory only. It does not re-read the source backup, current runtime state, segment manifest, or archived segments, and it does not prove that the runtime remains unchanged after preview generation.

The digest is an implementation integrity digest for accidental-change detection. It is not a digital signature, MAC, trusted timestamp, provenance credential, or protection against an actor who can rewrite both proof and digest.

## Interpretation

A valid policy-v1 proof establishes only that:

- the preview parsed the exact managed-ledger set available through the ordered readers;
- terminal-ledger keys were unique within each terminal ledger;
- the generated proof reports all scanned lines retained;
- the generated proof reports zero removable lines; and
- the preview and verifier did not authorize or report a rewrite.

It does not establish that compaction is safe under a different policy, that a replacement transaction has completed, that a backup can be restored, that lifecycle semantics are preserved by a future rewrite, or that retention deletion is authorized.

## Evidence bundle

Retain together:

- application commit and binary identity;
- host and resolved state-directory identity;
- quiescence start and end evidence;
- verified backup-v2 directory and manifest;
- segment and derived-index verification output;
- preview command output;
- both proof files;
- independent proof-verification output; and
- operator identity, timestamp, and change record.

Do not use proof-file validity alone as cutover approval.

## Non-goals

Policy v1 and QL-5C2 do not provide:

- runtime ledger rewrite or replacement;
- deletion, retention, truncation, or compression;
- duplicate repair or conflict resolution;
- event merging or history summarization;
- mutation locking or transactional snapshots;
- cryptographic proof signing;
- automatic approval or cutover;
- distributed coordination; or
- formal soak or privileged reference-host qualification evidence.

The policy-v2 replacement boundary and later transaction phases are documented separately. Any actual rewrite must satisfy the replacement policy, deterministic planning, source-to-replacement provenance, lifecycle/status/metrics/eligibility/idempotency equivalence, interrupted-transition recovery, and explicit retention approval where deletion is proposed.

## Related documents

- [`QUARANTINE_JSONL_MAINTENANCE.md`](./QUARANTINE_JSONL_MAINTENANCE.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)
- [`QUARANTINE_CONCURRENCY.md`](./QUARANTINE_CONCURRENCY.md)
- [`QUARANTINE_REPLACEMENT_POLICY.md`](./QUARANTINE_REPLACEMENT_POLICY.md)

## Release boundary

This documentation normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak remains unperformed, privileged reference-host qualification/rehearsal remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.
