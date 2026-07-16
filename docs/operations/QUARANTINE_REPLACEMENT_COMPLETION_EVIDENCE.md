# Quarantine Replacement Completion Evidence

**Status: draft for v0.4.0** | **Record version: `lingonberry-quarantine-replacement-completion-evidence/v1`** | **Tracking issue: #65**

## 1. Purpose

This document defines the authoritative durable time source used by retention policy. Filesystem timestamps are not protocol evidence and must never authorize cleanup.

The completion evidence record is separate from the existing v1 replacement transaction journal. This preserves v0.3.0 compatibility and avoids silently changing the meaning or digest of an existing versioned record.

## 2. File contract

A terminal replacement transaction may publish:

```text
quarantine-replacement-completion-evidence.json
quarantine-replacement-completion-evidence.digest
```

The JSON record uses canonical JSON and the digest covers the exact canonical bytes.

```json
{
  "version": "lingonberry-quarantine-replacement-completion-evidence/v1",
  "transactionId": "<transaction-id>",
  "terminalState": "committed",
  "terminalSequence": 6,
  "completedAt": "2026-07-16T10:00:00Z",
  "journalDigest": "fnv1a64:<digest>",
  "generationDigest": "fnv1a64:<digest>"
}
```

For a rolled-back transaction with no durable generation, `generationDigest` is `null`.

## 3. Authoritative write boundary

The record is created only after the terminal journal state and all state required to explain that terminal state are durable.

Required sequence:

1. write and fsync the terminal journal
2. verify the terminal journal and its digest
3. verify the committed generation and generation digest when terminal state is `committed`
4. construct canonical completion evidence
5. write the record to a temporary file and fsync it
6. write and fsync the digest
7. rename both artifacts into their final names using the existing single-file atomic publication rules
8. fsync the transaction directory
9. re-read and verify the published record

A completion record must not be written before the transaction reaches a terminal state.

## 4. Verification predicates

Verification succeeds only when:

- version is exactly supported
- transaction ID matches the containing transaction journal
- terminal state is `committed` or `rolled-back`
- terminal state and terminal sequence match the journal
- `completedAt` is a canonical UTC timestamp
- `completedAt` is not later than the verifier's bound evaluation time
- journal digest matches the current verified journal bytes
- committed evidence binds the current verified generation digest
- rolled-back evidence does not claim an unrelated generation digest
- record digest matches the canonical record bytes
- neither artifact is a symbolic link or unexpected entry type

Any mismatch is fail-closed.

## 5. Retention age calculation

Retention preview binds an explicit evaluation timestamp. Durable age is calculated as:

```text
evaluationTime - completedAt
```

The calculation must reject:

- missing completion evidence
- malformed or non-UTC timestamps
- future timestamps
- arithmetic overflow
- journal or generation mismatch
- unsupported record versions

The evaluation timestamp, completion timestamp, evidence digest, and resulting age seconds are included in the cleanup proof.

## 6. v0.3.0 compatibility

Existing v0.3.0 transactions do not contain authoritative completion evidence. They remain readable and valid replacement transactions, but retention preview reports:

```text
durable-age-evidence-missing
```

No implementation may infer or backfill completion time from:

- file modification, creation, or access times
- directory age
- Git commit or release time
- audit ingestion time
- first inspection time
- operator-supplied unverified timestamps

A future backfill requires a separate versioned policy and verified evidence source. It is not part of the initial v0.4.0 cleanup path.

## 7. Crash semantics

- crash before terminal journal durability: no completion evidence is valid
- crash after terminal journal durability but before completion evidence publication: transaction remains terminal but cleanup-ineligible until evidence publication is safely resumed
- crash during temporary evidence writes: final artifacts remain absent and cleanup is ineligible
- crash after one final artifact exists without the other: verification fails closed and repair/recovery is required
- crash after directory fsync and successful verification: evidence is durable and may be consumed by read-only retention preview

Evidence publication recovery must be idempotent. Existing matching final artifacts are accepted; conflicting artifacts are never overwritten silently.

## 8. Safety invariants

1. Filesystem timestamps never authorize cleanup.
2. Completion evidence never changes an existing v1 journal digest.
3. The record is not evidence of eligibility by itself; all retention predicates still apply.
4. Missing evidence is an explicit ineligible state, not an operational success.
5. A future-dated record is rejected.
6. A committed record without a verified generation binding is rejected.
7. Conflicting final artifacts require recovery and are not replaced automatically.
8. Cleanup apply revalidates the evidence under the same-host operation lock.