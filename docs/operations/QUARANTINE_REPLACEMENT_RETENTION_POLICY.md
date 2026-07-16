# Quarantine Replacement Retention Policy

**Status: draft for v0.4.0** | **Policy version: `lingonberry-quarantine-replacement-retention-policy/v1`** | **Tracking issue: #62**

## 1. Purpose

This document defines the authorization boundary for removing inactive quarantine replacement generations and terminal replacement transaction workspaces.

A retention report is evidence for classification, not authorization for deletion. A subject becomes eligible only when every policy predicate, proof binding, runtime revalidation, and operator-consent requirement succeeds.

## 2. Managed subjects

The policy recognizes two independent subject types:

- `generation`: a directory under `quarantine-generations/<transaction-id>/`
- `transaction-workspace`: a replacement transaction directory supplied explicitly by the operator

Generation cleanup and workspace cleanup must remain separately selectable and separately evidenced. Eligibility of one does not imply eligibility of the other.

## 3. Required normalized policy inputs

A cleanup policy instance must contain:

```json
{
  "version": "lingonberry-quarantine-replacement-retention-policy/v1",
  "minimumPreviousCommittedGenerations": 1,
  "minimumAgeSeconds": 604800,
  "allowPreviousCommittedGenerations": true,
  "allowRolledBackGenerations": true,
  "allowCommittedTransactionWorkspaces": true,
  "allowRolledBackTransactionWorkspaces": true,
  "selectedSubjects": []
}
```

Rules:

- `minimumPreviousCommittedGenerations` must be at least `1` in v0.4.0.
- `minimumAgeSeconds` must be non-negative and must be evaluated from bound durable protocol metadata.
- `selectedSubjects` must identify exact subject type and transaction ID.
- wildcard, prefix, glob, implicit "all", and directory discovery selections are forbidden for apply.
- policy normalization must be deterministic before digest calculation.

## 4. Eligibility predicates

A selected subject is eligible only when all applicable predicates succeed.

### 4.1 Common predicates

- the subject was explicitly selected by the operator
- the subject resolves below the expected managed root
- every path component is a real directory entry and not a symbolic link
- the subject name is valid UTF-8 and exactly matches its bound transaction ID
- the subject appears exactly once in the plan
- the active pointer, journals, manifests, digests, inventories, and runtime fingerprint match the verified proof
- the same-host operation lock is held
- no replacement or cleanup transaction is concurrently mutating managed state
- minimum age is satisfied using bound durable metadata

### 4.2 Generation predicates

`previous-committed-generation` is eligible only when:

- the journal state is `committed`
- generation manifest and generation digest verify completely
- it is not referenced by the active pointer
- removing it does not violate `minimumPreviousCommittedGenerations`
- no selected workspace or retained evidence requires the generation for recovery explanation

`rolled-back-generation` is eligible only when:

- the journal state is `rolled-back`
- generation manifest and generation digest verify completely
- it is not referenced by the active pointer
- rollback and audit evidence remain available after cleanup

The following classifications are categorically ineligible:

- `active-committed-generation`
- `incomplete-transaction-generation`
- `orphan-unreferenced-generation`
- `unknown-or-corrupt`
- `legacy-root-layout`

### 4.3 Transaction workspace predicates

A committed workspace is eligible only when:

- its journal is terminal `committed`
- its committed generation is present and verifies, or later policy explicitly documents a safely retained equivalent evidence source
- no recovery operation can validly consume the workspace
- required audit and journal evidence is preserved outside the removable payload

A rolled-back workspace is eligible only when:

- its journal is terminal `rolled-back`
- rollback evidence remains sufficient after cleanup
- no generation or active pointer references workspace-local data

Non-terminal, unreadable, duplicate-ID, missing-journal, or ambiguous workspaces are ineligible.

## 5. Durable age source

Filesystem creation, modification, and access timestamps are not authoritative retention evidence.

For v0.4.0, age must be derived from a versioned durable timestamp already bound to verified transaction or generation metadata. The preview must identify the exact source field used for each subject. Apply must reject the proof if that source is absent, malformed, inconsistent, or changed.

