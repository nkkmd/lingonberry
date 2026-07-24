# Technical Decision ADR

**Status: v1.0 pre-release normative**

This ADR records the implemented technical boundaries of the Lingonberry v1.0 reference implementation. English is normative.

## 1. Decision

The v1.0 reference implementation uses Rust for the protocol, identity, validation, core storage behavior, indexing, relay/runtime, and storage tooling.

The checked-in Cargo workspace contains:

- `packages/protocol`;
- `packages/identity`;
- `packages/validation`;
- `packages/core`;
- `packages/indexer`;
- `packages/relay`;
- `packages/storage`.

The implementation does not contain separate `packages/api`, `packages/cli`, or `packages/codecs` workspace members. Public HTTP and command-line behavior are implemented through the relay/runtime package, while storage-specific commands are implemented through the storage package.

The primary interactive carrier is HTTP. Local relay/runtime commands and file/archive export and import are also implemented carrier surfaces. This ADR does not claim that a network pub/sub protocol, federated synchronization, runtime carrier negotiation, or remote handshake exists.

## 2. Storage decision

The default runtime storage backend is SQLite under the configured Lingonberry state directory.

The runtime preserves distinct representations for:

- the original publish-request material required for replay and carrier identity;
- finalized canonical knowledge objects;
- derived index state;
- quarantine and resolution records where applicable.

The implementation also retains file-backed helpers and emits JSONL archive material, but the production runtime selected by the reference relay is not accurately described as a simple filesystem raw log plus an unrelated SQLite catalog.

File/archive export writes:

- `manifest.json`;
- `wire-log.jsonl`;
- `canonical-catalog.jsonl`.

Archive import validates its implemented manifest boundary, validates imported objects, applies the local acceptance policy, finalizes objects, and appends through the configured storage backend.

## 3. Protocol and validation decision

Protocol-native JSON is the shared semantic representation across implemented carriers.

The current implementation fixes these identifiers:

| Identifier | Value |
|---|---|
| protocol version | `0.1.0` |
| knowledge-object schema version | `0.1.0` |
| HTTP publish-request schema version | `0.1.0` |
| capability version | `1` |
| archive version | `1` |

Validation, acceptance, finalization, and storage classification are separate stages:

1. parse the request or object;
2. validate structural, schema, signature, identity, and semantic invariants;
3. evaluate the configured acceptance policy;
4. accept, reject, or defer;
5. finalize the knowledge object;
6. classify storage as new, duplicate, conflict, or operational failure.

A successful compatibility check does not guarantee acceptance. Acceptance does not guarantee that storage will not detect a conflict or operational failure.

## 4. Carrier decision

### 4.1 HTTP

HTTP is the primary interactive ingress and retrieval carrier.

The public listener provides the checked-in versioned routes, including the core surfaces:

- `POST /v1/objects`;
- `GET /v1/objects/<canonical-id>`;
- `GET /v1/capabilities`;
- `GET /v1/ready`.

The HTTP server contains a bounded HTTP/1.1 parser suitable for the checked-in contract tests. It is not a production reverse proxy, TLS terminator, streaming server, or general-purpose web framework.

Internet-facing operation requires an appropriate reverse proxy and the operator controls documented by the deployment runbooks.

### 4.2 Relay/runtime CLI

The relay/runtime binary provides local commands for validation, publication, retrieval, listing, subscription-like filtering over stored records, replay, capability output, archive operations, index operations, and quarantine workflows.

The local `subscribe` and `replay` commands do not establish a network subscription protocol. They do not provide remote delivery acknowledgements, ordering negotiation, resumable sessions, or federated synchronization.

### 4.3 File/archive

File/archive export and import are implemented operational carriers. They are not merely deferred design options.

Archive portability does not imply automatic remote synchronization, semantic translation, dynamic downgrade, or compatibility negotiation.

## 5. Capability decision

The implementation generates capability manifests for the relay/runtime CLI and public HTTP endpoint. Archive export writes a separate archive-specific manifest.

Capability discovery is descriptive. The implementation does not provide:

- runtime client/server negotiation;
- automatic mutually supported version selection;
- dynamic carrier fallback;
- protocol or schema downgrade;
- remote signed handshakes;
- semantic translation between incompatible contracts.

