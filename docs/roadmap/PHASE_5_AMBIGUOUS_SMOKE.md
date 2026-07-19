# Phase 5: Ambiguous index rejection smoke

The relay-boundary smoke test verifies that contradictory index content is never treated as a successful rebuild or checkpoint update.

- the canonical ID set remains unchanged;
- altered record content returns `LB_INDEX_AMBIGUOUS`;
- the affected canonical ID is reported in `ambiguousIds`;
- checkpoint persistence returns `LB_INDEX_CHECKPOINT_REFUSED`;
- the existing checkpoint bytes remain unchanged.

The production CLI does not expose a test-only ambiguous-state injection command. The smoke therefore exercises the same public storage, index verification, and checkpoint persistence APIs used by the relay boundary.
