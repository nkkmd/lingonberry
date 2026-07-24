# Quarantine Replacement Generation Contract

**Status: implemented** | **v1.0 pre-release normative operations contract**

This document defines how a verified quarantine replacement transaction is sealed as a complete generation and later made reader-visible through the versioned current-generation pointer. Generation publication avoids treating several independent ledger renames as one atomic operation.

## Version identities

```text
lingonberry-quarantine-replacement-generation/v1
lingonberry-quarantine-current-generation/v1
```

The checked-in constants are authoritative for the accepted manifest and pointer versions.

## State layout

After a generation has been published into state, the relevant layout is:

```text
<state-dir>/
├── quarantine-current-generation.json
├── quarantine-generations/
│   └── <transaction-id>/
│       ├── quarantine-replacement-generation.json
│       ├── quarantine-replacement-generation.digest
│       └── managed ledger files
├── legacy root-level managed ledger files
├── quarantine-ledger-index.json
├── quarantine-segments.json
└── quarantine-segments/
```

During transaction preparation, the sealed generation first exists under the transaction workspace:

```text
<transaction-dir>/publication/
```

Sealing and state publication are separate operations. The existence of a transaction-local `publication/` directory does not mean the generation is active.

## Legacy root behavior

When `quarantine-current-generation.json` is absent, managed active-ledger reads and writes resolve to the legacy root-level ledger paths.

When a valid pointer exists, managed active-ledger reads and writes resolve through the referenced generation. A present but invalid pointer is an error; resolution must not silently fall back to legacy root ledgers.

Publishing the first generation does not automatically delete, migrate, or compact the legacy root files. They remain retained state until governed by a separate verified retention and cleanup process.

## Sealing preconditions

A transaction may be sealed only when its durable journal is in the `verified` state.

The sealer reads the staging manifest and requires:

- the supported staging-manifest version;
- a transaction ID matching the transaction journal;
- the exact managed-ledger set;
- no missing or unexpected staging entries;
- consistent `present`, byte-count, line-count, replacement-count, and digest metadata; and
- staged bytes that still match their recorded digests.

A staged ledger that changes after verification causes sealing to fail closed.

## Generation manifest

The transaction-local generation contains:

```text
publication/
├── quarantine-replacement-generation.json
└── quarantine-replacement-generation.digest
```

The manifest binds:

- version;
- transaction ID;
- source directory identifier;
- exact managed-ledger membership;
- per-ledger presence;
- byte length;
- logical line count;
- replacement count; and
- ledger digest when present.

Absent ledgers must have zero bytes, zero lines, zero replacements, and a null digest.

The source directory is the checked-in staging directory identifier. Generation verification re-reads the staging files and confirms that the manifest still describes the exact staged data.

## Generation digest

The generation digest covers the canonical generation-manifest bytes. It is persisted in:

```text
quarantine-replacement-generation.digest
```

The current implementation uses the repository's FNV-1a 64-bit integrity-digest format. It detects mismatched manifest bytes but is not a digital signature, MAC, trusted timestamp, authorization token, or tamper-proof provenance mechanism.

Protect the transaction workspace, staged ledgers, manifest, digest, and state publication paths with deployment-level access controls.

## Durability sequence

Generation sealing performs the following durability operations:

1. require a verified transaction journal;
2. validate the staging manifest and staged ledger files;
3. create a new transaction-local `publication/` directory;
4. sync the transaction directory;
5. create and sync the canonical generation manifest;
6. create and sync the generation digest;
7. sync the publication directory; and
8. re-read and verify the sealed generation.

The publication directory must not already exist. The sealer does not silently overwrite or merge a previous generation attempt.

## Verification boundary

Generation verification checks:

- transaction journal readability and transaction ID agreement;
- manifest and digest availability;
- supported generation version;
- supported source-directory identifier;
- exact managed-ledger set;
- staging directory membership;
- present/absent metadata consistency;
- staged byte length; and
- staged byte digest.

Verification currently depends on the transaction's staging directory remaining available and unchanged. The manifest/digest pair alone is not a self-contained copy of all ledger bytes at this stage.

