# Timestamp Semantics

**Status:** normative for the v1.0.0 pre-release implementation  
**Timestamp rule:** `lb.timestamp.rfc3339.utc.v1`  
**Protocol version:** `0.1.0`

## 1. Scope

This document defines the checked-in timestamp contract for protocol objects and distinguishes producer requirements from the wider input language accepted by the current Rust validator.

The timestamp-bearing protocol fields covered here are:

- Knowledge Object `createdAt`;
- provenance source `observedAt`, when present;
- identity claim `issuedAt`;
- Transition Object `issuedAt`; and
- any later field that explicitly declares this timestamp rule.

Operational log times, filesystem modification times, scheduler times, HTTP `Date` headers, database timestamps, and release-gate start or end times are outside this protocol rule unless another checked-in contract explicitly adopts it.

## 2. Version boundary

`lb.timestamp.rfc3339.utc.v1` is an independently versioned timestamp-production rule. It is not the Lingonberry product version, protocol version, or object schema version.

Publishing product version `1.0.0` MUST NOT silently change the accepted or produced timestamp language for this rule. A change to timezone requirements, calendar validation, fractional precision, leap-second treatment, comparison, or normalization requires a separately reviewed rule revision.

## 3. Producer contract

A producer claiming conformance to `lb.timestamp.rfc3339.utc.v1` MUST emit UTC using:

```text
YYYY-MM-DDTHH:MM:SS[.fraction]Z
```

The separator MUST be uppercase `T`, and the UTC designator MUST be uppercase `Z`.

Examples:

```text
2026-07-20T00:00:00Z
2026-07-20T00:00:00.123Z
```

A conforming producer:

- MUST include a timezone designator;
- MUST NOT emit a local time without an offset;
- SHOULD emit only the fractional precision needed to represent its source value; and
- MUST NOT claim that an explicit non-zero offset is the canonical producer form for this rule.

The producer rule does not define a default timestamp. An implementation MUST NOT invent `createdAt`, `observedAt`, or `issuedAt` merely because a field is missing.

## 4. Checked-in schema requirements

The checked-in Draft 7 schemas declare the relevant fields as strings with `format: date-time`:

- [`schemas/knowledge-object.schema.json`](../../schemas/knowledge-object.schema.json);
- [`schemas/transition-object.schema.json`](../../schemas/transition-object.schema.json).

The schemas require:

- Knowledge Object `createdAt`;
- identity claim `issuedAt` when an identity claim exists;
- Transition Object `issuedAt`;
- provenance `observedAt` only when supplied.

A JSON Schema engine may treat `format` as assertion, annotation, or configurable validation. Therefore, schema declaration alone MUST NOT be described as proving that the checked-in Rust timestamp validator ran.

## 5. Checked-in Rust acceptance language

The Rust protocol validator uses its internal `is_rfc3339_datetime` helper for Knowledge Object `createdAt`, provenance `observedAt`, and identity claim `issuedAt`. Transition validation paths must be assessed against their own checked-in implementation and schema use; this document does not infer runtime validation merely from the Transition Object schema.

The current Rust helper accepts:

- uppercase `T` between date and time;
- uppercase `Z`, a `+HH:MM` offset, or a `-HH:MM` offset;
- optional fractional seconds containing one or more ASCII digits;
- seconds from `00` through `60`;
- offset hours from `00` through `23`; and
- offset minutes from `00` through `59`.

The helper rejects timestamps without a zone and rejects lowercase `t` or `z`.

The helper performs only limited calendar validation. It checks month `01` through `12` and day `01` through `31`, but it does not validate month-specific day counts or leap years. For example, its implementation shape does not establish that every accepted date exists in the Gregorian calendar.

The helper is therefore an RFC-3339-style implementation check, not a complete claim of strict RFC 3339 or civil-calendar conformance. Documentation and APIs MUST NOT describe it more strongly.

## 6. Producer versus consumer distinction

The producer rule is intentionally narrower than the current Rust acceptance language:

- conforming production uses UTC `Z` form;
- the Rust validator also accepts explicit numeric offsets.

Acceptance of an explicit offset does not make that spelling the canonical producer form. A consumer MAY preserve and process an accepted explicit-offset value, but it MUST NOT silently relabel it as having been produced under the UTC-only rule.

Compatibility acceptance MUST occur before any operation that requires a valid object. It MUST NOT rewrite the accepted spelling before canonicalization, identity derivation, digest construction, or signature verification unless a separately versioned rule explicitly requires that rewrite.

## 7. Canonicalization and preservation

`lb.canonical.json.v1` treats a timestamp as an ordinary JSON string. It does not:

- convert an offset to UTC;
- add or remove fractional seconds;
- change letter case;
- alter precision;
- validate a civil-calendar date;
- resolve leap seconds; or
- compare instants.

The checked-in normalization and finalization paths preserve an accepted timestamp string. Consequently, textually different representations produce different canonical bytes even when an external date-time library would interpret them as the same instant.

Any timestamp-normalization procedure must run under its own versioned rule. Changing existing timestamp text after signature or digest creation invalidates any byte-based result that covered the original text.

## 8. Identity, signatures, and digests

When an identity, signature, or digest rule covers a timestamp-bearing object or field, it covers the exact JSON string present in the applicable canonical input.

Implementations MUST NOT parse and reformat a timestamp before deriving identity keys, signature targets, digest targets, transition identifiers, or evidence hashes unless the selected versioned rule explicitly requires normalization.

Timestamp syntax does not establish:

- trusted clock origin;
- authority of the producer;
- ordering between distributed writers;
- freshness;
- absence of clock skew;
- causality; or
- truth of an asserted observation or issuance time.

Those properties require separate authority, signature, transition, storage, or operational evidence.

## 9. Ordering and equality

The protocol does not define lexical timestamp ordering as a universal substitute for instant ordering.

For timestamps using the same UTC producer form and compatible precision, lexical comparison may be useful to an implementation, but this document does not make it a general semantic guarantee. Explicit offsets, different fractional precision, leap-second spelling, invalid-but-validator-accepted calendar dates, and clock skew prevent such a claim.

Unless another checked-in rule states otherwise:

- timestamp string equality means exact string equality;
- canonical-byte equality includes exact timestamp spelling; and
- semantic conflict or transition authority MUST NOT be inferred solely from timestamp order.

## 10. Failure and reporting boundaries

A path that invokes the checked-in Rust validator MUST reject a required timestamp that fails the helper described above. A path using only generic schema validation MUST report only the guarantees of that schema engine and configuration.

A path MUST NOT report a timestamp as cryptographically authenticated merely because it is syntactically valid. A path MUST NOT report an event as fresh, authoritative, or causally later merely because its timestamp compares later.

## 11. Conformance evidence

Applicable tests and fixtures should distinguish at least:

- valid UTC producer form;
- accepted explicit-offset consumer input;
- missing-zone rejection;
- lowercase separator or zone rejection;
- preservation of the exact timestamp string in canonical bytes;
- fractional-second preservation;
- the current limited calendar-validation boundary; and
- coverage of each protocol field that invokes the timestamp validator.

A fixture demonstrates the expected result for that fixture. It does not broaden the accepted language beyond checked-in schema and runtime behavior.

## 12. Release boundary

The fixed v1.0.0 release candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. This documentation change does not redefine that candidate.

Normal CI, documentation inventory, bilingual checks, documentation freeze checks, and documentation walkthrough evidence do not constitute privileged reference-host qualification or the formal 72-hour soak. Product version update, release PR, annotated tag, and GitHub Release remain gated separately.