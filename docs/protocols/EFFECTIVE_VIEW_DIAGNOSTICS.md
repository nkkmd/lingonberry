# Effective View Diagnostics

**Status: normative v1.0 pre-release protocol contract** | **Rule version: `lb.http.effective-view.diagnostics.v1`** | **Last reviewed: 2026-07-25**

This document defines the checked-in public diagnostic representation used when an effective-view observation cannot be completed from the evidence currently available to the relay.

Diagnostics describe unusable evidence. They do not authorize a transition, repair stored state, or prove that omitted evidence does not exist.

## 1. Response placement

Diagnostics are carried in:

```text
evidenceObservation.diagnostics
```

The observation also contains:

```text
generation
snapshotClassification
diagnosticSummary
```

The current relay snapshot additionally persists the complete diagnostic set as `evidenceObservation.allDiagnostics` so the generation-bound pagination endpoint can serve entries beyond the inline summary limit. Clients must use the documented diagnostics and pagination contracts rather than treating the relay snapshot file as a storage API.

Optional headers do not replace or override the body.

## 2. Observation classification

The implemented observation classifications are:

```text
complete
incomplete
```

When the diagnostic set is empty:

- `snapshotClassification` is `complete`;
- `diagnostics` is an empty array;
- the effective view is current.

When one or more diagnostics exist:

- `snapshotClassification` is `incomplete`;
- the relay returns either a stale last-known effective view or an unresolved unavailable view;
- the current evidence observation generation remains explicit;
- diagnostics describe why the current observation could not be completed.

A successful HTTP `200` response can therefore contain an incomplete observation. HTTP success is not proof that the effective view is current.

## 3. Diagnostic entry

The implemented entry shape is:

```json
{
  "kind": "transition",
  "evidenceId": "lb:transition:t2",
  "classification": "corrupt",
  "reasonCode": "LB_EVIDENCE_INVENTORY_CONFLICT"
}
```

Every public diagnostic entry must contain:

- `kind`: the protocol evidence kind;
- `evidenceId`: the stable protocol evidence identifier associated with the problem;
- `classification`: one of `unsupported`, `corrupt`, or `unreadable`;
- `reasonCode`: a stable externally documented code.

The current effective-view implementation emits transition diagnostics. The v1 field model reserves protocol evidence kinds such as `target`, `transition`, `delegation`, and `revocation`, but a reserved kind is not evidence that the checked-in relay currently emits it.

A supported evidence item is not a diagnostic.

## 4. Classification semantics

### `unsupported`

The evidence is sufficiently readable to determine that its declared rule or schema version is not supported by the implementation.

### `corrupt`

The evidence or evidence inventory contradicts structural, integrity, signature, graph, or binding requirements.

### `unreadable`

The evidence identity is known, but required bytes or related authoritative bytes cannot currently be read.

`unreadable` must not be silently converted to not-found. `corrupt` must not be silently converted to unsupported.

## 5. Stable reason-code registry

The v1 registry includes:

| Reason code | Classification | Contract meaning |
|---|---|---|
| `LB_EVIDENCE_RULE_UNSUPPORTED` | `unsupported` | The declared rule or schema version is not supported. |
| `LB_EVIDENCE_PARSE_FAILED` | `corrupt` | Readable bytes do not parse under the declared representation. |
| `LB_EVIDENCE_VALIDATION_FAILED` | `corrupt` | Parsed evidence violates required structure or semantics. |
| `LB_EVIDENCE_DIGEST_MISMATCH` | `corrupt` | Read bytes do not match the trusted immutable carrier digest. |
| `LB_EVIDENCE_SIGNATURE_INVALID` | `corrupt` | Required signature evidence is present but invalid. |
| `LB_EVIDENCE_BYTES_UNREADABLE` | `unreadable` | Required payload or related authoritative bytes cannot currently be read. |
| `LB_EVIDENCE_INVENTORY_CONFLICT` | `corrupt` | Evidence inventory or graph references conflict. |

The checked-in transition graph currently emits `LB_EVIDENCE_INVENTORY_CONFLICT` when a transition supersedes an identifier that is absent from the authorized transition set. It emits `LB_EVIDENCE_VALIDATION_FAILED` for an unsupported transition shape and `LB_EVIDENCE_BYTES_UNREADABLE` when required target-publisher or replacement bytes cannot be obtained.

A registry entry may exist before every producer path is implemented. Implementations must not fabricate a reason code for an unrelated internal error.

Changing the externally observable meaning of an existing code requires a versioned contract change.

## 6. Optional fields

A future or compatible producer may add only fields whose semantics are stable across implementations, such as:

```text
ruleVersion
digest
```

When present:

- `ruleVersion` identifies the declared unsupported rule version;
- `digest` is an immutable carrier digest using the separately defined digest format.

The current relay diagnostic constructor emits only the four required fields.

