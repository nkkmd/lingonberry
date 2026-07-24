# Carrier Decision Memo

**Status: v1.0 pre-release normative**

This memo records the carrier decisions implemented by the Lingonberry v1.0 reference implementation. English is normative.

## 1. Decision

The reference implementation uses protocol-native JSON across three implemented carrier surfaces:

1. an HTTP request/response carrier for interactive publication, retrieval, capability discovery, and readiness;
2. a local relay/runtime command surface backed by the same protocol, validation, acceptance, and storage layers;
3. a file/archive export and import carrier for portable replay material.

HTTP remains the primary interactive ingress. File/archive export and import are implemented operational carriers rather than merely future options. The relay/runtime exposes local subscription and replay commands, but v1.0 does not implement a network pub/sub protocol, a remote subscription handshake, delivery acknowledgements, or federated carrier synchronization.

The carrier choice does not change protocol semantics. Carriers transport protocol-native objects and route them into shared validation, acceptance, finalization, storage, and indexing behavior.

## 2. Implemented carrier surfaces

| Carrier surface | Implemented entry points | v1.0 role |
|---|---|---|
| HTTP | `POST /v1/objects`, `GET /v1/objects/<canonical-id>`, `GET /v1/capabilities`, `GET /v1/ready` and checked-in versioned read surfaces | primary interactive publication and retrieval carrier |
| relay/runtime CLI | `publish`, `get`, `raw`, `list`, `subscribe`, `replay`, `capabilities`, archive commands, and operator commands | local process and operator-facing access to the shared runtime |
| file/archive | `export-archive`, `import-archive`, `manifest.json`, `wire-log.jsonl`, `canonical-catalog.jsonl` | portable export/import and replay evidence |

Administrative quarantine HTTP operations use a separately authenticated listener. They are not part of the public HTTP carrier merely because they use HTTP framing.

## 3. Why HTTP is the primary interactive ingress

HTTP is retained as the primary interactive ingress because it provides a small, testable publication and retrieval loop while preserving protocol-native semantics.

The implemented flow is:

1. accept a bounded HTTP request;
2. parse the versioned publish-request envelope;
3. validate the envelope and contained knowledge object;
4. verify publisher signature and identity rules;
5. evaluate the local acceptance policy;
6. defer eligible requests to quarantine or reject failures;
7. finalize the knowledge object;
8. append through the configured storage backend;
9. classify stored, duplicate, conflict, or operational failure;
10. return the versioned ingestion result.

HTTP does not provide a semantic adapter, authoring profile, alternative canonicalization rule, or authorization bypass.

## 4. Protocol-native request boundary

The HTTP publish request contains:

```json
{
  "object": {
    "...": "protocol-native knowledge object"
  },
  "publisher": {
    "publicKey": "64-character lowercase hexadecimal public key",
    "signature": "128-character lowercase hexadecimal signature"
  }
}
```

The exact schema and signing bytes are defined by the checked-in schema and [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md).

The implemented request boundary does not accept `npub` as an alternative canonical publisher-key encoding. The validator requires the checked-in lowercase hexadecimal representation. Documentation must not describe unimplemented ingress conversion as a v1.0 feature.

The publisher envelope is not copied into the knowledge object as an author field. Signature verification, object provenance, identity claims, and acceptance policy remain distinct concerns.

## 5. HTTP response decision

The draft-era generic response envelopes are not the v1.0 contract. In particular, clients must not require:

```json
{
  "status": "ok",
  "data": {}
}
```

or assume that publication returns a wrapper containing only `id`, `canonical`, and `rawRef`.

Publication uses the versioned ingestion-result contract. Its stable semantic outcomes are:

- `stored`;
- `duplicate`;
- `deferred`;
- `rejected`;
- `conflict`;
- `failed`.

Clients must inspect both the HTTP status and the response body. Duplicate publication is idempotent success, deferred publication is not canonical storage, conflict is distinct from validation rejection, and operational failure is not acceptance-policy rejection.

Retrieval returns the implemented stored-object representation. It does not use the obsolete illustrative `status: ok` / `canonical` / `rawRef` wrapper unless those fields are present in the checked-in route contract.

## 6. HTTP parsing and deployment boundary

The reference relay contains a bounded HTTP/1.1 parser sufficient for the checked-in contract tests. It is not a production reverse proxy.

An Internet-facing deployment requires an appropriate reverse proxy for TLS termination and external publication policy. The carrier decision does not imply built-in guarantees for:

- TLS termination;
- HTTP/2 or HTTP/3;
- compression;
- streaming publication;
- universal CORS behavior;
- proxy trust configuration;
- production denial-of-service protection.

The reverse proxy must preserve the request method, path, body, and signature material required by the carrier contract.

## 7. Capability discovery boundary

The public HTTP carrier implements `GET /v1/capabilities`. The relay/runtime also implements the `capabilities` command.

