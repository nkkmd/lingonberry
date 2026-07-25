# Effective View Diagnostic Read Guard

**Status: normative v1.0 pre-release protocol contract with an explicit implementation gap** | **Rule version: `lb.http.effective-view.diagnostic-read-guard.v1`** | **Last reviewed: 2026-07-25**

This document defines the in-flight snapshot protection required when a relay claims support for effective-view diagnostic read guards.

The checked-in relay does **not** currently implement this rule. Its diagnostic page operation loads one target snapshot, validates the retained generation, slices an in-memory diagnostic array, and returns a page. It has no persisted guard record, guard expiry, atomic acquisition against garbage collection, release operation, or guard recovery after restart.

## 1. Capability boundary

A relay may advertise:

```text
lb.http.effective-view.diagnostic-read-guard.v1
```

only when it implements every required acquisition, lifetime, release, retention, and failure condition in this document.

Stateless page reads may still be generation-bound, but they must not be described as read-guard protected.

## 2. Purpose

A cursor or cursor lease binds a client request to a target, generation, and page position. It does not by itself prevent a retention worker from deleting the bound snapshot after cursor validation but before page materialization.

A read guard protects one in-flight page operation. It provides the serialization point between:

- a reader that intends to materialize a page from one retained generation; and
- a retention operation that intends to commit deletion of that snapshot.

A read guard is not a long-lived pagination lease and is not authorization to read otherwise protected data.

## 3. Acquisition requirements

Guard acquisition must be atomic, transactional, or equivalently serialized with all of these checks:

1. the target identifier is valid;
2. the requested generation is valid;
3. the cursor is valid for the target, generation, and requested page position;
4. any required cursor lease is active;
5. the requested limit is valid;
6. the exact diagnostic snapshot is retained and readable;
7. the snapshot identity matches the requested generation;
8. no committed delete claim already covers the snapshot;
9. the snapshot is eligible to be opened by this operation;
10. a unique guard can be recorded without overwriting another operation's state.

When a cursor-lease capability is also implemented, idle-lease extension may occur in the same serialized operation, but only under the cursor-lease contract. Guard acquisition alone must not reset an absolute cursor lifetime.

## 4. Guard binding

A successful guard is bound to exactly:

```text
targetId
evidenceGeneration
snapshotIdentity
pageOperationIdentity
guardIssuedAt
guardExpiresAt
read-guard rule version
```

The binding must prevent:

- a guard for one target from protecting another;
- a guard for one generation from protecting another;
- one page operation from releasing another operation's guard;
- a stale operation identity from reactivating an expired guard.

Guard identifiers and operation identities are relay-internal. They must not appear in public cursors or page responses.

## 5. Lifetime

The v1 initial guard lifetime is:

```text
120 seconds
```

A guard is active only while:

```text
now < guardExpiresAt
```

At exact equality, the guard is expired.

`guardExpiresAt` is fixed at acquisition unless the separately versioned heartbeat extension is implemented. The base read-guard rule does not authorize unbounded sliding renewal.

The implementation should use a monotonic process clock for active-operation decisions. Durable wall-clock fields may support recovery and audit, but wall-clock rollback must not extend a guard.

## 6. Page materialization

After acquisition, the reader must open and read only the exact guarded snapshot.

A successful page requires:

- the guard remains active through trustworthy page materialization, or a separately conformant heartbeat has extended it;
- the opened snapshot identity matches the guard;
- every diagnostic belongs to the guarded generation;
- page ordering and boundaries follow the pagination contract;
- the complete page is materialized without a storage or serialization error.

The implementation must return either:

- one complete, generation-consistent page; or
- an error without a partial diagnostic page.

It must not silently switch generations, continue from a replacement snapshot, or splice records from two snapshots.

## 7. Response delivery boundary

A guard protects snapshot materialization, not indefinite network delivery.

The relay should complete storage reads and construct the response body while the guard is active, then release the guard before or immediately after handing the complete body to the HTTP layer.

The implementation must not hold a long database transaction, filesystem lock, or retention serialization lock for the full duration of slow client network delivery.

If the response body is fully materialized under the guard and network delivery later fails, the storage read itself may still be recorded as complete. That failure must not cause the guard to be renewed indefinitely.

## 8. Release

A guard should be released after:

- successful page materialization; or
- any page-operation failure that no longer needs the snapshot.

Release must be idempotent.

A release request must verify the guard's target, generation, snapshot, and operation identity. It must not release another reader's guard.

