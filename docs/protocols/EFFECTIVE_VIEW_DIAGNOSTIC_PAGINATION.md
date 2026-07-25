# Effective View Diagnostic Pagination

**Status: normative v1.0 pre-release protocol contract** | **Rule version: `lb.http.effective-view.diagnostic-pagination.v1`** | **Last reviewed: 2026-07-25**

This document defines generation-bound pagination for effective-view diagnostics and records the exact behavior and limitations of the checked-in relay implementation.

## 1. Inline summary

`GET /v1/effective-objects/{targetId}` returns at most 20 diagnostics in the normal effective-view response.

The implemented summary shape is:

```json
{
  "evidenceObservation": {
    "generation": "evidence:sha256:<observation-generation>",
    "snapshotClassification": "incomplete",
    "diagnosticSummary": {
      "total": 245,
      "returned": 20,
      "truncated": true
    },
    "diagnostics": []
  }
}
```

Semantics:

- `total` is the size of the complete sorted and deduplicated diagnostic set stored in the snapshot;
- `returned` is the number included in the inline `diagnostics` array;
- `truncated` is exactly `returned < total`;
- the checked-in v2 implementation does not emit `byClassification` in `diagnosticSummary`;
- the inline diagnostics are the first entries in the deterministic canonical-JSON order defined by the diagnostics contract.

The snapshot also stores the complete set in an implementation field named `allDiagnostics`. Clients must access that set through the pagination endpoint, not by reading relay state files.

## 2. Complete diagnostic operation

The HTTP operation is:

```text
GET /v1/effective-objects/{targetId}/diagnostics
```

Inputs:

- `generation` is required and must begin with `evidence:sha256:`;
- `cursor` is optional;
- `limit` is optional;
- default `limit` is `100`;
- valid `limit` range is `1..=100`.

The response shape is:

```json
{
  "targetId": "lb:obj:target-1",
  "generation": "evidence:sha256:<observation-generation>",
  "diagnostics": [],
  "page": {
    "limit": 100,
    "returned": 100,
    "nextCursor": "<opaque-cursor-or-null>"
  }
}
```

`nextCursor` is `null` when the returned page reaches the end of the retained diagnostic set.

## 3. Snapshot source

The checked-in relay stores one effective-view snapshot per target under relay-managed state. The pagination operation:

1. loads the currently retained snapshot for the target;
2. reads its `evidenceObservation`;
3. requires the stored generation to equal the requested generation;
4. reads the stored complete diagnostic set;
5. slices that set by cursor offset and requested limit.

Pagination does not recompute diagnostics from current transition state.

The implementation does not retain an arbitrary history of immutable diagnostic snapshots. A newer effective-view computation may replace the target's stored snapshot. Therefore “generation-bound” means that a page is returned only when the currently retained snapshot still matches the requested generation; it is not a promise of indefinite generation retention.

## 4. Generation isolation

A page request must not silently switch to another generation.

- If the retained snapshot generation equals the requested generation, the operation returns diagnostics from that snapshot.
- If the target snapshot is absent, malformed for pagination, or has another generation, the operation returns:

