# Carrier Capability Discovery and Compatibility

**Status: v1.0 pre-release normative**

This document defines the implemented carrier capability discovery surface and the compatibility boundaries that consumers must apply for the Lingonberry v1.0 reference implementation. English is normative.

## 1. Scope

The v1 reference implementation publishes a capability manifest for discovery. The manifest describes implemented identifiers, supported schema versions, supported object types, carrier vocabulary, policy defaults, and selected helper surfaces.

The implementation does not perform runtime capability negotiation, automatic fallback, dynamic downgrade, remote handshakes, or semantic translation between incompatible versions. Consumers must treat discovery, compatibility evaluation, request validation, and local acceptance policy as separate responsibilities.

## 2. Implemented discovery surfaces

The implemented discovery surfaces are:

- the relay CLI command `lingonberry capabilities`;
- `GET /v1/capabilities` on the public HTTP listener;
- the archive export `manifest.json`, which carries archive-specific compatibility metadata.

The CLI capability command builds a relay manifest. The HTTP endpoint builds an HTTP manifest. Archive export writes a different archive manifest and archive import validates only the archive fields described in this document.

The v1 implementation does not implement:

- a registry-backed discovery protocol;
- a signed remote capability exchange;
- a client/server negotiation request;
- selection of a mutually supported version at runtime;
- retry against another carrier after incompatibility;
- automatic downgrade to an older protocol or schema;
- a remote handshake that establishes a session contract.

Manifest fields named under discovery helpers are descriptive vocabulary. Their presence is not proof that the corresponding remote protocol is implemented.

## 3. Capability manifest shape

The relay and HTTP discovery surfaces return a generated JSON object with this implemented shape:

```json
{
  "capabilityVersion": "1",
  "protocolVersion": "0.1.0",
  "carrierKind": "http",
  "supportedCarrierKinds": ["http", "archive", "relay"],
  "supportedSchemaVersions": [
    {
      "schema": "knowledge-object",
      "versions": ["0.1.0"],
      "preferred": "0.1.0",
      "breaking": false
    },
    {
      "schema": "http-publish-request",
      "versions": ["0.1.0"],
      "preferred": "0.1.0",
      "breaking": false
    }
  ],
  "supportedObjectTypes": [
    "inquiry",
    "observation",
    "claim",
    "evidence",
    "annotation",
    "synthesis",
    "translation",
    "reference",
    "concept"
  ],
  "supportedContentTypes": ["application/json"],
  "supportedAuthModes": [
    "public-key-signature",
    "relay-trusted-signature"
  ],
  "validationConstraints": [
    "required-fields",
    "schema-version-match",
    "identity-consistency"
  ],
  "finalizeConstraints": [
    "canonical-id-resolution",
    "rawref-preservation",
    "provenance-preservation"
  ],
  "supportedAccessScopes": ["public", "curated", "private"],
  "supportedRetentionHints": ["long-lived", "long-term", "ephemeral"],
  "multiNode": {},
  "defaults": {
    "accessScope": "public",
    "retentionHint": "long-lived"
  }
}
```

The `carrierKind` value depends on the surface:

| Surface | `carrierKind` |
|---|---|
| `lingonberry capabilities` | `relay` |
| `GET /v1/capabilities` | `http` |

`capabilityVersion` is the version of this discovery object. It is not the protocol version, a schema version, an archive version, or a negotiated session version.

## 4. Version identifiers

The checked-in v1 candidate implementation currently uses:

| Identifier | Implemented value | Meaning |
|---|---|---|
| capability version | `1` | generated capability-manifest format |
| protocol version | `0.1.0` | protocol compatibility identifier |
| knowledge object schema version | `0.1.0` | accepted `knowledge-object` schema version |
| HTTP publish request schema version | `0.1.0` | accepted `http-publish-request` schema version |
| archive version | `1` | archive layout and import contract identifier |

These values are independent compatibility boundaries. A consumer must not substitute one for another.

