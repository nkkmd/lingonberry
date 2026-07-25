# Identity and Provenance

**Status: normative v1.0 pre-release protocol contract** | **Last reviewed: 2026-07-25**

This document defines the implemented responsibilities and boundaries of the Lingonberry identity key, identity claims, provenance, and `rawRef` fields.

Identity keys are semantic comparison keys. They are not object identifiers, signatures, authorization proofs, lineage identifiers, storage addresses, or carrier identities.

## 1. Identity-key rule versions

The implemented identity-key rules are:

| Rule version | Digest | Encoding | Current role |
|---|---|---|---|
| `lb.identity.key.v1` | FNV-1a 64-bit | `lb:key:lb.identity.key.v1:fnv1a64:<16 lowercase hex>` | legacy and current default finalization output |
| `lb.identity.key.v2` | SHA-256 | `lb:key:lb.identity.key.v2:sha256:<64 lowercase hex>` | implemented stronger rule and claim-verification option |

The v1 output remains stable for compatibility with stored objects, fixtures, and callers of the protocol finalization API. FNV-1a is an integrity fingerprint, not a collision-resistant cryptographic identity.

The v2 rule hashes the same canonical semantic basis with SHA-256.

The current implementation does **not** automatically migrate a v1 claim to v2, rewrite an existing claim, or make v2 the default `FinalizedKnowledgeObject.identity_key`. New producers may emit v2 claims, but consumers must continue to recognize both implemented rule versions.

## 2. Semantic basis

Both v1 and v2 derive from exactly these root fields when they are present:

```text
type
createdAt
body
contexts
relations
status
lineage
attachments
labels
```

Fields absent from the object are absent from the basis. Default values are not inserted solely for identity-key derivation.

The following fields are excluded:

```text
id
schemaVersion
provenance
rawRef
identityClaims
meta
```

Unknown fields are not part of the implemented basis. Knowledge Object validation rejects unknown root fields before normal finalization.

For a non-object input, the standalone identity-key basis helper produces an empty object. Normal protocol finalization first requires a valid Knowledge Object, so this behavior is not permission to finalize a non-object value.

## 3. Canonicalization and derivation

The basis is serialized with `lb.canonical.json.v1` and encoded as UTF-8.

```text
selected semantic fields
  -> lb.canonical.json.v1
  -> UTF-8 bytes
  -> version-selected digest
  -> versioned identity key
```

Rust v2 derivation uses `to_canonical_json` and SHA-256. JavaScript v2 derivation uses recursively sorted keys, `JSON.stringify`, UTF-8 input, and SHA-256. Cross-runtime interoperability is established by the checked-in conformance vectors, not by assuming that arbitrary JSON implementations serialize identically.

## 4. Current finalization behavior

The protocol and validation finalization paths:

1. validate the Knowledge Object;
2. normalize JSON object ordering;
3. serialize the complete object canonically;
4. retain the object's `id` as `canonical_id`;
5. derive and return the **v1** identity key;
6. preserve the normalized object and complete canonical JSON.

Therefore, the `identity_key` returned by current generic finalization is:

```text
lb:key:lb.identity.key.v1:fnv1a64:...
```

The existence of v2 support does not change that default.

## 5. Identity claim schema

An `identityClaims` field, when present, must be an array. Each schema-valid claim contains:

```text
schemaVersion
claimType
ruleVersion
identityKey
canonicalId
issuer
issuedAt
verification
```

The current JSON Schema requires:

- `schemaVersion` to equal `1`;
- `claimType` to equal `identity`;
- `ruleVersion` to be a non-empty string;
- `identityKey` to match the bounded `lb:key:` shape;
- `canonicalId` to match the bounded `lb:obj:` shape;
- `issuer` to contain `protocol` and `sourceId`, with optional `signerId`;
- `issuedAt` to be a date-time string;
- `verification` to contain `method` and a `sha256:<64 lowercase hex>` payload hash;
- optional `verification.signature` to be non-empty;
- optional `verification.status` to be `pending`, `verified`, or `rejected`;
- no additional claim, issuer, or verification properties.

Schema validation establishes shape. It does not establish that the issuer exists, the stated signature verifies, the payload hash identifies the intended bytes, or the verification status is trustworthy.

## 6. Version-aware claim verification

The version-aware identity validator independently evaluates each array entry.

For each claim, it:

1. requires the claim to be an object;
2. requires a non-empty string `ruleVersion`;
3. selects the implemented v1 or v2 derivation rule;
4. recomputes the expected identity key from the enclosing object;
5. requires string `identityKey` to equal the expected value;
6. when both the enclosing object ID and a string claim `canonicalId` are available, requires them to match.

Unsupported rules produce a distinct error containing:

```text
ruleVersion is unsupported
```

The validation facade separates unsupported identity rules from ordinary mismatches through `IdentityValidationStatus::Unsupported`.

The version-aware validator itself does not additionally verify:

- `schemaVersion` or `claimType`;
- issuer authority;
- issuer signatures;
- `issuedAt` trust or freshness;
- `verification.method` semantics;
- `verification.payloadHash` against a reconstructed payload;
- `verification.signature`;
- `verification.status` provenance.

