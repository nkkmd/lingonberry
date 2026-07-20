# Versioning and Compatibility

**Status: draft for v0.6.0** | **Last updated: 2026-07-20**

## 1. Independent version axes

Lingonberry uses independent version axes. A change in one axis does not automatically change another.

| Axis | Identifies | Owned by | Unknown version behavior |
|---|---|---|---|
| Protocol version | Semantic object contract | Protocol specification | Reject as unsupported |
| Schema version | Structural constraints for an object/profile | Schema document | Reject as unsupported unless explicit compatibility exists |
| Canonicalization rule | Deterministic byte serialization | Canonicalization specification | Reject operations requiring canonical bytes |
| Identity rule | Semantic identity basis and identifier derivation | Identity specification | Reject claim as unsupported |
| Signature rule | Signature target, algorithm, and encoding | Signature specification | Reject signature as unsupported |
| API version | HTTP/CLI request and response contract | Public API specification | Return versioned API error |
| Storage format version | Durable node-internal layout | Storage implementation | Refuse write/open until supported migration exists |
| Journal version | Durable transaction/recovery record layout | Operation-specific journal contract | Refuse unsafe resume or mutation |
| Proof version | Cleanup/replacement/backup proof representation | Operation-specific proof contract | Reject proof as unsupported |

Implementations MUST NOT infer one version from another.

## 2. Compatibility classes

A version pair is classified as one of:

- **compatible**: the consumer can process the producer output without semantic loss;
- **compatible-with-restrictions**: processing is allowed only under documented constraints;
- **unsupported**: the version is known but not implemented;
- **invalid**: the version value or combination violates the contract;
- **unknown**: the version identifier is not recognized.

`unknown` and `unsupported` MUST NOT be converted to `compatible` by fallback.

## 3. Initial compatibility matrix

This matrix records the v0.6 foundation. It will be expanded as signature, replacement, withdrawal, and legacy fixtures are fixed.

| Producer artifact | Consumer capability | Result |
|---|---|---|
| `lb.canonical.json.v1` | `lb.canonical.json.v1` | compatible |
| unknown canonicalization rule | `lb.canonical.json.v1` only | unsupported |
| `lb.identity.key.v1` | v1 identity support | compatible |
| `lb.identity.key.v2` | v2 identity support | compatible |
| `lb.identity.key.v2` | v1 identity support only | unsupported |
| unknown identity rule | any known identity support | unsupported |
| schema `0.1.0` | explicit `0.1.0` support | compatible |
| unknown schema version | no declared support | unsupported |
| protocol object | storage format v1 | no direct compatibility relation; protocol must pass validation first |
| storage format v1 | protocol v1 | no direct compatibility relation; storage is node-internal |

## 4. Backward and forward compatibility

A change is backward compatible only when an older conforming consumer can process the new producer output without changing the meaning required by the older contract.

A change is forward compatible only when a newer conforming consumer explicitly declares support for the older artifact.

Adding an optional field is not automatically compatible. Compatibility depends on the selected schema and unknown-field policy.

Changing canonical bytes, digest targets, signature targets, required fields, identifier semantics, or acceptance classification is breaking and requires a new rule, schema, API, or protocol version as appropriate.

## 5. Legacy handling

Legacy fixtures MUST declare:

- the version or historical behavior represented;
- whether acceptance, rejection, or quarantine is expected;
- whether normalization is permitted;
- whether the fixture can be re-emitted by a current producer.

Legacy input MUST NOT be silently rewritten into a current object when that would change identity, signature validity, or provenance.

## 6. Fail-closed requirements

An implementation MUST fail closed when:

- the protocol or schema version is unknown;
- a required canonicalization, identity, digest, or signature rule is unsupported;
- version fields contradict one another;
- a storage, journal, or proof version is newer than supported;
- compatibility depends on an undocumented implementation detail.

## 7. Change procedure

A compatibility-affecting change requires:

1. a specification update;
2. an explicit version decision;
3. new or updated fixtures;
4. a compatibility matrix update;
5. cross-implementation tests;
6. release notes describing producer and consumer impact.

Fixtures MUST NOT be automatically regenerated merely to make an implementation pass.