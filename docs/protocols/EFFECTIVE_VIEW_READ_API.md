# Effective View Read API

**Status: normative v1.0 pre-release protocol contract** | **Rule version: `lb.http.effective-view.read.v1`** | **Last reviewed: 2026-07-25**

This document defines the checked-in public HTTP read contract for a target object's effective view. The endpoint can return a current view, a stale last-known-good view with a newer incomplete observation, or an unresolved unavailable view when no prior complete view exists.

HTTP success does not imply semantic freshness. Clients must inspect the response body.

## 1. Endpoint

```text
GET /v1/effective-objects/{targetId}
```

`targetId` must be a valid canonical Knowledge Object identifier beginning with:

```text
lb:obj:
```

The release HTTP router removes any query string before passing the target ID to the effective-view operation. This endpoint does not define query parameters.

## 2. Successful response model

A successful response contains:

```text
effectiveObject
originalObject
effectiveView
evidenceObservation
```

Example current response:

```json
{
  "effectiveObject": {
    "id": "lb:obj:replacement-1"
  },
  "originalObject": {
    "id": "lb:obj:target-1"
  },
  "effectiveView": {
    "classification": "replaced",
    "generation": "evidence:sha256:<semantic-generation>",
    "freshness": "current"
  },
  "evidenceObservation": {
    "generation": "evidence:sha256:<observation-generation>",
    "snapshotClassification": "complete",
    "diagnosticSummary": {
      "total": 0,
      "returned": 0,
      "truncated": false
    },
    "diagnostics": [],
    "allDiagnostics": []
  }
}
```

The checked-in relay persists and currently serializes `evidenceObservation.allDiagnostics` because the generation-bound diagnostic page operation reads the complete set from the retained snapshot. Clients should use `diagnostics`, `diagnosticSummary`, and the documented pagination endpoint rather than depend on `allDiagnostics` as a stable public field.

## 3. Effective-view classification

The implemented `effectiveView.classification` values are:

```text
original
replaced
withdrawn
ambiguous
unresolved
```

Semantics:

- `original`: no authorized active transition changes the target;
- `replaced`: one active authorized replacement transition selects a readable replacement object;
- `withdrawn`: one active authorized withdrawal transition removes the effective object, so `effectiveObject` is `null`;
- `ambiguous`: more than one authorized transition head remains and the relay preserves the original object rather than selecting one effect;
- `unresolved`: the latest observation is incomplete and no prior complete effective-view snapshot is available.

An unsupported transition shape produces an incomplete observation rather than a new public classification.

## 4. Freshness

The implemented `effectiveView.freshness` values are:

```text
current
stale
unavailable
```

### `current`

The effective view was computed from the same complete evidence observation identified by `effectiveView.generation` and `evidenceObservation.generation`.

### `stale`

The relay loaded a previously persisted effective-view snapshot, retained its semantic result and generation, changed `freshness` to `stale`, and replaced `evidenceObservation` with the newer incomplete observation.

Therefore, in a stale response:

```text
effectiveView.generation != evidenceObservation.generation
```

can be expected. The first generation identifies the last-known-good semantic view; the second identifies the latest evidence observation that could not be applied.

### `unavailable`

No prior complete effective-view snapshot was available. The relay returns:

```text
effectiveView.classification = unresolved
effectiveView.generation = null
effectiveView.freshness = unavailable
```

The original object remains present as both `effectiveObject` and `originalObject` in the checked-in implementation. Clients must not treat that fallback payload as a current semantic decision.

## 5. Evidence observation

`evidenceObservation` contains:

```text
generation
snapshotClassification
diagnosticSummary
diagnostics
allDiagnostics
```

The implemented snapshot classifications are:

```text
complete
incomplete
```

A complete observation has no diagnostics. An incomplete observation contains one or more diagnostics and cannot change the effective semantic generation.

Diagnostics follow `EFFECTIVE_VIEW_DIAGNOSTICS.md`.

The inline diagnostic limit is:

```text
20
```

`diagnosticSummary` contains exactly:

```text
total
returned
truncated
```

The checked-in implementation does not emit `byClassification`.

Diagnostics are sorted by canonical JSON text and exact duplicates are removed before summary construction. Array order does not represent chronology or severity.

## 6. Last-known-good behavior

When the latest observation is incomplete:

1. the relay attempts to load the previously persisted snapshot for the target;
2. when found, the prior effective object, original object, classification, and semantic generation are retained;
3. freshness becomes `stale`;
4. the latest incomplete `evidenceObservation` replaces the prior observation;
5. the updated stale snapshot is persisted on a best-effort basis;
6. the response is `200 OK`.