Those fields are shape-checked by the Knowledge Object schema. A future cryptographic claim verifier requires a separate versioned contract.

## 7. Validation and acceptance behavior

Full Knowledge Object validation combines:

- structural and semantic schema validation;
- version-aware identity claim validation;
- classification of unsupported identity rule versions.

The validation facade removes legacy schema messages that hard-code v1-only behavior, then applies the version-aware v1/v2 validator. This prevents a valid v2 claim from being rejected merely because older schema validation text expected v1.

The resulting identity status is:

```text
valid
invalid
unsupported
not-present
```

Acceptance policy determines whether unsupported rules are rejected or deferred. Unsupported does not mean the claim is a valid match, and it must not be silently treated as verified.

An object with no non-empty `identityClaims` array has status `not-present`. Identity claims are optional unless deployment acceptance policy requires them.

## 8. Separation from publisher signatures

An identity claim and an HTTP publisher signature are different evidence layers.

- The identity key compares the selected semantic basis.
- The identity claim states that a versioned identity key corresponds to a canonical object ID.
- The HTTP publisher signature covers a versioned publish-request payload.
- Provenance describes asserted sources and observations.
- `rawRef` points to source material or a retrieval locator.

A valid HTTP signature does not automatically verify an identity claim. A valid identity-key recomputation does not prove that the claim issuer authorized the object. Neither proves authorization to mutate an effective view.

## 9. Provenance contract

Knowledge Objects require a `provenance` object containing a non-empty `sources` array.

Each schema-valid source contains:

```text
protocol
sourceId
```

and may contain:

```text
authorId
observedAt
```

No additional provenance-source properties are allowed by the checked-in schema.

Provenance is excluded from the identity-key basis. Two objects with the same semantic basis but different provenance therefore derive the same identity key.

Current validation checks provenance structure, required source fields, allowed properties, and supported timestamp shape through the protocol validator and schema fixtures. It does not contact the named source, verify `authorId`, authenticate the source protocol, or reconstruct a transformation chain.

## 10. `rawRef` contract

Knowledge Objects require a `rawRef` object containing:

```text
protocol
sourceId
```

and may contain:

```text
locator
payloadHash
```

No additional `rawRef` properties are allowed by the checked-in schema.

`rawRef` is excluded from the identity-key basis. Its presence preserves a reference to source material without making transport location part of semantic identity.

Current validation does not retrieve `locator`, verify that the referenced content still exists, interpret `payloadHash`, or prove that the raw payload canonicalizes to the enclosing object.

## 11. Preservation boundary

Normal finalization preserves `provenance`, `rawRef`, and `identityClaims` in the normalized complete object and canonical JSON. They are excluded only from the identity-key basis.

Carrier, archive, and replay paths must not discard those fields when preserving the canonical object or signed wire request.

Identity-key equality alone must not be used to merge, overwrite, or deduplicate objects whose canonical IDs or complete canonical bytes differ. Duplicate and conflict classification is a separate storage contract.

## 12. Migration rules

For v1-to-v2 migration:

- existing v1 claims remain immutable;
- a producer may append a new v2 claim only by publishing a new valid object representation under the applicable append-only and identity rules;
- consumers verify each claim according to its declared rule version;
- unsupported versions remain unsupported rather than being coerced to v1 or v2;
- removal of v1 verification support requires a separately reviewed compatibility and migration specification;
- the current generic finalization API remains v1 until an explicit versioned implementation change is made.

Adding a v2 claim changes the complete canonical object bytes even when the semantic identity basis remains unchanged.

## 13. Security properties and non-goals

Implemented v2 SHA-256 derivation provides collision resistance appropriate to a digest-based comparison key, subject to the canonicalization and selected-field contract. It does not provide:

- issuer authentication;
- authorization;
- non-repudiation;
- proof of possession of a private key;
- freshness or replay protection;
- trusted timestamps;
- source availability;
- source-content verification;
- transformation-chain verification;
- confidentiality;
- uniqueness of the semantic concept represented by an object.

V1 FNV-1a must not be used where cryptographic collision resistance is required.

## 14. Reference implementations and conformance

Reference implementations are:

| Language | Location |
|---|---|
| Rust | `packages/identity/src/lib.rs` |
| JavaScript | `packages/identity/identity-key.mjs` |
| Rust v1 derivation and protocol validation | `packages/protocol/src/lib.rs` |
| Validation facade | `packages/validation/src/lib.rs` |

Relevant vectors and fixtures include:

```text
conformance/identity-key-v2/
conformance/identity-claims/
fixtures/knowledge-object/with-identity-claim.json
fixtures/knowledge-object/invalid-identity-claim-mismatch.json
fixtures/http-publish-request/with-identity-claim.json
fixtures/http-publish-request/invalid-identity-claim-mismatch.json
```

Any change to the selected basis, canonicalization, digest, key encoding, claim schema, version dispatch, unsupported-rule classification, or default finalization rule requires compatibility review and updated conformance vectors.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
