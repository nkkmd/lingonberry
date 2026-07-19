# Lingonberry Protocol Contract

**Status: draft for v0.6.0** | **Contract series: protocol v1 candidate** | **Last updated: 2026-07-20**

## 1. Purpose

This document defines the external contract required to implement a Lingonberry-compatible producer or consumer without depending on the Rust implementation.

Normative requirements use **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in their ordinary standards sense.

## 2. Contract boundaries

Lingonberry separates the following representations and versions.

1. **Wire representation**: carrier-specific request or event representation.
2. **Canonical protocol object**: validated semantic JSON object.
3. **Canonical bytes**: deterministic UTF-8 bytes produced by a named canonicalization rule.
4. **Storage representation**: node-internal durable representation.
5. **API representation**: versioned read/write response representation.

A storage, journal, proof, or API version MUST NOT be interpreted as a protocol version.

## 3. Canonical envelope

A canonical knowledge object MUST be a JSON object.

The protocol v1 candidate requires these fields:

- `id`
- `schemaVersion`
- `type`
- `createdAt`
- `body`
- `provenance`
- `rawRef`

The following fields are optional protocol extensions when permitted by the selected schema:

- `contexts`
- `relations`
- `status`
- `lineage`
- `attachments`
- `labels`
- `meta`
- `identityClaims`

Unknown fields MUST be handled according to the selected schema version. An implementation MUST NOT silently reinterpret an unknown protocol or schema version as a known version.

## 4. Canonical serialization

Canonical bytes are produced by `lb.canonical.json.v1`, defined in [CANONICALIZATION.md](./CANONICALIZATION.md).

A conforming implementation MUST:

- recursively sort object member names;
- preserve array order;
- serialize without insignificant whitespace;
- emit UTF-8 bytes without a trailing newline;
- preserve the distinction between missing, `null`, empty string, empty array, and empty object;
- avoid locale- or platform-dependent ordering.

The output bytes of an existing rule version MUST NOT change.

## 5. Identifier and identity rules

The canonical `id` is an opaque protocol identifier. Consumers MUST NOT infer semantics from undocumented substrings.

Identity claims are versioned independently from the protocol and schema. `lb.identity.key.v2` derives a SHA-256 identity key from the canonical JSON serialization of these semantic fields when present:

- `type`
- `createdAt`
- `body`
- `contexts`
- `relations`
- `status`
- `lineage`
- `attachments`
- `labels`

Transport and provenance fields are excluded from the v2 semantic identity basis.

Unknown identity rule versions MUST be reported as unsupported. They MUST NOT be accepted using a fallback rule.

## 6. Digest and signature targets

Every digest or signature operation MUST identify:

- the rule version;
- the exact JSON value or byte sequence covered;
- the canonicalization rule version;
- the hash or signature algorithm;
- the key, digest, and signature encoding.

An implementation MUST NOT sign a parsed object using runtime map iteration order. It MUST sign the bytes defined by the selected signature rule.

Changing the covered fields, canonicalization rule, algorithm, or encoding requires a new rule version.

## 7. Timestamp semantics

`createdAt` is part of the semantic object and identity basis where the selected identity rule includes it.

A producer MUST emit the timestamp form required by the selected schema. Canonicalization MUST NOT silently change timezone, precision, or textual representation. Timestamp normalization, when introduced, MUST be a separately versioned rule applied before canonical serialization.

## 8. Relations, lineage, replacement, and withdrawal

`relations` represents semantic statements between objects. `lineage` represents derivation or revision history. Implementations MUST NOT collapse the two concepts.

Replacement and withdrawal are append-only state transitions. They MUST NOT physically overwrite the existing canonical record. A conflict MUST NOT overwrite an existing canonical object.

The exact replacement and withdrawal schemas remain versioned protocol modules and will be fixed by dedicated v0.6 fixtures.

## 9. Validation levels

A conforming implementation distinguishes at least these levels:

1. **Parse validation**: valid JSON and safe representability.
2. **Envelope validation**: carrier/request framing.
3. **Schema validation**: fields and structural constraints.
4. **Semantic validation**: cross-field and protocol rules.
5. **Identity validation**: identifier and identity claim rules.
6. **Signature validation**: signature target and cryptographic verification.
7. **Acceptance classification**: accept, reject, quarantine, duplicate, or conflict.

Passing an earlier level MUST NOT imply that later levels passed. An object that fails required validation MUST NOT enter canonical storage.

## 10. Error and acceptance behavior

Results MUST be deterministic and machine-readable. Human-readable messages are supplemental.

Implementations MUST distinguish:

- invalid input;
- unsupported version;
- cryptographic failure;
- duplicate;
- conflict;
- quarantined input;
- storage or I/O failure;
- contradictory internal state.

Corruption, unsupported versions, and contradictory state MUST fail closed.

## 11. Conformance

The normative fixture corpus is rooted at `conformance/` and described by `conformance/manifest.v1.json`.

A producer conformance implementation MUST reproduce canonical bytes, identifiers, digest inputs, and signature inputs where fixtures provide them.

A consumer conformance implementation MUST reproduce the expected validation and acceptance classification.

The standalone JavaScript runner in `conformance/run.mjs` is a reference implementation, not the specification itself.

## 12. Compatibility

Compatibility is defined in [VERSIONING_AND_COMPATIBILITY.md](./VERSIONING_AND_COMPATIBILITY.md).

No implementation detail in a Rust crate, JavaScript package, database schema, or journal file overrides this external contract.