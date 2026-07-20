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
4. otherwise protected by the relay's configured recent-generation retention policy.

If one generation satisfies multiple conditions, it is stored as one logical snapshot.

## Garbage collection

A snapshot is eligible for garbage collection only when all of the following are true:

- it is not the current observation generation;
- it is not the semantic checkpoint generation;
- no active cursor lease references it;
- it is outside the configured recent-generation retention policy;
- deletion does not mutate or remove canonical evidence.

Garbage collection is idempotent. A failed collection attempt MUST NOT make a retained generation appear successfully deleted.

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
- Do not make cursor leases permanent.
- Do not silently continue pagination against a different generation.
- Do not expose storage paths, row identifiers, or lease identifiers in public cursors or errors.

## Unresolved retention parameter

This rule fixes the protected-generation categories and collection safety conditions. The remaining policy decision is whether non-protected recent generations are retained primarily by a per-target generation-count bound, by an age/time bound, or by a hybrid of both.
