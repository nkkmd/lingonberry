# Operational Premises Memo

**Status: v1.0 pre-release normative**  
**Last reviewed: 2026-07-24**

This memo records the operating premises that constrain the Lingonberry v1.0 reference deployment. English is normative.

## 1. Scope

These premises define responsibility boundaries for the protocol, relay, storage runtime, deployment, monitoring, and qualification. They do not replace the protocol schemas, carrier contracts, storage runbooks, or release checklist.

The fixed v1.0.0 pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Later documentation and tooling commits do not redefine that candidate.

## 2. Core premises

- Knowledge objects are append-only once accepted into canonical storage.
- Wire and canonical forms are representations of the same protocol object, not separate protocols.
- A carrier transports protocol-native objects and must not redefine canonical identity, provenance, acceptance policy, or storage semantics.
- Relay ingress and storage persistence are separate responsibilities.
- The core protocol remains domain-neutral. Domain vocabulary, curation, trust, and presentation rules belong to profiles or local policy.
- Secrets belong to deployment-managed injection paths and must not be committed to repository files, embedded in images, exposed in process arguments, or copied into public evidence.
- Documentation clarification must not claim qualification, soak completion, release publication, or compatibility guarantees that have not been demonstrated.

## 3. Relay boundary

The public relay is the ingress and retrieval surface. Its implemented responsibilities include:

- bounded HTTP request handling;
- route and request-envelope parsing;
- schema, identity, and signature validation;
- configured acceptance-policy evaluation;
- quarantine deferral where supported;
- finalization and storage-backend invocation;
- retrieval, capability discovery, and listener readiness;
- serialization of versioned route-specific results.

The relay does not own:

- the internal SQLite layout;
- backup or restore policy;
- migration execution;
- canonical truth about the real world;
- domain-specific curation;
- administrator authorization through the public listener;
- production TLS termination or Internet-edge denial-of-service protection.

A successful relay readiness response proves only that the listener accepted and routed the request. It is not deep storage verification.

## 4. Storage boundary

The reference storage runtime owns persistent canonical state and storage-specific operations, including:

- append and duplicate/conflict classification;
- retrieval and listing where implemented;
- replay and verification;
- backup and restore;
- archive export and import;
- explicit migration tooling;
- resolved storage configuration and diagnostics.

`lingonberry-storage ready` is a finite readiness gate, not a resident storage daemon. `lingonberry-storage run` prints the resolved runtime snapshot and exits.

Ordinary relay startup, storage readiness, or process restart must not perform an implicit migration. Migration, restore, and destructive replacement work require the relay to be stopped from writing the same active storage.

## 5. Reference deployment premises

The v1 reference host contract is:

- Ubuntu Server 24.04 LTS on x86_64;
- systemd-managed `lingonberry-storage-ready.service` and `lingonberry-relay.service`;
- an unprivileged `lingonberry` service account;
- SQLite as the default active storage backend;
- persistent local storage on a supported filesystem;
- separate active-data, backup, and temporary roots;
- a private relay bind address when a reverse proxy provides public TLS;
- Caddy, nginx, or another externally maintained reverse proxy as a deployment layer, not a protocol component.

Container execution is optional. It is supportable only when it preserves the same binaries, configuration precedence, persistent paths, readiness ordering, secret boundaries, and evidence requirements. The repository does not currently establish a normative image registry, Dockerfile, Compose stack, Kubernetes manifest, or container orchestrator contract.

PostgreSQL, external indexers, clustered storage, zero-downtime migration, automatic failover, and cross-node orchestration are not v1 reference guarantees.

## 6. Public, administrative, and private-data premises

The core v1 object path is designed for publicly shareable knowledge objects. `accessScope=public` and `retentionHint=long-lived` are metadata and policy inputs; they do not implement confidentiality, deletion authorization, or automatic expiry.

Private or encrypted application data requires a separately reviewed profile, policy, and deployment design. It must not be inferred from the public carrier contract.

The public HTTP listener and authenticated administrator listener are separate process surfaces. Public publisher signatures are not administrator credentials. Administrator authorization does not bypass object validation, acceptance policy, conflict detection, or quarantine state rules.

## 7. Core and profile boundary

Core responsibilities include:

- protocol object structure;
- canonical identity and canonicalization;
- provenance and signature boundaries;
- protocol-native carrier framing;
- validation, normalization, and finalization;
- append-only persistence semantics;
- replay compatibility within the implemented version contracts.

Profile or local-policy responsibilities include:

- domain-specific object subtypes;
- relation and context vocabularies;
- curation and trust rules;
- query and display priority;
- local acceptance allowlists or denylists;
- retention operations beyond the protocol metadata;
- confidentiality and access-control designs not implemented by the public v1 carrier.

Profile vocabulary must not silently change core identity, signing bytes, schema requirements, or storage classification.

## 8. Monitoring premises

Operational monitoring should distinguish at least these layers:

- process and service state;
- storage configuration and diagnostics;
- relay listener readiness;
- publish outcomes: `stored`, `duplicate`, `deferred`, `rejected`, `conflict`, and `failed`;
- quarantine persistence and administrator operations where enabled;
- backup, restore, migration, disk-pressure, and incident evidence under their governing runbooks.

The following are outside generic operational-health claims:

- truth of object content;
- domain-specific validity;
- profile-specific trust decisions;
- UI ordering or presentation quality;
- complete observability of a wider federation;
- formal qualification or soak status not supported by the recorded evidence.

A green hosted CI run is not proof of reference-host readiness. A successful local walkthrough is not privileged reference-host qualification and is not the formal 72-hour soak.

## 9. Change-control consequences

A change requires focused compatibility or operational review when it alters:

- protocol or schema version contracts;
- signing bytes or identity derivation;
- public or administrator routes;
- acceptance or quarantine semantics;
- publish-result statuses or HTTP mappings;
- storage schema or migration behavior;
- filesystem, account, service, or network boundaries;
- backup, restore, or rollback procedures;
- the fixed release candidate or qualification evidence.

Runtime fixes, protocol changes, and documentation inventory updates must remain in their appropriate separate pull requests.

## 10. Release boundary

The following remain incomplete unless separately recorded with evidence:

- formal 72-hour soak;
- privileged reference-host qualification and rehearsal;
- final version update;
- release pull request;
- v1.0.0 tag;
- GitHub Release publication.

Ordinary CI, documentation walkthroughs, archive tests, and nonprivileged rehearsals do not satisfy those release gates.

## Related documents

- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Relay and Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Technology Decision ADR](./TECH_DECISION_ADR.md)
