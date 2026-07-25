# Effective View Diagnostic Retention

**Status: normative v1.0 pre-release protocol contract** | **Rule version: `lb.http.effective-view.diagnostic-retention.v1`** | **Last reviewed: 2026-07-25**

This contract defines retention and garbage-collection rules for **derived diagnostic snapshots** used by effective-view diagnostic pagination. It does not authorize deletion or mutation of canonical Knowledge Objects, Transition Objects, delegation or revocation evidence, signatures, immutable carrier digests, quarantine evidence, or any other canonical record.

The checked-in relay does **not** implement this multi-generation retention capability. It currently stores one effective-view snapshot per target and replaces that file when a newer snapshot is persisted. It has no cursor leases, read guards, retention catalog, semantic-checkpoint pin, recent-generation set, delete claims, or diagnostic snapshot garbage collector. The relay must not advertise `lb.http.effective-view.diagnostic-retention.v1` until the complete capability is implemented and tested.

## 1. Scope

This rule applies only to derived snapshots that can serve:

```text
GET /v1/effective-objects/{targetId}/diagnostics
```

A derived snapshot contains a generation-bound effective-view response and its deterministic diagnostic set. Snapshot retention is an availability policy for stable paging. It is not evidence retention, legal retention, archival preservation, or a canonical-data lifecycle policy.

Canonical evidence must remain independently available according to its own storage contract even after every derived snapshot has expired.

## 2. Snapshot identity

Each retained snapshot must bind at least:

```text
target ID
evidence generation
snapshot identity
observedAt
snapshot digest
storage format version
```

A snapshot identity must not be inferred from a mutable path alone. The retained bytes, generation, and digest must agree before the snapshot can satisfy a page read or retention protection.

Multiple protection reasons for the same target and generation refer to one logical snapshot. Implementations must not create conflicting copies with different content under one generation identifier.

## 3. Protected generations

A conforming implementation must retain a valid derived snapshot while any of these protections applies:

1. it is the current observation generation for the target;
2. it is the semantic checkpoint generation for the target;
3. an unexpired diagnostic cursor lease protects the exact target and generation;
4. an unexpired read guard protects an in-flight page read from the exact snapshot;
5. it remains inside the recent-generation policy in section 4;
6. a committed retention or recovery hold explicitly protects it.

A snapshot must not be considered protected merely because an opaque cursor exists on a client. Protection is determined by trusted relay state.

## 4. Recent-generation policy

For valid snapshots not otherwise protected, the v1 defaults are:

```text
maximum recent generations per target = 8
maximum recent age = 86400 seconds
```

A snapshot is recent-policy retained only when **both** conditions are true at one captured decision time:

1. it is among the newest eight otherwise-unprotected snapshots for the target; and
2. its age is no greater than 86400 seconds.

The count and age bounds are conjunctive upper bounds. Meeting one bound does not compensate for failing the other.

Current observation, semantic checkpoint, cursor lease, read guard, and explicit recovery holds override these recent-policy bounds.

## 5. Ordering and age

Otherwise-unprotected snapshots are ordered by:

1. `observedAt` descending;
2. evidence generation ascending by unsigned UTF-8 byte order when `observedAt` values are equal.

`observedAt` is the durable time at which the observation snapshot was committed. Evidence receipt time, worker start time, process uptime, file modification time, cursor issue time, and garbage-collection scan time are not substitutes.

The age boundary is inclusive:

```text
ageSeconds <= 86400
```

A snapshot exactly 86400 seconds old remains age-eligible but is collectible when it falls outside the newest-eight count bound and has no other protection.

Negative age caused by clock rollback or corrupt metadata must not grant indefinite retention. The implementation must use a documented monotonic or conservatively validated time model.

## 6. Cursor-lease protection

A cursor lease protects only its exact target and generation for its bounded lifetime.

The lease contract is defined by `lb.http.effective-view.diagnostic-cursor-lease.v1`. Retention must use trusted lease state rather than accepting an opaque cursor token as proof of an active lease.

When a lease expires:

- it no longer pins the snapshot;
- the snapshot may remain retained for another protection reason;
- a later page request may return `409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE`;
- the relay must not silently switch the request to a different generation;
- retention must not create a fresh lease merely because the old cursor is presented.

## 7. Read-guard protection

An acquired read guard protects one in-flight page materialization from a conflicting delete claim.

Guard acquisition, heartbeat renewal, release, expiry, and retention delete-claim creation must share a serialization boundary. For one snapshot, a new guard or valid heartbeat and a deletion claim that assumes no active guard must not both commit.

A cursor lease alone is not an in-flight read guard. A heartbeat alone is not a retention pin; only the resulting active guard is protective.

## 8. Garbage-collection eligibility

A snapshot is eligible for deletion only when one atomic or serializable decision establishes all of the following:

