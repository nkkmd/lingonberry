# Quarantine Replacement Completion Evidence

**Status: implemented** | **v1.0 pre-release normative operations contract**

This document defines the durable evidence record used to bind a terminal quarantine replacement transaction to its journal state, optional committed generation, and retention-age calculation. Filesystem timestamps are not authoritative evidence and must never authorize cleanup.

## Record identity

```text
lingonberry-quarantine-replacement-completion-evidence/v1
```

The completion-evidence record is separate from the replacement transaction journal. It does not alter the journal schema or digest and does not retroactively add evidence to older transactions.

## Artifact pair

A terminal transaction may publish:

```text
quarantine-replacement-completion-evidence.json
quarantine-replacement-completion-evidence.digest
```

The JSON artifact is canonical JSON. The digest is an FNV-1a 64-bit integrity digest over the exact canonical bytes:

```text
fnv1a64:<16 lowercase or uppercase hexadecimal digits>
```

The digest detects artifact mismatch. It is not a digital signature, MAC, trusted timestamp, authorization token, or independent provenance proof.

## Record fields

The implemented record shape is:

```json
{
  "version": "lingonberry-quarantine-replacement-completion-evidence/v1",
  "transactionId": "<transaction-id>",
  "terminalState": "committed",
  "terminalSequence": 5,
  "completedAtUnixSeconds": 1780000000,
  "journalDigest": "fnv1a64:<digest>",
  "generationDigest": "fnv1a64:<digest>"
}
```

For a rolled-back transaction without a committed generation, `generationDigest` is `null`.

`completedAtUnixSeconds` is an unsigned Unix-seconds value. The implementation does not store an RFC 3339 timestamp in this artifact.

## Terminal-state boundary

Completion evidence is valid only for:

```text
committed
rolled-back
```

The record binds to the exact terminal state and terminal sequence from the verified transaction journal. Non-terminal states are rejected.

A committed record must bind the expected verified generation digest. A rolled-back record must match the expected absence or presence of a generation digest supplied by the verified terminal context; the verifier compares exact expected values rather than inferring intent from the state name alone.

## Publication procedure

Publication is an explicit transaction-directory operation:

1. construct the completion-evidence value from verified terminal state;
2. canonicalize the JSON;
3. compute the integrity digest;
4. create and sync the temporary JSON file with exclusive-create semantics;
5. create and sync the temporary digest file with exclusive-create semantics;
6. rename the JSON temporary file to its final name;
7. rename the digest temporary file to its final name; and
8. sync the transaction directory.

The implementation publishes two files with separate renames. It does not provide a cross-file atomic rename. A crash between renames can leave a partial final pair, which must fail closed and receive manual recovery review.

Publication does not silently overwrite an existing pair.

- An existing exact pair is accepted idempotently.
- A partial pair is rejected.
- A conflicting pair is rejected.
- A stale temporary artifact is rejected and requires manual review.

Temporary artifacts are removed on an error encountered by the current publication attempt where possible, but pre-existing stale temporary files are not automatically replaced.

## Artifact verification

Artifact verification must establish that:

- neither expected artifact is missing;
- each path is an ordinary file rather than an unexpected entry type;
- the digest file matches the canonical evidence bytes;
- the JSON parses into the supported version and exact field contract; and
- no unsupported or malformed value is accepted.

The artifact verifier and semantic verifier are separate layers. Matching bytes alone do not prove that the evidence agrees with the current journal and generation.

## Semantic verification predicates

Semantic verification accepts explicit expected values from already verified transaction context. It succeeds only when:

- `transactionId` is a non-empty ASCII path-safe identifier and equals the expected transaction ID;
- `terminalState` is `committed` or `rolled-back` and equals the expected state;
- `terminalSequence` equals the expected journal sequence;
- `journalDigest` has the supported FNV-1a shape and equals the expected verified journal digest;
- `generationDigest`, when present, has the supported shape;
- `generationDigest` exactly equals the expected generation digest, including `null` versus non-`null`; and
- `completedAtUnixSeconds` is not greater than the supplied evaluation time.

