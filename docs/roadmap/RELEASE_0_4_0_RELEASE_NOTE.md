# Lingonberry v0.4.0

Lingonberry v0.4.0 completes the verified lifecycle for inactive quarantine replacement generations.

## Highlights

- Deterministic retention evaluation with an explicit retained-generation floor.
- Durable terminal completion evidence bound to replacement journals and generation digests.
- Versioned cleanup plan and proof artifacts with canonical JSON and digest sidecars.
- Read-only state reconstruction that rejects stale pointers, journals, generation metadata, completion evidence, and managed-path inventories.
- Dedicated cleanup transaction journals with deterministic path-level progress.
- Transaction-local sealed inventories and resumable recovery semantics.
- Explicit separation between preparation and terminal processing.
- Fail-closed handling for partial artifacts, tampering, symbolic links, unsupported entry types, ambiguous state, and stale proofs.
- Explicit operator request plus a separate final-action acknowledgement.

## Operational boundary

v0.4.0 does not enable scheduled or unattended cleanup. Active, incomplete, orphan, corrupt, legacy-root, unverified, and insufficiently aged subjects remain ineligible.

Terminal cleanup transaction workspaces are retained. Their journals, digest sidecars, sealed inventories, progress records, and terminal states remain operational evidence in this release.

## Recovery semantics

Before terminal processing begins, the transaction may be rolled back. After that boundary, interruption is represented explicitly as `recovery-required` or `partially-deleted`; the system does not claim rollback is available.

## Compatibility

Existing legacy-root state remains readable and is never selected implicitly. Generation-aware layouts continue to use the active pointer and verified generation metadata. Existing replacement apply, resume, rollback, backup, index, and segment verification behavior remains compatible.

## Upgrade notes

- Review the v0.4.0 release checklist and operations documentation before enabling operator workflows.
- Preserve all transaction workspaces and evidence artifacts during the initial deployment period.
- Treat contradictory journal, pointer, inventory, or digest state as a manual-review condition.

## Deferred

A future separately versioned policy may define retention for terminal cleanup transaction workspaces. That work is not part of v0.4.0 and requires an independent review and authorization boundary.