When no previous snapshot exists, the relay builds and best-effort persists an unresolved unavailable response and still returns `200 OK`.

A failed best-effort snapshot write in an incomplete-response path does not replace the semantic error with an HTTP `500`. In contrast, failure to persist a newly computed complete view returns a storage error.

## 7. Persistence boundary

The checked-in relay stores one effective-view snapshot per target beneath the relay state directory. A new persisted response replaces the previously retained snapshot for that target.

This storage behavior provides last-known-good fallback and generation-bound diagnostic pagination for the currently retained snapshot. It does not provide:

- multi-generation diagnostic retention;
- cursor leases;
- read guards;
- snapshot garbage-collection coordination;
- distributed snapshot replication;
- transactional consistency with concurrent external writers.

The read operation recomputes the evidence generation from the target object and transition evidence on each request before selecting current, stale, or unavailable behavior.

## 8. Authorization and transition projection

The relay reads transition records associated with the target and compares each transition publisher key with the target publisher key.

Current projection behavior includes:

- unauthorized publisher transitions are not applied;
- a missing readable target publisher produces an `unreadable` diagnostic;
- superseding a transition ID absent from the authorized transition set produces a `corrupt` inventory-conflict diagnostic;
- one active `replace` transition requires the replacement object to be readable;
- one active `withdraw` transition produces `withdrawn`;
- multiple active heads produce `ambiguous`;
- an unsupported single active transition shape produces a validation diagnostic.

The endpoint does not repair transition evidence, resolve ambiguous heads, or authorize publishers.

## 9. HTTP status codes

The implemented status behavior is:

| Status | Code | Meaning |
|---|---|---|
| `200 OK` | none | Current, stale, or unavailable effective-view response was constructed. |
| `400 Bad Request` | `LB_TARGET_ID_INVALID` | The target ID does not satisfy the canonical target-ID shape. |
| `404 Not Found` | `LB_TARGET_NOT_FOUND` | No target object is stored for the requested ID. |
| `500 Internal Server Error` | storage backend code | Reading the target object failed. |
| `500 Internal Server Error` | `LB_TRANSITION_STORAGE_ERROR` | Transition evidence could not be read or parsed. |
| `500 Internal Server Error` | `LB_EVIDENCE_GENERATION_FAILED` | The evidence generation could not be constructed. |
| `500 Internal Server Error` | `LB_EFFECTIVE_VIEW_STORAGE_ERROR` | A newly computed complete snapshot could not be persisted. |

An incomplete latest observation alone is not a `409`, `500`, or `503` condition.

Error responses use the shared shape:

```json
{
  "status": "error",
  "code": "LB_TARGET_NOT_FOUND",
  "message": "target object not found"
}
```

Error messages are protocol-facing summaries. Filesystem paths, stack traces, database row identifiers, credentials, and exception internals must not be exposed.

## 10. Headers and representation

The checked-in release server returns:

```text
Content-Type: application/json; charset=utf-8
Connection: close
```

The body is serialized using the repository's canonical JSON serializer.

The current server does not emit `Lingonberry-View-Freshness`. Clients must use `effectiveView.freshness` in the response body and must not require an optional freshness header.

## 11. Diagnostic pagination relation

The complete retained diagnostic set can be requested through:

```text
GET /v1/effective-objects/{targetId}/diagnostics?generation=<generation>&limit=<1..100>&cursor=<optional>
```

Pagination is bound to the exact retained observation generation. It does not silently switch to the current semantic generation or a newly computed generation.

The normal effective-view endpoint has no pagination parameters and returns at most 20 entries in `evidenceObservation.diagnostics`.

## 12. Safety requirements

Implementations and clients must not:

- treat `200 OK` as proof that freshness is `current`;
- label a stale semantic generation as current;
- apply semantic effects from an incomplete observation;
- omit the latest observation generation from stale or unavailable responses;
- treat an unavailable fallback object as a resolved semantic result;
- hide inline diagnostic truncation;
- require the removed `byClassification` summary field;
- silently switch diagnostic pagination to a different generation;
- convert storage corruption or I/O failure into a normal stale response;
- expose relay-local paths, process identity, database identifiers, credentials, or exception details;
- infer distributed or transactional snapshot guarantees from the local retained snapshot.

## 13. Non-goals

This contract does not provide:

- semantic conflict resolution;
- automatic repair of corrupt evidence;
- historical effective-view queries;
- multi-generation retention guarantees;
- cursor lease or read-guard protection;
- cryptographic authentication of the retained snapshot file;
- distributed consistency;
- reference-host qualification, formal soak completion, or release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
