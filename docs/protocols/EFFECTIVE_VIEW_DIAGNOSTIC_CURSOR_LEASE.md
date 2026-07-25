# Effective View Diagnostic Cursor Lease

**Status: normative v1.0 pre-release protocol contract with an explicit implementation gap** | **Rule version: `lb.http.effective-view.diagnostic-cursor-lease.v1`** | **Last reviewed: 2026-07-25**

This document defines the lease semantics required when a relay claims support for generation-protecting effective-view diagnostic cursors.

The checked-in relay does **not** currently implement this lease rule. Its current cursor is a stateless token containing target and generation FNV-1a prefixes plus an offset. It has no authenticated lease state, idle timeout, absolute lifetime, renewal, restart recovery, or snapshot-protection guard. Operators and clients must not treat the current cursor as retaining a generation.

## 1. Capability boundary

A relay may claim support for:

```text
lb.http.effective-view.diagnostic-cursor-lease.v1
```

only when it implements every requirement in this document, including durable or conservatively reconstructable lease state and coordination with snapshot retention.

A relay that implements only stateless diagnostic pagination must not advertise this capability and must return generation-unavailable when the requested snapshot no longer exists.

## 2. Lease timing model

The v1 lease uses both an idle timeout and an absolute lifetime:

```text
idle timeout: 900 seconds
absolute lifetime: 3600 seconds
```

When a lease is first issued:

```text
issuedAt = now
absoluteExpiresAt = issuedAt + 3600 seconds
idleExpiresAt = min(now + 900 seconds, absoluteExpiresAt)
```

After an eligible successful page response completes:

```text
idleExpiresAt = min(pageCompletedAt + 900 seconds, absoluteExpiresAt)
```

`absoluteExpiresAt` and `issuedAt` never move.

The time source used for expiry decisions must be monotonic for the lifetime of a process. Durable records may also carry wall-clock values for audit, but wall-clock rollback must not extend an active lease.

## 3. Required lease binding

A lease must bind at least:

```text
targetId
generation
cursor position or cursor family
issuedAt
idleExpiresAt
absoluteExpiresAt
lease rule version
```

The binding must prevent a cursor issued for one target or generation from being accepted for another.

The public token may be opaque, signed, MAC-protected, or an unpredictable reference to server-side state. FNV-1a prefixes alone are not authentication and do not satisfy this rule.

## 4. Extension eligibility

A lease is extended only after all of the following succeed:

- the cursor is structurally valid and authentic under the relay's lease mechanism;
- the cursor target matches the request target;
- the cursor generation matches the required generation;
- the lease is active at protected-read acquisition time;
- the requested generation snapshot is retained;
- the requested limit is valid;
- the requested cursor position is valid;
- the snapshot read guard is acquired;
- the requested page is read completely from one generation;
- the page response is successfully produced.

Malformed cursors, context mismatch, invalid limits, invalid positions, expired leases, unavailable snapshots, storage failures, authorization failures, and unsuccessful responses must not extend the lease.

A repeated successful request for the same page may extend idle expiry, but it must not move absolute expiry or change the cursor's generation binding.

## 5. Expiry boundary

A lease is active only while:

```text
now < idleExpiresAt
and
now < absoluteExpiresAt
```

At either exact expiry instant, the lease is expired.

An expired cursor:

- does not protect a snapshot from collection;
- must not be renewed by a late request;
- must not receive a new absolute lifetime;
- must not be silently rebound to a newer generation.

When the lease or protected generation is unavailable, the page operation returns:

