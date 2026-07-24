# Canonicalization

**Status: normative v1.0 pre-release protocol contract** | **Rule version: `lb.canonical.json.v1`** | **Last reviewed: 2026-07-25**

This document defines the checked-in canonical JSON behavior used by Lingonberry protocol code, identity-key derivation, publish-request signatures, archive and evidence serialization, and conformance fixtures.

The v1 rule describes the existing Rust and JavaScript reference implementations. It is not a claim of full RFC 8785 compliance.

## 1. Canonicalization boundary

Canonicalization receives an already parsed JSON value and emits one JSON text without insignificant whitespace or a trailing newline.

```text
JSON text
  -> implementation parser
  -> parsed JSON value
  -> recursive object-key ordering
  -> compact JSON serialization
  -> UTF-8 bytes
```

Canonicalization does not validate a knowledge-object schema, add defaults, remove optional fields, normalize timestamps or language tags, sort arrays semantically, or choose a cryptographic hash algorithm.

## 2. Rule identifier

The frozen rule identifier is:

```text
lb.canonical.json.v1
```

Changing output for values already covered by this rule requires a new rule identifier. A future rule must not silently reinterpret existing v1 identity keys, signatures, digests, or fixtures.

## 3. Reference implementations

The checked-in implementations are:

| Runtime | Parse and value model | Canonical operation |
|---|---|---|
| Rust | `packages/protocol/src/lib.rs` custom parser and `JsonValue` | `normalize_json` and `to_canonical_json` |
| JavaScript | `JSON.parse` and native JavaScript values | `sortKeys` and `JSON.stringify` |

The shared fixture is under:

```text
conformance/canonicalization/
```

The current fixture covers object-key ordering, nested objects, preserved array order, non-ASCII strings, compact output, and idempotence.

## 4. Object ordering

Every object is serialized with keys in ascending implementation string order. The operation is recursive for nested objects, including objects contained in arrays.

Rust stores object members in `BTreeMap<String, JsonValue>`. JavaScript applies:

```text
Object.keys(value).sort()
```

The checked-in fixture uses keys for which both implementations produce the same order. Protocol producers must not assume that untested edge cases involving supplementary Unicode key characters are interoperable merely because each runtime is internally deterministic.

Example:

```json
{"z":1,"a":{"z":2,"a":3}}
```

becomes:

```json
{"a":{"a":3,"z":2},"z":1}
```

## 5. Arrays

Array element order is preserved exactly.

Canonicalization recursively orders object members inside array elements, but it does not:

- sort array values;
- remove duplicates;
- normalize relation order;
- normalize label order;
- treat arrays as sets.

Any semantic array normalization must occur in a separately versioned rule before canonicalization.

## 6. Strings and UTF-8

Canonical output is encoded as UTF-8.

The Rust serializer emits ordinary Unicode scalar values directly and escapes:

```text
quotation mark
reverse solidus
backspace
form feed
line feed
carriage return
tab
other U+0000 through U+001F control characters
```

Other control characters use lowercase four-digit `\u` escapes. Solidus is not escaped on output.

No Unicode normalization is performed. Canonically equivalent NFC and NFD strings remain different byte sequences.

The Rust parser rejects invalid UTF-8 and unescaped control characters. Its current `\u` parser accepts one Unicode scalar value per escape and rejects surrogate code units rather than combining surrogate pairs. Producers that require Rust and JavaScript interoperability must emit valid UTF-8 scalar values directly and must not rely on escaped surrogate pairs.

## 7. Numbers

`lb.canonical.json.v1` does not define a cross-runtime mathematical number normalization algorithm.

Rust stores a parsed number as its validated source lexeme and emits that lexeme unchanged. Therefore these may remain byte-distinct in Rust:

```text
1
1.0
1e0
1E+0
```

JavaScript parses numbers into the native `Number` type and `JSON.stringify` selects its own output representation. Consequently, arbitrary JSON number lexemes are not guaranteed to produce identical Rust and JavaScript canonical bytes.

Protocol schemas, identity bases, signed payloads, and conformance fixtures must restrict numeric values to an interoperability-safe subset that has been tested across both implementations. A stronger number-normalization algorithm requires a new canonicalization rule version.

Non-finite JavaScript values are outside the JSON input model and must not enter canonicalization.

## 8. Duplicate object members

The two parsers do not expose duplicate members as a preserved sequence.