The capability manifest advertises only the single implemented knowledge-object schema version and the single implemented HTTP publish-request schema version. The `preferred` member repeats the implemented version. The `breaking` member is descriptive metadata; the runtime validators do not use it to negotiate compatibility.

## 5. Carrier kind and supported carrier kinds

`carrierKind` identifies the surface that produced the manifest. `supportedCarrierKinds` is a vocabulary list containing `http`, `archive`, and `relay`.

The list does not mean that one running endpoint can dynamically switch among those carriers, proxy them, or downgrade from one to another. Carrier selection is made by the caller or operator before invoking the corresponding surface.

The relay CLI manifest and the HTTP manifest share most fields because both use the same manifest builder. Archive export does not reuse that full object.

## 6. HTTP capability discovery

`GET /v1/capabilities` returns the generated capability manifest with:

- `carrierKind` set to `http`;
- default access scope set to `public`;
- default retention hint set to `long-lived`.

The endpoint is discovery only. It does not:

- authorize a publish request;
- prove readiness of the storage backend;
- select a schema version for the caller;
- create a negotiated session;
- guarantee that every advertised policy vocabulary value is operationally enabled;
- bypass request validation or acceptance policy.

Consumers must still submit the implemented HTTP publish-request shape and inspect the actual route result.

## 7. Archive manifest

Archive export writes `manifest.json` with an archive-specific shape. Its principal fields are:

```json
{
  "archiveVersion": "1",
  "capabilityVersion": "1",
  "protocolVersion": "0.1.0",
  "carrierKind": "archive",
  "createdAt": "...",
  "itemCount": 0,
  "schemaVersions": {
    "knowledgeObject": "0.1.0",
    "httpPublishRequest": "0.1.0"
  },
  "policy": {
    "defaultAccess": "public",
    "defaultRetention": "long-lived",
    "privateEnabled": false,
    "scrubMode": "operator-controlled"
  },
  "paths": {
    "manifest": "manifest.json",
    "wireLog": "wire-log.jsonl",
    "catalog": "canonical-catalog.jsonl"
  }
}
```

The archive manifest is not the HTTP capability manifest serialized to disk. Field names and enforcement behavior differ.

Archive import currently enforces only:

- `archiveVersion` exists and equals `1`;
- `protocolVersion` exists and equals `0.1.0`;
- `carrierKind` exists and equals `archive`.

Archive import does not currently reject an archive solely because `capabilityVersion`, `schemaVersions`, `policy`, `paths`, `createdAt`, or `itemCount` is absent or differs. After manifest validation, imported records are parsed, validated, evaluated under the local acceptance policy, finalized, and appended through the storage backend.

Consumers and operators must distinguish fields that are present in the archive manifest from fields that the archive importer enforces.

## 8. Manifest description versus enforced validation

The discovery manifest describes more than the consumer path automatically validates.

The protocol validators enforce the actual knowledge-object and publish-request contracts, including required fields, exact knowledge-object schema version, supported object type, publisher material, signature verification, identity rules, and additional structural constraints.

The archive importer separately enforces the three archive manifest fields listed above. It then applies local object validation and local acceptance policy to each imported record.

No generic capability-manifest validator currently rejects a remote HTTP manifest based on all advertised fields. A client that depends on a field must implement and test its own compatibility check before using the remote surface.

In particular, these advertised values are not equivalent to runtime authorization or policy enablement:

- `supportedAuthModes`;
- `supportedAccessScopes`;
- `supportedRetentionHints`;
- `validationConstraints`;
- `finalizeConstraints`;
- helper entries under `multiNode`.

## 9. Compatibility evaluation

Compatibility evaluation belongs to the consumer of a discovered manifest. A conservative consumer should:

