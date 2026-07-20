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
    "diagnostics": [
      {
        "kind": "transition",
        "evidenceId": "lb:transition:t2",
        "classification": "corrupt"
      }
    ]
  }
}
```

`effectiveView.freshness` is `current`, `stale`, or `unavailable`.

The body is authoritative. Implementations may also emit `Lingonberry-View-Freshness`, but clients MUST NOT depend on the header instead of the body.

## Status codes

- `200`: target and effective-view state are readable, including stale last-known-good state.
- `404`: target Knowledge Object is not known.
- `500`: storage corruption or I/O failure prevents a trustworthy response.

An incomplete latest observation alone is not a `409` or `503` condition.

## Safety

- Do not label a stale semantic result as current.
- Do not omit the latest observation generation.
- Do not suppress diagnostics for unusable evidence.
- Do not apply semantic effects from an incomplete generation.
- Do not convert storage corruption into a normal stale response.