Any mismatch fails with the completion-evidence error family. Verification does not repair, rewrite, or backfill evidence.

## Durable-age calculation

The semantic verifier computes:

```text
durableAgeSeconds = nowUnixSeconds - completedAtUnixSeconds
```

A future completion value is rejected before subtraction. The verifier receives the evaluation time explicitly; callers that need reproducible retention evidence must persist that evaluation time in the retention decision and cleanup proof.

The completion-evidence report contains:

- transaction ID;
- terminal state;
- terminal sequence;
- completion Unix seconds; and
- durable age seconds.

## Retention and cleanup boundary

Completion evidence supplies one durable age input. It does not by itself make a generation cleanup-eligible.

Retention and cleanup must separately verify all applicable conditions, including:

- completion-evidence artifact and semantic validity;
- replacement completion and generation state;
- active-generation exclusion;
- retention-policy age threshold;
- exact candidate generation selection;
- cleanup plan and proof;
- current-state revalidation; and
- required operator acknowledgements.

Cleanup preparation must revalidate completion evidence under the same-host quarantine operation lock according to the cleanup execution contract.

## Older transactions and missing evidence

Transactions created before completion-evidence publication support may remain readable and valid as replacement transactions while lacking authoritative cleanup age.

Missing completion evidence must be classified explicitly as cleanup-ineligible, such as:

```text
durable-age-evidence-missing
```

The implementation must not infer completion time from:

- file creation, modification, or access times;
- directory age;
- Git commit, release, or deployment time;
- audit ingestion time;
- first inspection time; or
- an unverified operator-supplied timestamp.

Any future backfill requires a separate versioned policy and trusted evidence source.

## Crash and recovery semantics

- Before terminal journal durability, no completion evidence is valid.
- After terminal journal durability but before evidence publication, the transaction remains terminal but cleanup-ineligible.
- A stale temporary artifact requires manual review before publication can proceed.
- A partial final artifact pair fails closed.
- An exact existing final pair makes publication idempotent.
- A conflicting existing final pair is never overwritten silently.
- A synced and semantically verified pair may be consumed by retention preview.

The presence of both final files is not sufficient without digest and semantic verification.

## Security and trust boundary

Completion evidence binds local versioned artifacts through checked values. It does not establish:

- the identity of the operator who caused the terminal transition;
- an externally trusted completion timestamp;
- tamper resistance against an actor able to rewrite the evidence, journal, generation, and all matching digests;
- remote attestation;
- distributed consensus; or
- authorization to delete retained generations.

Protect transaction directories, evidence files, journals, generation manifests, and backups with deployment-level access controls and evidence retention policy.

## Operational evidence

For qualification or cleanup authorization, retain:

- application commit and binary identity;
- transaction ID and transaction-directory path;
- verified terminal journal and digest;
- verified generation manifest and digest for committed transactions;
- completion-evidence JSON and digest;
- semantic verification report and evaluation time;
- retention decision and cleanup plan/proof;
- operator acknowledgements; and
- any recovery actions for stale, partial, or conflicting artifacts.

## Non-goals

The v1.0 completion-evidence contract does not provide:

- automatic publication for every historical transaction;
- automatic timestamp backfill;
- cleanup eligibility by itself;
- cryptographic signatures or trusted timestamping;
- multi-file atomic publication;
- automatic repair of partial or conflicting artifact pairs;
- cross-host locking or distributed coordination; or
- proof of formal soak or privileged reference-host qualification.

## Related documents

- [`QUARANTINE_REPLACEMENT_POLICY.md`](./QUARANTINE_REPLACEMENT_POLICY.md)
- [`QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
- [`QUARANTINE_CONCURRENCY.md`](./QUARANTINE_CONCURRENCY.md)

## Release boundary

This normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence remains unperformed, privileged reference-host qualification/rehearsal remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.
