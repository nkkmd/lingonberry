# Quarantine Replacement Policy and Semantic-equivalence Contract

**Status: proposed for QL-5C3A** | **Policy target: v2** | **Last updated: 2026-07-13**

This document defines the safety contract that must be satisfied before Lingonberry may implement a mutation-capable quarantine ledger rewrite. It does not authorize rewriting, truncation, retention deletion, or publication of replacement ledgers.

## 1. Normative language

The terms **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are normative.

## 2. Policy identity

```text
lingonberry-quarantine-compaction-policy/v2
```

Policy v2 is intentionally incompatible with policy v1 proof semantics. A v1 proof demonstrates that no line is removable. A v2 proof may describe canonical replacement, but only after every requirement in this document is machine-verified.

## 3. Managed-ledger classification

| Ledger | Classification | Replacement permission |
|---|---|---|
| `quarantine.jsonl` | immutable source evidence | forbidden |
| `quarantine-annotations.jsonl` | immutable reviewer evidence | forbidden |
| `admin-auth-audit.jsonl` | immutable security audit evidence | forbidden |
| `quarantine-resolutions.jsonl` | terminal single-event evidence | canonical replacement only |
| `quarantine-dismissals.jsonl` | terminal single-event evidence | canonical replacement only |
| `quarantine-rejections.jsonl` | terminal single-event evidence | canonical replacement only |

Unknown ledgers MUST be rejected. Absence of an explicit permission means replacement is forbidden.

## 4. Permitted replacement

The initial policy v2 permits only **canonical representation replacement** for terminal single-event ledgers.

A source line MAY be replaced only when:

1. the source line is valid JSON and valid for its ledger type;
2. its `quarantineId` is present and unique within that logical ledger stream;
3. canonical serialization preserves the complete parsed JSON value;
4. no field, array element, numeric value, string value, boolean, or null value is added, removed, or changed;
5. the replacement line is the canonical JSON serialization of the exact parsed source value;
6. the source-to-replacement mapping is one-to-one;
7. logical ledger order is preserved;
8. the source line digest and replacement line digest are recorded in the proof.

Whitespace normalization, object-key ordering, and canonical newline normalization MAY change bytes. Semantic content MUST NOT change.

## 5. Explicitly forbidden transformations

Policy v2 MUST reject plans that perform any of the following:

- remove a valid source event;
- merge multiple source events;
- split one source event into multiple replacement events;
- deduplicate repeated terminal events;
- resolve conflicts by choosing one event;
- change `quarantineId`;
- change terminal disposition or associated metadata;
- reorder logical events;
- rewrite immutable-evidence ledgers;
- mix retention deletion with representation replacement;
- infer missing fields or apply schema defaults;
- discard unknown fields;
- normalize values beyond canonical JSON serialization.

Duplicate terminal events remain corruption and are never replacement candidates.

## 6. Replacement identity

For terminal single-event ledgers, the replacement key is:

```text
<ledger-name> + "\u0000" + <quarantineId>
```

The key MUST be unique within the complete archive-aware logical stream. A duplicate key MUST fail with corruption semantics before a replacement plan is produced.

## 7. Ordering semantics

The logical order is:

```text
verified archive segments in manifest sequence
→ active ledger
```

A replacement plan MUST preserve this order exactly. The nth valid source event MUST map to the nth replacement event for the same managed ledger.

## 8. Provenance requirements

Each replacement entry MUST record:

```text
ledger
logicalOrdinal
replacementKey
sourceSegment or active-ledger marker
sourceLineNumber
sourceLineDigest
replacementLineDigest
sourceValueDigest
replacementValueDigest
transformation
```

`transformation` MUST equal:

```text
canonical-json-representation
```

`sourceValueDigest` and `replacementValueDigest` MUST be computed from canonical serialization of the parsed JSON values and MUST match.

The proof MUST retain enough location information to trace every replacement to one exact source line in the verified backup v2 input.

## 9. Semantic-equivalence contract

A plan is semantically equivalent only when all dimensions below pass.

### 9.1 Record identity

- the managed-ledger set is unchanged;
- every immutable source line is retained byte-for-byte;
- every terminal replacement key appears exactly once before and after;
- no new replacement key appears;
- no existing replacement key disappears.

### 9.2 Terminal state

For each quarantine ID, the terminal state observed from the complete lifecycle MUST be identical before and after replacement.

Conflicting terminal states MUST fail as corruption. The verifier MUST NOT choose a winner.

### 9.3 Status equivalence

The following counts MUST match:

```text
quarantined
promoted
dismissed
permanentlyRejected
annotated
eligibleForPromotion
```

Any future status field introduced before policy application MUST either be included in the contract or cause the policy version to be rejected as insufficient.

### 9.4 Metrics equivalence

All quarantine metrics derived from managed ledger state MUST match exactly, excluding process-local counters that are explicitly documented as non-state-derived.

At minimum:

```text
quarantine record count
promotion count
dismissal count
permanent rejection count
annotation count
eligible record count
terminal conflict/corruption result
```

### 9.5 Eligibility equivalence

For every quarantine record, promotion eligibility and its rejection reason MUST match before and after replacement.

### 9.6 Idempotency equivalence

For each terminal action already represented in state:

- repeating the same action MUST produce the same idempotent result;
- attempting a conflicting terminal action MUST produce the same conflict result;
- batch operation results, including per-record outcome classification, MUST match.

### 9.7 Reader equivalence

All public and admin ordered readers MUST return equivalent parsed values in the same logical order. Byte-for-byte equality is not required for terminal replacement lines; parsed-value equality is required.

### 9.8 Corruption behavior

Inputs rejected as corruption before replacement MUST remain rejected as corruption. Replacement MUST NOT conceal or repair corruption.

## 10. Determinism

Given identical:

- verified backup v2 manifest and files;
- segment manifest and immutable segments;
- policy version;
- canonicalization implementation version;

…the replacement plan, per-entry mapping, semantic-equivalence report, and proof digest MUST be identical, excluding a separately stored non-digest metadata timestamp if one is retained.

Generated timestamps MUST NOT participate in the deterministic proof digest.

## 11. Required preconditions

Before preview or apply:

1. backup v2 verification succeeds;
2. archive segment verification succeeds;
3. the exact managed-ledger set is present;
4. runtime or backup fingerprints match the proof inputs;
5. policy version is supported;
6. canonicalization implementation version is supported;
7. no duplicate terminal replacement key exists;
8. no terminal-state conflict exists.

Apply additionally requires the same-host operation lock and revalidation of all preconditions inside the lock.

## 12. Mandatory rejection conditions

The verifier MUST reject:

- unsupported proof or policy version;
- unsupported canonicalization version;
- unknown or missing managed ledger;
- replacement entry targeting immutable evidence;
- incomplete, duplicate, or non-bijective provenance mapping;
- changed parsed JSON value;
- changed logical order;
- changed status, metrics, eligibility, idempotency, or corruption behavior;
- stale backup, segment manifest, index, or runtime fingerprint;
- retention deletion request;
- plan containing an unrecognized transformation;
- generated plan whose deterministic digest does not match recomputation.

## 13. Proof boundary

QL-5C3A defines this contract only. QL-5C3B may implement read-only planning and proof generation. QL-5C3C may implement mutation only after QL-5C3B proves all requirements above and its format is reviewed.

## 14. Test vectors

The normative examples are stored under:

```text
docs/operations/fixtures/quarantine-replacement-policy/
```

A conforming implementation MUST include equivalent automated tests before policy v2 is accepted by any CLI command.
