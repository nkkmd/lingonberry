# Storage migration and upgrade contract

**Status: v0.7.0 implementation contract** | **Last updated: 2026-07-21**

## 1. Purpose

This document defines the single-node storage migration boundary for Lingonberry v0.7.0. It supplements the release roadmap and does not replace backup, restore, replacement, cleanup, or recovery contracts.

The migration system must preserve existing durable evidence while allowing a supported legacy data directory to move to the current storage format. Unknown, corrupt, unreadable, contradictory, or changed-after-plan state fails closed.

## 2. Storage format manifest

The data directory contains `storage-format.manifest` after initialization or successful migration.

The v1 manifest records:

- `format_version`
- `layout_id`
- `created_by`
- optional `source_format_version`

The current contract is:

```text
format_version: 1
layout_id: single-node-canonical-v1
```

The manifest is written atomically and durably. A manifest is not evidence that migration completed unless the durable migration journal also reaches `committed` after verification.

## 3. Inspection classification

Read-only inspection produces exactly one classification:

- `empty`: no durable entries and no manifest
- `legacy_unversioned`: durable entries exist but no manifest exists
- `supported`: manifest version and layout are supported by this binary
- `unknown_newer`: manifest format version is newer than this binary supports
- `corrupt`: malformed, contradictory, unsupported, or unreadable state

Inspection inventories regular files and directories deterministically. Symlinks and unsupported filesystem entry types are rejected. Manifest and migration-journal files are excluded from the source inventory digest so journal progress does not invalidate the inspected source binding.

## 4. Deterministic migration plan

A migration plan is bound to:

- the inspected source inventory digest
- the source format classification
- the target storage format version

The resulting `plan_id` is deterministic. Before mutation, the implementation must re-check that the current source inventory still matches the journal's bound source digest.

A non-empty legacy or older supported state requires verified backup evidence. An empty directory may be initialized without a backup because no durable source evidence exists.

## 5. Durable migration journal

The journal file is `storage-migration.journal`.

Allowed forward path:

```text
planned
→ backup_verified   # required when the plan requires backup
→ migrating
→ verified
→ committed
```

An empty initialization may move directly from `planned` to `migrating`.

Allowed rollback path:

```text
planned | backup_verified | migrating | verified
→ rolling_back
→ rolled_back
```

No transition may skip verification and publish `committed`. Backup evidence may only be attached when entering `backup_verified`.

## 6. Required migration phases

1. **Inspect** — classify source state and seal deterministic inventory.
2. **Plan** — generate the target-bound migration plan without mutation.
3. **Verified backup** — create and verify a backup bound to the source digest and plan ID.
4. **Migrate** — perform idempotent target-format steps while recording durable progress.
5. **Verify** — verify manifest, canonical reads, index consistency, backup readability, and recovery evidence.
6. **Commit** — durably publish the target format only after verification succeeds.
7. **Resume or rollback** — classify interrupted state from durable evidence and continue deterministically.

## 7. Fail-closed rules

The implementation must not mutate storage when:

- the manifest format is newer than supported
- the manifest is malformed or has unknown fields
- the layout identifier is unsupported
- the data directory contains symlinks or unsupported entry types
- the source inventory differs from the plan binding
- required verified backup evidence is absent
- the journal contains an invalid stage or transition
- verification has not succeeded durably

Errors must not be converted into an empty or legacy classification.

## 8. Upgrade policy

- Supported legacy states are upgraded only through an explicit inspected plan.
- Upgrade is not an implicit side effect of ordinary server startup.
- A migration requiring backup must not enter `migrating` until backup evidence is verified and durably recorded.
- Re-running an interrupted migration uses the existing journal and plan binding; it must not silently generate a replacement plan over changed source state.
- Canonical Knowledge Objects and transition evidence are not semantically rewritten during a format-only migration.

## 9. Downgrade policy

Automatic downgrade is not supported in v0.7.0.

A binary that encounters a newer format must stop before mutation. Downgrade requires restoration of a verified backup created by a version compatible with the target binary. Removing or editing the manifest to force startup is unsupported and must not be documented as a recovery method.

## 10. Deprecated configuration policy

Configuration deprecation must include:

- the replacement key or procedure
- the first version emitting a warning
- the earliest removal version
- deterministic precedence during the transition period
- a migration example

A deprecated configuration key must not change meaning before removal. Conflicting old and new keys fail closed unless an explicit precedence contract has already been published.

## 11. v0.7.0 completion evidence

The release gate requires an integration fixture representing v0.4.0-equivalent durable state and evidence that migration to v0.7.0 preserves:

- read
- write
- index verification
- verified backup
- crash recovery
- deterministic resume or rollback
- fail-closed rejection of unknown newer format

The implementation must retain failure-point coverage around backup verification, manifest publication, verification, commit, resume, and rollback.