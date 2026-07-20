# Last-Known-Good Effective View

## Status

Normative foundation for v0.6.0 effective-view continuity.

Rule version: `lb.transition.effective-view.last-known-good.v1`

## Principle

When a complete evidence generation has produced a valid semantic result and a later current generation is incomplete, the relay preserves the last-known-good semantic result while marking it stale.

An incomplete generation MUST NOT create a new semantic effect and MUST NOT erase a previously established authorized effect merely because currently observed evidence is unsupported, corrupt, or unreadable.

## Separate checkpoints

A relay maintains two logically separate checkpoints:

- `semanticCheckpoint`: the most recent complete generation whose authority and graph projection were durably evaluated;
- `observationCheckpoint`: the most recent current evidence generation, including complete or incomplete classification and diagnostics.

The observation checkpoint may advance while the semantic checkpoint remains unchanged.

## Incomplete current generation

If current generation `g2` is incomplete and the last complete semantic generation is `g1`:

```json
{
  "effectiveView": {
    "generation": "g1",
    "freshness": "stale",
    "classification": "replaced"
  },
  "evidenceObservation": {
    "generation": "g2",
    "snapshotClassification": "incomplete",
    "applyToEffectiveView": false
  }
}
```

The relay MUST expose enough diagnostics to identify which evidence markers caused incompleteness without interpreting those markers as valid authority or transition evidence.

## Recovery

When a later generation becomes complete:

1. evaluate authority and the full transition graph for that generation;
2. verify that the generation is still current;
3. durably commit the new semantic result;
4. advance both semantic and observation checkpoints;
5. mark the effective view `current`.

A complete result may retain, replace, withdraw, or restore the original target according to the normal graph rules. Recovery is not required to preserve the previous semantic result if the newly complete generation validly produces a different result.

## No prior semantic checkpoint

If the first observed generation is incomplete, there is no last-known-good derived result to preserve. The original target remains visible and the derived effective view is reported as `unresolved`, not as an authorized replacement or withdrawal.

## Safety requirements

- Do not advance the semantic checkpoint from an incomplete generation.
- Do not describe a stale effective view as current.
- Do not erase the observation that newer incomplete evidence exists.
- Do not allow incomplete evidence to create, replace, withdraw, or supersede semantic effects.
- Do not roll back canonical Knowledge Objects or Transition Objects when derived state becomes incomplete.
