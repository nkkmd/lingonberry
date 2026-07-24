# Migration and Schema Versioning

**Status: v1.0 pre-release normative**

This document defines the implemented version and migration boundaries of the Lingonberry v1.0 reference implementation. English is normative.

## 1. Scope

Lingonberry has several independent versioned contracts. They must not be treated as interchangeable:

- protocol version;
- knowledge-object schema version;
- HTTP publish-request schema version;
- publish-ingestion result contract version;
- capability manifest version;
- archive layout version;
- storage schema and migration state.

A version match does not by itself authorize publication, prove storage health, or make two nodes operationally equivalent. Protocol validation, local acceptance policy, storage classification, and operator qualification remain separate checks.

## 2. Implemented v1.0 pre-release baseline

| Contract | Implemented value | Authority |
|---|---:|---|
| protocol | `0.1.0` | `PROTOCOL_VERSION` |
| knowledge object | `0.1.0` | `KNOWLEDGE_OBJECT_SCHEMA_VERSION` and the checked-in schema |
| HTTP publish request | `0.1.0` | `HTTP_PUBLISH_REQUEST_SCHEMA_VERSION` and the checked-in schema |
| publish-ingestion result | `1` | `PUBLISH_INGESTION_CONTRACT_VERSION` |
| capability manifest | `1` | `CAPABILITY_VERSION` |
| archive layout | `1` | `ARCHIVE_VERSION` |

The identity-claim substructure has its own checked-in validation rules. Its version must not be substituted for the knowledge-object or HTTP request version.

The repository does not implement runtime selection among multiple protocol or schema versions. The generated capability manifest currently advertises one preferred version for each implemented schema family.

## 3. Version semantics

### 3.1 Protocol version

The protocol version identifies the top-level semantic contract shared by validation, finalization, storage, archive processing, and capability publication.

Changing it may affect:

- accepted wire semantics;
- canonicalization or identifier derivation;
- provenance and `rawRef` preservation;
- replay results;
- cross-node compatibility.

A protocol-version change requires an explicit compatibility decision. The v1.0 implementation does not automatically downgrade or translate between protocol versions.

### 3.2 Schema versions

The knowledge-object and HTTP publish-request schemas are separate contracts, even though both currently use `0.1.0`.

The knowledge object carries `schemaVersion` in the payload. The HTTP request envelope is validated against the checked-in request schema and contains the knowledge object plus publisher material.

A schema change must be evaluated against actual validator and finalizer behavior. A filename or JSON Schema `$id` change alone does not create runtime compatibility.

### 3.3 Publish-ingestion result version

Publication returns the versioned ingestion-result contract. Contract version `1` includes machine-readable outcomes such as:

- `stored`;
- `duplicate`;
- `deferred`;
- `rejected`;
- `conflict`;
- `failed`.

This response contract is independent from the request schema version. Clients must not infer request compatibility solely from the response contract version.

### 3.4 Capability version

Capability version `1` describes the shape of generated capability manifests. The manifest publishes the implemented protocol version, schema versions, carrier kinds, object types, content types, auth modes, validation/finalization constraints, and policy hints.

Capability publication is discovery, not negotiation. The v1.0 implementation does not provide:

- automatic mutual-version selection;
- schema conversion;
- dynamic fallback;
- protocol downgrade;
- signed remote negotiation;
- a migration-path registry.

### 3.5 Archive version

Archive version `1` identifies the implemented archive bundle layout. Export writes:

- `manifest.json`;
- `wire-log.jsonl`;
- `canonical-catalog.jsonl`.

At manifest validation time, archive import enforces the exact implemented:

- `archiveVersion`;
- `protocolVersion`;
- `carrierKind` of `archive`.

The current manifest-stage validator does not enforce every descriptive manifest field. In particular, documentation must not claim that `capabilityVersion`, schema-version metadata, policy metadata, paths, creation time, or item count are all independently enforced at that stage.

After manifest validation, imported records still pass through parsing, protocol validation, local acceptance policy, finalization, and storage classification. A valid manifest therefore does not guarantee that every record will be stored.

## 4. Implemented migration boundaries

### 4.1 Wire and request migration

The v1.0 reference implementation validates the versions it implements. It does not contain a generic migration engine that accepts arbitrary older payloads and upgrades them to the current schema.

The safe processing order is:

1. parse the request or archive record;
2. validate the implemented request and object contracts;
3. verify publisher identity and signature rules where applicable;
4. evaluate the local acceptance policy;
5. finalize the object;
6. append through the configured storage backend;
7. classify stored, duplicate, conflict, deferred, rejected, or failed outcomes.

Documentation and clients must not assume a hidden `validate -> migrate -> normalize` compatibility layer when no such path is checked in.

### 4.2 Storage migration

