# Quarantine Replacement Policy and Semantic-equivalence Contract

**Status: implemented** | **v1.0 pre-release normative operations contract**

This document defines the policy boundary for quarantine ledger replacement planning, proof generation, staging, publication, recovery, and later cleanup. Replacement is a controlled representation change; it is not retention deletion, conflict repair, deduplication, or permission to rewrite immutable evidence.

## Policy identity

```text
lingonberry-quarantine-compaction-policy/v2
```

Policy v2 is intentionally distinct from compaction policy v1. Policy v1 proves that no managed-ledger line is removable. Policy v2 permits only one-to-one canonical representation replacement for terminal single-event ledgers, subject to the checked-in planning, verification, transaction, publication, and recovery contracts.

## Managed-ledger classification

| Ledger | Classification | Replacement permission |
|---|---|---|
| `quarantine.jsonl` | immutable source evidence | forbidden |
| `quarantine-annotations.jsonl` | immutable reviewer evidence | forbidden |
| `admin-auth-audit.jsonl` | immutable security audit evidence | forbidden |
| `quarantine-resolutions.jsonl` | terminal single-event evidence | canonical representation replacement only |
| `quarantine-dismissals.jsonl` | terminal single-event evidence | canonical representation replacement only |
| `quarantine-rejections.jsonl` | terminal single-event evidence | canonical representation replacement only |

The exact managed-ledger set is closed. Unknown, missing, or duplicate ledger entries are verification failures. Absence of explicit permission means replacement is forbidden.

## Permitted transformation

For each terminal single-event source line, the replacement may change only JSON representation:

- insignificant whitespace;
- object-key ordering produced by the canonical serializer; and
- the canonical line terminator used by the staged ledger.

The parsed JSON value must remain identical. No field, array element, number, string, boolean, or null value may be added, removed, inferred, defaulted, or changed.

The source-to-replacement mapping must be one-to-one and order preserving. Each source line maps to exactly one replacement line, and the nth logical source event maps to the nth logical replacement event for that ledger.

## Replacement identity and corruption

The terminal replacement key is:

```text
<ledger-name> + "\u0000" + <quarantineId>
```

The key must be unique across the complete archive-aware logical stream. Duplicate terminal events are corruption. Planning and apply operations must not deduplicate them, choose a winner, or convert them into a valid replacement.

Conflicting terminal states likewise remain corruption or lifecycle conflict according to the underlying ledger contract. Replacement must not conceal or repair them.

## Archive-aware logical order

The authoritative logical order is:

```text
verified archive segments in manifest order
→ active ledger
```

Planning verifies the segment manifest and immutable segments before reading the logical stream. A replacement plan records source location and line number so each replacement can be traced to one exact logical source line.

## Plan and proof artifacts

Read-only preview produces:

```text
quarantine-replacement-plan.json
quarantine-replacement-plan.digest
quarantine-replacement-proof.json
quarantine-replacement-proof.digest
```

The current versions are implementation-defined constants exposed by the core library. The plan records the exact managed-ledger set, runtime fingerprint, source backup-manifest digest, optional segment-manifest digest, per-ledger entries, and semantic-equivalence expectations.

The proof binds to the canonical plan digest and records source, replacement, and retained line totals together with:

```text
mutationAllowed: false
rewritePerformed: false
```

Preview and proof verification do not mutate runtime state.

## Provenance entries

A terminal replacement entry records sufficient information to bind one logical source line to one canonical replacement line, including:

- ledger name;
- logical ordinal;
- replacement key;
- source segment or active-ledger location;
- source line number;
- source-line digest;
- replacement-line digest;
- source-value digest;
- replacement-value digest; and
- transformation identifier.

The transformation identifier is:

```text
canonical-json-representation
```

Source-value and replacement-value digests are computed from canonical serialization of the parsed values and must match.

Immutable-evidence ledger entries are retained, not replacement candidates.

## Preview preconditions

Replacement preview requires:

1. successful archive-segment verification;
2. a verified archive-inclusive quarantine backup v2;
3. an empty artifact output directory;
4. the exact managed-ledger set;
5. valid JSON for every logical line;
6. unique terminal replacement keys; and
7. a stable runtime fingerprint before and after scanning.

Preview does not acquire the mutation lock. It detects observed changes by comparing runtime fingerprints before and after reading. Operators producing qualification or maintenance evidence must still quiesce writers; a matching before-and-after fingerprint is not a transactional snapshot guarantee.

## Proof verification boundary

