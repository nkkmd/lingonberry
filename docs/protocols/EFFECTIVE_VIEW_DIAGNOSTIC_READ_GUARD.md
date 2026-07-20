# Effective View Diagnostic Read Guard

## Status

Normative foundation for v0.6.0 diagnostic page-read protection.

Rule version: `lb.http.effective-view.diagnostic-read-guard.v1`

## Purpose

A valid diagnostic cursor lease protects a client's right to continue pagination. It does not by itself prove that a snapshot cannot be garbage-collected between lease validation and page materialization.

A relay therefore acquires a short-lived read guard before reading a diagnostic page.

## Acquisition

Guard acquisition MUST be atomic with all of the following checks:

1. cursor structure and integrity are valid;
2. target binding matches the request;
3. generation binding matches the request;
4. cursor lease is unexpired;
5. requested limit is valid;
6. the exact derived snapshot is retained and readable;
7. no garbage-collection delete claim has already committed for that snapshot.

The acquisition transaction MAY also extend the cursor idle expiry under `lb.http.effective-view.diagnostic-cursor-lease.v1`.

A successful acquisition records a guard bound to exactly:

- target ID;
- evidence generation;
- snapshot identity;
- page operation identity;
- guard issuance time;
- guard absolute expiry.

Guard identifiers and operation identities are relay-internal and MUST NOT appear in public cursors or responses.

## Lifetime

Initial v0.6.0 parameters:

```text
guard lifetime = 120 seconds
```

A guard is active only while:

```text
now < guardExpiresAt
```

Equality with `guardExpiresAt` is expired.

A normal page operation SHOULD complete within one guard lifetime. The initial v0.6.0 contract does not permit unbounded sliding renewal.

## Page read

After guard acquisition, the relay reads only the exact guarded snapshot and generation.

A page response MUST NOT be returned when:

- the guard was not acquired;
- the guard expired before the snapshot read completed;
- the guarded snapshot identity no longer matches the opened snapshot;
- a different generation was read;
- the page was only partially materialized after a storage error.

The implementation MUST either return one complete generation-consistent page or return an error without a partial diagnostic page.

## Release and crash recovery

A guard SHOULD be released after the page response is fully materialized or after the page operation fails.

Release is idempotent.

If a process crashes before release, the guard expires automatically at its absolute expiry. Restart MUST NOT reset or extend the original guard expiry.

Expired guards are ignored by garbage collection and MAY be deleted by reconciliation.

## Garbage collection

Garbage collection MAY delete a derived snapshot only when, in the same atomic decision:

- the snapshot is not current observation;
- the snapshot is not the semantic checkpoint;
- no unexpired cursor lease protects it;
- no unexpired read guard protects it;
- it is outside the hybrid recent-generation policy;
- no prior reader has already acquired a guard that covers the deletion decision.

Guard acquisition and garbage-collection claim creation MUST use transaction, compare-and-swap, or an equivalent serialization mechanism so that exactly one of these outcomes occurs:

1. reader acquires the guard and GC does not delete the snapshot until the guard expires or releases; or
2. GC commits the delete claim and the reader fails guard acquisition.

A reader MUST NOT succeed against a snapshot whose delete claim already committed.

## Error behavior

When guard acquisition loses to a committed garbage-collection claim or the generation is otherwise unavailable:

```text
409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

When storage fails after acquisition but before a trustworthy complete page is materialized:

```text
500 LB_DIAGNOSTIC_PAGE_READ_FAILED
```

The relay MUST NOT silently switch generation or return a partial page.

## Safety requirements

- Do not hold a long database transaction for network response delivery.
- Do not treat a cursor lease alone as an in-flight read guard.
- Do not allow expired guards to pin snapshots permanently.
- Do not renew guards indefinitely.
- Do not release or expire a guard for one target or generation using another operation.
- Do not permit both a committed GC claim and a successful new guard acquisition for the same snapshot.
- Do not expose guard IDs, storage paths, row IDs, process IDs, or lease records publicly.
