# Effective View Diagnostic Pagination

## Status

Normative foundation for v0.6.0 diagnostic reads.

Rule version: `lb.http.effective-view.diagnostic-pagination.v1`

## Summary response

`GET /v1/effective-objects/{targetId}` returns at most 20 diagnostics in the normal response.

```json
{
  "evidenceObservation": {
    "generation": "evidence:sha256:<observation generation>",
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
    "diagnostics": []
  }
}
```

`total` and `byClassification` describe the complete diagnostic set for the stated observation generation. `returned` is the number included in this response. `truncated` is exactly `returned < total`.

Diagnostics use the deterministic order defined by `lb.http.effective-view.diagnostics.v1`. The summary response contains the first 20 entries in that order.

## Complete diagnostic endpoint

```text
GET /v1/effective-objects/{targetId}/diagnostics
```

Query parameters:

- `generation` is REQUIRED and MUST be an evidence generation identifier.
- `cursor` is optional and opaque.
- `limit` is optional, defaults to 100, and MUST be between 1 and 100 inclusive.

The endpoint returns diagnostics only for the requested immutable observation generation.

```json
{
  "targetId": "lb:obj:target-1",
  "generation": "evidence:sha256:<observation generation>",
  "diagnostics": [],
  "page": {
    "limit": 100,
    "returned": 100,
    "nextCursor": "<opaque cursor or null>"
  }
}
```

## Snapshot isolation

A page request MUST NOT silently switch to a newer observation generation.

- If the requested generation remains available, return that generation even when a newer generation is current.
- If the requested generation is unknown or no longer retained, return `409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE`.
- A cursor is valid only with the target ID and generation for which it was issued.
- Cursor mismatch, tampering, or malformed encoding returns `400 LB_DIAGNOSTIC_CURSOR_INVALID`.

## Cursor boundary

The cursor is an opaque implementation token. It MUST NOT expose filesystem paths, database row IDs, table names, ingestion sequence numbers, or worker identifiers. Clients MUST NOT parse it.

## Status codes

- `200`: page returned for the requested generation.
- `400 LB_DIAGNOSTIC_CURSOR_INVALID`: malformed, tampered, or context-mismatched cursor or invalid limit.
- `404 LB_TARGET_NOT_FOUND`: target Knowledge Object is unknown.
- `409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE`: requested generation is not available for stable paging.
- `500 LB_DIAGNOSTIC_STORAGE_ERROR`: a trustworthy response cannot be produced.

## Safety

- Do not return an unbounded diagnostic array from the normal effective-view endpoint.
- Do not report an approximate `total` as exact.
- Do not mix diagnostics from different generations in one pagination sequence.
- Do not hide truncation.
- Do not encode relay-internal storage identifiers in public cursors.