```text
409 LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

- The operation does not fall back to the newest generation.
- The operation does not merge diagnostics from multiple generations.
- The response repeats the exact requested generation.

A generation identifier is an integrity-oriented observation identifier. It is not an authorization token or a guarantee that the snapshot remains retained.

## 5. Current cursor encoding

The public cursor is an implementation token and clients must treat it as opaque.

The checked-in relay currently encodes:

```text
<fnv1a64(targetId)>.<fnv1a64(generation)>.<offset>
```

On decode, it verifies the expected target and generation prefixes and parses the decimal offset.

Current limitations:

- FNV-1a prefixes are not cryptographic authentication;
- the cursor is not confidential;
- the cursor has no issue time, idle expiry, or absolute expiry;
- the cursor does not retain or pin a snapshot;
- the cursor is not durable server-side lease state;
- the offset can be replayed while the same generation snapshot remains retained.

The cursor-lease contract is a separate capability. The current stateless cursor is not conformant with that lease contract.

## 6. Cursor validation

The operation returns `400 LB_DIAGNOSTIC_CURSOR_INVALID` when:

- the cursor does not contain the expected target and generation prefixes;
- the offset is not parseable as an unsigned position;
- the offset is greater than the complete diagnostic count.

A cursor produced for one target or generation must not be accepted for another.

The current implementation detects context mismatch through deterministic prefixes, not through a signature or MAC. Documentation and capability advertisement must not describe this as cryptographic tamper protection.

## 7. Ordering and page stability

Before persistence, diagnostics are:

1. serialized as canonical JSON;
2. sorted by that canonical JSON string;
3. deduplicated by exact equality.

Pagination preserves this stored order.

For one retained snapshot and one cursor sequence:

- page boundaries are deterministic for the same limit;
- no entry is intentionally reordered between page requests;
- cursor offset identifies the next array position;
- page order does not imply chronology, severity, or evidence precedence.

Because the checked-in implementation has no lease or read guard, it does not guarantee that a snapshot cannot be replaced between separate requests. If replacement occurs, the next request fails generation matching rather than switching generations.

## 8. Limits and arithmetic

The implementation validates `limit` before reading the page.

```text
minimum: 1
maximum: 100
default: 100
```

The page end is the smaller of:

```text
offset + limit
diagnostic count
```

An implementation must avoid integer overflow when computing page boundaries. The checked-in Rust implementation uses bounded request limits and snapshot lengths represented by `usize`.

## 9. Error boundary

The checked-in operation uses these stable request errors:

| HTTP status | Code | Meaning |
|---|---|---|
| `400` | `LB_TARGET_ID_INVALID` | Target identifier is invalid. |
| `400` | `LB_DIAGNOSTIC_GENERATION_INVALID` | Generation syntax is invalid. |
| `400` | `LB_DIAGNOSTIC_LIMIT_INVALID` | Limit is outside `1..=100`. |
| `400` | `LB_DIAGNOSTIC_CURSOR_INVALID` | Cursor context, encoding, or position is invalid. |
| `409` | `LB_DIAGNOSTIC_GENERATION_UNAVAILABLE` | A matching retained snapshot is unavailable. |

The implementation-level operation does not independently fetch the target object before serving a retained page. HTTP routing or another layer may impose additional target existence checks, but clients must not assume a `404` is produced by the pagination function itself.

Unexpected storage or serialization failures must not be converted to a successful empty page.

## 10. Public-data boundary

The cursor and response must not expose:

- filesystem or object-storage paths;
- database row IDs or table names;
- process, worker, thread, lease, or host identifiers;
- credentials, request headers, environment values, or signing material;
- raw exception text;
- mutable local ingestion sequence numbers presented as protocol identifiers.

The current cursor exposes deterministic hashes and an offset. Clients must still treat it as opaque and untrusted.

## 11. Retention, lease, and read-guard interaction

This pagination contract does not itself provide retention.

A relay that advertises the separate cursor-lease and read-guard capabilities must coordinate pagination with those contracts. The checked-in relay currently does not implement those capabilities.

Without them:

- a generation remains page-readable only while its target snapshot remains stored;
- separate requests are not protected by a durable lease;
- an in-flight read is not represented by a protocol-defined read guard;
- generation mismatch fails closed with `LB_DIAGNOSTIC_GENERATION_UNAVAILABLE`.

## 12. Conformance requirements

A conforming pagination implementation must test at least:

1. inline truncation at 20 entries;
2. exact `total`, `returned`, and `truncated` values;
3. default, minimum, and maximum limits;
4. invalid limit rejection;
5. first, middle, and final pages;
6. empty diagnostic set;
7. target and generation cursor binding;
8. malformed cursor rejection;
9. offset beyond the retained set;
10. generation mismatch rejection;
11. no silent generation switch;
12. deterministic ordering and exact deduplication;
13. `nextCursor = null` at completion;
14. replacement of the retained snapshot between requests;
15. no disclosure of relay-local storage identifiers.

## 13. Non-goals

This contract does not provide:

- indefinite diagnostic snapshot history;
- cryptographic cursor authentication in the checked-in implementation;
- cursor lease or snapshot pinning;
- read-guard protection;
- distributed snapshot consensus;
- authorization to access otherwise protected data;
- proof that diagnostics omitted from a collected snapshot never existed;
- formal release qualification, soak evidence, or reference-host qualification.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
