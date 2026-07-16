# Quarantine Replacement v0.4.0 Smoke Test

## Purpose

Validate upgrade compatibility, verified retention evaluation, preview/proof stability, recovery classification, and evidence preservation using disposable fixtures only.

## Required fixtures

- v0.3.0 legacy-root layout;
- generation layout with one active generation and multiple previous terminal generations;
- stale-state fixture;
- altered-inventory fixture;
- interrupted-transaction fixture.

## Validation sequence

1. Run all Rust and JavaScript tests.
2. Verify the v0.3.0 legacy-root fixture remains readable.
3. Verify the generation-layout active pointer and generation metadata.
4. Verify terminal completion evidence for candidate generations.
5. Evaluate retention policy and confirm the active generation and retention floor are excluded.
6. Build preview/proof artifacts twice and confirm canonical byte stability.
7. Revalidate the proof against unchanged state.
8. Change one bound state artifact and confirm stale-state rejection.
9. Verify transaction preparation produces a sealed inventory.
10. Verify a reversible fixture returns to its original state and retains evidence.
11. Verify a terminal fixture records deterministic path-level progress and retains evidence.
12. Change an inventory artifact in a separate fixture and confirm fail-closed behavior.
13. Verify interrupted fixtures classify as `recovery-required` or `partially-deleted` as defined by the journal frontier.
14. Confirm no scheduled or unattended cleanup entry point is enabled.

## Bounded output record

Record only version, state, classification, operation, outcome, phase, stable error family, and counts. Do not use secrets, paths, identifiers, digests, record IDs, or free-form errors as metric labels.

## Release evidence

Archive the main commit SHA, CI run identifiers, checklist revision, release-note revision, failure inventory revision, crash matrix revision, and confirmation that terminal workspaces remain retained.
