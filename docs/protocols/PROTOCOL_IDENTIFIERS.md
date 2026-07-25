# Protocol Identifier Contract

**Status:** normative for the v1.0.0 pre-release implementation  
**Rule version:** `lb.protocol.id.ascii.v1`

## 1. Purpose

Protocol identifiers are case-sensitive machine identifiers. They are not display text, file paths, URLs, authorization tokens, signatures, or collision-resistant digests.

This document distinguishes the complete identifier grammar defined by the checked-in JSON Schemas from the weaker validation performed by some runtime paths.

## 2. Registered identifier classes

The registered protocol prefixes are:

```text
Knowledge Object ID:  lb:obj:<suffix>
Transition Object ID: lb:transition:<suffix>
identity key:         lb:key:<suffix>
```

For identifiers governed by `lb.protocol.id.ascii.v1`, `<suffix>` MUST be non-empty and contain only:

```text
A-Z a-z 0-9 . _ ~ : -
```

Equivalent complete regular expressions are:

```text
^lb:obj:[A-Za-z0-9._~:-]+$
^lb:transition:[A-Za-z0-9._~:-]+$
^lb:key:[A-Za-z0-9._~:-]+$
```

Whitespace, control characters, percent escapes, `/`, `\\`, `?`, `#`, and non-ASCII Unicode code points are outside this rule.

## 3. Length limits

Limits include the prefix and are measured over the literal UTF-8 representation.

```text
Knowledge Object ID:  maximum 255 bytes
Transition Object ID: maximum 255 bytes
identity key:         maximum 512 bytes
```

Because conforming identifiers are ASCII-only, character count and UTF-8 byte count are equal for values that already satisfy the grammar.

A validating path MUST reject an over-limit identifier. It MUST NOT truncate, case-fold, hash, percent-decode, Unicode-normalize, or otherwise rewrite the supplied identifier into a different valid identifier.

## 4. Checked-in schema enforcement

The checked-in Knowledge Object schema enforces:

- `id`: `^lb:obj:[A-Za-z0-9._~:-]+$`, maximum 255 characters;
- identity-claim `identityKey`: `^lb:key:[A-Za-z0-9._~:-]+$`, maximum 512 characters;
- identity-claim `canonicalId`: the Knowledge Object grammar, maximum 255 characters.

The checked-in Transition Object schema enforces:

- `id`: the Transition Object grammar, maximum 255 characters;
- `targetId` and `replacementId`: the Knowledge Object grammar, maximum 255 characters;
- every `supersedesTransitionIds` entry: the Transition Object grammar, maximum 255 characters;
- non-empty and unique `supersedesTransitionIds` when the field is present.

Schema acceptance establishes identifier shape only. It does not establish uniqueness, authenticity, ownership, authority, existence, retrievability, or collision resistance.

## 5. Runtime enforcement boundaries

### 5.1 Transition HTTP path

The checked-in transition ingestion implementation enforces the ASCII suffix grammar and 255-byte limit for:

- the transition `id`;
- `targetId`;
- `replacementId` for a `replace` transition;
- every `supersedesTransitionIds` entry.

It also rejects:

- an empty parent array;
- duplicate parent identifiers;
- self-supersession.

The transition path therefore implements the principal Transition Object identifier constraints described by this document.

### 5.2 Rust Knowledge Object validator

The checked-in Rust `is_lb_object_id` helper currently verifies only that a value:

- starts with `lb:obj:`; and
- contains no whitespace.

That helper does **not** by itself enforce the complete ASCII suffix allowlist or the 255-byte limit. Consequently, a runtime path that invokes only this helper MUST NOT be described as enforcing the complete `lb.protocol.id.ascii.v1` Knowledge Object grammar.

A path requiring the complete contract must invoke equivalent schema-strength validation before canonical storage, indexing, graph insertion, or effective-view evaluation.

This documentation change does not add missing runtime enforcement.

## 6. Identity-key formats

The generic Knowledge Object finalizer currently emits a v1 identity key in this fixed form:

```text
lb:key:lb.identity.key.v1:fnv1a64:<16-lowercase-hex>
```

The repository also implements `lb.identity.key.v2`, whose digest component is derived with SHA-256. Generic finalization does not currently select v2 by default.

The `lb:key:` prefix identifies an identity-key namespace. It does not make an arbitrary suffix cryptographically secure. In particular, FNV-1a values MUST NOT be used as signatures, authentication proofs, authorization tokens, or adversarial collision-resistant digests.

Unknown identity-rule versions MUST be reported as unsupported by version-aware verification. They MUST NOT be silently interpreted using v1 or v2.

## 7. Comparison, ordering, and preservation

Identifiers are compared literally and are case-sensitive.

```text
lb:obj:Example != lb:obj:example
```

Conforming ASCII identifiers are ordered by ascending unsigned byte value where a rule explicitly requires lexical ordering. Locale-aware collation MUST NOT be used.

For `lb.transition.identity.v1`, a valid copy of `supersedesTransitionIds` is sorted before identity derivation. This rule does not authorize mutation or reordering of the stored Transition Object array.

Consumers MUST preserve the exact accepted spelling. They MUST NOT:

- lowercase or uppercase it;
- trim it;
- Unicode-normalize it;
- percent-decode it;
- interpret it as a path or URL;
- infer authority or semantic meaning from undocumented suffix components.

## 8. Storage and security boundaries

An identifier may be used as a lookup key only after the receiving component applies the validation required by that path.

Implementations MUST NOT concatenate an unvalidated identifier directly into a filesystem path. A node-local path key, filename digest, cursor prefix, or FNV-derived storage key is an implementation detail and MUST NOT be exposed as the protocol identifier itself.

Identifier equality does not prove content equality. Duplicate and immutable-content conflict handling must compare the canonical content required by the relevant storage or API contract.

## 9. Legacy evidence

A node may retain historical evidence containing a value that does not satisfy the current grammar. Retention does not make the value conforming and does not authorize its use as:

- a newly generated object or transition identifier;
- a new transition target or parent;
- a new identity claim;
- a newly accepted canonical object.

A path that encounters unsupported legacy identifier syntax must fail closed for any new semantic effect. It must not silently rewrite the identifier.

## 10. Conformance expectations

Identifier conformance should cover at least:

- valid examples for all three registered prefixes;
- rejection of empty suffixes;
- rejection of whitespace, controls, path separators, query and fragment delimiters, percent escapes, and non-ASCII Unicode;
- exact 255/256-byte boundaries for object and transition identifiers;
- exact 512/513-byte boundaries for identity keys;
- case-sensitive comparison;
- deterministic parent-set ordering for transition identity derivation;
- a negative test demonstrating that the weak Rust Knowledge Object helper is not equivalent to schema-strength validation.

## 11. Release boundary

This contract describes the checked-in v1.0.0 pre-release implementation and its known enforcement differences. It does not redefine the fixed release candidate, complete the formal 72-hour soak, complete privileged reference-host qualification, update the release version, create a release tag, or publish a GitHub Release.
