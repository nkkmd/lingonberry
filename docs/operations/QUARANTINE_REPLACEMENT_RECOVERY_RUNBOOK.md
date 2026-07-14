# Quarantine Replacement Transaction Recovery Runbook

**Status: operator runbook for QL-5C3C** | **Last updated: 2026-07-14**

This runbook covers the generation-directory replacement transaction implemented by QL-5C3C. It applies only to a verified policy-v2 replacement proof and a verified archive-inclusive backup v2.

## 1. Safety model

Publication never overwrites active ledgers in place. A complete generation is materialized under:

```text
<state-dir>/quarantine-generations/<transaction-id>/
```

Readers and writers resolve the active generation through:

```text
<state-dir>/quarantine-current-generation.json
```

The pointer is replaced by one atomic rename. When no pointer exists, the legacy root-ledger layout remains active. An invalid pointer or invalid generation fails closed; it never falls back to the legacy root.

Immutable evidence ledgers and archive segments are not rewritten. Retention deletion, deduplication, event collapse, conflict resolution, schema migration, and archive-boundary movement remain forbidden.

## 2. Preconditions

Before applying a replacement:

1. stop concurrent maintenance processes that are not coordinated by the same-host lock;
2. verify the backup v2;
3. verify the replacement proof;
4. choose a new transaction directory whose basename is a bounded ASCII transaction ID;
5. keep the backup, proof, transaction workspace, and state directory available until the transaction is committed or rolled back.

Example workspace:

```text
/var/lib/lingonberry-maintenance/tx-20260714-001
```

The basename `tx-20260714-001` becomes the generation ID.

## 3. Apply

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-apply \
  /var/backups/lingonberry/quarantine-v2 \
  /var/lib/lingonberry-maintenance/proof-20260714 \
  /var/lib/lingonberry-maintenance/tx-20260714-001
```

A successful command returns canonical JSON with:

```text
state: committed
classification: committed
activeGeneration: tx-20260714-001
```

Do not delete the transaction workspace merely because the pointer exists. `committed` is the completion boundary.

## 4. Inspect status

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-status \
  /var/lib/lingonberry-maintenance/tx-20260714-001
```

Expected classifications include:

```text
prepared
writing
staged
verified
resumable-before-switch
resumable-after-switch
recovery-required
committed
rolled-back
```

A contradictory pointer, journal, generation manifest, digest, or input binding is an error and must not be manually reclassified as success.

## 5. Resume

Use resume after interruption when status is resumable or recovery-required:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-recover \
  /var/lib/lingonberry-maintenance/tx-20260714-001 \
  --resume
```

Resume re-verifies the journal-bound backup and proof, staged generation, publication intent, current pointer, ledger index, and archive segments. Repeating resume after completion is idempotent.

## 6. Roll back

Rollback is available only before the transaction reaches `committed`:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-recover \
  /var/lib/lingonberry-maintenance/tx-20260714-001 \
  --rollback
```

Rollback atomically restores the exact previous pointer. If the transaction began from the legacy root layout, rollback removes the generation pointer and makes the legacy root active again. It then rebuilds and verifies the derived index and verifies archive segments before recording `rolled-back`.

A committed transaction is terminal. Reverting a committed generation requires a new independently verified replacement transaction; do not edit the pointer manually.

## 7. Fail-closed conditions

Stop and preserve all evidence when any of the following occurs:

- journal or digest mismatch;
- backup or proof no longer matches the journal binding;
- runtime fingerprint changed before publication;
- generation directory is missing, incomplete, or contains unexpected files;
- staged or materialized ledger digest mismatch;
- current pointer is unrelated to the publication intent;
- immutable ledger differs byte-for-byte;
- archive segment or segment manifest verification fails;
- transaction state transition is invalid;
- status reports a contradictory or corrupt state.

Do not copy staged files over active files, edit the pointer, delete the generation directory, or alter journal files to force recovery.

## 8. Evidence to retain

For diagnosis, retain:

```text
transaction journal and digest
transaction input binding
staging manifest and staged ledgers
sealed generation manifest and digest
publication preparation pointer
publication intent and digest
verified backup v2
verified replacement plan and proof
active current-generation pointer
materialized generation directory
ledger index and segment manifest
application and filesystem error logs
```

## 9. Post-commit verification

After `committed`:

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance verify-index

LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance verify-segments

LINGONBERRY_STATE_DIR=/var/lib/lingonberry \
lingonberry-quarantine-maintenance replacement-status \
  /var/lib/lingonberry-maintenance/tx-20260714-001
```

All commands must succeed, and status must identify the transaction generation as active.
