# Transition Evidence Generation

## Status

Normative foundation for v0.6.0 target-scoped re-evaluation.

Rule version: `lb.transition.evidence-generation.v1`

## Purpose

A re-evaluation worker needs a deterministic generation identifier for the complete evidence snapshot associated with one target Knowledge Object. The generation is content-derived rather than a relay-local mutable counter.

```text
evidence generation = evidence:sha256:<64 lowercase hex>
```

The same supported evidence set MUST produce the same generation after restart, rebuild, or ingestion in a different order.

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
- immutable content `digest`

Receipt time, ingestion order, queue delivery count, worker identity, and relay-local row identifiers are excluded.

## Supported evidence kinds

The initial kind order is:

1. `target`
2. `transition`
3. `delegation`
4. `revocation`

Within one kind, entries are ordered by ASCII byte ascending `id`, then `classification`, then `digest`.

## Duplicate handling

An exact duplicate entry with the same `kind`, `id`, `classification`, and `digest` does not change the evidence set or generation.

Two entries with the same `kind` and `id` but conflicting classification or digest are not silently selected, merged, or ordered into a valid generation. Generation construction fails closed until the evidence conflict is represented by a normative classification rule.

## Current conformance boundary

The v0.6 foundation vector currently fixes generation for evidence classified as `supported`.

The treatment of `unsupported`, `corrupt`, and unreadable evidence is intentionally not inferred. A relay MUST NOT omit such evidence and claim that the resulting supported-only digest represents a complete current snapshot unless the protocol contract explicitly defines that behavior.

## Safety requirements

- Do not use a relay-local counter as the protocol generation.
- Do not include ingestion order or timestamps that are not evidence content.
- Do not silently discard conflicting evidence IDs.
- Do not advance a derived checkpoint if generation construction is incomplete or ambiguous.
- Do not allow a stale worker to commit a result for a generation different from the current recomputed generation.