A successful generation report contains the publication directory, manifest path, digest path, generation digest, and ledger count. It does not activate the generation.

## State publication boundary

State publication copies or moves the verified generation into the versioned generation directory according to the replacement transaction workflow, prepares a publication intent, and switches reader-visible state through the current-generation pointer.

The current-generation pointer binds exactly:

```text
version
transactionId
generationDigest
```

Pointer validation confirms the supported pointer version and exact expected transaction ID and generation digest. Pointer validation alone does not verify all generation files; callers must also verify the referenced generation through the applicable publication and resolver path.

## Atomic switch

The reader-visible boundary is the atomic rename of one prepared current-generation pointer file. Multiple managed-ledger renames are not treated as collectively atomic.

Before the pointer switch, readers continue to use the previous active generation or legacy root layout. After a valid pointer switch, readers resolve all managed active-ledger paths through the referenced generation.

The pointer rename is local-filesystem atomicity only. It does not provide:

- cross-filesystem atomicity;
- distributed consensus;
- replication ordering;
- cross-host fencing; or
- network-filesystem lease semantics.

## Fail-closed resolution

The resolver must fail when any of the following applies:

- pointer JSON is malformed;
- pointer version is unsupported;
- transaction ID is invalid or unexpected;
- generation digest differs from the expected verified digest;
- referenced generation directory is missing;
- generation manifest or digest is missing or inconsistent;
- managed-ledger membership differs from the exact set;
- a required ledger is missing;
- an unexpected ledger is present; or
- ledger bytes differ from sealed metadata.

Once a pointer is present, invalid generation state must not cause fallback to root-level ledgers.

## Failure and recovery

Failure injection covers generation-manifest write and durability boundaries. If generation durability fails after the transaction-local publication directory has been created, the checked-in recovery path attempts to:

- remove the incomplete publication directory;
- sync the transaction directory; and
- advance a transaction still in `verified` state to `recovery-required`.

Operators must inspect the durable journal and filesystem rather than infer success from the presence or absence of `publication/`.

The broader transaction and publication-intent workflow distinguishes:

```text
before pointer switch
after pointer switch but before committed
committed
rolled-back
recovery-required or contradictory state
```

Resume may repeat only documented idempotent steps. Rollback is available only where the transaction state machine explicitly permits it. A committed generation is terminal; changing active state afterward requires another verified transaction or a separate supported recovery procedure.

## Backup and cleanup

A published generation, its manifest/digest, current pointer, related transaction journal, publication intent, completion evidence, and retained legacy state are operationally related evidence.

Do not delete a previous or rolled-back generation merely because it is no longer active. Cleanup requires:

- verified completion evidence;
- explicit durable-age evaluation;
- a retention decision;
- an exact cleanup plan/proof;
- active-generation exclusion;
- same-host locked revalidation; and
- separate irreversible-delete confirmation.

## Operational evidence

Retain:

- application commit and binary identity;
- transaction ID;
- verified staging manifest and ledger digests;
- generation manifest and digest;
- generation-verification report;
- publication intent and digest;
- previous and new pointer bytes;
- transaction journal and digest;
- completion evidence and digest;
- pre- and post-publication status and metrics; and
- any recovery or rollback decisions.

## Non-goals

The v1.0 generation contract does not provide:

- automatic migration or deletion of legacy root ledgers;
- activation merely by sealing `publication/`;
- a cryptographic signature over the generation;
- self-contained verification after staging has been removed unless the later publication format provides the required bytes;
- multi-host or distributed atomic publication;
- automatic failover to root ledgers after pointer corruption;
- automatic retention cleanup; or
- proof of formal soak or privileged reference-host qualification.

## Related documents

- [`QUARANTINE_REPLACEMENT_POLICY.md`](./QUARANTINE_REPLACEMENT_POLICY.md)
- [`QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md`](./QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md)
- [`QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
- [`QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)

## Release boundary

This normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence remains unperformed, privileged reference-host qualification/rehearsal remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.