Consumers are responsible for compatibility checks for the fields on which they depend. Protocol validation and local acceptance policy remain independent of capability discovery.

## 6. Identity and provenance decision

Publisher signatures, protocol identity claims, provenance, administrator authentication, and authorization are distinct mechanisms.

The HTTP publish-request validator requires the checked-in lowercase hexadecimal public-key and signature encodings. The v1.0 implementation does not promise ingress conversion from `npub` or other external encodings.

A valid publisher signature does not establish the truth of object content and does not bypass schema validation, identity validation, acceptance policy, quarantine, conflict detection, or storage failures.

Administrator credentials apply only to the separately authenticated administrative listener and do not replace publisher signatures.

## 7. Access and retention decision

The implemented defaults are:

- access scope: `public`;
- retention hint: `long-lived`.

Access scopes and retention hints are policy metadata. They do not themselves implement confidentiality, private-route authorization, automatic expiry, deletion, or archival enforcement.

The archive manifest reports `privateEnabled: false` and operator-controlled scrubbing. Operators must use the access, retention, backup, and archive runbooks rather than infer enforcement from manifest vocabulary.

## 8. Indexing decision

Indexing is implemented as derived state. It does not replace the canonical stored object or original request material.

The checked-in `packages/indexer` package builds queryable snapshots and graph views from stored canonical records. Index rebuilds must remain reproducible from authoritative storage.

The v1.0 reference implementation does not guarantee PostgreSQL, an external search service, distributed index replication, or a separately deployed indexer service.

## 9. Deployment and process boundaries

The reference implementation supports local process execution and documented systemd-oriented operation. Public HTTP, administrative HTTP, and storage verification have separate operational responsibilities.

Readiness of the public HTTP listener does not prove:

- storage verification;
- backup validity;
- migration completion;
- quarantine consistency;
- privileged disk-pressure qualification;
- reference-host qualification;
- formal soak completion.

Those claims require their own runbooks and evidence.

## 10. Alternatives not selected for v1.0

The v1.0 reference implementation does not use the following as its core runtime:

- a TypeScript or Node.js relay/storage implementation;
- a Go-based protocol core;
- PostgreSQL as the default storage backend;
- a Toitoi-specific transport as the Lingonberry protocol core;
- an unversioned semantic translation gateway;
- a network pub/sub carrier inferred solely from the word `relay`.

These may be evaluated in future versions only through explicit compatibility, migration, and operational review.

## 11. Consequences

The selected architecture provides:

- one Rust workspace for the implemented protocol and runtime components;
- deterministic protocol and validation behavior;
- a shared ingestion pipeline across HTTP, local runtime, and archive import;
- explicit separation of compatibility, validation, acceptance, and storage outcomes;
- reproducible archive and index workflows;
- a small deployable reference surface.

The costs include:

- Rust implementation and review complexity;
- responsibility for explicit reverse-proxy and system-service configuration;
- limited dynamic interoperability in v1.0;
- no automatic downgrade, fallback, or federated synchronization;
- required operator discipline for backup, migration, quarantine, and evidence handling.

## 12. Change control

A compatibility or architecture review is required before changing:

- workspace package boundaries that expose public behavior;
- protocol, schema, capability, or archive identifiers;
- canonicalization or identity rules;
- signature encoding or signing bytes;
- public versus administrative listener placement;
- storage authority or replay material;
- archive manifest enforcement;
- acceptance-policy responsibility;
- duplicate and conflict semantics;
- default storage backend;
- release qualification requirements.

Documentation-only clarification must not redefine the selected release candidate or claim qualification that has not occurred.

## 13. Release boundary

For the current v1.0.0 release process:

- v1.0.0 is not released;
- the fixed candidate is `f9543019f2c219aea3b085ff90f2da201b268a48`;
- documentation and tooling commits do not redefine that candidate;
- formal 72-hour soak has not been performed;
- privileged reference-host qualification and rehearsal are incomplete;
- version update has not been performed;
- the release tag has not been created;
- the GitHub Release has not been created.

Normal CI, documentation walkthroughs, local rehearsals, and non-privileged tests are not substitutes for formal soak or privileged reference-host qualification.

## Related documents

- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [Carrier Capability Discovery and Compatibility](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Acceptance Policy](./ACCEPTANCE_POLICY.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
