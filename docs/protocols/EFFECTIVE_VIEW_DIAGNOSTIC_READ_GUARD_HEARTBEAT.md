# Effective View Diagnostic Read Guard Heartbeat

## Status

Normative foundation for v0.6.0 bounded read-guard renewal.

Rule version: `lb.http.effective-view.diagnostic-read-guard-heartbeat.v1`

## Parameters

```text
guard idle lifetime: 120 seconds
guard absolute lifetime: 600 seconds
```

A guard heartbeat may extend only the idle expiry:

```text
guardExpiresAt = min(heartbeatAt + 120 seconds, guardIssuedAt + 600 seconds)
```

The absolute expiry is fixed at guard creation and MUST NOT be extended or reset after restart.

## Progress requirement

A heartbeat is valid only when the guarded page operation has made durable or directly observable materialization progress since the previous heartbeat. Timer-only heartbeats, duplicate progress tokens, and heartbeats from another target, generation, snapshot identity, or operation identity are invalid.

A valid progress token is relay-internal and monotonic for the exact guarded operation. It MUST NOT be exposed in the public HTTP response.

## Completion

A page may be returned only when materialization completes before both idle and absolute expiry. If either expiry is reached, the operation fails closed and MUST NOT return a partial page.

The cursor lease and read guard are independent. Expiry of the cursor lease after a page operation has acquired a valid read guard does not cancel that already-running operation. The read guard still cannot survive its absolute expiry.

## Failure behavior

Invalid heartbeat, stale progress, identity mismatch, expired guard, or absolute-expiry exhaustion does not extend the guard. A read that cannot complete under a valid guard returns:

```text
500 LB_DIAGNOSTIC_PAGE_READ_FAILED
```

The implementation MUST release the guard idempotently after success or failure. Expired guards are eligible for reconciliation and garbage collection.

## Safety

- Do not renew a guard without materialization progress.
- Do not extend the 600-second absolute expiry.
- Do not reset guard age after restart.
- Do not let a heartbeat change target, generation, snapshot identity, or operation identity.
- Do not return a partial page after guard expiry.
- Do not expose guard identifiers or progress tokens through the public API.
