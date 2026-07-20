# Index Generation Digest Rule

**Status: draft for v0.6.0** | **Rule version: `lb.index.generation.v1`** | **Last updated: 2026-07-20**

## 1. Purpose

This document fixes the deterministic digest and generation rules used to compare an index snapshot with canonical storage.

This rule describes derived index state. It is not a protocol object digest, signature digest, cryptographic integrity proof, or storage format version.

## 2. Digest primitive

`lb.index.generation.v1` uses FNV-1a 64-bit with:

| Parameter | Value |
|---|---|
| Offset basis | `0xcbf29ce484222325` |
| Prime | `0x100000001b3` |
| Arithmetic | unsigned 64-bit wrapping multiplication |
| Input encoding | UTF-8 |
| Line delimiter | one byte `0x0a` appended after every input line |
| Output | `fnv1a64:` followed by 16 lowercase hexadecimal digits |

The digest function starts from the offset basis. For each input line, it processes every UTF-8 byte and then one newline byte. For each byte it XORs the current state with the byte and multiplies by the prime using wrapping 64-bit arithmetic.

The line delimiter is part of the contract. Implementations MUST also append it after the final line.

## 3. Record fingerprint

A stored catalog record fingerprint is calculated over exactly three lines in this order:

1. `carrierIdentity`
2. `storedAt`
3. canonical JSON of the stored object using `lb.canonical.json.v1`

```text
recordFingerprint = fnv1a64_lines([
  carrierIdentity,
  storedAt,
  canonicalJson(object)
])
```

Changing the carrier identity or stored timestamp changes this derived-state fingerprint even when the canonical object is unchanged.

## 4. ID digest

For a snapshot, canonical IDs are sorted in ascending byte-independent string order as implemented by the protocol's deterministic ordered map/set representation.

The ID digest is the line digest of the sorted canonical IDs:

```text
idDigest = fnv1a64_lines(sortedCanonicalIds)
```

## 5. Content digest

For every record in canonical-ID order, construct one line:

```text
canonicalId + U+0000 + recordFingerprint
```

The content digest is the line digest of those constructed lines:

```text
contentDigest = fnv1a64_lines(sortedContentLines)
```

The U+0000 separator is part of the rule and MUST NOT be replaced with a visible delimiter.

## 6. Generation identifier

The generation identifier is:

```text
generation = "idx:" + idDigest
```

It intentionally identifies the canonical ID set. Consumers MUST compare `recordCount`, `idDigest`, and `contentDigest` when verifying full snapshot consistency.

Equal generation identifiers alone do not prove equal record content.

## 7. Security properties

FNV-1a 64-bit is not collision resistant and MUST NOT be used as:

- a protocol object digest;
- a signature prehash;
- an authenticity proof;
- an adversarial corruption proof;
- a replacement, cleanup, backup, or migration authorization proof.

Its purpose is deterministic operational comparison and drift detection inside the index lifecycle. A future cryptographic rule requires a new version.

## 8. Failure behavior

An implementation MUST classify snapshots with equal ID sets but unequal record fingerprints or content digests as ambiguous or inconsistent. It MUST NOT overwrite the canonical source of truth or update a consistent checkpoint from an inconsistent result.

Unknown digest rule versions MUST be treated as unsupported.

## 9. Conformance vector

The initial vector is rooted at:

```text
conformance/index-generation-digest/
```

It fixes canonical JSON, record fingerprint, ID digest, content digest, generation identifier, and record count for a one-record snapshot.
