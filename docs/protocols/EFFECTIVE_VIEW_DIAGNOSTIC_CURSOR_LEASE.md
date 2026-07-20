# Effective View Diagnostic Cursor Lease

## Status

Normative foundation for v0.6.0 diagnostic pagination leases.

Rule version: `lb.http.effective-view.diagnostic-cursor-lease.v1`

## Lease model

A diagnostic pagination cursor lease uses both an idle timeout and an absolute lifetime.

```text
idle timeout: 900 seconds
absolute lifetime: 3600 seconds
```

When a cursor is first issued:

```text
issuedAt = now
absoluteExpiresAt = issuedAt + 3600 seconds
idleExpiresAt = min(now + 900 seconds, absoluteExpiresAt)
```

After a successful valid page response:

```text
idleExpiresAt = min(pageCompletedAt + 900 seconds, absoluteExpiresAt)
```

The absolute expiry never moves.

## Extension eligibility

A lease is extended only after a page request has all of the following properties:

- the cursor is structurally valid and authentic;
- the cursor target matches the request target;
- the cursor generation matches the required generation;
- the requested generation snapshot is retained;
- the requested limit is valid;
- the requested page is successfully read and returned.

Malformed cursors, generation mismatches, invalid limits, storage failures, authorization failures, and unsuccessful page responses MUST NOT extend the lease.

A repeated successful request for the same page MAY extend the idle expiry, but MUST NOT move the absolute expiry. Implementations SHOULD make repeated page reads idempotent.

## Expiry boundary

A lease is active only while both of the following are true:

```text
now < idleExpiresAt
now < absoluteExpiresAt
```

At either exact expiry instant, the lease is expired. An expired cursor does not protect its derived snapshot from garbage collection.

## Garbage collection race

Lease validation and snapshot acquisition MUST be coordinated so that a page request cannot successfully validate an active lease and then read a snapshot concurrently collected before that page completes.

A conforming implementation may use a transaction, lease reference count, read guard, or equivalent compare-and-swap protocol. It MUST NOT report a successful page containing partial or mixed snapshot data.

If the lease has expired or the snapshot is unavailable before a protected read is acquired, return:

```text
409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

Do not silently switch generations.

## Restart behavior

Lease state that is required to protect a snapshot MUST be durable or conservatively reconstructable after restart. A restart MUST NOT extend `absoluteExpiresAt` or reset `issuedAt`.

If lease state cannot be trusted after restart, the relay may reject the cursor as unavailable, but MUST NOT grant a fresh absolute lifetime to the old cursor.

## Public boundary

The public cursor remains opaque. Public responses do not expose:

- `issuedAt`;
- internal lease identifiers;
- reference counts;
- database row identifiers;
- storage paths;
- worker or process identifiers.

## Safety requirements

- Never extend a lease after an invalid or unsuccessful request.
- Never move the absolute expiry.
- Never treat an exact expiry instant as active.
- Never reset cursor lifetime after restart.
- Never collect a snapshot while a successful page read holds its protection guard.
- Never mix generations within one cursor walk.
