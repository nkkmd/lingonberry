# Quarantine Replacement Preview Runbook

**Status: QL-5C3B operator procedure** | **Last updated: 2026-07-13**

This runbook executes and verifies the non-mutating policy-v2 replacement preview. It does not authorize or apply a ledger rewrite.

## 1. Preconditions

Use a quiescent or controlled runtime state and prepare:

- the runtime state directory;
- a verified complete backup with version `lingonberry-quarantine-backup/v2`;
- a new or empty proof output directory;
- the `lingonberry-quarantine-maintenance` binary built from the reviewed release candidate.

The command rejects an unverified backup, backup v1, a non-empty output directory, corrupt archive segments, duplicate terminal keys, and runtime-state changes detected during the scan.

## 2. Generate the preview

```bash
lingonberry-quarantine-maintenance replacement-preview \
  <verified-backup-v2-dir> \
  <empty-proof-dir>
```

The runtime state and backup are read-only inputs. Only the proof directory may be written.

Expected artifacts:

```text
quarantine-replacement-plan.json
quarantine-replacement-plan.digest
quarantine-replacement-proof.json
quarantine-replacement-proof.digest
```

A successful report must state:

```text
mutationAllowed = false
rewritePerformed = false
```

## 3. Verify the generated proof

```bash
lingonberry-quarantine-maintenance verify-replacement-proof \
  <proof-dir>
```

Verification checks both artifact digests, supported versions, exact managed-ledger membership, logical ordinals, immutable-ledger retention, replacement-key uniqueness, one-to-one provenance, source/replacement value equivalence, and all required semantic-equivalence dimensions.

Do not continue when verification fails. Preserve the runtime state, backup, proof directory, command output, and error code for diagnosis.

## 4. Review the plan

For immutable evidence ledgers, every entry must be:

```text
retain-byte-for-byte
```

For terminal single-event ledgers, the only permitted decisions are:

```text
retain-byte-for-byte
canonical-json-representation
```

`canonical-json-representation` may change JSON text representation only. It must not change the parsed JSON value, logical order, terminal state, metrics, eligibility, reader behavior, or idempotency.

Reject the preview if it proposes deletion, deduplication, merging, splitting, conflict resolution, schema migration, default insertion, unknown-field removal, retention cleanup, or archive-boundary movement.

## 5. Reproducibility check

Run the preview twice against the same verified runtime state and backup, using two different empty output directories.

The following files must be byte-identical:

```text
quarantine-replacement-plan.json
quarantine-replacement-plan.digest
```

The proof may contain a different `generatedAt` value. Generated timestamps are outside the deterministic plan digest boundary.

## 6. Error handling

Stable error-code families:

```text
LB_QUARANTINE_REPLACEMENT_BACKUP
LB_QUARANTINE_REPLACEMENT_CHANGED
LB_QUARANTINE_REPLACEMENT_CONFLICT
LB_QUARANTINE_REPLACEMENT_CORRUPT
LB_QUARANTINE_REPLACEMENT_POLICY
LB_QUARANTINE_REPLACEMENT_PROOF
LB_QUARANTINE_REPLACEMENT_SEMANTICS
```

Treat every failure as fail-closed. Do not manually edit plan/proof artifacts and do not retry against a runtime state that changed without producing a new verified backup.

## 7. Safety boundary

QL-5C3B stops after verified plan/proof publication. It does not:

- rewrite active ledgers;
- rewrite archive segments;
- publish staging ledgers;
- create a transaction journal;
- perform rollback or recovery;
- delete retained evidence.

Transactional application and recovery belong to QL-5C3C and require a separate reviewed contract and implementation.