1. require a recognized `capabilityVersion` before interpreting the object;
2. require the expected `carrierKind` for the selected surface;
3. compare `protocolVersion` with the protocol version it implements;
4. locate the required schema entries by schema name;
5. require an exact implemented schema version unless the consumer has an independently tested compatibility rule;
6. require every object type, content type, or authentication mode that its operation actually depends on;
7. reject missing or ambiguous required fields;
8. avoid inferring fallback, downgrade, or translation support from the manifest.

For the v1 reference implementation, exact equality is the safest rule for protocol and schema identifiers. The implementation does not provide a semver range evaluator or a negotiated downgrade path.

Compatibility evaluation answers whether a consumer can attempt the operation using a known contract. It does not answer whether a particular object will be accepted.

## 10. Acceptance policy boundary

Acceptance policy is local runtime policy applied after structural and identity validation. It may accept, reject, or defer an otherwise parseable request.

Capability discovery and compatibility evaluation must not be used to predict or replace acceptance policy. A compatible publish-request contract can still be:

- rejected for validation or identity errors;
- deferred to quarantine by local policy;
- rejected by a local policy profile;
- classified as a duplicate or conflict by storage;
- failed by an operational dependency.

Conversely, acceptance policy must not reinterpret an incompatible protocol or schema version as compatible.

The responsibility split is:

| Concern | Responsible layer |
|---|---|
| describe the local surface | capability manifest producer |
| decide whether a known client contract can be attempted | consumer compatibility check |
| validate request and object structure | protocol and validation layers |
| decide accept, reject, or defer | local acceptance policy |
| classify duplicate, conflict, or storage failure | storage and ingestion layers |

## 11. Defaults and policy vocabulary

The generated manifest reports:

- `defaults.accessScope = public`;
- `defaults.retentionHint = long-lived`.

The supported lists also contain `curated`, `private`, `long-term`, and `ephemeral`. These are policy vocabulary, not proof that confidentiality, private authorization, automatic expiry, deletion, or retention enforcement is enabled.

The archive manifest additionally reports `privateEnabled: false` and `scrubMode: operator-controlled`.

Consumers must use [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md) and the actual carrier contract to interpret these values.

## 12. Non-goals for v1.0

The v1.0 reference implementation does not guarantee:

- bidirectional runtime capability negotiation;
- automatic selection from version ranges;
- dynamic fallback to another carrier;
- dynamic protocol or schema downgrade;
- semantic translation between incompatible objects;
- signed remote handshakes;
- central-registry coordination;
- universal enforcement of every manifest field;
- authorization derived from capability discovery.

Adding any of these behaviors requires a separate implementation change, tests, compatibility review, and documentation update. It must not be introduced by documentation wording alone.

## 13. Operator verification

For a controlled checkout:

1. run `lingonberry capabilities` and record the relay manifest;
2. start the public HTTP listener;
3. request `GET /v1/capabilities` and confirm `carrierKind` is `http`;
4. compare both outputs with the constants and manifest builder in the checked-in implementation;
5. export an archive and inspect `manifest.json`;
6. verify archive import rejects mismatched `archiveVersion`, `protocolVersion`, and `carrierKind`;
7. verify imported records still pass local validation and acceptance policy;
8. confirm no test or runtime path claims automatic negotiation, fallback, downgrade, or remote handshake;
9. record the exact commit, commands, fixtures, and resulting evidence.

A local or CI verification run is not formal 72-hour soak evidence and is not privileged reference-host qualification.

## 14. Release boundary

This document describes the fixed v1.0 candidate implementation. Documentation or tooling commits after candidate selection do not redefine the candidate.

Until separate evidence exists, the following remain incomplete:

- formal 72-hour soak;
- privileged reference-host qualification and rehearsal;
- version update;
- release pull request;
- tag creation;
- GitHub Release publication.

## Related documents

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Acceptance Policy](./ACCEPTANCE_POLICY.md)
- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [`knowledge-object` schema](../../schemas/knowledge-object.schema.json)
- [`http-publish-request` schema](../../schemas/http-publish-request.schema.json)
