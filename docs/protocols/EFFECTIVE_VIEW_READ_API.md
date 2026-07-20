# Effective View Read API

## Status

Normative foundation for v0.6.0 effective-view reads.

Rule version: `lb.http.effective-view.read.v1`

## Endpoint

```text
GET /v1/effective-objects/{targetId}
```

A last-known-good result is returned with `200 OK` even when the newest evidence observation is incomplete. Staleness and diagnostics are mandatory in the response body.

## Response body

```json
{
  "effectiveObject": {
    "id": "lb:obj:replacement-1"
  },
  "effectiveView": {
    "classification": "replaced",
    "generation": "evidence:sha256:<semantic generation>",
    "freshness": "stale"
  },
  "evidenceObservation": {
    "generation": "evidence:sha256:<latest observation generation>",
    "snapshotClassification": "incomplete",
    "diagnosticSummary": {
      "total": 245,
      "returned": 20,
      "truncated": true,
      "byClassification": {
        "unsupported": 12,
        "corrupt": 228,
        "unreadable": 5
      }
    },
    "diagnostics": [
      {
        "kind": "transition",
        "evidenceId": "lb:transition:t2",
        "classification": "corrupt",
        "reasonCode": "LB_EVIDENCE_PARSE_FAILED"
      }
    ]
  }
}
```

`effectiveView.freshness` is `current`, `stale`, or `unavailable`.

The body is authoritative. Implementations may also emit `Lingonberry-View-Freshness`, but clients MUST NOT depend on the header instead of the body.

## Public diagnostics

`evidenceObservation.diagnostics` follows `lb.http.effective-view.diagnostics.v1` in `EFFECTIVE_VIEW_DIAGNOSTICS.md`.

Public diagnostics expose stable protocol fields and reason codes only. Filesystem paths, database identifiers, stack traces, parser exception text, worker identifiers, and other relay-internal details are forbidden.

Diagnostics are deterministically ordered and exact duplicates are collapsed. Conflicting diagnostic content for the same evidence kind and identifier is not silently selected.

## Diagnostic response bound

The normal effective-view response returns at most 20 diagnostics. `diagnosticSummary.total` and `byClassification` describe the complete diagnostic set for the stated observation generation. `returned` is the number present in this response and `truncated` is exactly `returned < total`.

The complete diagnostic set is available through the generation-bound pagination contract in `EFFECTIVE_VIEW_DIAGNOSTIC_PAGINATION.md`:

```text
GET /v1/effective-objects/{targetId}/diagnostics
```

The normal read endpoint MUST NOT return an unbounded diagnostic array.

## Status codes

- `200`: target and effective-view state are readable, including stale last-known-good state.
- `404`: target Knowledge Object is not known.
- `500`: storage corruption or I/O failure prevents a trustworthy response.

An incomplete latest observation alone is not a `409` or `503` condition.

## Safety

- Do not label a stale semantic result as current.
- Do not omit the latest observation generation.
- Do not suppress the diagnostic summary for unusable evidence.
- Do not hide diagnostic truncation.
- Do not expose implementation-specific error details through the public API.
- Do not apply semantic effects from an incomplete generation.
- Do not convert storage corruption into a normal stale response.