```text
409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

A malformed, unauthentic, or context-mismatched cursor remains a cursor error rather than an expiry result.

## 6. Snapshot read guard

Lease validation and snapshot acquisition must be coordinated so that a request cannot:

1. validate an active lease;
2. lose the protected snapshot to collection;
3. still return a successful page.

A conforming implementation may use a transaction, reference count, read guard, lease pin, compare-and-swap state transition, or an equivalent mechanism.

The guard must ensure that a successful page contains records from exactly one retained generation. Partial, mixed-generation, or post-collection reads are forbidden.

The read guard and cursor lease are related but distinct:

- the lease expresses retention eligibility across requests;
- the read guard protects one in-flight read;
- an active lease does not remove the need for a read guard;
- a read guard does not extend the lease unless the page completes successfully.

## 7. Retention interaction

Snapshot retention may collect a generation only when:

- no active lease protects it;
- no in-flight read guard protects it;
- the generation otherwise satisfies the retention policy.

The retention implementation must evaluate lease expiry and read-guard state atomically enough to prevent a successful reader from racing with collection.

A lease protects only the bound diagnostic snapshot. It does not retain arbitrary target objects, transition ledgers, logs, backups, or unrelated generations.

## 8. Restart behavior

Lease state required for retention protection must be durable or conservatively reconstructable after restart.

A restart must not:

- reset `issuedAt`;
- extend `absoluteExpiresAt`;
- grant a fresh idle timeout without a successful eligible page;
- convert an expired lease into an active one;
- trust incomplete or contradictory lease state.

When persisted lease state cannot be trusted, the relay may fail closed and report the generation unavailable. It must not manufacture a new lease for an old cursor.

## 9. Failure and recovery behavior

Lease-state corruption, missing bound snapshots, contradictory expiry fields, invalid authentication, or retention-state conflicts must fail closed.

The relay must not repair such state by silently changing the target, generation, cursor position, or expiry times.

Operator logs may record stable internal diagnostics, but public errors must not expose secrets, storage paths, database identifiers, lease-row identifiers, process identifiers, or raw authentication material.

## 10. Public cursor boundary

The public cursor remains opaque. Clients must not parse it or depend on its encoding.

Public responses must not expose:

- `issuedAt`, `idleExpiresAt`, or `absoluteExpiresAt` unless a separate versioned public contract defines them;
- internal lease identifiers;
- reference counts or read-guard identifiers;
- database rows, table names, or storage paths;
- worker, process, thread, or host identifiers;
- MAC keys, signing keys, bearer tokens, or authentication tags as separate fields.

An opaque cursor is not necessarily confidential. It must still be handled as untrusted input by the server.

## 11. Current checked-in implementation

The current relay implementation in `packages/relay/src/effective_view_v2.rs` encodes cursors as:

```text
<fnv1a64(targetId)>.<fnv1a64(generation)>.<offset>
```

Current behavior:

- verifies the expected target and generation prefixes;
- parses an offset;
- reads the currently retained snapshot for the target;
- rejects a generation mismatch or missing snapshot;
- does not persist cursor state;
- does not authenticate the cursor;
- does not track issue time, idle expiry, or absolute expiry;
- does not renew a lease;
- does not pin snapshots against retention;
- does not reconstruct lease state after restart.

Therefore the current implementation provides generation-bound stateless pagination only. It is not conformant with `lb.http.effective-view.diagnostic-cursor-lease.v1` and must not advertise that lease capability.

## 12. Conformance requirements

A conforming lease implementation must test at least:

1. initial issue-time calculations;
2. idle renewal after a successful page;
3. no renewal after each class of failed request;
4. absolute expiry never moves;
5. exact expiry is inactive;
6. target and generation mismatch rejection;
7. authentication or reference integrity failure;
8. repeated page-read idempotence;
9. restart without lifetime reset;
10. retention blocked by an active lease;
11. retention blocked by an in-flight read guard;
12. collection immediately after expiry when otherwise eligible;
13. no generation switch during one cursor walk;
14. no mixed or partial page during a retention race.

## 13. Non-goals

This rule does not provide:

- indefinite generation retention;
- client-controlled lease durations;
- distributed lease consensus unless separately implemented;
- authorization to read otherwise protected data;
- protection for unrelated snapshots or operational artifacts;
- cryptographic security from FNV-1a identifiers;
- formal release qualification, soak evidence, or reference-host qualification.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
