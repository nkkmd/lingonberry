# Protocol-Native Wire Format Contract

**Status:** normative for the v1.0.0 pre-release implementation  
**Protocol version:** `0.1.0`  
**Knowledge Object schema version:** `0.1.0`  
**HTTP publish-request schema version:** `0.1.0`

## 1. Scope

This document defines the checked-in JSON wire representation used for Lingonberry Knowledge Objects and the HTTP publish-request envelope that carries them.

A protocol-native Knowledge Object is both the semantic protocol object and the JSON object exchanged by a carrier. The implementation does not insert a separate semantic conversion format between receipt and canonical storage. Carrier framing and request metadata remain outside the Knowledge Object itself.

This contract distinguishes:

- requirements enforced by the checked-in JSON Schemas;
- requirements enforced by the checked-in Rust validator and finalizer;
- deterministic serialization behavior implemented by the Rust protocol package; and
- properties that remain design goals rather than implemented guarantees.

Transition Objects use a separate schema and ingestion path and are outside this document.

## 2. JSON framing and input limits

The checked-in Rust JSON parser accepts exactly one JSON value followed only by JSON whitespace. Trailing non-whitespace content is rejected.

A Rust path using `lingonberry_protocol::parse_json` rejects:

- input larger than `1,048,576` bytes; and
- JSON nesting deeper than `128` levels.

These are implementation limits, not fields in either JSON Schema. A different conforming implementation may use different resource limits, but a deployment claiming parity with the checked-in Rust ingestion path must apply equivalent limits or document the difference.

The wire encoding is UTF-8 JSON. Neither schema defines an HTTP media type, record delimiter, streaming frame, compression format, or archive container. Those remain carrier-specific concerns.

## 3. Knowledge Object wire shape

A Knowledge Object is a JSON object. The following fields are required by both the checked-in schema and the Rust validator:

- `id`
- `schemaVersion`
- `type`
- `createdAt`
- `body`
- `provenance`
- `rawRef`

The following root fields are optional:

- `contexts`
- `relations`
- `status`
- `lineage`
- `identityClaims`
- `attachments`
- `labels`
- `meta`

Unknown root fields are rejected by both the schema and the Rust validator.

The authoritative structural reference is [`schemas/knowledge-object.schema.json`](../../schemas/knowledge-object.schema.json). The fixtures under [`fixtures/knowledge-object/`](../../fixtures/knowledge-object/) provide examples, but a fixture does not override the schema or runtime behavior.

## 4. Required field contract

### 4.1 `id`

`id` is the canonical Knowledge Object identifier supplied by the object.

The JSON Schema requires the complete `lb.protocol.id.ascii.v1` Knowledge Object grammar and a maximum length of 255 characters. The checked-in Rust Knowledge Object validator currently performs weaker validation: it requires only the `lb:obj:` prefix and absence of whitespace.

Consequently, schema acceptance and Rust-validator acceptance are not equivalent. A path that invokes only the Rust validator MUST NOT be described as enforcing the complete identifier grammar or length limit. See [`PROTOCOL_IDENTIFIERS.md`](./PROTOCOL_IDENTIFIERS.md).

### 4.2 `schemaVersion`

`schemaVersion` MUST be the literal string `0.1.0`.

This is a protocol object schema version. It is not the product release version and MUST NOT be changed merely because the Lingonberry product version changes.

### 4.3 `type`

`type` MUST be one of:

```text
inquiry
observation
claim
evidence
annotation
synthesis
translation
reference
concept
```

### 4.4 `createdAt`

`createdAt` MUST be a date-time string accepted by the validating path.

The JSON Schema declares the Draft 7 `date-time` format. The Rust validator uses its checked-in RFC 3339 date-time validation helper. This document does not claim that every external JSON Schema engine and the Rust helper accept exactly the same date-time language.

Validation does not rewrite the timestamp to UTC, change fractional-second precision, or replace the supplied spelling.

### 4.5 `body`

`body` MUST contain exactly:

- non-empty string `text`; and
- `language`, matching the repository's BCP47-style syntax.

Additional `body` properties are rejected.

The implementation validates the language tag's syntax. It does not canonicalize case, expand aliases, verify registry membership, infer language from text, or translate content.

### 4.6 `provenance`

`provenance` MUST contain a non-empty `sources` array. Each source requires non-empty `protocol` and `sourceId` strings and may include `authorId` and `observedAt`.

The validator checks shape and basic date-time syntax. It does not contact the named protocol, prove that the source exists, authenticate the author, or establish evidentiary truth.

### 4.7 `rawRef`

