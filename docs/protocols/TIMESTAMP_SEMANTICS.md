# Timestamp Semantics

**Status: draft for v0.6.0** | **Rule version: `lb.timestamp.rfc3339.utc.v1`** | **Last updated: 2026-07-20**

## 1. Scope

This rule applies to protocol timestamps including `createdAt`, provenance `observedAt`, and identity claim `issuedAt` when present.

## 2. Producer requirement

A conforming producer MUST emit an RFC 3339 timestamp in UTC using an uppercase `T` separator and uppercase `Z` suffix.

The accepted producer form is:

```text
YYYY-MM-DDTHH:MM:SS[.fraction]Z
```

Examples:

```text
2026-07-20T00:00:00Z
2026-07-20T00:00:00.123Z
```

A producer MUST NOT emit a local time without an offset. A producer SHOULD use only the precision needed to represent the source timestamp.

## 3. Consumer requirement

A consumer MUST reject a required timestamp that is not a syntactically valid RFC 3339 date-time.

For the v0.6 protocol contract, a consumer MAY accept an equivalent explicit offset form for legacy input only when the applicable compatibility policy declares it. Such acceptance MUST NOT silently rewrite the timestamp before identity, digest, or signature verification.

## 4. Canonicalization

`lb.canonical.json.v1` treats a timestamp as an ordinary JSON string. It does not:

- convert offsets to UTC;
- add or remove fractional seconds;
- change letter case;
- alter precision;
- validate calendar correctness.

Therefore, textually different timestamp representations produce different canonical bytes even when they denote the same instant.

Any timestamp normalization rule must run before canonicalization and requires its own version.

## 5. Identity and signatures

When a selected identity or signature rule covers a timestamp field, the exact timestamp string is covered.

Implementations MUST NOT parse and reformat the timestamp before deriving identity keys, digest targets, or signature targets unless the selected versioned rule explicitly requires normalization.

## 6. Conformance

Timestamp fixtures distinguish:

- valid producer UTC form;
- invalid missing-zone form;
- preservation of the exact timestamp string in canonical bytes.
