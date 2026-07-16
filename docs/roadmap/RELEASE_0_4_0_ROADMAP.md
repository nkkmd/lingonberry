# v0.4.0 Roadmap: Verified Retention Cleanup

**Status: active** | **Target release: v0.4.0** | **Started: 2026-07-16** | **Tracking issue: #62**

## 1. Purpose

v0.4.0 introduces a verified, operator-controlled lifecycle for removing inactive quarantine replacement generations and terminal transaction workspaces.

v0.3.0 deliberately stopped at read-only retention inspection. v0.4.0 begins from that inspection boundary and adds policy, deterministic preview/proof, transactional apply/recovery, and operational hardening without weakening the verified replacement transaction.

The first release remains operator-triggered. Background or schedule-driven automatic deletion is not part of v0.4.0.

## 2. Starting point

The following v0.3.0 capabilities are prerequisites and remain authoritative:

- `lingonberry-quarantine-replacement-retention-report/v1`
- active generation pointer verification
- sealed generation manifest and generation digest
- replacement transaction journal and terminal states
- same-host operation lock
- verified backup v2
- bounded metrics, append-only audit, and deterministic failure injection

The existing retention report classifies generations as:

- `active-committed-generation`
- `previous-committed-generation`
- `rolled-back-generation`
- `incomplete-transaction-generation`
- `orphan-unreferenced-generation`
- `unknown-or-corrupt`
- `legacy-root-layout`

Classification alone never authorizes deletion.

## 3. Release scope

### 3.1 Policy and eligibility contract

Define a versioned retention policy that determines whether a managed subject may enter a cleanup plan.

Required policy inputs:

- exact subject type and transaction ID
- retention-report classification
- active-pointer snapshot
- transaction-journal terminal state
- generation manifest and digest verification
- minimum retained committed generations
- minimum age threshold based on durable metadata, not directory timestamps alone
- explicit operator-selected subject set

### 3.2 Deterministic cleanup preview and proof

Add a read-only preview that emits a deterministic plan and proof.

The proof must bind:

- policy version and normalized policy inputs
- state-directory identity
- active-pointer content and digest
- selected generation/workspace identities
- transaction journal digests
- generation manifest and generation digest
- complete managed-path inventory
- runtime fingerprint
- plan digest and proof digest

Preview must not rename, truncate, delete, or rewrite managed state.

### 3.3 Verified cleanup transaction and recovery

Apply a verified proof through a dedicated cleanup transaction.

Required properties:

- acquire the same-host operation lock
- revalidate all proof inputs immediately before mutation
- reject stale pointer, journal, manifest, digest, path inventory, or runtime fingerprint
- move selected subjects to a transaction-local tomb area using same-filesystem rename
- fsync each durable boundary
- begin irreversible deletion only after the tomb set is sealed and verified
- resume idempotently after interruption
- allow rollback only before irreversible deletion begins
- distinguish committed, rolled-back, recovery-required, and partially-deleted terminal evidence

### 3.4 Operations and release hardening

- versioned cleanup status
- bounded-cardinality Prometheus metrics
- secret-free append-only audit
- machine-readable failure-point inventory
- deterministic failure injection at durable boundaries
- operator runbook and end-to-end smoke test
- compatibility from v0.3.0 legacy-root and generation layouts
- release checklist, release notes, package version, tag, and GitHub Release

## 4. Eligibility matrix

| Subject | Default eligibility | Required evidence | Notes |
|---|---|---|---|
| active committed generation | never eligible | n/a | active pointer target must never be deleted |
| previous committed generation | conditionally eligible | terminal committed journal, verified manifest/digest, retention floor satisfied | keep at least the configured number of previous committed generations |
| rolled-back generation | conditionally eligible | terminal rolled-back journal, verified manifest/digest | rollback evidence remains in append-only audit/journal |
| incomplete transaction generation | never eligible | n/a | recovery or manual investigation required |
| orphan unreferenced generation | never automatically eligible | manual investigation | absence of a journal is not proof of disposability |
| unknown or corrupt generation | never eligible | n/a | fail closed |
| legacy root layout | never eligible | n/a | no generation directory subject exists |
| committed transaction workspace | conditionally eligible | bound committed generation, terminal journal, proof that no recovery input depends on workspace | workspace cleanup is separate from generation cleanup |
| rolled-back transaction workspace | conditionally eligible | terminal rolled-back journal and preserved rollback evidence | apply retention floor and age policy |
| non-terminal transaction workspace | never eligible | n/a | recovery may still require it |

## 5. Safety invariants

1. Never delete the active generation.
2. Never delete archive segments or immutable evidence ledgers.
3. Never infer eligibility from filesystem timestamps alone.
4. Never delete incomplete, orphan, unknown, or corrupt state automatically.
5. Never apply a stale plan or proof.
6. Never follow symlinks or escape managed state roots.
7. Never mix cleanup mutation into a replacement transaction.
8. Never claim rollback is possible after irreversible deletion begins.
9. Never remove journals or audit evidence required to explain a deletion.
10. Never run cleanup without the same-host operation lock.
11. Never expose paths, transaction IDs, or free-form errors as metric labels.
12. Never enable unattended automatic deletion in v0.4.0.

## 6. Transaction states

Proposed cleanup transaction states:

```text
prepared
validated
quarantining
quarantined
deleting
committed
rolled-back
recovery-required
partially-deleted
```

`partially-deleted` is an explicit terminal evidence state. It must not be normalized into `committed` merely because some subjects were removed.

## 7. Work breakdown

### V4-1: Retention policy and evidence contract

- authoritative policy document
- subject model and eligibility matrix
- normalized policy representation
- path and symlink safety rules
- proof binding and stale-state rejection rules

### V4-2: Cleanup preview and proof

- deterministic plan/proof schemas
- read-only CLI
- verifier
- tamper, duplicate, traversal, symlink, and stale-state tests

### V4-3: Cleanup transaction and recovery

- journal and state machine
- tomb-area rename and sealing
- irreversible deletion boundary
- resume and pre-delete rollback
- partial deletion classification

### V4-4: Operations and release hardening

- status, metrics, audit
- failure injection and crash matrix
- smoke test and upgrade compatibility
- release documentation and v0.4.0 publication

## 8. Initial non-scope

- background scheduled deletion
- distributed cleanup coordination
- remote backup or archive deletion
- archive-segment rewrite or deletion
- immutable evidence mutation
- deduplication or semantic event collapse
- schema migration or conflict resolution
- cryptographic signing of cleanup proofs
- secure erase guarantees
- cross-filesystem atomic cleanup

## 9. First implementation gate

No destructive code is introduced until all of the following are reviewed:

- eligibility matrix
- minimum retention floor
- age-source semantics
- managed path inventory rules
- proof binding and stale-state rejection
- symlink and traversal rejection
- rollback cutoff and irreversible boundary
- partial deletion evidence semantics

## 10. Release completion criteria

v0.4.0 is complete only when:

- cleanup eligibility is deterministic and fail-closed
- preview/proof is read-only, reproducible, and tamper-detecting
- apply revalidates all bound state under lock
- crashes at every durable boundary have deterministic recovery classification
- active, incomplete, orphan, corrupt, archive, and immutable evidence state cannot be deleted
- operator smoke tests pass from v0.3.0 state
- all workspace tests and main-branch CI pass
- release checklist, notes, tag, and GitHub Release are published