`rawRef` MUST contain non-empty `protocol` and `sourceId` strings. It may also contain non-empty `locator` and `payloadHash` strings.

The current schema and Rust validator treat `payloadHash` as an opaque non-empty string. They do not require a named hash algorithm, verify the digest, retrieve the locator, or prove that the referenced payload is available.

A Knowledge Object missing `rawRef` is invalid. Retaining `rawRef` preserves a reference supplied by the producer; it does not guarantee durable retention of the referenced raw bytes.

## 5. Optional field contract

The schema and Rust validator define the shape of `contexts`, `relations`, `status`, `lineage`, `identityClaims`, `attachments`, `labels`, and `meta`.

Schema defaults on arrays such as `relations`, `lineage`, `attachments`, and `labels` are annotations. The checked-in Rust finalizer does not insert omitted default arrays.

Neither validation nor finalization:

- sorts relations, lineage edges, labels, attachments, or identity claims;
- deduplicates arrays;
- canonicalizes language tags;
- rewrites timestamps;
- supplies a default lifecycle status; or
- removes optional fields.

Order therefore remains significant in the serialized representation unless another versioned rule explicitly defines order-insensitive processing for a specific derivation.

## 6. Identity claims

`identityClaims` is optional. When present, each claim must have the shape defined in the Knowledge Object schema.

The checked-in Rust validator is stricter than the generic schema in several semantic respects. It currently requires the supported identity rule version and checks that claim values correspond to the containing object and the implementation-derived identity key.

The generic finalizer derives and returns an identity key using `lb.identity.key.v1` and FNV-1a 64. It does not automatically add an `identityClaims` array to an object that lacks one.

An identity key is a deterministic implementation output. It is not a signature, authorization credential, proof of authorship, or adversarial collision-resistant digest.

See [`IDENTITY_AND_PROVENANCE.md`](./IDENTITY_AND_PROVENANCE.md) and [`PROTOCOL_IDENTIFIERS.md`](./PROTOCOL_IDENTIFIERS.md).

## 7. HTTP publish-request envelope

The HTTP publish-request wire shape is defined by [`schemas/http-publish-request.schema.json`](../../schemas/http-publish-request.schema.json).

It contains exactly:

- `object`: a Knowledge Object; and
- `publisher`: an object containing `publicKey` and `signature`.

The schema requires:

- `publicKey`: 64 lowercase hexadecimal characters; and
- `signature`: 128 lowercase hexadecimal characters.

The Rust `validate_publish_request` path validates the nested Knowledge Object, validates the publisher fields, rejects additional envelope and publisher properties, and invokes the checked-in signature verifier.

`publisher` is carrier/request metadata. It is not a required field of the Knowledge Object and is not inserted into the finalized Knowledge Object.

The schema description alone does not define the signed byte sequence. Implementations MUST use the checked-in, versioned signature-verification rule for interoperability rather than inventing a signature scope from field order or raw request bytes.

## 8. Validation pipeline

For a checked-in Rust ingestion path, the processing model is:

1. parse one bounded JSON value;
2. detect whether the value is a Knowledge Object or an HTTP publish request when the caller uses shape detection;
3. validate the selected shape;
4. for a publish request, verify publisher metadata and signature through the checked-in verifier;
5. finalize the nested or direct Knowledge Object; and
6. pass the finalized object and canonical JSON to the relevant storage or indexing path.

Shape detection is intentionally narrow: an object containing both `object` and `publisher` is classified as a publish request; other values are classified as a Knowledge Object and then validated. Shape detection is not validation.

A carrier may reject an otherwise structurally valid object because of authentication, authorization, capability, storage, duplicate/conflict, quarantine, or operational policy.

## 9. Deterministic normalization and serialization

The checked-in `normalize_json` function recursively rebuilds JSON objects into ordered maps. Arrays retain their supplied order, and scalar values retain their supplied values.

The checked-in `to_canonical_json` function emits a compact JSON representation from that ordered tree. In this implementation, deterministic object-member ordering is provided by the ordered map representation.

The implemented normalization guarantee is limited to deterministic recursive object-key ordering and deterministic serialization of the resulting in-memory value. It does **not** include:

- Unicode normalization;
- whitespace normalization inside strings;
- language-tag canonicalization;
- timestamp canonicalization;
- numeric semantic normalization beyond the parser/serializer's stored number token behavior;
- array sorting;
- default insertion; or
- semantic relation or lineage normalization.

Consumers MUST NOT claim a broader canonicalization algorithm unless that algorithm is separately versioned and implemented.

## 10. Finalization

`finalize_knowledge_object` first runs the Rust Knowledge Object validator. On success it:

