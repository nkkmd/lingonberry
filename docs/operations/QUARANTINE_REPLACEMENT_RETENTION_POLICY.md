# Quarantine Replacement Retention Policy

**Status: normative v1.0 pre-release operations contract** | **Policy version: `lingonberry-quarantine-replacement-retention-policy/v1`** | **Last reviewed: 2026-07-24**

This document defines the implemented authorization boundary for selecting inactive quarantine replacement generations for cleanup. A retention decision report is classification evidence only. It does not delete data, create a cleanup transaction, or authorize irreversible deletion by itself.

The v1 policy evaluates **generation subjects only**. It does not select or authorize deletion of replacement transaction workspaces, cleanup transaction workspaces, archive segments, legacy root ledgers, or unrelated runtime files.

## 1. Policy model

The implementation evaluates a normalized policy with these fields:

```text
minimumPreviousCommittedGenerations
minimumAgeSeconds
allowPreviousCommittedGenerations
allowRolledBackGenerations
selectedGenerationIds
```

The policy version is fixed by the implementation as:

```text
lingonberry-quarantine-replacement-retention-policy/v1
```

Rules:

- `minimumPreviousCommittedGenerations` must be at least `1`;
- `minimumAgeSeconds` is an unsigned duration;
- `selectedGenerationIds` must contain at least one exact generation ID;
- duplicate selections are rejected;
- empty IDs, `.`, `..`, path separators, wildcards, glob syntax, and bracket expressions are rejected;
- selection is exact and deterministic; there is no prefix, wildcard, discovery, or implicit-all mode;
- generation and workspace cleanup are not interchangeable.

## 2. Candidate model

Each retention candidate contains:

```text
generationId
classification
terminalTransactionState
verificationStatus
durableAgeSeconds
```

Candidate generation IDs must be unique. Duplicate candidate IDs make the evaluation fail closed.

The implemented classifications are:

```text
previous-committed-generation
rolled-back-generation
active-committed-generation
incomplete-transaction-generation
orphan-unreferenced-generation
unknown-or-corrupt
legacy-root-layout
```

Unsupported classifications are ineligible.

## 3. Eligible classifications

### 3.1 Previous committed generation

A `previous-committed-generation` is provisionally eligible only when:

- `allowPreviousCommittedGenerations` is true;
- `terminalTransactionState` is exactly `committed`;
- `verificationStatus` is exactly `verified`;
- durable age evidence is present;
- `durableAgeSeconds` is at least `minimumAgeSeconds`.

After provisional evaluation, the retention floor is applied across the complete candidate set. The evaluator counts every candidate classified as `previous-committed-generation` and permits removal of no more than:

```text
previous committed total - minimumPreviousCommittedGenerations
```

The selected generation IDs are normalized into sorted order. When more selected previous committed generations are provisionally eligible than the retention floor permits, only the first permitted IDs in that deterministic order remain eligible; the rest receive:

```text
minimum-retention-floor
```

The retention floor protects previous committed generations only. It does not imply that rolled-back generations are safe to remove.

### 3.2 Rolled-back generation

A `rolled-back-generation` is eligible only when:

- `allowRolledBackGenerations` is true;
- `terminalTransactionState` is exactly `rolled-back`;
- `verificationStatus` is exactly `verified`;
- durable age evidence is present;
- `durableAgeSeconds` is at least `minimumAgeSeconds`.

Rolled-back generations are not included in `minimumPreviousCommittedGenerations`.

## 4. Categorically ineligible classifications

The evaluator rejects:

| Classification | Reason code |
|---|---|
| `active-committed-generation` | `active-generation` |
| `incomplete-transaction-generation` | `non-terminal-transaction` |
| `orphan-unreferenced-generation` | `orphan-requires-manual-review` |
| `unknown-or-corrupt` | `unknown-or-corrupt` |
| `legacy-root-layout` | `legacy-root-layout` |
| unsupported classification | `unsupported-classification` |

A missing selected candidate is represented by an ineligible decision with:

```text
subject-not-found
```

The policy must not convert orphan, incomplete, corrupt, unknown, or legacy-root state into an automatically deletable subject.

## 5. Other rejection reasons

The evaluator uses stable reason codes including:

```text
classification-disabled-by-policy
terminal-state-mismatch
generation-not-verified
durable-age-evidence-missing
minimum-age-not-satisfied
minimum-retention-floor
eligible
```

Eligibility is all-or-nothing per selected generation. Age alone never establishes eligibility.

## 6. Durable age evidence

Filesystem creation, modification, and access timestamps are not authoritative policy inputs.

