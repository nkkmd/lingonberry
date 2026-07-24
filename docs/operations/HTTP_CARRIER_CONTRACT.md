# HTTP Carrier Contract

**Status: v1.0 pre-release normative**

This document defines the implemented HTTP carrier surface for the Lingonberry v1.0 reference relay. English is normative.

## 1. Scope

The public HTTP listener exposes protocol-object publication, object retrieval, capability discovery, readiness, and the implemented transition/read surfaces. Administrative quarantine operations are served by a separate authenticated listener.

HTTP is a carrier for protocol-native request and response objects. It does not redefine canonicalization, identity, provenance, acceptance policy, storage recovery, or administrator authorization.

## 2. Listener separation

### 2.1 Public listener

The public listener is started by the relay HTTP command and accepts public carrier routes. Requests for administrator-only paths on this listener return `404 Not Found` rather than exposing the existence of the administrative API.

The public listener does not accept administrator Bearer credentials as a way to unlock hidden routes.

### 2.2 Administrator listener

The administrator listener is a separately bound process surface. It requires configured role credentials before binding and applies role-based authentication and authorization before routing the request.

Administrator authentication and authorization are defined by [Secret Management](./SECRET_MANAGEMENT.md) and the quarantine-specific operation documents. They are not part of public object acceptance.

Expected authentication outcomes are:

| Condition | HTTP status |
|---|---:|
| Missing or unknown administrator credential | `401` |
| Recognized role without permission for the route | `403` |
| Route is not an administrator route | `404` |

Authentication or authorization does not bypass object validation, acceptance policy, conflict detection, or quarantine state rules.

## 3. Public routes