If no acceptable durable timestamp exists for a subject, the subject is ineligible. Implementations must not fall back to filesystem timestamps.

## 6. Managed path inventory

The proof must contain a complete, deterministic inventory for each selected subject.

Each entry binds:

- normalized relative path
- entry type
- byte length for regular files
- content digest for regular files
- directory membership

Rules:

- absolute paths are forbidden in portable proof content
- `.` and `..` path components are forbidden
- symbolic links, hard-link ambiguity, devices, sockets, FIFOs, and unknown entry types are forbidden
- any extra, missing, changed, or reordered semantic inventory entry causes rejection
- apply must open and inspect entries without following symbolic links

## 7. Plan and proof

The cleanup preview produces:

- `lingonberry-quarantine-replacement-cleanup-plan/v1`
- `lingonberry-quarantine-replacement-cleanup-proof/v1`

The plan describes requested actions. The proof establishes that each requested action was eligible against an exact state snapshot.

The proof must bind at least:

- normalized policy and policy digest
- state layout
- active pointer bytes and digest, or explicit verified absence
- retention report version and normalized subject classifications
- journals and journal digests
- generation manifests and generation digests
- managed path inventories
- durable age sources
- runtime fingerprint
- plan digest
- proof digest

A proof is single-purpose. It may not authorize additional subjects or a different policy.

## 8. Apply-time revalidation

Before the first mutation, apply must:

1. acquire the same-host operation lock
2. verify proof and plan digests
3. resolve every subject without following symbolic links
4. re-read and compare the active pointer
5. re-read and compare all bound journals
6. reverify manifests and generation digests
7. rebuild and compare path inventories
8. recompute retention-floor eligibility across the complete current generation set
9. recompute durable age eligibility
10. compare the runtime fingerprint

Any mismatch is a preflight rejection. No partial selection is allowed: the entire apply request fails before mutation.

## 9. Mutation boundary

Cleanup mutation must use a dedicated transaction and journal.

The reversible phase may:

- create a transaction-local tomb directory on the same filesystem
- rename selected subjects into the tomb directory
- fsync renamed subjects, tomb directory, managed parent directories, and journal state

The irreversible phase begins before the first file or directory entry is deleted from the sealed tomb set.

After the irreversible phase begins:

- rollback must not be advertised
- interruption must resume deletion or report `recovery-required` / `partially-deleted`
- missing tomb entries must be reconciled against the sealed inventory, never assumed successful

## 10. Evidence preservation

Cleanup must preserve enough append-only evidence to answer:

- which operator-triggered transaction requested cleanup
- which policy and proof authorized it
- which exact subjects were selected
- which state snapshot was verified
- which durable boundaries completed
- whether deletion committed, rolled back, requires recovery, or partially completed

Metrics and audit records must not expose secrets, full paths, transaction IDs as unbounded labels, or free-form error text.

## 11. Operator consent

v0.4.0 requires explicit double opt-in:

- a verified proof naming exact subjects
- a separate apply invocation containing an explicit destructive-action acknowledgement

Interactive confirmation alone is insufficient for automation safety. A command must be unambiguously non-destructive unless the destructive acknowledgement is present.

## 12. Forbidden behavior

Implementations must not:

- delete the active generation
- delete archive segments
- mutate immutable evidence ledgers
- delete unknown, corrupt, orphan, incomplete, or legacy-root state
- infer eligibility from age alone
- follow symbolic links
- accept wildcard subject selection for apply
- apply stale or partially matching proofs
- silently skip failed subjects and report global success
- combine cleanup with replacement apply/resume/rollback
- enable background scheduled deletion in v0.4.0
- promise secure erase semantics

## 13. Review gate before destructive implementation

Destructive implementation may begin only after review accepts:

- the subject model
- eligibility predicates
- retention floor
- durable age source
- inventory and symlink rules
- proof binding
- apply-time revalidation
- reversible and irreversible boundaries
- partial deletion evidence semantics
