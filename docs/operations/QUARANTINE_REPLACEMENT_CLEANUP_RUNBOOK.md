# Quarantine Replacement Cleanup Runbook

**Applies to:** Lingonberry v0.4.0

## Operating model

Cleanup is exact-subject, proof-bound, operator-triggered, and double opt-in. No scheduled or unattended cleanup is supported. Terminal cleanup transaction workspaces remain retained in v0.4.0.

## Preflight

- Verify replacement completion evidence.
- Produce a retention report and select exact eligible generation IDs.
- Build and verify the cleanup plan/proof.
- Revalidate the plan/proof against current state immediately before preparation.
- Confirm the active generation is not selected.
- Confirm no concurrent replacement or cleanup operation is active.

Any mismatch is a stop condition. Do not attempt automatic repair.

## Preparation

Preparation moves the exact approved managed-path set into a transaction-local tomb area and seals a canonical inventory. At the end of preparation the journal must be `tomb-sealed`, the inventory digest must verify, and rollback must remain available.

## Final processing

Final processing requires a separate explicit acknowledgement. Managed paths are processed in deterministic `generation_id/managed_path` order, and progress is recorded durably after each path.

Expected terminal state: `committed`.

## Rollback

Rollback is allowed only before final processing starts. Expected terminal state: `rolled-back`.

After final processing starts, rollback must not be advertised. Interruption is represented as `recovery-required` or `partially-deleted`.

## Recovery classifications

- `prepared` / `revalidated`: inspect and resume preparation.
- `renaming-to-tomb`: verify source and tomb state; do not infer success from absence.
- `tomb-sealed`: inspect, roll back, or provide the final acknowledgement.
- `deleting`: resume from the durable progress frontier.
- `recovery-required`: verify journal and inventory before resuming.
- `partially-deleted`: preserve evidence and escalate for manual recovery.
- `committed` / `rolled-back`: status-only; retain the workspace.

## Evidence preservation

Retain the journal and digest, plan/proof and digests, sealed inventory and digest, path-level progress, terminal state, and append-only audit records. v0.4.0 provides no automatic terminal-workspace retention mechanism.

## Observability

Metrics may use only bounded labels such as state, operation, phase, outcome, and stable error family. Paths, transaction IDs, generation IDs, digests, record IDs, and free-form errors are prohibited as metric labels.

## Incident handling

1. Stop further operator invocations.
2. Preserve the transaction workspace byte-for-byte.
3. Record the bounded error family and journal state.
4. Verify journal and inventory digest pairs.
5. Compare actual entries with the sealed inventory without following symbolic links.
6. Escalate contradictory state for manual review.
