# Lingonberry Protocol Contract

**Status: normative for the v1.0.0 pre-release implementation**  
**Protocol version: `0.1.0`**  
**Knowledge Object schema version: `0.1.0`**

## 1. Purpose

This document defines the checked-in external contract for Lingonberry protocol objects, canonical bytes, validation, identifiers, signatures, transitions, and conformance.

Normative terms such as **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** apply only to behavior explicitly described as implemented or required by the referenced versioned rule. Future design goals are identified separately and are not current runtime guarantees.

## 2. Version and representation boundaries

The implementation separates:

1. carrier or request representation;
2. parsed JSON values;
3. schema-valid protocol objects;
4. canonical JSON bytes;
5. node-local storage records;
6. HTTP API request and response representations;
7. independently versioned identity, signature, transition, digest, and diagnostic rules.

The following checked-in version constants are distinct:

- protocol version: `0.1.0`;
- Knowledge Object schema version: `0.1.0`;
- HTTP publish-request schema version: `0.1.0`;
- archive format version: `1`;
- capability manifest version: `1`.

A storage, archive, journal, capability, API, identity, signature, transition, or diagnostic version MUST NOT be interpreted as the protocol version.

## 3. Parser limits and JSON model

The Rust protocol parser accepts one JSON value followed only by JSON whitespace.

Current limits are:

- maximum UTF-8 input length: 1,048,576 bytes;
- maximum nesting depth: 128.

The checked-in Rust JSON model stores number tokens as their original lexical strings. Canonicalization does not numerically normalize them.

Object members are stored in a `BTreeMap<String, JsonValue>`. Duplicate JSON member names are therefore collapsed with the last parsed value winning. Producers MUST NOT rely on duplicate-member behavior, and duplicate members are outside the interoperable protocol contract.

The parser does not currently accept escaped UTF-16 surrogate-pair composition as a portable interoperability feature. Producers SHOULD emit Unicode scalar values directly as UTF-8 where possible.

## 4. Knowledge Object envelope

A Knowledge Object MUST be a JSON object with these required root fields:

- `id`;
- `schemaVersion`;
- `type`;
- `createdAt`;
- `body`;
- `provenance`;
- `rawRef`.

The current schema permits these optional root fields:

- `contexts`;
- `relations`;
- `status`;
- `lineage`;
- `identityClaims`;
- `attachments`;
- `labels`;
- `meta`.

Unknown root fields are rejected. The implementation does not silently reinterpret an unknown schema version as `0.1.0`.

`id` MUST begin with `lb:obj:` and MUST contain no whitespace. This validation does not prove uniqueness, authenticity, ownership, or collision resistance.

The supported `type` values are:

- `inquiry`;
- `observation`;
- `claim`;
- `evidence`;
- `annotation`;
- `synthesis`;
- `translation`;
- `reference`;
- `concept`.

`createdAt` MUST pass the checked-in RFC 3339 date-time validator. Canonicalization preserves the accepted timestamp text; it does not silently change timezone, fractional precision, or lexical representation.

## 5. Canonical serialization

Canonical bytes are produced by `lb.canonical.json.v1`, as defined in [CANONICALIZATION.md](./CANONICALIZATION.md).

The rule:

- orders object keys using the checked-in string ordering;
- preserves array order;
- emits no insignificant whitespace;
- emits UTF-8 without a trailing newline;
- preserves number lexemes in the Rust implementation;
- preserves distinctions among missing values, `null`, empty strings, empty arrays, and empty objects.

`lb.canonical.json.v1` is not a claim of RFC 8785 compatibility. Cross-runtime compatibility is limited to the shared conformance fixtures and the value domain supported by every participating implementation.

The output bytes of an existing rule version MUST NOT be changed in place. Any change to ordering, escaping, number treatment, covered fields, or encoding requires a new rule version.

## 6. Finalization and identity

`finalize_knowledge_object`:

1. validates the Knowledge Object;
2. recursively normalizes it into the checked-in JSON model;
3. produces canonical JSON;
4. returns the object `id` as `canonical_id`;
5. derives the default identity key.

The current default identity derivation is `lb.identity.key.v1`, using FNV-1a 64-bit over the canonical JSON of its semantic basis. The returned form is:

```text
lb:key:lb.identity.key.v1:fnv1a64:<16-lowercase-hex>
```

`lb.identity.key.v2` is also implemented and documented, but generic Knowledge Object finalization does not currently select it by default. Implementations MUST NOT describe a v1-derived key as v2.

Identity claims are independently versioned. The checked-in verifier validates the declared rule version, recomputed identity key, and enclosing-object `canonicalId` consistency. It does not by itself establish issuer authority, signature authenticity, trusted issuance time, provenance authenticity, or source retrievability.

See [IDENTITY_AND_PROVENANCE.md](./IDENTITY_AND_PROVENANCE.md).

## 7. HTTP publish-request envelope and signature

The HTTP publish-request root object permits exactly:

- `object`;
- `publisher`.

`publisher` permits exactly:

- `publicKey`: 64 lowercase hexadecimal characters;
- `signature`: 128 lowercase hexadecimal characters.