The v1 reference relay provides these core public routes:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/v1/objects` | Publish one HTTP publish-request envelope |
| `GET` | `/v1/objects/<canonical-id>` | Retrieve one stored canonical object |
| `GET` | `/v1/capabilities` | Read the relay capability manifest |
| `GET` | `/v1/ready` | Confirm that the public HTTP listener accepted and routed a readiness request |

Additional versioned transition and effective-view routes are documented by their protocol contracts. Their presence must be discovered from the checked-in capability surface and tests rather than inferred from this core route table.

Unknown routes return `404`. Unsupported methods return `405` where the matched route implements explicit method handling.

## 4. Publish request

### 4.1 Endpoint

```text
POST /v1/objects
Content-Type: application/json
```

The request body is the versioned HTTP publish-request envelope defined by the schema and signature contract. The envelope contains a protocol-native knowledge object and publisher material required by the active request contract.

A simplified shape is:

```json
{
  "object": {
    "...": "protocol-native knowledge object"
  },
  "publisher": {
    "publicKey": "...",
    "signature": "..."
  }
}
```

The schema and [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md) document are authoritative for exact fields, canonical signing bytes, key encoding, and signature verification. This operations document does not create an alternative signature format.

### 4.2 Processing order

The relay processes a publish request through these boundaries:

1. read the HTTP request body;
2. parse JSON and validate the publish-request structure;
3. apply schema and identity validation;
4. apply the configured acceptance policy;
5. defer eligible unsupported identity rules to quarantine, or reject hard failures;
6. finalize the knowledge object;
7. append through the storage backend;
8. classify stored, duplicate, conflict, or operational failure;
9. serialize the versioned publish-ingestion result.

The acceptance decision and the final ingestion result are different layers. An accepted request can still become a storage conflict or operational failure.

## 5. Publish response contract

Publish responses use the versioned ingestion-result object. The stable fields include:

```json
{
  "contractVersion": "1",
  "status": "stored",
  "code": "LB_OBJECT_STORED",
  "stored": true,
  "duplicate": false,
  "errors": [],
  "canonicalId": "...",
  "identityKey": "...",
  "carrierIdentity": "...",
  "storedAt": "...",
  "object": {}
}
```

Optional fields are present only when meaningful. In particular, rejected, deferred, conflict, and failed results do not claim a canonical ID or stored object. A deferred result may include `quarantineId`.

The semantic statuses are:

| Status | Meaning |
|---|---|
| `stored` | A new canonical record was appended |
| `duplicate` | The request resolved to an already stored canonical record; this is idempotent success |
| `deferred` | The active acceptance policy placed the original request in quarantine |
| `rejected` | Parsing, validation, signature, or acceptance requirements rejected the request |
| `conflict` | The canonical identity conflicts with existing stored state |
| `failed` | Storage, quarantine persistence, or another operational dependency failed |

Clients must inspect both the HTTP status and the response body. They must not reduce all `2xx` responses to `stored`, and must not treat every non-`2xx` response as a validation rejection.

## 6. HTTP status mapping

The checked-in implementation maps semantic outcomes to HTTP transport results. Exact mappings are part of the relay tests and may be refined only through a versioned compatibility review.

The v1 operational interpretation is:

| Semantic outcome | HTTP class | Notes |
|---|---:|---|
| newly stored | `2xx` | successful write |
| duplicate | `2xx` | idempotent success, explicitly marked duplicate |
| deferred | `202` | request is not in canonical storage; inspect `quarantineId` |
| malformed or rejected | `4xx` | inspect classified `code` and `errors` |
| not found | `404` | route or requested object absent |
| conflict | `409` | existing canonical state prevents the requested write |
| method not allowed | `405` | route exists but method is unsupported |
| operational failure | `5xx` | do not relabel as policy rejection |

No generic `requestId` wrapper is guaranteed by the v1 public carrier. Clients must not require undocumented `status: ok`, `data`, or nested `error` envelopes when the route returns a route-specific contract object.

A generic error body produced by route handling has an error classification and message, but it is not a promise that all route-specific failures share one universal schema.

## 7. Retrieve

### 7.1 Endpoint

```text
GET /v1/objects/<canonical-id>
```

A successful response returns the implemented stored-object representation. Clients must use the actual response contract and must not assume the draft-era wrapper:

```json
{
  "status": "ok",
  "canonical": {},
  "rawRef": {}
}
```

unless those fields are present in the route response being consumed.

Retrieval does not re-run publish acceptance and does not grant permission to mutate the record. Missing objects return `404`.

## 8. Capabilities

### 8.1 Endpoint

```text
GET /v1/capabilities
```

The response is generated by the implementation and is the discovery source for supported operations, schemas, protocol identifiers, and feature-specific surfaces.

Clients must not hard-code the obsolete illustrative capability object that previously appeared in this document. In particular, values such as protocol version `0.1.0`, object-type lists, authentication modes, content-type lists, access-scope lists, and retention-hint lists are not guaranteed merely because they appeared in an old example.

Capability discovery is descriptive, not authorization. A capability entry does not make an administrator route public, does not bypass acceptance policy, and does not prove storage readiness.

## 9. Readiness

### 9.1 Endpoint

```text
GET /v1/ready
```

A successful response demonstrates that the public relay listener is bound, accepted the connection, parsed the request, and routed the readiness endpoint.

It does not prove:

- strict storage verification;
- backup validity;
- migration completion;
- quarantine consistency;
- privileged disk-pressure qualification;
- formal soak completion.

Deep storage checks remain the responsibility of `lingonberry-storage ready`, `doctor`, or `verify` under the operator runbooks.

## 10. Request framing

The reference server implements a bounded HTTP/1.1 request parser suitable for the checked-in contract tests. Operators must place an appropriate reverse proxy in front of an Internet-facing listener and must not infer production-grade proxy, TLS, compression, streaming, CORS, or denial-of-service behavior that is not explicitly implemented and tested.

The reverse proxy is responsible for public TLS termination and external publication policy. It must preserve the request method, path, body, and required signature material.

## 11. Signature and provenance boundary

Publisher signature verification protects the versioned publish-request contract. It is not administrator authentication and it does not establish the truth of the object content.

A valid signature does not guarantee acceptance. The request can still be rejected for schema or identity errors, deferred under acceptance policy, conflict with stored state, or fail operationally.

The relay must not rewrite signed fields before verification. Canonical object identity and stored provenance are produced by the protocol, validation, finalization, and storage layers rather than invented by the HTTP status code.

## 12. Access and retention

The implemented defaults are governed by [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md). HTTP transport does not implement `public`, `curated`, and `private` as three authorization modes merely because those terms appeared in an earlier draft.

`accessScope=public` and `retentionHint=long-lived` are protocol or carrier policy metadata. They do not provide confidentiality, automatic expiry, or deletion authorization.

## 13. Quarantine and administrator routes

Deferred publication writes the original request and classified reasons to quarantine rather than canonical storage. Public publish responses may disclose the resulting quarantine ID as part of the versioned ingestion result.

Inspection, annotation, promotion, dismissal, permanent rejection, replacement operations, status, and quarantine metrics are administrator operations where implemented. They belong on the authenticated administrator listener and are governed by role permissions.

The public listener intentionally returns `404` for administrator paths.

Administrator metrics may use a route-specific text format. This does not establish a universal Prometheus endpoint for the public carrier.

## 14. Observability boundary

The v1 carrier does not guarantee a universal structured-log schema containing `requestId`, `durationMs`, event names, or generic metric families for every request.

Use route responses, process exit status, systemd state, journald diagnostics, storage snapshots, and the limited authenticated administrator metrics described by [Observability](./OBSERVABILITY.md).

Do not include Bearer credentials, publisher private keys, environment files, or unredacted sensitive request bodies in evidence bundles.

## 15. Compatibility rules

A change requires compatibility review when it alters any of the following:

- route path or method;
- request schema or signing bytes;
- publish-ingestion `contractVersion`;
- semantic status or classified code;
- HTTP status mapping;
- retrieval response shape;
- public versus administrator listener placement;
- authentication or role requirements;
- capability identifiers used by clients.

Documentation-only clarification must not redefine the selected release candidate or claim that formal qualification has occurred.

## 16. Operator verification

For a controlled environment:

1. verify storage with the operator runbook;
2. start the public relay listener;
3. request `/v1/ready` and `/v1/capabilities`;
4. publish a signed fixture and retain the complete HTTP status and body;
5. publish the same fixture again and confirm explicit duplicate semantics;
6. retrieve the returned canonical ID;
7. exercise one rejected fixture and preserve its classified code;
8. if defer policy is enabled, exercise one deferred fixture and verify it is absent from canonical storage and present in quarantine;
9. confirm an administrator path on the public listener returns `404`;
10. confirm the same administrator route on the administrator listener enforces `401`, `403`, and authorized behavior as applicable;
11. record the tested commit, configuration, listener addresses, and evidence classification.

A disposable local test is not privileged reference-host qualification and is not formal soak evidence.

## Related documents

- [Acceptance Policy](./ACCEPTANCE_POLICY.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [`http-publish-request` schema](../../schemas/http-publish-request.schema.json)
