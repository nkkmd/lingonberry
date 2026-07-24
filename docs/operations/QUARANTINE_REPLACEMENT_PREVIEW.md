# Quarantine Replacement Preview and Proof

**Status: implemented** | **v1.0 pre-release normative operations contract**

This document defines the non-mutating replacement-preview boundary for quarantine compaction policy v2. Preview produces a deterministic replacement plan and verification proof. It does not authorize staging, publication, pointer switching, retention deletion, or cleanup.

## Safety boundary

Preview is read-only for runtime quarantine state and the supplied backup. It may write only into a caller-supplied empty output directory.

```text
runtime state: read-only
verified backup v2: read-only
proof output directory: newly published artifacts only
```

Preview must not rewrite, truncate, rotate, rename, delete, publish, or otherwise mutate a managed ledger, segment, index, generation, pointer, or transaction workspace.

Preview does not acquire the same-host quarantine mutation lock. Operators using it for qualification or maintenance evidence must quiesce writers. Runtime fingerprints are compared before and after the logical scan, but a matching pair is not a transactional snapshot guarantee.

## Commands

```bash
lingonberry-quarantine-maintenance replacement-preview \
  <verified-backup-v2-dir> <empty-proof-dir>

lingonberry-quarantine-maintenance verify-replacement-proof \
  <proof-dir>
```

Policy-v1 compaction preview and proof remain separate commands and formats.

## Versions and artifacts

```text
policy: lingonberry-quarantine-compaction-policy/v2
plan: lingonberry-quarantine-replacement-plan/v1
proof: lingonberry-quarantine-replacement-proof/v1
```

Preview publishes:

```text
quarantine-replacement-plan.json
quarantine-replacement-plan.digest
quarantine-replacement-proof.json
quarantine-replacement-proof.digest
```

The plan is canonical JSON and contains deterministic input bindings and per-ledger decisions. The proof is canonical JSON and records verification results and aggregate counts. `generatedAt` belongs to the proof and is outside the plan digest.

## Preconditions

Before publishing artifacts, preview must:

1. verify the archive-segment manifest and referenced immutable segments;
2. verify the supplied quarantine backup;
3. require backup version `lingonberry-quarantine-backup/v2`;
4. require an empty output directory;
5. compute an initial runtime fingerprint;
6. require the exact managed-ledger set;
7. parse every logical ledger line; and
8. reject duplicate terminal replacement keys.

After scanning, preview recomputes the runtime fingerprint. A mismatch fails with `LB_QUARANTINE_REPLACEMENT_CHANGED`, and no artifact set may be accepted as valid.

The backup-manifest digest and optional segment-manifest digest bind manifest bytes. They do not independently prove that backup contents equal live runtime state; runtime fingerprint and apply-time revalidation are separate checks.

## Managed-ledger classification

Immutable evidence ledgers are:

```text
quarantine.jsonl
quarantine-annotations.jsonl
admin-auth-audit.jsonl
```

They are retained as immutable evidence. Preview must not emit representation-replacement entries for them.

Terminal single-event ledgers are:

```text
quarantine-resolutions.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
```

For each valid logical line, preview chooses:

```text
retain-byte-for-byte
canonical-json-representation
```

Canonical representation may change insignificant whitespace, object-key ordering, and the line terminator. The parsed JSON value must remain identical.

## Archive-aware logical scan

The authoritative order is:

```text
verified archive segments in manifest order
→ active ledger
```

Each event receives a zero-based `logicalOrdinal` scoped to its ledger. Provenance records the active-ledger marker or immutable segment location and the source line number within that physical file.

The terminal replacement key follows the replacement-policy contract. A duplicate key in the complete archive-aware stream is corruption, not a deduplication opportunity.

## Plan entries

A terminal plan entry binds:

- ledger name and logical ordinal;
- replacement key and decision;
- source location and line number;
- source-line and source-value digests;
- replacement-line and replacement-value digests; and
- transformation identifier.