The protocol library validator performs Knowledge Object validation and publish-request signature verification. The signature target and field exclusion rule are defined in [HTTP_PUBLISH_SIGNATURE.md](./HTTP_PUBLISH_SIGNATURE.md).

A syntactically valid key or signature string is not sufficient. Verification MUST succeed where the validating path invokes cryptographic verification.

Other ingestion paths MUST be assessed independently. A path that does not invoke the verifier MUST NOT be described as authenticated merely because the object contains signature-shaped fields.

## 8. Relations, lineage, transitions, and effective views

`relations` represents semantic statements between objects. `lineage` represents derivation or revision history. They are not interchangeable.

Replacement and withdrawal are represented by append-only Transition Objects. Publishing a transition does not overwrite or delete the target Knowledge Object.

The checked-in HTTP transition path validates its request, verifies its signature, appends a canonical transition record, and separately appends a reevaluation intent. Those writes are not atomic. A queue-write failure can occur after the transition is durable.

The current effective-view authority implementation primarily compares the target publisher key with the transition publisher key. The broader delegation, revocation, and authorization-time model described by transition authority documents is not fully implemented in this path.

Transition supersession and graph projection fail closed. Missing, corrupt, unreadable, conflicting, or ambiguous evidence MUST NOT be converted into a newly authorized replacement or withdrawal.

Derived effective-view state does not mutate canonical Knowledge Objects or Transition Objects.

See:

- [TRANSITION_OBJECT.md](./TRANSITION_OBJECT.md);
- [TRANSITION_AUTHORITY.md](./TRANSITION_AUTHORITY.md);
- [TRANSITION_SUPERSESSION.md](./TRANSITION_SUPERSESSION.md);
- [ORPHAN_TRANSITIONS.md](./ORPHAN_TRANSITIONS.md);
- [EFFECTIVE_VIEW_READ_API.md](./EFFECTIVE_VIEW_READ_API.md).

## 9. Validation and acceptance boundaries

The repository contains distinct checks for:

1. JSON parsing and resource limits;
2. request-envelope shape;
3. Knowledge Object or Transition Object schema constraints;
4. cross-field semantic rules;
5. canonical identifier and identity consistency;
6. signature verification where invoked;
7. duplicate and immutable-content conflict classification;
8. storage and I/O outcomes;
9. quarantine or rejection behavior in paths that support them.

Passing one layer does not imply that later layers passed.

A path MUST NOT report cryptographic authentication unless it actually ran the relevant verifier successfully. A path MUST NOT report durable storage if its required durable write failed. Partial persistence boundaries, such as transition append followed by queue failure, MUST be documented rather than hidden.

## 10. Errors and fail-closed behavior

Machine-readable codes are the stable integration surface where a specific API defines them. Human-readable messages are supplemental and may change.

The implementation distinguishes, in the relevant paths:

- malformed or invalid input;
- unsupported schema or rule versions;
- signature failure;
- exact duplicate;
- immutable-content conflict;
- quarantine or rejection outcomes;
- storage and I/O failure;
- incomplete, corrupt, unreadable, or ambiguous transition evidence;
- contradictory or unavailable derived state.

Not every API exposes every category or uses the same HTTP status. Clients MUST follow the contract for the specific endpoint rather than infer a universal mapping from this overview.

Unsupported versions, corrupt evidence, ambiguous graphs, and contradictory derived state fail closed. Fail closed means that the implementation does not invent authority or a new semantic transition effect; it does not necessarily mean that every operation returns an HTTP error.

## 11. Capability manifests

The capability manifest reports protocol version `0.1.0`, supported schema versions, object types, carrier kinds, content types, auth modes, validation constraints, defaults, and multi-node policy metadata.

Capability metadata is descriptive. It MUST NOT be treated as proof that every advertised policy is fully enforced by every active runtime path. Endpoint-level behavior and checked-in tests remain necessary when evaluating an implementation.

## 12. Conformance

The fixture corpus is rooted at `conformance/` and described by `conformance/manifest.v1.json`.

A producer claiming conformance MUST reproduce the expected canonical bytes, identifiers, digest inputs, or signature inputs for the fixtures that apply to its claimed feature set.

A consumer claiming conformance MUST reproduce the expected validation or acceptance outcomes for those fixtures.

The JavaScript runner in `conformance/run.mjs` is a reference implementation and test harness. It does not override this contract or the versioned rule documents.

Conformance to a fixture set does not establish operational durability, multi-node safety, formal soak completion, or reference-host qualification.

## 13. Compatibility and change control

Compatibility is governed by [VERSIONING_AND_COMPATIBILITY.md](./VERSIONING_AND_COMPATIBILITY.md).

A behavior change requires a new rule, schema, protocol, or API version when it changes any externally observable element such as:

- accepted field set;
- canonical bytes;
- identifier or identity derivation;
- signature target;
- validation result;
- transition authority or supersession result;
- response shape or machine-readable code.

Documentation MUST describe checked-in behavior accurately. It MUST NOT convert an unimplemented design requirement into a claim about the current runtime.

## 14. Release boundary

This document and its checks do not constitute the formal 72-hour soak or privileged reference-host qualification.

The fixed v1.0.0 candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Later documentation, inventory, and tooling commits do not redefine that candidate.
