# Transition Evidence Generation

## Status

Normative foundation for v0.6.0 target-scoped re-evaluation.

Rule version: `lb.transition.evidence-generation.v1`

## Purpose

A re-evaluation worker needs a deterministic generation identifier for the complete evidence snapshot associated with one target Knowledge Object. The generation is content-derived rather than a relay-local mutable counter.

```text
evidence generation = evidence:sha256:<64 lowercase hex>
```

The same evidence set MUST produce the same generation after restart, rebuild, or ingestion in a different order.

## Basis

The SHA-256 input is the UTF-8 encoding of `lb.canonical.json.v1` applied to:

```json
{
  "ruleVersion": "lb.transition.evidence-generation.v1",
  "targetId": "lb:obj:target-1",
  "evidence": [
    {
      "kind": "target",
      "id": "lb:obj:target-1",
      "classification": "supported",
      "digest": "sha256:<record digest>"
    }
  ]
}
```

Each evidence entry contains only:

- `kind`
- protocol `id`
- evidence `classification`
- immutable content or carrier `digest`

Receipt time, ingestion order, queue delivery count, worker identity, and relay-local row identifiers are excluded.

## Evidence kinds and ordering

The initial kind order is:

1. `target`
2. `transition`
3. `delegation`
4. `revocation`

Within one kind, entries are ordered by ASCII byte ascending `id`, then `classification`, then `digest`.

## Classifications

The initial classifications are:

- `supported`: structurally valid and understood by the implementation
- `unsupported`: preserved evidence whose required rule or version is not implemented
- `corrupt`: evidence bytes or integrity metadata fail validation
- `unreadable`: evidence is known to exist but its stored payload cannot currently be read

`unsupported`, `corrupt`, and `unreadable` entries MUST NOT be omitted from the generation basis. Their existence is part of the target snapshot.

A classified unusable entry uses the SHA-256 digest of the immutable stored carrier bytes. For an unreadable payload, the marker MUST use an immutable carrier digest captured and durably stored before or during successful ingestion. A relay that has neither readable bytes nor a trusted stored digest cannot construct a complete generation and MUST report an evidence-inventory error rather than inventing a marker.

## Semantic effect

Generation construction and semantic applicability are separate.

If every target-scoped entry is `supported`, the snapshot classification is `complete` and normal authority and graph evaluation may proceed.

If any entry is `unsupported`, `corrupt`, or `unreadable`:

```text
snapshot classification = incomplete
authority classification = unknown
apply to effective view  = false
```

The incomplete generation and its diagnostic classifications may be durably recorded. The unusable evidence is not interpreted as a valid transition, delegation, or revocation.

When evidence changes from an unusable classification to `supported`, or its immutable digest changes through an explicit replacement or repair record, the generation MUST change and trigger target-scoped re-evaluation.

## Duplicate handling

An exact duplicate entry with the same `kind`, `id`, `classification`, and `digest` does not change the evidence set or generation.

Two entries with the same `kind` and `id` but conflicting classification or digest are not silently selected, merged, or ordered into a valid generation. Generation construction fails closed until the evidence conflict is represented by a normative resolution rule.

## Safety requirements

- Do not use a relay-local counter as the protocol generation.
- Do not include ingestion order or timestamps that are not evidence content.
- Do not silently discard unusable or conflicting evidence.
- Do not treat a classified marker as semantically valid evidence.
- Do not advance an effective-view result from an incomplete snapshot.
- Do not allow a stale worker to commit a result for a generation different from the current recomputed generation.
