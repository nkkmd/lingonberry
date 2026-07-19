# Phase 5: Index recovery smoke

The real-binary recovery scenario uses one durable state directory and verifies:

- a published object is present in canonical storage;
- `catch-up-index` creates a missing checkpoint and returns `rebuilt` / `LB_INDEX_REBUILT`;
- a second run returns `up-to-date` / `LB_INDEX_UP_TO_DATE` without rewriting the checkpoint;
- a corrupt checkpoint returns `failed` / `LB_INDEX_CHECKPOINT_CORRUPT`;
- corrupt checkpoint bytes are not overwritten automatically.

Canonical storage remains the source of truth. Catch-up is allowed for missing or stale checkpoints, while corruption and unsupported checkpoint versions remain fail-closed operator-visible states.
