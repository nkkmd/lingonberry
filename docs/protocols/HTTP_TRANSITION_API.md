# HTTP Transition Publish API

**Status: normative v1.0 pre-release protocol contract** | **API route: `POST /v1/transitions`** | **Last reviewed: 2026-07-25**

This document defines the implemented HTTP boundary for publishing append-only Transition Objects. Transition publication is isolated from ordinary Knowledge Object publication and persists its own signed wire request, transition record, and reevaluation intent.

## 1. Route isolation

The relay exposes separate publication routes:

```text
POST /v1/objects
POST /v1/transitions
```

`POST /v1/transitions` accepts only a transition publish request. The implementation does not infer a request type and redirect it to another route.

The transition envelope permits exactly these top-level fields:

```text
transition
publisher
```

Unknown top-level fields are rejected. A transition sent under `object`, a request containing both `object` and `transition`, or a request missing `transition` or `publisher` is invalid.

## 2. Request envelope

A transition request has this shape:

```json
{
  "transition": {
    "id": "lb:transition:replace-0001",
    "schemaVersion": "0.1.0",
    "objectType": "transition",
    "transitionType": "replace",
    "targetId": "lb:obj:example-0001",
    "replacementId": "lb:obj:example-0002",
    "issuedAt": "2026-07-20T01:40:00Z",
    "provenance": {
      "sources": [
        {
          "protocol": "lingonberry",
          "sourceId": "transition:draft:replace-0001"
        }
      ]
    },
    "rawRef": {
      "protocol": "lingonberry",
      "sourceId": "transition:draft:replace-0001"
    }
  },
  "publisher": {
    "publicKey": "<64-lowercase-hex>",
    "signature": "<128-lowercase-hex>"
  }
}
```

The publisher object permits exactly:

```text
publicKey
signature
```

Both fields are required.

## 3. Implemented Transition Object validation

The implementation accepts only transition schema version:

```text
0.1.0
```

Required transition fields are:

```text
id
schemaVersion
objectType
transitionType
targetId
issuedAt
provenance
rawRef
```

Optional implemented fields are:

```text
replacementId
supersedesTransitionIds
reason
identityClaims
meta
```

No other transition fields are accepted.

### 3.1 Identifiers

- `id` must be an ASCII identifier beginning with `lb:transition:`;
- `targetId` must be an ASCII identifier beginning with `lb:obj:`;
- `replacementId`, when required, must begin with `lb:obj:`;
- each identifier is limited to 255 bytes;
- accepted identifier characters are ASCII alphanumeric characters plus `.`, `_`, `~`, `:`, and `-`.

### 3.2 Transition types

The implemented transition types are:

```text
replace
withdraw
```

For `replace`:

- `replacementId` is required;
- `replacementId` must be a valid bounded `lb:obj:` identifier.

For `withdraw`:

- `replacementId` must be absent.

### 3.3 Supersession

When `supersedesTransitionIds` is present:

- it must be a non-empty array;
- every entry must be a valid bounded `lb:transition:` identifier;
- duplicate entries are rejected;
- a transition must not supersede itself.

### 3.4 Timestamp and nested objects

`issuedAt` must be a string matching the implementation's RFC3339-with-zone shape check. The current validator checks for a `T` separator and either a trailing `Z` or a timezone sign. It does not perform a full calendar-validity parse.

`provenance` and `rawRef` must be JSON objects. The transition API implementation does not currently perform their full protocol-schema validation in this path.

`reason`, when present, must be a non-empty string.

Although `identityClaims` and `meta` are accepted fields, this transition ingestion path does not currently validate transition identity claims or apply a versioned identity-rule classification.

## 4. Signature contract

Transition requests use Ed25519 over canonical request bytes.

The implementation constructs the signed payload by:

1. parsing the request;
2. preserving the `transition` value;
3. copying the `publisher` object;
4. removing only `publisher.signature`;
5. serializing an object containing `publisher` and `transition` with `lb.canonical.json.v1`;
6. verifying Ed25519 directly over the resulting UTF-8 bytes.

The route name is not part of the signed payload. The `transition` envelope field is part of the payload.

Encoding requirements:

- `publisher.publicKey`: exactly 64 lowercase hexadecimal characters, decoding to 32 bytes;
- `publisher.signature`: exactly 128 lowercase hexadecimal characters, decoding to 64 bytes.

The current implementation invokes the host `openssl` executable using Ed25519 raw-message verification. A missing or failing OpenSSL process causes signature verification to fail. Deployments serving this route therefore require a compatible `openssl pkeyutl` command.

The implementation does not map an unavailable verifier to a distinct service-error code; it returns `LB_TRANSITION_SIGNATURE_INVALID` with HTTP 401.

## 5. Processing order

The implemented processing order is:

1. parse JSON;
2. require a top-level object;
3. reject unknown envelope fields;
4. require `transition` and `publisher`;
5. require the transition to be an object;
6. validate publisher field names and lowercase-hex shapes;
7. validate the implemented transition fields and invariants;
8. verify the Ed25519 signature;
9. canonicalize the complete signed request;
10. inspect the append-only transition log for the same transition ID;
11. classify exact duplicate or immutable-content conflict;
12. append a new transition record and synchronize its file data;
13. inspect target availability;
14. append a pending reevaluation intent and synchronize its file data;
15. return the HTTP result.

