# Quarantine Replacement Generation Directory

**Status: implemented by QL-5C3C** | **Last updated: 2026-07-14**

QL-5C3C publishes a verified replacement as a complete generation directory rather than replacing several active ledger paths independently.

## Layout

```text
<state-dir>/
├── quarantine-current-generation.json
├── quarantine-generations/
│   └── <transaction-id>/
│       ├── quarantine-replacement-generation.json
│       ├── quarantine-replacement-generation.digest
│       └── managed ledger files
├── legacy managed ledger files
├── quarantine-ledger-index.json
├── quarantine-segments.json
└── quarantine-segments/
```

When `quarantine-current-generation.json` is absent, existing root-level managed ledgers remain active. When the pointer exists, all managed active-ledger reads and writes resolve through the referenced generation directory.

## Atomic publication boundary

The generation directory is completely materialized, fsynced, and verified before publication. The reader-visible switch is one atomic rename of the current-generation pointer. Multiple ledger renames are never treated as collectively atomic.

The pointer binds:

```text
version
transactionId
generationDigest
```

The referenced generation manifest binds exact ledger membership, presence, byte length, line count, replacement count, and file digest.

## Fail-closed resolution

A present but invalid pointer is an error. A missing generation directory, mismatched generation digest, missing or unexpected ledger, or invalid transaction ID is an error. The resolver does not fall back to root-level ledgers after a pointer has been published.

## Legacy compatibility

Legacy root ledgers are not migrated or deleted merely by enabling generation resolution. The first successful publication switches readers and writers to the new generation. A pre-commit rollback removes or restores the exact previous pointer and therefore reactivates the prior generation, including the legacy root when no previous pointer existed.

## Recovery

The transaction journal and publication intent distinguish:

```text
before pointer switch
after pointer switch but before committed
committed
rolled back
contradictory or corrupt
```

Resume may repeat only idempotent steps. Rollback is allowed only before `committed`. A committed generation is terminal and can be superseded only by a new verified transaction.

See also:

```text
docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md
docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md
```