These surfaces publish generated capability manifests. They support discovery; they do not perform runtime negotiation.

The v1.0 implementation does not provide:

- a client/server negotiation request;
- automatic selection of a mutually supported version;
- dynamic fallback to another carrier;
- protocol or schema downgrade;
- a signed remote capability handshake;
- semantic translation between incompatible contracts.

Consumer compatibility checks, protocol validation, acceptance policy, and storage classification are separate responsibilities. See [Carrier Capability Discovery and Compatibility](./CARRIER_CAPABILITY_NEGOTIATION.md).

## 8. File/archive carrier decision

File/archive export and import are implemented.

Archive export writes:

- `manifest.json`;
- `wire-log.jsonl`;
- `canonical-catalog.jsonl`.

The archive manifest identifies the archive version, capability version, protocol version, archive carrier kind, schema versions, policy metadata, paths, creation time, and item count.

Archive import currently enforces the manifest's:

- `archiveVersion`;
- `protocolVersion`;
- `carrierKind`.

It then parses each wire-log record, validates the contained object, applies the local acceptance policy, finalizes the object, and appends it through the configured storage backend.

The archive carrier is not an authorization boundary, encrypted backup format, automatic scrubber, or remote synchronization protocol. `privateEnabled: false` and `scrubMode: operator-controlled` must be interpreted as archive policy metadata, not as implemented confidentiality or automatic redaction.

## 9. Relay and subscription boundary

The runtime implements local `subscribe` and `replay` commands over stored records. These are process-level query and replay surfaces.

They do not establish a v1.0 network pub/sub protocol. The following remain outside the implemented guarantee:

- remote subscriber registration;
- long-lived push delivery;
- delivery acknowledgement;
- redelivery scheduling;
- ordering across nodes;
- remote backpressure negotiation;
- network handshake semantics;
- federated relay synchronization.

Documentation must not infer those features from the words `relay`, `subscribe`, `replay`, or from descriptive entries in the capability manifest.

## 10. Shared semantic boundary

All implemented carriers must preserve the shared protocol boundary:

- protocol-native knowledge objects remain protocol-native;
- carriers do not invent domain-specific semantic translations;
- validation and finalization use the checked-in protocol and validation layers;
- canonical identity is not derived from an HTTP status or archive filename;
- provenance and raw references remain object-level protocol data;
- duplicate and conflict classification belongs to ingestion and storage;
- acceptance policy remains local runtime policy;
- application profiles remain outside the core carrier contract.

A Toitoi or other application profile may use HTTP as its connection surface, but domain-specific routing, presentation, and curation remain profile concerns.

## 11. Access and retention boundary

The generated capability manifest reports:

- default access scope `public`;
- default retention hint `long-lived`;
- additional supported policy vocabulary.

The archive manifest reports corresponding archive policy metadata.

These values do not implement confidentiality, private authorization, automatic expiration, deletion authorization, or retention enforcement. Those operational meanings are governed by [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md) and the actual storage and administrator contracts.

## 12. Authentication and authorization boundary

Publisher signature verification is implemented for the HTTP publish-request contract. It establishes that the request carries a valid signature under the checked-in signing contract; it is not administrator authentication and does not prove the truth of the published content.

Administrator authentication and role authorization are separate and apply only to the authenticated administrator listener and implemented administrative operations.

A valid publisher signature does not bypass:

- schema validation;
- identity validation;
- acceptance policy;
- duplicate/conflict classification;
- quarantine rules;
- storage failures.

## 13. Rejected alternatives for v1.0

The v1.0 reference implementation does not introduce:

- a custom binary carrier;
- a Toitoi-specific transport contract;
- a gateway carrier that depends on semantic translation;
- a remote federated-sync protocol;
- an automatic carrier-selection protocol.

These would add compatibility and operational boundaries that are not qualified by the fixed candidate.

## 14. Compatibility review triggers

A carrier change requires explicit compatibility review when it alters:

- route paths or methods;
- request schema or signing bytes;
- publish-ingestion result contract;
- semantic statuses or classified codes;
- HTTP status mapping;
- retrieval response shape;
- capability identifiers;
- archive layout or enforced manifest fields;
- public versus administrator listener placement;
- authentication or role requirements;
- storage append, duplicate, conflict, or replay behavior.

A documentation clarification must not redefine the selected release candidate or claim that formal qualification has occurred.

## 15. Release boundary

This memo describes the implemented carrier decision. It does not establish that v1.0.0 has been published.

The fixed v1.0.0 candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Later documentation or tooling commits do not redefine that candidate.

Normal CI, documentation walkthroughs, local archive tests, or non-privileged rehearsals are not the formal 72-hour soak and do not establish privileged reference-host qualification.

## Related documents

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Discovery and Compatibility](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Acceptance Policy](./ACCEPTANCE_POLICY.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
