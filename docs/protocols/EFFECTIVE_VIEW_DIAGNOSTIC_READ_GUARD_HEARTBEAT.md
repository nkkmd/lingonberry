# Effective View Diagnostic Read Guard Heartbeat

**Status: normative v1.0 pre-release protocol contract** | **Rule version: `lb.http.effective-view.diagnostic-read-guard-heartbeat.v1`** | **Last reviewed: 2026-07-25**

This contract defines bounded renewal for an already-acquired diagnostic read guard. A heartbeat is an internal continuation mechanism for a slow page-materialization operation. It is not a public API, a cursor renewal, a snapshot lease, or permission to retain a snapshot indefinitely.

The checked-in relay does **not** implement read guards, guard heartbeats, progress tokens, guard reconciliation, or heartbeat-aware retention. Its diagnostic pagination remains a stateless generation check over the currently retained per-target snapshot. The relay must not advertise this heartbeat capability until the complete guard and retention protocol is implemented and tested.

## 1. Timing model

A conforming guard-heartbeat implementation uses:

```text
guard idle lifetime: 120 seconds
guard absolute lifetime: 600 seconds
```

At guard creation:

```text
guardIssuedAt = now
guardAbsoluteExpiresAt = guardIssuedAt + 600 seconds
guardIdleExpiresAt = min(now + 120 seconds, guardAbsoluteExpiresAt)
```

After an accepted heartbeat:

```text
guardIdleExpiresAt = min(heartbeatAcceptedAt + 120 seconds, guardAbsoluteExpiresAt)
```

The absolute expiry never moves. At either exact expiry instant, the guard is expired.

Clock comparisons must use one documented time basis. Wall-clock rollback, restart, or process migration must not increase the remaining absolute lifetime.

## 2. Required binding

Every heartbeat is bound to the exact active guard and page operation:

```text
guard identity
target ID
evidence generation
snapshot identity
page operation identity
progress token
guard issued time
current idle expiry
absolute expiry
```

A heartbeat must not change or rebind any identity. A heartbeat from another target, generation, snapshot, operation, worker, or guard is invalid.

Guard identities and progress tokens are relay-internal. They must not appear in public cursors, response bodies, URLs, logs intended for untrusted readers, or metrics labels.

## 3. Progress requirement

A heartbeat is eligible only when the page operation has made new materialization progress since the previously accepted heartbeat.

Progress must be directly observable or durably recorded for the exact operation. The progress token must be monotonic and operation-scoped. Accepted progress can represent, for example, a strictly larger verified materialized-record count or a later durable internal phase.

The following do not establish progress:

- elapsed time alone;
- a repeated progress token;
- a lower or incomparable token;
- network-write progress after page materialization has already completed;
- work performed for another target, generation, snapshot, page, or operation;
- an unverified worker assertion;
- process liveness without materialization progress.

Timer-only heartbeats are forbidden.

## 4. Acceptance transaction

Heartbeat acceptance must atomically verify:

1. the guard exists and is active;
2. both idle and absolute expiries have not been reached;
3. all guard and operation bindings match;
4. no successful release or terminal failure has already committed;
5. no conflicting retention or deletion claim has committed;
6. the progress token is valid and strictly advances the accepted progress state;
7. the resulting idle expiry does not exceed the fixed absolute expiry.

The progress update and idle-expiry update must commit together. A crash must not leave a renewed expiry without the corresponding accepted progress record.

A rejected heartbeat must not modify guard state.

## 5. Relationship to cursor leases

A diagnostic cursor lease and a read guard are independent protections.

- A cursor lease protects eligibility to request another page while its snapshot is retained.
- A read guard protects one in-flight materialization operation against a conflicting retention deletion.
- A guard heartbeat can extend only the guard idle expiry.
- A heartbeat must not renew or recreate the cursor lease.
- Cursor expiry after valid guard acquisition does not cancel that already-running guarded operation.
- Guard expiry does not grant a new cursor lease.