Artifact verification checks digest pairs, supported versions, exact ledger structure, mapping completeness, canonical replacement values, line counts, ordering, and the declared semantic-equivalence fields.

Verification of an artifact directory does not by itself re-read the source runtime state, re-verify the original backup directory, or prove that the runtime still matches the plan. Apply preparation must revalidate plan inputs against current state.

The repository integrity digest detects accidental or uncoordinated artifact changes. It is not a digital signature, MAC, trusted timestamp, authorization token, or independent provenance proof.

## Semantic-equivalence requirements

A conforming replacement must preserve all of the following across the complete logical state:

- managed-ledger membership;
- parsed values and logical order;
- quarantine record identity;
- promoted, dismissed, permanently rejected, and pending classifications;
- annotations and immutable audit evidence;
- status output;
- state-derived metrics;
- promotion eligibility and rejection reasons;
- idempotent repeated terminal actions;
- conflicting-action outcomes;
- batch outcome classification;
- public and administrator ordered-reader results; and
- corruption behavior.

Byte-for-byte equality is required for immutable evidence. Terminal replacement lines require parsed-value equality and one-to-one provenance, not byte equality.

Replacement must not introduce a new key, remove an existing key, reorder events, or change a terminal disposition.

## Apply and publication boundary

Mutation-capable replacement is implemented as a transaction and generation publication workflow rather than in-place truncation of the active ledgers.

Apply preparation and publication must:

- acquire the same-host quarantine operation lock where required by the implementation;
- revalidate the plan, proof, runtime fingerprint, backup inputs, and segment state;
- stage the complete replacement generation;
- verify staged ledger and semantic-equivalence artifacts;
- seal generation metadata and digests;
- persist transaction and publication intent state;
- publish or switch the current-generation pointer only through the checked-in transition path; and
- expose resume, rollback, or recovery-required state according to the durable journal.

The active-generation pointer is deployment state. Its validation and transition rules do not provide distributed consensus or cross-host atomicity.

## Interruption and recovery

Operators must inspect the durable replacement transaction state before retrying an interrupted operation. They must not infer success solely from the presence or absence of a generation directory or pointer file.

Rollback is available only where the transaction contract explicitly permits it. Once publication has crossed a non-reversible boundary, the supported action is resume or manual recovery, not an advertised automatic rollback.

Failure injection and automated tests exercise transition boundaries, but passing tests do not replace host qualification or an operator evidence bundle.

## Replacement completion evidence

Completion evidence is a separate verified artifact. It binds the completed transaction, published generation, input and output artifacts, terminal state, and relevant digests. It must be verified before a generation is considered eligible for the separate retention and cleanup workflow.

Completion evidence does not authorize deletion by itself.

## Cleanup separation

Replacement cleanup is governed by separate retention-decision, cleanup-plan/proof, tomb preparation, deletion acknowledgement, journal, rollback, and recovery contracts.

Representation replacement and retention deletion must never be combined into one implicit operation. The active generation must not be selected for cleanup.

## Explicitly forbidden behavior

The v1.0 replacement policy does not permit:

- deleting valid history as part of replacement;
- merging or splitting events;
- deduplicating terminal events;
- resolving conflicts by selecting one event;
- changing `quarantineId` or terminal metadata;
- rewriting immutable-evidence ledgers;
- inferring missing fields or applying schema defaults;
- discarding unknown fields;
- changing logical order;
- publishing an unverified staged generation;
- unattended or scheduled replacement;
- automatic cleanup of previous generations;
- distributed locking, leader election, or multi-node consensus; or
- treating a digest file as operator authorization.

## Operational evidence

A replacement evidence bundle should retain:

- application commit and binary identity;
- verified backup v2 and its manifest;
- segment verification output;
- plan, proof, and digest files;
- runtime fingerprint and input metadata;
- staging verification output;
- transaction journal and digest;
- publication intent and generation manifest artifacts;
- completion evidence and digest;
- pre- and post-operation status and metrics; and
- operator decisions, timestamps, and recovery actions.

Evidence completeness does not establish formal soak or privileged reference-host qualification unless those procedures were separately executed and recorded.

## Related documents

- [`QUARANTINE_COMPACTION_PROOF.md`](./QUARANTINE_COMPACTION_PROOF.md)
- [`QUARANTINE_CONCURRENCY.md`](./QUARANTINE_CONCURRENCY.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)
- [`QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md`](./QUARANTINE_REPLACEMENT_COMPLETION_EVIDENCE.md)
- [`QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md`](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)

## Release boundary

This normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence remains unperformed, privileged reference-host qualification/rehearsal remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.