The current Rust parser inserts members into a `BTreeMap`; a later duplicate member replaces the earlier value. `JSON.parse` likewise exposes only one resulting property value.

Because duplicate-member provenance is lost and parser behavior is not a safe protocol distinction:

- conforming producers must never emit duplicate object member names;
- signed, hashed, or identity-bearing input with duplicate members is unsupported;
- validators or ingress layers may reject duplicate members before the reference parser;
- canonical output must not be used to prove which duplicate occurrence appeared in the original JSON text.

This is a producer prohibition, not a claim that the current Rust parser detects duplicates.

## 9. Booleans and null

The exact lowercase tokens are:

```text
true
false
null
```

## 10. Whitespace and line termination

Canonical output contains no insignificant whitespace around object members, array elements, colons, or commas.

The canonical string has no trailing newline. JSONL writers that append `\n` do so outside the canonicalization function as a record-framing operation.

## 11. Missing and empty values

Canonicalization preserves the distinction among:

```text
missing member
null
empty string
empty array
empty object
```

It does not add, remove, or replace fields. Default insertion and semantic normalization belong to separately versioned schema or normalization logic.

## 12. Parser limits and accepted input

The Rust reference parser enforces:

```text
maximum input size: 1,048,576 bytes
maximum nesting depth: 128
no trailing non-whitespace content
valid JSON number grammar
valid UTF-8
```

These are parser limits, not properties of the emitted canonical JSON format. Another implementation may impose stricter resource limits, but protocol-facing limits must be documented and must fail closed.

## 13. Identity-key basis

Identity-key derivation does not canonicalize the complete knowledge object indiscriminately. The checked-in v1 identity basis selects these members when present:

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

The selected object is then serialized canonically and hashed by the identity-key rule. Fields outside the basis, including `id`, provenance, raw references, identity claims, and metadata, do not enter this specific v1 basis.

Canonicalization supplies deterministic bytes; it does not define which fields belong to an identity basis.

## 14. Publish-request signature payload

The publish-request signature payload is a canonical object containing:

```text
object
publisher
```

Before canonicalization, `publisher.signature` is removed. The publisher public key remains in the payload.

The canonical UTF-8 bytes are passed to Ed25519 verification. Canonicalization does not authenticate the key, authorize the publisher, or establish provenance independently of signature verification and policy checks.

## 15. Canonicalization and digests

Canonical bytes are often inputs to hashes or integrity digests, but this rule does not choose or strengthen the digest algorithm.

In particular:

- FNV-1a integrity digests used by some operational artifacts are not signatures or MACs;
- canonical output does not establish trusted time;
- canonical output does not prove the original formatting of parsed JSON;
- canonical output does not preserve duplicate-member occurrence or original number spelling in JavaScript;
- canonical equality is not automatically semantic equality for every application profile.

## 16. Conformance requirements

A conformance case should contain at least:

```text
input JSON
expected canonical JSON
```

For every supported case, an implementation must verify:

1. the input is accepted by that implementation;
2. the emitted canonical string exactly matches the expected file;
3. the UTF-8 bytes match exactly;
4. canonicalizing the parsed canonical output is idempotent;
5. array order is preserved;
6. nested object ordering is recursive.

Cross-runtime claims must be limited to shared fixtures that both runtimes execute successfully. Untested numeric lexemes, duplicate members, escaped surrogate pairs, and Unicode key-order edge cases are outside the demonstrated interoperability set.

## 17. Compatibility and versioning

`lb.canonical.json.v1` output for covered values is frozen.

A behavior change involving any of the following requires explicit compatibility analysis and normally a new version such as `lb.canonical.json.v2`:

- number normalization;
- duplicate-member rejection in the core parser;
- Unicode normalization;
- surrogate-pair handling;
- object-key ordering semantics;
- string escaping;
- semantic array ordering;
- default-field insertion or removal.

Capability declarations may advertise supported canonicalization rule identifiers, but advertisement does not replace conformance testing.

## 18. Non-goals

This rule does not perform:

- schema validation;
- semantic field insertion or deletion;
- timestamp timezone conversion;
- language-tag case normalization;
- Unicode NFC normalization;
- relation or label sorting;
- duplicate-event resolution;
- canonical ID assignment;
- signature creation or key authorization;
- cryptographic algorithm migration;
- release qualification or release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