No heartbeat can extend either protection beyond its own fixed absolute lifetime.

## 6. Completion and response boundary

A page may be returned only when complete materialization finishes while the guard remains active.

The relay must fail closed when:

- either guard expiry is reached before complete materialization;
- a heartbeat is rejected and the operation cannot finish before the existing expiry;
- progress state becomes corrupt or contradictory;
- the guarded snapshot identity changes;
- storage fails before a trustworthy complete page is materialized;
- a retention deletion claim wins the required serialization boundary.

The relay must never return a partial page, silently switch generation, or create a replacement guard after expiry.

Network delivery should occur after complete page materialization. A guard heartbeat must not be used to hold storage transactions or retention locks during arbitrary client download time.

## 7. Failure behavior

A read that cannot complete under a valid guard returns the protocol-level failure:

```text
500 LB_DIAGNOSTIC_PAGE_READ_FAILED
```

An expired or unavailable snapshot discovered before guard acquisition remains:

```text
409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

Invalid heartbeat input is internal implementation failure or rejected internal state; it is not a separate public endpoint error because the heartbeat is not public.

Rejection reasons should use stable bounded internal reason codes. They must not expose paths, database identifiers, exception strings, credentials, guard IDs, or progress-token values.

## 8. Release and restart

The guard must be released idempotently after success or terminal failure.

After a crash:

- accepted progress and expiry state must be durable or conservatively reconstructable;
- restart must not reset `guardIssuedAt`;
- restart must not move `guardAbsoluteExpiresAt`;
- uncertain or contradictory state must be treated as expired or unavailable, not renewed;
- expired guard records may be reconciled without deleting a snapshot protected by another active guard or lease.

A process restart is not progress and must not produce an automatic heartbeat.

## 9. Retention serialization

Heartbeat acceptance, guard release, and retention deletion claims must share a serialization boundary sufficient to prevent both of these outcomes from succeeding for the same protected snapshot:

1. a heartbeat renews an active guard; and
2. retention commits a deletion claim that assumes no active guard exists.

A conforming implementation may use a transaction, compare-and-swap, serialized coordinator, or equivalent mechanism. Host-local in-memory coordination alone is insufficient when retention and readers can run in different processes or hosts.

The heartbeat does not itself retain a snapshot. Only the resulting active guard participates in retention eligibility.

## 10. Current implementation boundary

The checked-in relay currently:

- encodes target hash, generation hash, and offset in a stateless cursor;
- verifies that the requested generation equals the currently stored snapshot generation;
- reads and slices diagnostics without a guard record;
- has no guard idle or absolute expiry;
- has no progress token or heartbeat state;
- has no guard release or restart reconciliation;
- has no atomic heartbeat-versus-retention deletion protocol.

Therefore the current relay does not conform to or advertise `lb.http.effective-view.diagnostic-read-guard-heartbeat.v1`. This document defines the capability contract for an implementation that adds the complete mechanism; it does not claim that the mechanism exists in the fixed candidate.

## 11. Conformance requirements

A conforming implementation must test at least:

- accepted strictly increasing progress;
- duplicate, stale, lower, or cross-operation progress rejection;
- exact idle-expiry rejection;
- exact absolute-expiry rejection;
- absolute lifetime remaining fixed across every heartbeat;
- restart without lifetime reset;
- crash between progress validation and commit;
- heartbeat racing guard release;
- heartbeat racing retention deletion claim;
- cursor expiry during a valid guarded operation;
- no partial response after guard expiry;
- idempotent release after success and failure;
- absence of guard and progress identifiers from public surfaces.

## 12. Non-goals

This contract does not provide:

- public heartbeat endpoints;
- client-controlled guard renewal;
- unlimited or sliding absolute lifetime;
- cursor-lease renewal;
- snapshot retention by heartbeat alone;
- distributed coordination without an explicit shared mechanism;
- authentication through FNV-1a cursor prefixes;
- formal soak evidence, reference-host qualification, or release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