- recursively orders object keys through `normalize_json`;
- serializes the normalized value with `to_canonical_json`;
- returns the object's existing `id` as `canonical_id`;
- derives an `lb.identity.key.v1` identity key; and
- returns the normalized object and canonical JSON.

Finalization does not:

- generate a new Knowledge Object ID;
- replace or repair an invalid field;
- add provenance;
- add or verify the availability of raw data;
- add an identity claim;
- sign the object;
- assign carrier metadata;
- persist the object by itself; or
- prove that later storage, indexing, or effective-view processing succeeded.

The supplied `id` remains the canonical identifier reference axis after successful finalization.

## 11. Replay, append-only storage, and retrieval

The wire shape is designed to permit deterministic replay, but replayability is not established by schema validation alone.

A complete replay guarantee depends on the receiving path preserving the accepted object, the applicable duplicate/conflict rules, archive and storage behavior, supported schema and rule versions, and availability of any external raw reference required by an operator.

Likewise, describing a carrier or storage backend as append-only is a property of that backend and its operational policy, not an intrinsic property of a JSON object.

## 12. Compatibility

A consumer evaluating compatibility MUST distinguish at least:

- product release version;
- protocol version;
- Knowledge Object schema version;
- HTTP publish-request schema version;
- identity-key rule version;
- signature rule or verification behavior; and
- carrier capability version.

A product-version change does not automatically change any protocol or rule version.

Before accepting a new wire version, an implementation should determine whether it can:

- parse and validate existing objects;
- preserve accepted identifiers and semantic fields;
- reproduce the required deterministic derivations;
- verify publish signatures under the applicable rule;
- replay stored records without silent rewriting; and
- rebuild indexes from preserved canonical objects.

Unknown or unsupported versions MUST fail explicitly at the boundary that requires their semantics. They MUST NOT be silently interpreted as a known version.

## 13. Conformance evidence

Relevant checked-in evidence includes:

- Knowledge Object schema validation;
- HTTP publish-request schema validation;
- Rust protocol validator and finalizer tests;
- publish-signature verification tests;
- fixtures under `fixtures/knowledge-object/` and `fixtures/http-publish-request/`; and
- ingestion, duplicate/conflict, archive, quarantine, storage, retrieval, and index tests that consume finalized Knowledge Objects.

Conformance evidence should cover at least:

- the minimal valid Knowledge Object;
- required-field rejection, including missing `rawRef`;
- unsupported schema and object types;
- unknown-property rejection;
- valid and invalid body, provenance, and raw-reference shapes;
- optional-field handling without implicit defaults;
- deterministic object-key ordering with array-order preservation;
- identity-claim semantic checks;
- valid and invalid publisher key/signature encodings;
- signature verification failure;
- parser byte and nesting limits; and
- the known difference between schema-strength identifier validation and the weaker Rust Knowledge Object identifier helper.

## 14. Security boundaries

Structural validity does not establish authenticity, authorization, trustworthiness, non-repudiation, source availability, or semantic truth.

Implementations MUST bound parser resource use, verify signatures where the carrier contract requires them, validate identifiers at the strength required by the receiving path, and avoid treating `rawRef.locator`, relation targets, attachment URIs, labels, or metadata as trusted filesystem paths, commands, or authorization decisions.

Canonical JSON and FNV-derived identity keys MUST NOT be treated as cryptographic signatures or collision-resistant content digests.

## 15. Release boundary

This contract describes the checked-in v1.0.0 pre-release implementation. It does not redefine the fixed release candidate `f9543019f2c219aea3b085ff90f2da201b268a48`.

This documentation change does not complete privileged reference-host qualification, start or complete the formal 72-hour soak, update the product version, create a release tag, or publish a GitHub Release.

## 16. Related documents

- [`PROTOCOL_CONTRACT.md`](./PROTOCOL_CONTRACT.md)
- [`PROTOCOL_IDENTIFIERS.md`](./PROTOCOL_IDENTIFIERS.md)
- [`IDENTITY_AND_PROVENANCE.md`](./IDENTITY_AND_PROVENANCE.md)
- [`TIMESTAMP_SEMANTICS.md`](./TIMESTAMP_SEMANTICS.md)
- [`../concepts/CARRIER.md`](../concepts/CARRIER.md)
- [`../concepts/CONCEPT_MODEL.md`](../concepts/CONCEPT_MODEL.md)
- [`schemas/knowledge-object.schema.json`](../../schemas/knowledge-object.schema.json)
- [`schemas/http-publish-request.schema.json`](../../schemas/http-publish-request.schema.json)