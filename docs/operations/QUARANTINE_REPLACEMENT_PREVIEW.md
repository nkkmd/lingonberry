# Quarantine Replacement Preview and Proof

**Status: implementation contract for QL-5C3B** | **Last updated: 2026-07-13**

This document defines the implementation boundary for the non-mutating policy-v2 replacement preview approved by `QUARANTINE_REPLACEMENT_POLICY.md`.

## 1. Safety boundary

The preview is read-only. It must not rewrite, replace, truncate, delete, rename, rotate, publish, or otherwise mutate runtime ledger state.

The preview may write only into a caller-supplied empty output directory.

```text
runtime state: read-only
verified backup v2: read-only
proof output directory: write-only publication target
```

QL-5C3B does not authorize application of a replacement plan. Transactional application remains QL-5C3C.

## 2. Proposed CLI

```bash
lingonberry-quarantine-maintenance replacement-preview \
  <verified-backup-v2-dir> <empty-proof-dir>

lingonberry-quarantine-maintenance verify-replacement-proof \
  <proof-dir>
```

The existing policy-v1 commands remain supported without behavior changes.

## 3. Versions and files

```text
policy: lingonberry-quarantine-compaction-policy/v2
proof: lingonberry-quarantine-replacement-proof/v1
plan: lingonberry-quarantine-replacement-plan/v1
```

Output directory:

```text
quarantine-replacement-plan.json
quarantine-replacement-plan.digest
quarantine-replacement-proof.json
quarantine-replacement-proof.digest
```

The plan contains deterministic inputs and decisions. The proof contains verification results and may contain a generated timestamp. Generated timestamps are excluded from the plan digest boundary.

## 4. Preconditions

Before scanning runtime state, the implementation must:

1. verify archive segments;
2. verify the supplied backup;
3. require backup version `lingonberry-quarantine-backup/v2`;
4. require an empty output directory;
5. compute a runtime fingerprint;
6. reject unknown managed ledgers or files presented as managed input.

After scanning and before publishing output, the runtime fingerprint must be recomputed. Any difference fails with no published proof.

## 5. Managed-ledger policy

### Immutable evidence

```text
quarantine.jsonl
quarantine-annotations.jsonl
admin-auth-audit.jsonl
```

Required decision:

```text
retain-byte-for-byte
```

No replacement entry may refer to these ledgers.

### Terminal single-event representation

```text
quarantine-resolutions.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
```

Allowed decision per logical record:

```text
retain-byte-for-byte
canonical-json-representation
```

`canonical-json-representation` is permitted only when parsing the source line and replacement line produces exactly the same JSON value.

## 6. Deterministic logical scan

Logical read order remains:

```text
verified archive segments in manifest sequence
→ active ledger
```

Each logical record receives a zero-based `logicalOrdinal` scoped to its managed ledger.

For each terminal record, the preview computes:

```text
replacementKey = quarantineId
sourceLineDigest
sourceValueDigest
canonicalLine
canonicalLineDigest
canonicalValueDigest
```

The preview must reject duplicate `replacementKey` values within a terminal ledger. A duplicate is corruption, not a deduplication opportunity.

## 7. Plan entry

A replacement-plan entry has the following normative fields:

```json
{
  "ledger": "quarantine-resolutions.jsonl",
  "logicalOrdinal": 0,
  "replacementKey": "q-123",
  "decision": "canonical-json-representation",
  "source": {
    "location": "active-ledger",
    "lineNumber": 1,
    "lineDigest": "...",
    "valueDigest": "..."
  },
  "replacement": {
    "lineDigest": "...",
    "valueDigest": "..."
  },
  "transformation": "canonical-json-representation"
}
```

For archived input, `source.location` identifies the immutable segment and `lineNumber` is scoped to that segment.

A byte-identical canonical line may use `retain-byte-for-byte` and must not be counted as a replacement.

## 8. Plan digest boundary

The plan digest is computed from canonical JSON over:

```text
plan version
policy version
source backup manifest digest
source segment manifest digest
runtime fingerprint
ordered managed-ledger plans
semantic-equivalence expectations
```

The following are excluded:

```text
generatedAt
output paths
hostnames
process IDs
wall-clock duration
```

Identical verified inputs must produce identical canonical plan bytes and the same digest.

## 9. Semantic-equivalence report

The verifier must machine-check:

```text
record identity equivalence
terminal state equivalence
logical order equivalence
status-count equivalence
Prometheus-metric equivalence
promotion-eligibility equivalence
single-operation idempotency equivalence
batch-operation idempotency equivalence
reader-result equivalence
corruption-detection equivalence
complete one-to-one provenance
```

A report is successful only when every required dimension is `true`.

Line count, byte count, or digest equality alone is insufficient.

## 10. Proof verification

Verification must reject:

- plan or proof digest mismatch;
- unsupported plan, proof, or policy version;
- missing, unknown, or duplicate managed-ledger entries;
- a replacement entry for an immutable ledger;
- missing or duplicate provenance mapping;
- source and replacement parsed-value mismatch;
- changed `replacementKey` or `logicalOrdinal`;
- changed terminal state or logical order;
- duplicate terminal keys;
- any deletion, collapse, merge, or retention operation;
- any failed semantic-equivalence dimension;
- non-deterministic fields inside the plan digest boundary.

## 11. Atomic proof publication

The implementation writes temporary files inside the output directory, verifies all generated artifacts, and then renames them to final names.

A failed preview must not leave a valid-looking final plan/proof pair.

## 12. Error-code families

Proposed stable families:

```text
LB_QUARANTINE_REPLACEMENT_BACKUP
LB_QUARANTINE_REPLACEMENT_CHANGED
LB_QUARANTINE_REPLACEMENT_CONFLICT
LB_QUARANTINE_REPLACEMENT_CORRUPT
LB_QUARANTINE_REPLACEMENT_POLICY
LB_QUARANTINE_REPLACEMENT_PROOF
LB_QUARANTINE_REPLACEMENT_SEMANTICS
```

Messages may add detail, but callers should branch only on the stable code.

## 13. Required tests

```text
valid canonical representation replacement
already-canonical no-op
deterministic plan bytes and digest
duplicate terminal-key rejection
semantic-change rejection
immutable-ledger replacement rejection
unknown-ledger rejection
incomplete provenance rejection
proof and plan tampering rejection
runtime-fingerprint change rejection
backup-v1 rejection
policy-v1 proof verification regression
output-directory atomicity
```

The approved JSON vectors under `docs/operations/fixtures/quarantine-replacement-policy/` are normative test inputs.

## 14. Non-goals

- rewrite application;
- transaction journal;
- staging-ledger publication;
- rollback or resume;
- retention deletion;
- distributed locking;
- remote backup or archive storage;
- cryptographic signing.