- the snapshot is valid enough to identify unambiguously;
- it is not the current observation generation;
- it is not the semantic checkpoint generation;
- no active cursor lease protects it;
- no active read guard protects it;
- no recovery or operator hold protects it;
- it is outside the recent-generation policy;
- no earlier reader acquisition has already won the deletion race;
- deletion affects only derived snapshot state;
- the exact snapshot identity and digest still match the evaluated candidate.

Unknown, corrupt, contradictory, partially written, or symlinked snapshot state must not be silently deleted as an ordinary eligible snapshot. It requires fail-closed quarantine or operator investigation.

The garbage-collection decision must use one captured decision time for the entire target scan.

## 9. Delete claims and execution

Eligibility evaluation and physical deletion must not be separated by an unprotected time-of-check/time-of-use window.

A conforming implementation must use a transaction, compare-and-swap delete claim, serialized coordinator, or equivalent protocol so that:

1. a reader can acquire protection and prevent deletion; or
2. retention can commit the delete claim and cause later guard acquisition to fail.

Deletion must be idempotent. A failed attempt must not report success, remove protection metadata while leaving ambiguous bytes, or make a retained generation appear absent without a terminal deletion record.

When physical deletion follows a durable claim, crash recovery must distinguish at least:

```text
claim-not-executed
deleting
deleted
recovery-required
```

The protocol does not promise secure erase.

## 10. Rebuild behavior

A deleted derived snapshot may be reproducible from canonical evidence, but an existing cursor walk must not trigger an implicit rebuild and pretend continuity.

A separately authorized rebuild may create a new snapshot for the same deterministic evidence generation only when the rebuilt bytes and identity satisfy the current snapshot contract. Existing cursor and lease validity is still governed by their original binding and expiry; rebuilding does not revive expired protection.

If deterministic reconstruction cannot be proven, the rebuilt result is a new observation and must use its own generation identity.

## 11. Observability

Operator-visible metrics and status should distinguish bounded categories such as:

```text
current observation
semantic checkpoint
cursor-lease protected
read-guard protected
recent-policy retained
recovery-held
eligible for deletion
delete claimed
deleted
corrupt or recovery-required
unavailable-generation responses
```

Public responses must not expose lease identifiers, guard identifiers, delete-claim IDs, database keys, filesystem paths, worker identities, or internal retention reasons beyond stable public error codes.

Metrics must not use target IDs, generation IDs, cursor values, or snapshot paths as unbounded labels.

## 12. Current implementation boundary

The checked-in relay currently persists snapshots at a per-target path derived from an FNV-1a hash of the target ID. Persistence writes one temporary file and renames it over the target's snapshot path. Loading reads only that current file.

Consequences of the current implementation:

- there is at most one stored effective-view snapshot per target through this path;
- persisting a new snapshot replaces the earlier derived snapshot;
- historical generations are not retained for lease-stable pagination;
- no semantic checkpoint snapshot is separately pinned;
- no cursor lease or read guard is stored;
- no recent-generation catalog or eight-generation/24-hour policy is enforced;
- no retention delete claim or garbage-collection state machine exists;
- no retention metrics described above are emitted.

Generation mismatch is handled by rejecting the page request rather than silently reading the newer snapshot. That behavior does not constitute multi-generation retention conformance.

## 13. Capability advertisement

A relay may advertise `lb.http.effective-view.diagnostic-retention.v1` only when it implements and tests the complete retention contract together with compatible cursor-lease and read-guard semantics.

Advertising pagination support alone must not imply historical snapshot retention. A capability response should distinguish at least:

```text
stateless-current-snapshot pagination
multi-generation retention
cursor leases
read guards
guard heartbeat
retention garbage collection
```

The fixed candidate must be treated as supporting only the behavior actually implemented and tested.

## 14. Conformance requirements

A conforming implementation must test at least:

- current observation protection;
- semantic checkpoint protection;
- active and exact-expiry cursor leases;
- active, released, and exact-expiry read guards;
- heartbeat-versus-delete serialization;
- newest-eight ordering and timestamp ties;
- inclusive 86400-second age boundary;
- count and age bounds being conjunctive;
- one captured decision time per scan;
- delete claim racing new guard acquisition;
- crash before and during physical deletion;
- idempotent retry and reconciliation;
- corrupt, missing, partial, and symlinked state;
- rebuild without reviving an expired cursor;
- canonical evidence remaining unchanged;
- absence of sensitive or unbounded identifiers from metrics and public responses.

## 15. Non-goals

This contract does not provide:

- canonical evidence deletion;
- legal or regulatory retention policy;
- secure erase;
- unlimited client-controlled retention;
- automatic repair of corrupt snapshots;
- implicit rebuild during an existing cursor walk;
- distributed coordination without an explicit shared mechanism;
- formal soak evidence, reference-host qualification, or release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