Failure to delete an already expired guard record must not reactivate it or extend its lifetime.

## 9. Crash and restart behavior

If a process crashes before release, the guard expires at its original `guardExpiresAt`.

Restart must not:

- reset the guard issue time;
- extend the guard expiry;
- convert an expired guard into an active guard;
- trust incomplete or contradictory guard state;
- create a new guard solely because an old guard record exists.

Expired guards may be removed by reconciliation. When durable guard state cannot be trusted, retention must fail closed or conservatively preserve the snapshot until the conflict is resolved. It must not claim both successful deletion and successful protection for the same serialization point.

## 10. Garbage-collection serialization

A retention operation may commit a delete claim for a diagnostic snapshot only when, in one serialized decision:

- the snapshot is not the current observation when the retention policy protects it;
- the snapshot is not another protected checkpoint;
- no active cursor lease protects it;
- no active read guard protects it;
- it satisfies the separately defined retention policy;
- no earlier reader has already acquired a guard covering the deletion decision.

Guard acquisition and delete-claim creation must produce exactly one outcome:

1. the reader acquires a guard, and deletion cannot commit until the guard releases or expires; or
2. deletion commits its claim, and the reader fails guard acquisition.

A reader must not succeed against a snapshot whose delete claim already committed.

## 11. Error behavior

When guard acquisition loses to a committed delete claim, the generation is absent, or the snapshot cannot be protected, return:

```text
409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

When storage, serialization, identity verification, or guard state fails after acquisition and before a trustworthy complete page is materialized, a conforming implementation returns a stable server error such as:

```text
500 LB_DIAGNOSTIC_PAGE_READ_FAILED
```

The checked-in relay currently does not emit `LB_DIAGNOSTIC_PAGE_READ_FAILED` from a read-guard implementation because no such implementation exists. Capability documentation must not imply otherwise.

Unknown internal exceptions must not be copied into public error codes or messages.

## 12. Heartbeat interaction

A separately versioned heartbeat rule may extend a guard for a legitimately long page read.

The base read-guard rule requires that any heartbeat:

- prove ownership of the exact active guard;
- preserve target, generation, snapshot, and operation binding;
- fail after guard expiry;
- use a bounded absolute extension policy;
- serialize against deletion claims;
- stop when the page completes or fails.

A heartbeat is not a cursor-lease renewal and must not extend unrelated guards.

## 13. Current checked-in implementation

The current implementation in `packages/relay/src/effective_view_v2.rs`:

- loads the currently retained snapshot for one target;
- verifies its observation generation equals the requested generation;
- clones the stored diagnostic array;
- validates a cursor offset;
- slices the array and returns the page;
- fails generation mismatch with `LB_DIAGNOSTIC_GENERATION_UNAVAILABLE`.

It does not:

- create a read-guard record;
- coordinate atomically with retention;
- track guard issue or expiry time;
- release or reconcile guards;
- implement heartbeat;
- prove snapshot pinning across a retention race.

The current implementation therefore provides stateless generation checking, not `lb.http.effective-view.diagnostic-read-guard.v1` conformance.

## 14. Public-data boundary

Public responses and cursors must not expose:

- guard identifiers;
- operation identities;
- guard issue or expiry records unless a separate public contract defines them;
- database row IDs, table names, or storage paths;
- process, worker, thread, or host identifiers;
- lock tokens, lease records, MAC keys, bearer tokens, or signing material;
- raw storage or transaction errors.

## 15. Conformance requirements

A conforming implementation must test at least:

1. successful atomic guard acquisition;
2. rejection after a committed delete claim;
3. retention blocked by an active guard;
4. exact-expiry behavior;
5. idempotent release;
6. cross-target and cross-generation release rejection;
7. process crash without expiry reset;
8. restart with corrupt guard state;
9. complete page materialization under the guard;
10. storage failure without a partial page;
11. snapshot identity mismatch;
12. no generation switch;
13. reader-versus-retention race serialization;
14. bounded heartbeat integration when advertised;
15. no public guard-state disclosure.

## 16. Non-goals

This rule does not provide:

- a durable cursor lease by itself;
- indefinite snapshot retention;
- client-controlled guard duration;
- long-lived network connection protection;
- distributed consensus unless separately implemented;
- authorization to read otherwise restricted diagnostics;
- secure cursor authentication in the checked-in implementation;
- formal release qualification, soak evidence, or reference-host qualification.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