The route does not currently perform authority classification or effective-view graph projection inline.

## 6. Duplicate and conflict classification

The transition log is searched by `transitionId`.

An existing record is an exact duplicate when its stored request canonicalizes to the same bytes as the incoming complete request. The relay returns:

```text
HTTP 200
LB_TRANSITION_DUPLICATE
```

The duplicate path does not append a second transition record or a new reevaluation intent.

When the same transition ID is already present with different immutable request content, the relay returns:

```text
HTTP 409
LB_TRANSITION_CONFLICT
```

No replacement or mutation of the existing transition is performed.

## 7. Persistence model

New transitions are appended to:

```text
<state-dir>/transitions/append-only.jsonl
```

Each record contains:

```text
storedAtUnixSeconds
transitionId
targetId
request
```

After the transition record is appended, a pending reevaluation intent is appended to:

```text
<state-dir>/transitions/reevaluation-queue.jsonl
```

The intent contains:

```text
ruleVersion: lb.transition.reevaluation.queue.v1
targetId
triggerTransitionId
status: pending
```

Both append operations use append mode and call `sync_data`.

These two appends are not one atomic transaction. If the transition record append succeeds and the queue append fails, the API returns `LB_TRANSITION_STORAGE_ERROR`, but the transition record may already be durable. Operators and recovery tooling must not assume that every HTTP 500 means no transition was stored.

The implementation does not currently provide an automatic repair transaction for a missing reevaluation intent.

## 8. Target availability

After storing a new transition, or while returning an exact duplicate, the relay queries the Knowledge Object backend for `targetId`.

The response value is:

```text
available
missing
```

A backend result of `Ok(None)` is reported as `missing`. A backend read error is also currently collapsed to `missing`; the transition response does not distinguish unavailable storage from a genuinely absent target.

Target absence does not reject an otherwise valid transition. Out-of-order arrival is therefore permitted.

## 9. Success responses

A new transition returns HTTP 201:

```json
{
  "status": "stored",
  "code": "LB_TRANSITION_STORED",
  "transitionId": "lb:transition:replace-0001",
  "targetId": "lb:obj:example-0001",
  "targetStatus": "available"
}
```

An exact duplicate returns HTTP 200 with:

```text
status: duplicate
code: LB_TRANSITION_DUPLICATE
```

The implemented success response contains only:

```text
status
code
transitionId
targetId
targetStatus
```

It does not currently include authority classification, projection status, graph generation, queue status, or effective-view application fields.

## 10. Error responses

Errors use this body shape:

```json
{
  "status": "error",
  "code": "LB_TRANSITION_INVALID",
  "message": "..."
}
```

Implemented response classes are:

| HTTP | Code | Meaning |
|---:|---|---|
| 400 | `LB_INVALID_JSON` | JSON parsing failed |
| 400 | `LB_TRANSITION_ENVELOPE_INVALID` | envelope or publisher shape is invalid |
| 400 | `LB_TRANSITION_INVALID` | implemented transition validation failed |
| 401 | `LB_TRANSITION_SIGNATURE_INVALID` | signature encoding, verification, or verifier execution failed |
| 409 | `LB_TRANSITION_CONFLICT` | transition ID exists with different request content |
| 500 | `LB_TRANSITION_STORAGE_ERROR` | directory creation, log inspection, transition append, or queue append failed |

The current route does not emit `LB_TRANSITION_RULE_UNSUPPORTED` or HTTP 422 as a separate classification. Unsupported `schemaVersion` is returned as `LB_TRANSITION_INVALID` with HTTP 400.

## 11. Separation from effective-view processing

Publishing a Transition Object does not mutate or delete the target Knowledge Object.

The route persists the transition and enqueues a reevaluation intent. It does not prove that:

- authority was accepted;
- the target was successfully loaded;
- a transition graph was projected;
- the transition affects the current effective view;
- a later arriving target will be reevaluated automatically;
- the pending intent has been consumed.

Those guarantees require separate reevaluation, graph, authority, and effective-view processing contracts.

## 12. Security and operational boundary

A valid publisher signature proves possession of the private key corresponding to `publisher.publicKey` for the canonical transition request bytes. It does not by itself establish authorization to affect a target.

The transition route does not currently provide:

- replay-window enforcement;
- nonce or request-expiry enforcement;
- key revocation;
- publisher authorization policy;
- trusted timestamp validation;
- rate limiting within the transition module;
- atomic transaction-plus-queue publication;
- distributed locking or multi-host serialization;
- cryptographic acknowledgement of durable storage.

Filesystem permissions must protect the append-only log and reevaluation queue from unauthorized mutation.

## 13. Compatibility

Any change to the transition envelope, signature payload, identifier rules, validation semantics, duplicate classification, storage record shape, queue intent shape, or response contract requires compatibility review and may require a new versioned contract.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
