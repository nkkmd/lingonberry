# Protocol identifier grammar

**Rule version:** `lb.protocol.id.ascii.v1`  
**Status:** draft for v0.6.0

## Purpose

Protocol identifiers are opaque machine identifiers, not display text. This rule fixes a cross-language ASCII grammar so validation and lexical ordering do not depend on Unicode normalization, locale, or runtime-specific string comparison.

## Grammar

Protocol-generated canonical identifiers MUST use one of the registered prefixes and a non-empty ASCII-safe suffix.

```text
object ID:     lb:obj:<suffix>
transition ID: lb:transition:<suffix>
identity key:  lb:key:<suffix>

suffix character set:
A-Z a-z 0-9 . _ ~ : -
```

Equivalent regular expressions are:

```text
^lb:obj:[A-Za-z0-9._~:-]+$
^lb:transition:[A-Za-z0-9._~:-]+$
^lb:key:[A-Za-z0-9._~:-]+$
```

Whitespace, control characters, percent escapes, path separators, query delimiters, fragment delimiters, and non-ASCII Unicode code points are not valid protocol-ID characters.

## Comparison and ordering

Protocol IDs are compared as their literal ASCII bytes. For `supersedesTransitionIds`, producers and consumers sort IDs by ascending unsigned ASCII byte value before deriving `lb.transition.identity.v1`.

Because every permitted character is a single ASCII byte, ASCII byte ordering and Unicode code-point ordering produce the same result for valid IDs. Locale-aware collation MUST NOT be used.

## Preservation

Consumers MUST preserve the exact valid ID spelling. They MUST NOT lowercase, uppercase, Unicode-normalize, percent-decode, trim, or otherwise rewrite an ID.

IDs are case-sensitive. `lb:obj:Example` and `lb:obj:example` are distinct identifiers.

## Compatibility

Existing IDs that already satisfy this grammar remain valid. A pre-v0.6 record containing a non-ASCII protocol ID may be retained as legacy evidence, but it MUST NOT be emitted as a conforming v0.6 protocol ID or used as a newly generated transition parent.

Legacy retention does not imply acceptance into a v0.6 effective view. Implementations MUST report the unsupported identifier grammar explicitly rather than silently rewriting the identifier.

## Conformance

The conformance corpus includes:

- valid object, transition, and identity-key examples using the full safe character classes;
- rejection of Japanese and full-width Unicode characters;
- transition parent-set sorting over valid ASCII IDs.
