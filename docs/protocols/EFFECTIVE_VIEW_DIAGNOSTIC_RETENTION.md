# Effective View Diagnostic Retention

## Status

Normative foundation for v0.6.0 derived diagnostic retention.

Rule version: `lb.http.effective-view.diagnostic-retention.v1`

## Scope

Canonical Knowledge Objects, Transition Objects, delegation evidence, revocation evidence, signatures, and immutable carrier digests remain governed by their append-only canonical-storage rules.

This rule applies only to derived diagnostic snapshots and pagination state produced for:

```text
GET /v1/effective-objects/{targetId}/diagnostics
```

Derived snapshots MAY be garbage-collected. Canonical evidence MUST NOT be deleted merely because a derived snapshot expires.

## Protected generations

A relay MUST retain a derived diagnostic snapshot while it is any of the following:

1. the current observation generation for the target;
2. the semantic checkpoint generation for the target;
3. referenced by a non-expired active diagnostic cursor lease;
4. otherwise protected by the recent-generation hybrid retention policy below.

If one generation satisfies multiple conditions, it is stored as one logical snapshot.

## Recent-generation hybrid policy

For generations that are not protected by current observation, semantic checkpoint, or an active cursor lease, the protocol defaults are:

```text
maximum recent generations per target = 8
maximum recent age                   = 86400 seconds (24 hours)
```

A non-protected generation is recent-policy retained only when both conditions are true:

1. it is among the newest eight non-protected generations for the target; and
2. its observation age is less than or equal to 86400 seconds at the garbage-collection decision time.

The count and age limits are conjunctive upper bounds. Meeting only one condition is insufficient.

Current observation, semantic checkpoint, and active cursor-lease protections override both recent-policy bounds.

## Ordering for the count bound

Non-protected snapshots are ordered by:

1. `observedAt` descending;
2. generation identifier ascending by unsigned ASCII bytes when timestamps are equal.

`observedAt` is the durable time at which the relay committed the observation checkpoint for that generation. Receipt time of an individual evidence carrier, worker start time, and garbage-collection scan time are not substitutes.

The time boundary is inclusive:

```text
ageSeconds <= 86400
```

A snapshot exactly 86400 seconds old is age-eligible, but it is still collectible when outside the newest-eight count bound.

## Garbage collection

A snapshot is eligible for garbage collection only when all of the following are true:

- it is not the current observation generation;
- it is not the semantic checkpoint generation;
- no active cursor lease references it;
- it is outside the hybrid recent-generation policy;
- deletion does not mutate or remove canonical evidence.

Garbage collection is idempotent. A failed collection attempt MUST NOT make a retained generation appear successfully deleted.

The garbage-collection decision MUST use one captured decision time for the entire target scan. An implementation MUST NOT allow the age boundary to move independently between snapshot comparisons in one scan.

## Cursor leases

Issuing a pagination cursor creates or extends a bounded lease for the cursor's target and generation.

A cursor lease:

- protects only the exact target and generation bound into the cursor;
- has a finite expiry;
- is not a permanent retention pin;
- does not expose relay-local identifiers;
- does not survive beyond its specified expiry merely because the client retained the opaque token.

When a lease expires, a later page request MAY return:

```text
409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

A relay MUST NOT silently switch the request to another generation.

## Rebuild behavior

An expired derived snapshot may be reproducible from canonical evidence, but the pagination endpoint MUST NOT automatically rebuild it during an existing cursor walk and pretend that the old cursor remains valid.

A separately requested rebuild may create a new retained snapshot for the same deterministic generation. Cursor validity is still governed by the cursor lease and cursor-binding rules.

## Required observability

Operator-visible metrics SHOULD distinguish:

- retained current-observation snapshots;
- retained semantic-checkpoint snapshots;
- cursor-pinned snapshots;
- policy-retained recent snapshots;
- garbage-collected snapshots;
- unavailable-generation responses.

These operator details are not part of the public protocol response.

## Safety requirements

- Do not garbage-collect canonical evidence through this policy.
- Do not collect the current observation generation.
- Do not collect the semantic checkpoint generation.
- Do not collect a generation protected by an unexpired cursor lease.
- Do not treat count-only or age-only eligibility as sufficient for recent retention.
- Do not use evidence receipt time in place of durable observation-checkpoint time.
- Do not make cursor leases permanent.
- Do not silently continue pagination against a different generation.
- Do not expose storage paths, row identifiers, or lease identifiers in public cursors or errors.

## Remaining policy decision

The hybrid recent-generation bounds are fixed. The next decision is whether an active cursor lease uses a fixed expiry established when the first cursor is issued, or a sliding expiry extended by each valid page request.