Storage migration is an operator-controlled backend concern. It is distinct from protocol/schema migration.

The checked-in SQLite runtime and storage tooling define their own initialization, verification, backup, restore, migration, and rollback procedures. Operators must follow the storage migration and upgrade runbook rather than editing database metadata or tables manually.

Storage migration does not authorize changes to canonicalization, identifiers, provenance, `rawRef`, duplicate classification, or conflict behavior. If a storage change would alter those semantics, it is also a protocol compatibility change and requires separate review.

The public readiness endpoint is not evidence that backup, restore, migration, rollback, disk-pressure, or crash-recovery qualification has been completed.

### 4.3 Archive migration

The reference implementation imports only the exact archive and protocol versions accepted by the manifest validator. It does not automatically rewrite an unsupported archive version into version `1`.

An archive produced by an incompatible implementation must be converted by a separately specified and tested tool before import. No generic archive converter is part of the v1.0 contract.

Conversion must preserve or explicitly account for:

- original wire records;
- canonical identifiers and identity keys;
- provenance;
- `rawRef`;
- record ordering needed for deterministic replay;
- duplicate and conflict semantics;
- evidence of the source and destination versions.

## 5. Compatibility policy

### 5.1 Exact-version behavior

The v1.0 reference implementation is exact-version oriented. Producers and consumers should verify the advertised and checked-in versions before exchanging data.

A consumer may perform a local compatibility check against a capability manifest. That check does not cause the remote node to negotiate, migrate, or switch carriers.

### 5.2 Additive changes

A proposed additive schema change is compatible only when all of the following remain true:

- existing valid payloads remain valid under the actual validator;
- canonicalization and identifier derivation are unchanged for existing payloads;
- provenance and `rawRef` semantics are unchanged;
- replay produces the same canonical state;
- existing clients can safely ignore the addition;
- capability and conformance fixtures are updated consistently.

The label `minor` is not sufficient evidence by itself.

### 5.3 Breaking changes

A change is breaking when it changes any required field, accepted value, signing bytes, canonicalization rule, identifier derivation, provenance rule, replay result, archive boundary, or machine-readable response relied on by existing consumers.

A breaking change requires:

1. a new explicitly versioned contract;
2. implementation and conformance fixtures for each supported version;
3. a defined coexistence or cutover policy;
4. archive and storage impact analysis;
5. migration or rejection behavior that is explicit and tested;
6. updated capability publication;
7. operator upgrade and rollback instructions.

## 6. Deprecation and removal

The current implementation does not advertise deprecated alternate schema versions. Future deprecation support must not be documented as implemented until the validator, capability manifest, fixtures, and tests support it.

Before removing a supported version, maintainers must establish evidence that:

- no required publisher or importer still depends on it;
- replay and archive recovery remain possible;
- a conversion path exists where preservation is required;
- the capability manifest no longer advertises it;
- unsupported input fails closed with a stable machine-readable result;
- upgrade and rollback procedures have been exercised.

## 7. Required change procedure

For any protocol, schema, response, capability, archive, or storage-version change:

1. identify the exact contract being changed;
2. compare validator, finalizer, signing, storage, archive, and response behavior;
3. determine whether canonical IDs, identity keys, provenance, `rawRef`, replay, duplicates, or conflicts change;
4. update constants, schemas, fixtures, tests, and conformance material together;
5. update capability publication only for behavior that is implemented;
6. define unsupported-version behavior explicitly;
7. add migration tooling only in a separate implementation change with its own tests;
8. update operator backup, upgrade, rollback, and evidence procedures;
9. run the frozen candidate walkthrough and applicable qualification suites;
10. preserve the fixed release-candidate boundary unless a separate release decision replaces it.

## 8. Non-guarantees

The v1.0 pre-release implementation does not guarantee:

- transparent migration from arbitrary historical schemas;
- simultaneous acceptance of multiple knowledge-object or request versions;
- automatic archive conversion;
- automatic database downgrade;
- online zero-downtime schema migration;
- protocol or schema fallback;
- cross-node migration orchestration;
- semantic translation between incompatible versions;
- compatibility merely because version strings resemble semantic versioning.

## 9. Release boundary

Documentation and tooling commits after the fixed candidate do not redefine the candidate.

- fixed candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`;
- latest public release: `v0.9.0`;
- `v1.0.0`: unreleased;
- formal 72-hour soak: not performed;
- privileged reference-host qualification and rehearsal: incomplete;
- version update, release PR, tag, and GitHub Release: incomplete.

Routine CI, documentation walkthroughs, archive tests, and non-privileged rehearsals must not be reported as the formal soak or privileged reference-host qualification.

## 10. Related documents

- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Discovery and Compatibility](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Technical Decision ADR](./TECH_DECISION_ADR.md)
- [Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
