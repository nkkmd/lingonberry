# Quarantine Compaction Preview and Semantic Proof

**Status: implemented through QL-5C2** | **Last updated: 2026-07-13**

This runbook defines the first safe compaction phase. It produces a read-only preview and a versioned proof. It does not rewrite, replace, truncate, or delete runtime state.

## Policy v1

```text
lingonberry-quarantine-compaction-policy/v1
```

Managed ledgers are classified as follows.

### Immutable evidence

```text
quarantine.jsonl
quarantine-annotations.jsonl
admin-auth-audit.jsonl
```

Every line is retained. These records are audit evidence and have no removal rule under policy v1.

### Terminal single-event ledgers

```text
quarantine-resolutions.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
```

Each quarantine ID may appear once per ledger. A duplicate is corruption, not a compaction opportunity. Every valid line is retained under policy v1.

Therefore the safe removable-line count for policy v1 is always zero.

## Prerequisites

Create and verify an archive-inclusive backup v2 immediately before preview:

```bash
lingonberry-quarantine-backup export /srv/backups/lingonberry/pre-compaction
lingonberry-quarantine-backup verify /srv/backups/lingonberry/pre-compaction
```

A v1 backup is rejected because it cannot prove preservation of archived segments.

## Create preview and proof

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay

lingonberry-quarantine-maintenance compaction-preview \
  /srv/backups/lingonberry/pre-compaction \
  /srv/backups/lingonberry/compaction-proof
```

The output directory must be empty. The command creates:

```text
quarantine-compaction-proof.json
quarantine-compaction-proof.digest
```

The proof records:

- proof and policy versions
- backup-manifest digest
- segment-manifest digest when present
- per-ledger logical line count and byte count
- per-ledger ordered-stream digest
- retained and removable line counts
- record-key uniqueness counts
- promoted, dismissed, and permanently rejected counts
- `mutationAllowed: false`
- `rewritePerformed: false`

The runtime state is fingerprinted before and after scanning. Any observed change causes the preview to fail.

## Verify proof

```bash
lingonberry-quarantine-maintenance verify-compaction-proof \
  /srv/backups/lingonberry/compaction-proof
```

Verification rejects:

- proof digest mismatch
- unsupported proof or policy versions
- missing or duplicate managed-ledger entries
- any non-zero removable-line count under policy v1
- `mutationAllowed: true`
- `rewritePerformed: true`

## Concurrency boundary

Preview is read-only and does not acquire the mutation lock. It detects changes by comparing runtime fingerprints before and after scanning. Operators should still stop scheduled mutation traffic when producing an operational proof.

## Safety boundary

QL-5C2 does not authorize compaction. A proof produced under policy v1 demonstrates that no current ledger line is safely removable.

The proposed policy v2 boundary is specified in [QUARANTINE_REPLACEMENT_POLICY.md](./QUARANTINE_REPLACEMENT_POLICY.md). Policy v2 initially permits only one-to-one canonical representation replacement for terminal single-event ledgers. It does not permit history deletion, event merging, deduplication, conflict resolution, or changes to immutable evidence.

Actual rewrite remains blocked until later QL-5C3 phases implement and verify:

- the complete replacement-policy contract
- lifecycle, status, metrics, eligibility, and idempotency equivalence
- source-to-replacement provenance
- deterministic planning and proof verification
- interrupted-transition recovery
- separate approval for any retention deletion
