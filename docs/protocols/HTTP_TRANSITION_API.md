# HTTP Transition Publish API

**Status: draft for v0.6.0** | **API route: `POST /v1/transitions`** | **Last updated: 2026-07-20**

## 1. Purpose

Transition Objects use a dedicated HTTP publish boundary. They remain append-only protocol objects, but their schema dispatch, authority classification, graph projection, observability, and error reporting are kept separate from ordinary Knowledge Object publication.

## 2. Routes

```text
POST /v1/objects      Knowledge Object publication only
POST /v1/transitions  Transition Object publication only
```

A relay MUST NOT infer the object type and silently redirect a request to the other route.

## 3. Request envelope

A transition publish request is:

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

The top-level field MUST be named `transition`. A Transition Object wrapped in the Knowledge Object `object` field is an envelope error rather than a valid transition publication.

## 4. Signature

The request uses `lb.http.publish.signature.v1` unchanged:

1. remove only `publisher.signature`;
2. preserve `transition`, `publisher.publicKey`, and every other field;
3. canonicalize the complete remaining request with `lb.canonical.json.v1`;
4. verify Ed25519 over those UTF-8 bytes.

The route name is not part of the signature target. The envelope field name is part of the signed JSON and therefore distinguishes Knowledge Object and Transition requests cryptographically.

## 5. Validation order

A conforming relay evaluates:

1. HTTP route and method;
2. non-empty JSON request body;
3. transition publish-envelope shape;
4. Transition Object schema and semantic invariants;
5. protocol identifier and timestamp rules;
6. transition identity claims when present;
7. publisher key and signature encoding;
8. publisher signature verification;
9. append-only duplicate or conflict classification;
10. target availability classification;
11. authority classification;
12. effective-view graph projection.

Failure at an earlier stage MUST NOT be represented as success at a later stage.

## 6. Response classes

The API uses deterministic machine-readable codes.

| HTTP | Code | Meaning |
|---:|---|---|
| 201 | `LB_TRANSITION_STORED` | new transition bytes durably stored, including an orphan transition |
| 200 | `LB_TRANSITION_DUPLICATE` | exact idempotent duplicate already stored |
| 400 | `LB_TRANSITION_ENVELOPE_INVALID` | wrong or malformed request envelope |
| 400 | `LB_TRANSITION_INVALID` | structural or semantic validation failure |
| 401 | `LB_TRANSITION_SIGNATURE_INVALID` | malformed or invalid signature |
| 409 | `LB_TRANSITION_CONFLICT` | same transition ID conflicts with stored bytes |
| 422 | `LB_TRANSITION_RULE_UNSUPPORTED` | required schema, identity, or signature rule unsupported |
| 500 | `LB_TRANSITION_STORAGE_ERROR` | durable storage failure |

Authority classification or target absence is not an HTTP rejection after a structurally valid signed transition is stored. The response includes target status, authority classification, and whether the transition currently affects the effective view.

A missing target response contains fields equivalent to:

```json
{
  "status": "stored",
  "code": "LB_TRANSITION_STORED",
  "targetStatus": "missing",
  "authority": {
    "classification": "unknown",
    "basis": "target-unavailable"
  },
  "effectiveView": {
    "applied": false
  }
}
```

The relay MUST NOT return target-not-found solely because related protocol records arrived out of order.

## 7. Route isolation

The relay MUST reject:

- a Transition Object sent to `POST /v1/objects`;
- a Knowledge Object sent to `POST /v1/transitions`;
- a transition request using the top-level `object` field;
- an object request using the top-level `transition` field;
- an envelope containing both `object` and `transition`.

No route mismatch is automatically rewritten or forwarded.

## 8. Storage boundary

Publishing through `/v1/transitions` never mutates or deletes the target Knowledge Object. The Transition Object, its signed wire request, authority result, target resolution status, and derived projection state have separate lifecycle responsibilities.

Target absence is governed by `lb.transition.orphan.v1` in [ORPHAN_TRANSITIONS.md](./ORPHAN_TRANSITIONS.md). Target arrival may trigger derived-state re-evaluation but MUST NOT rewrite the stored transition, transition identity, or signature evidence.