Optional fields must not contain relay-local paths, row identifiers, mutable receipt metadata, host identity, process identity, lease identity, or exception text.

## 7. Deterministic ordering and duplicate handling

Before an observation is persisted, the checked-in relay:

1. serializes each diagnostic as canonical JSON;
2. sorts entries by that canonical JSON string;
3. removes exact duplicate entries.

This produces deterministic output for the current fixed entry shape. It is not implemented as a separate kind-priority comparator.

Clients must not infer chronology, ingestion order, severity, or transition precedence from diagnostic array order.

Entries that differ in any field are not exact duplicates. A conflict must remain observable rather than being silently selected as the winner.

## 8. Inline diagnostic summary

The inline summary limit is:

```text
20 entries
```

`diagnosticSummary` contains:

```json
{
  "total": 27,
  "returned": 20,
  "truncated": true
}
```

Semantics:

- `total` is the size of the complete deduplicated diagnostic set;
- `returned` is the number included in `diagnostics`;
- `truncated` is true when `returned < total`.

The current v2 implementation does not include a `byClassification` object in this summary. Clients must not require that older implementation detail.

When `truncated` is true, the inline array is not the complete diagnostic set. The generation-bound pagination API is the authoritative continuation mechanism.

## 9. Generation binding

Every observation carries an evidence generation identifier with this prefix:

```text
evidence:sha256:
```

The generation binds the observation to the target object and the transition evidence set used by the relay. Diagnostic pagination requires the exact retained generation.

A diagnostic entry by itself does not carry enough information to establish that it belongs to the current observation. Consumers must retain the enclosing target and generation context.

## 10. Pagination interaction

The checked-in diagnostic page operation accepts:

```text
target ID
generation
optional cursor
optional limit
```

Limits:

```text
default limit: 100
maximum limit: 100
minimum limit: 1
```

The page response includes:

```text
targetId
generation
diagnostics
page.limit
page.returned
page.nextCursor
```

The operation reads a retained effective-view snapshot and requires its `evidenceObservation.generation` to equal the requested generation. If no matching snapshot is retained, it fails with:

```text
LB_DIAGNOSTIC_GENERATION_UNAVAILABLE
```

Invalid generation syntax, limits, cursors, or cursor positions fail with stable request errors. Pagination does not recompute diagnostics from live state and does not promise access to an observation after its snapshot has been replaced or removed.

Cursor format, lease, read-guard, heartbeat, and retention semantics are defined by their separate protocol contracts.

## 11. Last-known-good behavior

When the current observation is incomplete and a prior snapshot exists, the checked-in relay returns that prior effective view with:

```text
effectiveView.freshness = stale
```

It replaces the response's `evidenceObservation` with the current incomplete observation and persists the resulting response snapshot.

When no prior snapshot exists, the relay returns:

```text
effectiveView.classification = unresolved
effectiveView.generation = null
effectiveView.freshness = unavailable
```

The original object remains available in the response, but it is not promoted to a current effective-view result.

A stale response is not authorization to apply the incomplete evidence set. An unavailable response is not evidence that the original object has no transitions.

## 12. Error boundary

Not every failure becomes a diagnostic entry.

The current relay returns an HTTP error when it cannot safely construct the observation, including failures such as:

- invalid target identifier;
- target not found;
- backend failure;
- transition ledger read or parse failure;
- evidence-generation hashing failure;
- effective-view snapshot persistence failure after a complete projection.

These request-level errors are distinct from evidence diagnostics inside an HTTP `200` response.

The relay must not convert an unknown internal failure into an unrelated stable evidence reason code.

## 13. Forbidden public data

Diagnostics and their pagination responses must not expose:

- filesystem or object-storage paths;
- database row IDs, table names, or local sequence numbers;
- stack traces or parser exception text;
- process, host, worker, thread, lease, or lock identifiers;
- credentials, bearer tokens, signing secrets, request headers, or environment values;
- mutable receipt timestamps unless another protocol contract explicitly defines them;
- storage implementation names used only for local diagnosis.

Such details belong in access-controlled operator logs and tooling.

## 14. Security and privacy boundary

Diagnostics are public protocol output when returned by the public effective-view API. Operators must assume that `evidenceId`, reason code, classification, target ID, and generation may be observable by unauthenticated clients.

Reason codes must remain bounded. Exception text and attacker-controlled strings must not be copied into reason-code fields.

A digest field, when present, is an integrity identifier. It is not automatically an authorization token, secret, signature, proof of trusted time, or proof of ownership.

## 15. Non-goals

This contract does not:

- authorize or execute semantic transitions;
- repair corrupt evidence;
- retry unreadable storage;
- define operator logging formats;
- guarantee indefinite diagnostic-generation retention;
- make the diagnostic cursor a bearer authorization credential;
- provide distributed snapshot consensus;
- prove that a last-known-good view remains semantically current;
- redefine the candidate release commit;
- satisfy formal soak or privileged reference-host qualification.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