`durableAgeSeconds` must be derived by the generation inspection and evidence pipeline from durable transaction or completion metadata. The retention evaluator consumes the supplied value; it does not independently inspect filesystem timestamps or reconstruct provenance.

When durable age evidence is absent, the candidate is ineligible. The evaluator does not fall back to file metadata.

## 7. Decision report

Evaluation produces:

```text
lingonberry-quarantine-replacement-retention-decision-report/v1
```

Each decision contains:

```text
generationId
classification
eligible
reasonCode
```

The report includes one decision for every exact selected generation ID. It is evidence for the next preview stage, not deletion authority.

The report does not contain a cleanup transaction, path inventory, destructive acknowledgement, tomb manifest, deletion progress, or secure-erasure claim.

## 8. Cleanup preview binding

The cleanup preview accepts a retention decision report only when its policy version matches the implemented v1 policy.

The plan builder requires at least one eligible generation. It also requires the supplied subject set to bind the **entire eligible generation set** exactly:

- a subject not marked eligible is rejected;
- duplicate subjects are rejected;
- omission of any eligible generation is rejected;
- subject order is normalized by generation ID;
- each subject binds its classification, transaction journal digest, generation digest, completion evidence digest, and exact managed-path inventory;
- all digests use the implemented `fnv1a64:<16 hex>` integrity format.

A retention decision report therefore cannot be reused to authorize a different subset after preview construction.

## 9. State-backed preview verification

The state-backed preview builder requires:

- a real state directory;
- a regular current-generation pointer file;
- a real, unique transaction directory for each selected generation;
- exact transaction ID and generation ID agreement;
- terminal journal state matching the classification;
- active transaction ID and active generation digest exclusion;
- bound journal and completion-evidence digest files;
- completion evidence verification;
- exact managed-path inventory without symlinks or unsupported file types;
- normalized relative paths only.

For a `previous-committed-generation`, the sealed generation is verified and its digest must match the expected generation digest.

For a `rolled-back-generation`, the builder verifies the rolled-back journal state, completion evidence, bound digest, active-pointer exclusion, and exact managed inventory. It does not claim that the rolled-back generation is an active or committed generation.

The preview binds:

```text
stateIdentity
activePointerDigest
runtimeFingerprint
policyVersion
subjects
```

The plan and proof formats are:

```text
lingonberry-quarantine-replacement-cleanup-plan/v1
lingonberry-quarantine-replacement-cleanup-proof/v1
```

## 10. Apply-time boundary

A verified retention report and cleanup proof remain insufficient by themselves for deletion.

Cleanup apply must separately enforce the cleanup transaction contract, including:

- the host-local operation lock;
- proof and plan verification;
- active-pointer and runtime-state revalidation;
- exact managed-path inventory revalidation;
- generation and completion-evidence bindings;
- a dedicated cleanup journal;
- a reversible tomb-preparation phase;
- explicit destructive acknowledgement before irreversible deletion;
- progress evidence and fail-closed recovery.

No partial selection may be silently skipped and reported as global success.

## 11. Transaction workspace retention

Replacement transaction workspaces and cleanup transaction workspaces are outside the implemented retention-policy subject model.

This policy does not define fields such as:

```text
allowCommittedTransactionWorkspaces
allowRolledBackTransactionWorkspaces
selectedWorkspaces
```

Terminal workspaces, journals, completion evidence, cleanup proofs, tomb inventories, and path-level deletion progress remain retained unless a separate future policy and implementation explicitly authorize their retirement.

## 12. Forbidden behavior

The retention policy and its consumers must not:

- select or delete the active generation;
- delete archive segments or mutate immutable evidence ledgers;
- treat legacy root layout as a generation cleanup subject;
- automatically delete orphan, unknown, corrupt, incomplete, or unsupported state;
- infer eligibility from age alone;
- follow symbolic links;
- accept wildcard or implicit-all selection;
- use a decision report for subjects not fully bound into its cleanup preview;
- combine replacement apply, resume, rollback, and cleanup into one transaction;
- schedule background deletion as an implied consequence of eligibility;
- promise secure erase semantics;
- treat FNV-1a digests as signatures, MACs, or trusted provenance.

## 13. Non-goals

The v1 retention policy does not provide:

- transaction-workspace retirement;
- cleanup-workspace retirement;
- archive retention or compaction;
- automated orphan repair;
- distributed cleanup coordination;
- remote storage deletion;
- cryptographic operator authorization;
- retention scheduling;
- secure deletion;
- release qualification or release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