The replacement transformation identifier is:

```text
canonical-json-representation
```

Source-value and replacement-value digests are computed from canonical serialization of parsed values and must match. An already byte-identical canonical line is retained and is not counted as a replacement.

## Plan digest boundary

The canonical plan binds:

- plan and policy versions;
- source backup-manifest digest;
- optional segment-manifest digest;
- runtime fingerprint;
- ordered managed-ledger plans; and
- semantic-equivalence expectations.

It excludes output paths, hostnames, process IDs, elapsed duration, and proof-generation time. Identical verified inputs must produce identical canonical plan bytes and the same digest.

## Proof contents

The proof binds the plan digest and records:

```text
mutationAllowed: false
rewritePerformed: false
sourceLines
replacementLines
retainedLines
semanticEquivalence
```

All required semantic-equivalence fields must be true. They cover record identity, terminal state, logical order, status and state-derived metrics, promotion eligibility, idempotent terminal operations, conflict outcomes, batch classification, ordered reader results, corruption behavior, and complete one-to-one provenance.

Matching line counts, byte counts, or digests alone is insufficient.

## Artifact verification boundary

`verify-replacement-proof` checks the artifact directory: digest pairs, supported versions, exact ledger structure, immutable-ledger restrictions, entry mapping, canonical replacement values, aggregate counts, ordering, and declared semantic-equivalence fields.

It does not re-read runtime state, re-verify the original backup directory, prove that runtime still matches the fingerprint, authorize mutation, or authenticate the artifact author. Apply preparation must revalidate plan inputs against current state under the transaction and lock contract.

## Publication behavior

The output directory must be empty. Preview writes temporary artifacts, verifies generated values, and publishes final names through the checked-in artifact path.

The four files are not one cross-file atomic object. A partial or conflicting set is invalid. File presence alone is not success; later commands must require the complete verified artifact set.

## Digest boundary

Plan and proof digests use the repository integrity-digest implementation. They detect accidental or uncoordinated byte changes. They are not digital signatures, trusted timestamps, authorization tokens, or independent provenance attestations.

## Fail-closed conditions

Preview or verification must fail for:

- unsupported backup, plan, proof, or policy version;
- backup or segment verification failure;
- a non-empty output directory;
- missing, unknown, or duplicate managed-ledger entries;
- malformed logical JSON;
- duplicate terminal replacement keys;
- replacement targeting immutable evidence;
- incomplete, duplicate, or non-bijective provenance;
- changed parsed values, logical order, counts, or semantic-equivalence results;
- runtime fingerprint change during preview;
- digest mismatch;
- partial or conflicting artifacts; or
- deletion, merge, collapse, deduplication, retention, or conflict-repair decisions.

Errors remain distinguishable through stable `LB_QUARANTINE_REPLACEMENT_*` families. Human-readable messages are diagnostic and are not the sole programmatic contract.

## Operational evidence

Retain the application commit and binary identity, verified backup v2 and manifest, segment verification output, command and exit status, plan/proof and digest files, runtime fingerprint and manifest digests, pre/post status, operator identity, timestamp, and recovery notes.

Preview evidence does not establish formal soak or privileged reference-host qualification unless those procedures were separately executed and recorded.

## Non-goals

Preview does not provide replacement application, transaction creation, generation sealing or publication, pointer switching, rollback, resume, retention deletion, writer exclusion, a transactional snapshot, distributed locking, remote backup storage, or cryptographic signing.

## Related documents

- [`QUARANTINE_REPLACEMENT_POLICY.md`](./QUARANTINE_REPLACEMENT_POLICY.md)
- [`QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [`QUARANTINE_REPLACEMENT_TRANSACTION.md`](./QUARANTINE_REPLACEMENT_TRANSACTION.md)
- [`QUARANTINE_REPLACEMENT_GENERATION.md`](./QUARANTINE_REPLACEMENT_GENERATION.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)
