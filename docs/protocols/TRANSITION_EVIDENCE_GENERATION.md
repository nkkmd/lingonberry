# Transition Evidence Generation

## Status

Normative pre-release contract for Lingonberry v1.0.0 target-scoped evidence generations.

Rule version: `lb.transition.evidence-generation.v1`

This document defines the protocol-level generation basis and fail-closed semantics. The repository also contains a production Rust implementation for the current relay effective-view path and an executable JavaScript conformance model. Those implementations overlap, but they do not yet cover every evidence kind and failure mode defined here.

## Purpose

A target-scoped re-evaluation worker needs a deterministic identifier for the complete evidence snapshot associated with one target Knowledge Object. The generation is content-derived rather than a relay-local mutable counter.

```text
evidence generation = evidence:sha256:<64 lowercase hex>
```

The same normalized evidence set MUST produce the same generation after restart, rebuild, or ingestion in a different order.

A generation identifies an observed evidence snapshot. It does not itself prove that every entry is semantically usable, authorized, current, or trustworthy.

## Generation basis

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
      "digest": "sha256:<64 lowercase hex>"
    }
  ]
}
```

Each evidence entry contains exactly the generation-relevant fields:

- `kind`;
- protocol `id`;
- evidence `classification`;
- immutable content or carrier `digest`.

Receipt time, ingestion order, queue delivery count, worker identity, storage path, relay-local row identifiers, and other implementation metadata are excluded.

## Evidence kinds and ordering

The initial kind order is:

1. `target`;
2. `transition`;
3. `delegation`;
4. `revocation`.

Within one kind, entries are ordered by ASCII-byte ascending:

1. `id`;
2. `classification`;
3. `digest`.

All participating identifiers and digest strings MUST be valid for their governing protocol rules before generation construction. Implementations MUST NOT use locale-aware ordering.

## Classifications

The initial classifications are:

- `supported`: structurally valid and understood by the implementation;
- `unsupported`: preserved evidence whose required rule or version is not implemented;
- `corrupt`: evidence bytes or integrity metadata fail validation;
- `unreadable`: evidence is known to exist but its stored payload cannot currently be read.

`unsupported`, `corrupt`, and `unreadable` entries MUST NOT be omitted from a complete inventory merely because they are unusable. Their existence is part of the target-scoped observation.

A classified unusable entry uses the SHA-256 digest of immutable stored carrier bytes. For an unreadable payload, the marker MUST use an immutable carrier digest captured and durably stored before or during successful ingestion. A relay that has neither readable bytes nor a trusted stored digest cannot construct a complete generation and MUST report an evidence-inventory error rather than inventing a marker.

A classification marker records the evidence observation. It does not convert unusable bytes into valid protocol evidence.

## Duplicate and conflict handling

An exact duplicate with the same `kind`, `id`, `classification`, and `digest` represents one set member and does not change the generation.

Two entries with the same `kind` and `id` but different classifications or digests are a conflict. An implementation MUST NOT silently select one, merge them, or treat their sort order as resolution. Generation construction fails closed until a normative conflict-resolution rule produces a single resolved evidence entry or an explicit conflict marker defined by a later rule version.

## Snapshot semantic effect

Generation construction and semantic applicability are separate.

If every target-scoped entry is `supported`, the snapshot classification is `complete` and the implementation may continue with authority, supersession, graph, and effective-view evaluation.

If any required entry is `unsupported`, `corrupt`, or `unreadable`:

```text
snapshot classification = incomplete
authority classification = unknown
apply to effective view  = false
```

The incomplete generation and stable diagnostics may be durably recorded. The unusable evidence MUST NOT be interpreted as a valid transition, delegation, or revocation.

When an unusable entry becomes `supported`, or its immutable digest changes through an explicit replacement or repair record, the generation MUST change and trigger target-scoped re-evaluation.

## Last-known-good behavior

An incomplete current observation MUST NOT overwrite a previously committed complete semantic checkpoint.

When a last-known-good effective view exists, an implementation may return that semantic view with freshness `stale`, while separately reporting the current incomplete observation generation and diagnostics. The semantic checkpoint does not advance; the observation checkpoint may advance.

When no last-known-good semantic checkpoint exists, the implementation MUST expose an unresolved or unavailable effective-view state rather than manufacturing a current semantic result.

## Stale-worker guard

A worker MUST recompute or verify the current target evidence generation before committing a derived result. A result calculated for generation `G1` MUST NOT be committed when the current generation is `G2`.

Queue retries, coalescing, worker identity, and delivery count do not alter the protocol generation. They are operational state governed by the re-evaluation queue and coalescing contracts.

## Current repository implementation boundary

The executable JavaScript conformance test `conformance/transition-evidence-generation.test.mjs` implements:

- all four initial evidence kinds;
- deterministic kind and ASCII-byte ordering;
- exact duplicate collapse;
- same-kind-and-ID conflict rejection;
- unusable classifications and fail-closed snapshot effect;
- generation changes after repair;
- last-known-good stale semantics and stable diagnostics.

The current Rust relay path in `packages/relay/src/effective_view.rs` implements a narrower production subset:

- generation basis includes the target and stored transitions currently loaded for that target;
- target and transition entries are emitted as `supported`;
- target digest is computed from canonical target JSON;
- transition digest is computed from the canonical stored publish request;
- stored transitions are loaded in transition-ID byte order;
- the basis is canonicalized and hashed to the `evidence:sha256:` identifier;
- incomplete graph or read diagnostics preserve a last-known-good snapshot when available.

The current Rust path does not, by this function alone, provide the complete protocol inventory model for delegation and revocation evidence, unusable-entry markers, exact duplicate collapse, or same-ID conflict resolution. Operators and integrators MUST NOT infer full `lb.transition.evidence-generation.v1` coverage merely from the presence of the Rust generation function.

The Rust implementation currently invokes the local `openssl` executable to calculate SHA-256. This is an implementation dependency, not part of the protocol output or hash definition.

## Conformance responsibilities

A conforming complete implementation MUST demonstrate that:

1. canonical basis construction is deterministic;
2. input order does not affect the generation;
3. exact duplicates do not affect set identity;
4. conflicting entries fail closed;
5. unusable evidence remains represented when a trusted immutable digest exists;
6. incomplete snapshots do not advance effective-view semantics;
7. repaired or replaced evidence changes the generation;
8. stale workers cannot commit against a superseded generation;
9. implementation metadata is excluded from the protocol basis.

Passing the repository conformance tests demonstrates the tested reference behavior. It does not independently prove durable storage, crash consistency, complete evidence discovery, or correct production integration across every carrier and deployment topology.

## Safety requirements

- Do not use a relay-local counter as the protocol generation.
- Do not include ingestion order or non-evidence timestamps.
- Do not silently discard unusable or conflicting evidence.
- Do not treat a classified marker as semantically valid evidence.
- Do not advance an effective-view result from an incomplete snapshot.
- Do not allow a stale worker to commit a result for a generation different from the current recomputed generation.
- Do not claim complete generation coverage when required evidence kinds are not inventoried by the active runtime path.

## Related contracts

- [CANONICALIZATION.md](./CANONICALIZATION.md)
- [TRANSITION_AUTHORITY.md](./TRANSITION_AUTHORITY.md)
- [TRANSITION_REEVALUATION_QUEUE.md](./TRANSITION_REEVALUATION_QUEUE.md)
- [TRANSITION_REEVALUATION_COALESCING.md](./TRANSITION_REEVALUATION_COALESCING.md)
- [LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md](./LAST_KNOWN_GOOD_EFFECTIVE_VIEW.md)
- [EFFECTIVE_VIEW_DIAGNOSTICS.md](./EFFECTIVE_VIEW_DIAGNOSTICS.md)

## Release boundary

This documentation normalization does not change the fixed v1.0.0 release candidate `f9543019f2c219aea3b085ff90f2da201b268a48`.

Formal 72-hour soak, privileged reference-host qualification, version update, release PR, tag, and GitHub Release remain separate release gates.