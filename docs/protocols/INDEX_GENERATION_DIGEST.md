# Index Generation Digest Rule

**Status: normative v1.0 pre-release protocol contract** | **Digest rule: `lb.index.generation.v1`** | **Lifecycle contract version: `1`** | **Last reviewed: 2026-07-25**

This document defines the deterministic fingerprints and generation values emitted by the checked-in index lifecycle implementation when comparing a derived index snapshot with canonical storage.

The rule describes derived operational state. It is not a protocol-object digest, storage-format version, signature digest, authorization proof, or cryptographic integrity guarantee.

## 1. Implemented result model

Index rebuild and verification return an `IndexRebuildResult` with:

```text
contractVersion
status
code
message
storage
index
missingFromIndex
unexpectedInIndex
ambiguousIds
```

The implemented lifecycle contract version is:

```text
1
```

Each non-null `storage` or `index` generation contains:

```text
generation
recordCount
idDigest
contentDigest
```

The digest rule name `lb.index.generation.v1` identifies the calculation documented here. The current result structure does not carry a separate runtime `digestRuleVersion` field.

## 2. Digest primitive

`lb.index.generation.v1` uses FNV-1a 64-bit with:

| Parameter | Value |
|---|---|
| Offset basis | `0xcbf29ce484222325` |
| Prime | `0x100000001b3` |
| Arithmetic | unsigned 64-bit wrapping multiplication |
| Input encoding | UTF-8 |
| Per-line delimiter | one byte `0x0a` |
| Output | `fnv1a64:` followed by 16 lowercase hexadecimal digits |

The digest starts from the offset basis. For every input line, the implementation processes each UTF-8 byte and then one newline byte. For each byte it XORs the current state with the byte and multiplies by the prime with wrapping 64-bit arithmetic.

The newline after the final line is part of the rule. An empty line set produces the unchanged offset-basis digest.

## 3. Record fingerprint

A stored catalog record fingerprint is calculated over exactly three lines in this order:

1. `carrierIdentity`;
2. `storedAt`;
3. canonical JSON of the stored object using `lb.canonical.json.v1`.

```text
recordFingerprint = fnv1a64_lines([
  carrierIdentity,
  storedAt,
  canonicalJson(object)
])
```

The complete strings are used without trimming or timestamp normalization at this stage. Changing `carrierIdentity` or `storedAt` changes the fingerprint even when the object has identical semantic content.

The record fingerprint is an index-comparison value. It is not the object identity key and is not an authenticity claim.

## 4. Canonical-ID ordering

The implementation collects canonical IDs in Rust `BTreeSet<String>` and record fingerprints in `BTreeMap<String, String>`. IDs are therefore processed in Rust string lexical order.

Implementations producing compatible values must reproduce that ordering over the exact canonical-ID strings. They must not use locale-aware collation, case folding, normalization, insertion order, or carrier order.

Duplicate canonical IDs cannot coexist in the fingerprint map. Construction of the map retains one value per canonical ID according to the upstream collection path; callers must not interpret this digest rule as duplicate-record arbitration.

## 5. ID digest

The ID digest is the line digest of canonical IDs in the deterministic order described above:

```text
idDigest = fnv1a64_lines(sortedCanonicalIds)
```

For an empty snapshot, the input line set is empty.

## 6. Content digest

For every record in canonical-ID order, construct one line:

```text
canonicalId + U+0000 + recordFingerprint
```

The content digest is the line digest of those lines:

```text
contentDigest = fnv1a64_lines(contentLinesInCanonicalIdOrder)
```

The U+0000 separator is part of the rule. It must not be replaced by a visible delimiter, JSON encoding, or multiple lines.

The implementation obtains ordering directly from the fingerprint `BTreeMap`; it does not independently sort the constructed content lines by their complete string value.

## 7. Generation identifier

The generation identifier is:

```text
generation = "idx:" + idDigest
```

It identifies the canonical-ID set only. Equal generation strings do not establish equal record content, metadata, storage state, or index consistency.

A complete comparison requires all of:

```text
recordCount
idDigest
contentDigest
```

and the per-ID difference sets produced by verification.

## 8. Verification and status classification

The implementation independently derives generation values for canonical storage and the supplied index snapshot.

It calculates:

```text
missingFromIndex
unexpectedInIndex
ambiguousIds
```

`ambiguousIds` contains IDs present on both sides whose record fingerprints differ.

The result is `consistent` with code `LB_INDEX_CONSISTENT` only when:

- `missingFromIndex` is empty;
- `unexpectedInIndex` is empty;
- `ambiguousIds` is empty;
- record counts match;
- ID digests match;
- content digests match.

When ID sets match but one or more record fingerprints differ, the result is `inconsistent` with code:

```text
LB_INDEX_AMBIGUOUS
```

Other comparison mismatches produce:

```text
LB_INDEX_INCONSISTENT
```

A storage subscription failure produces status `failed`, preserves the storage error code and message, sets both generation objects to null, and returns no snapshot.

## 9. Snapshot handling boundary

For a completed comparison, the result currently includes the compared `IndexSnapshot` even when status is `inconsistent`. Inclusion of that snapshot is diagnostic output and does not authorize publication, checkpoint advancement, serving, or replacement of a previously consistent index.

The lifecycle calculation itself does not manage a durable generation pointer, manifest, checkpoint, rollback record, or atomic activation transaction. A caller that persists or activates snapshots must separately enforce its own fail-closed publication policy.

This document therefore does not claim that inconsistent snapshots are automatically discarded or that consistent snapshots are automatically committed.

## 10. Version handling boundary

The checked-in calculation is a fixed implementation of `lb.index.generation.v1`. The current lifecycle API does not accept an externally supplied digest-rule version and does not emit an `unsupported rule` result for this calculation.

A future implementation that negotiates or reads versioned digest metadata must reject unsupported versions rather than silently applying v1. Such behavior requires an explicit API and is not provided by the current result contract.

## 11. Security properties

FNV-1a 64-bit is not collision resistant. These values must not be used as:

- a protocol-object digest;
- a signature prehash;
- a MAC or authenticity proof;
- an adversarial corruption proof;
- a trusted timestamp;
- a replacement, cleanup, backup, migration, or release authorization proof;
- evidence that canonical storage itself is intact.

Its implemented purpose is deterministic comparison and drift diagnosis inside the index lifecycle.

A future cryptographic digest requires a new rule version and compatibility fixtures. Merely wrapping the current string in a signed document does not change the collision properties of the underlying FNV-1a value.

## 12. Conformance

The shared conformance vector is rooted at:

```text
conformance/index-generation-digest/
```

The one-record vector fixes:

- canonical JSON;
- record fingerprint;
- ID digest;
- content digest;
- generation identifier;
- record count.

Conformance to that vector verifies deterministic calculation for the covered input. It does not verify storage durability, index activation, recovery, authorization, or collision resistance.

## 13. Non-goals

This rule does not define:

- canonical-object identity;
- archive or backup digests;
- index file serialization;
- a generation manifest schema;
- a durable current-generation pointer;
- atomic index activation;
- retention or garbage collection;
- distributed index consensus;
- cryptographic attestation;
- release qualification.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
