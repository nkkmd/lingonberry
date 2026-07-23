# Storage Migration and Upgrade Contract

**Status: v1.0 pre-release implementation contract**  
**Normative language: English**

## 1. Purpose and boundary

This document defines the implemented single-node storage-format migration primitive used by Lingonberry. It describes source inspection, deterministic planning, verified migration backup, durable journal transitions, target-manifest publication, resume, commit, and rollback.

It does not replace the operator-facing upgrade procedure in [`V1_0_UPGRADE_AND_ROLLBACK.md`](./V1_0_UPGRADE_AND_ROLLBACK.md). Binary replacement, systemd lifecycle, post-upgrade health checks, canonical read/write validation, index verification, and backup-based release rollback remain operator and release-qualification responsibilities.

The latest published release is `v0.9.0`. `v1.0.0` remains unpublished. This contract does not indicate that formal qualification, the 72-hour soak, version update, tag, or GitHub Release has completed.

## 2. Files and current format

The migration implementation uses these files inside the configured storage data directory:

```text
storage-format.manifest
storage-migration.journal
```

The current supported format is:

```text
format_version=1
layout_id=single-node-canonical-v1
```

The manifest records:

- `format_version`
- `layout_id`
- `created_by`
- optional `source_format_version`

Both manifest and journal are written through a temporary file, synchronized, renamed, and followed by parent-directory synchronization. Operators must not edit either file manually.

## 3. Inspection classifications

`lingonberry-storage-migrate inspect` performs read-only inspection and reports one of:

```text
empty
legacy_unversioned
supported
unknown_newer
corrupt
```

Meanings:

- `empty`: no durable inventory and no manifest;
- `legacy_unversioned`: durable inventory exists without a manifest;
- `supported`: the manifest version and layout match the current binary;
- `unknown_newer`: the manifest version is newer than the current binary;
- `corrupt`: the manifest is malformed, contradictory, unsupported, or unreadable.

Inspection recursively inventories regular files and directories in deterministic order. Symlinks and unsupported filesystem entry types fail closed. The manifest and migration journal are excluded from the source inventory digest so journal progress does not change the source binding.

## 4. Deterministic plan and journal creation

`lingonberry-storage-migrate plan`:

1. refuses to replace an existing migration journal;
2. inspects the configured data directory;
3. creates a deterministic plan bound to:
   - source inventory digest;
   - source format version or legacy classification;
   - target format version;
4. writes a durable journal in `planned` stage.

A directory already at the current supported format does not receive a new migration plan. `unknown_newer` and `corrupt` states are refused.

A non-empty legacy source requires verified backup evidence. An empty directory may proceed without backup because no source durable inventory exists.

## 5. Migration stages

Allowed forward transitions are:

```text
planned
→ backup_verified      # required for a legacy non-empty source
→ migrating
→ verified
→ committed
```

For an empty source, the allowed forward path may omit `backup_verified`:

```text
planned
→ migrating
→ verified
→ committed
```

Allowed rollback transitions are:

```text
planned | backup_verified | migrating | verified
→ rolling_back
→ rolled_back
```

A committed migration cannot be rolled back by the migration primitive.

## 6. Verified migration backup

`lingonberry-storage-migrate backup` requires the journal to be in `planned` stage.

The implementation:

1. verifies that the current source inventory still matches the plan binding;
2. creates a new backup directory under the configured backup root using the deterministic plan ID;
3. refuses an existing destination;
4. copies the durable tree while excluding the migration manifest and journal;
5. rejects symlinks and unsupported entry types;
6. synchronizes copied files and directories;
7. re-inspects both source and backup;
8. requires both inventory digests to match the planned source digest;
9. records that digest as backup evidence and advances to `backup_verified`.

This is a byte-and-inventory binding check for the migration source. It is not the full operator backup/restore drill described in the v1 upgrade runbook.

## 7. Apply and verification boundary

`lingonberry-storage-migrate apply` accepts `planned` for an empty source or `backup_verified` for a legacy source.

The implementation:

1. re-verifies the source inventory binding;
2. advances the journal to `migrating`;
3. writes the current storage manifest;
4. re-inspects the data directory;
5. requires the durable inventory digest to remain equal to the planned source digest;
6. requires inspection to report the target supported format;
7. advances the journal to `verified`.

The migration primitive's `verified` stage proves the inventory binding and target manifest contract. It does not by itself prove application-level canonical reads, new writes, index consistency, relay readiness, backup restoration, or disaster recovery. Those checks remain mandatory in the operator runbook and release qualification.

## 8. Verify and commit commands

`lingonberry-storage-migrate verify` is read-only. It requires the journal to be `verified` or `committed`, then confirms that the current inventory still matches the journal source binding.

`lingonberry-storage-migrate commit`:

- requires `verified` stage;
- repeats migrated-storage verification;
- advances durably to `committed`;
- is idempotent when already committed.

Ordinary service startup must not perform implicit migration or implicit commit.

## 9. Resume semantics

`lingonberry-storage-migrate resume` follows the durable journal:

- `planned` or `backup_verified`: continue through apply;
- `migrating`: create the manifest if absent, verify the target state, and advance to `verified`;
- `verified`: commit;
- `committed`: return the existing committed state;
- `rolling_back` or `rolled_back`: refuse resume.

Resume uses the existing plan binding. It must not silently generate a replacement plan over changed source state.

## 10. Rollback semantics

`lingonberry-storage-migrate rollback` is available only before commit.

The implementation:

1. advances to `rolling_back` when necessary;
2. removes the target storage manifest if present;
3. synchronizes the data directory;
4. advances to `rolled_back`;
5. returns the existing result when already rolled back.

Rollback does not restore files from the migration backup because the implemented format migration does not rewrite canonical durable inventory. Recovery from binary incompatibility, data loss, or post-commit failure requires the backup-based procedure in [`V1_0_UPGRADE_AND_ROLLBACK.md`](./V1_0_UPGRADE_AND_ROLLBACK.md).

## 11. Fail-closed rules

Migration must stop without mutation when:

- the manifest is newer than supported;
- the manifest is malformed or contains unknown fields;
- the layout identifier is unsupported;
- the data directory is not a directory;
- a symlink or unsupported entry type is encountered;
- the source inventory differs from the plan binding;
- a required verified migration backup is absent;
- a backup destination already exists;
- a journal already exists when planning;
- the journal stage or transition is invalid;
- commit is requested before `verified`;
- rollback is requested after `committed`.

Operators must not remove or edit the manifest or journal to bypass these checks.

## 12. CLI sequence

Normal legacy migration:

```text
inspect
→ plan
→ backup
→ apply
→ verify
→ commit
```

Empty-directory initialization:

```text
inspect
→ plan
→ apply
→ verify
→ commit
```

Interrupted operation:

```text
status
→ resume
```

Pre-commit abandonment:

```text
status
→ rollback
```

The migration binary emits operator-oriented `key=value` output rather than canonical JSON. Exit-code details are defined in [`OPERATOR_CLI_CONTRACT.md`](./OPERATOR_CLI_CONTRACT.md).

## 13. Upgrade and downgrade policy

- Storage migration is always explicit.
- Ordinary relay or storage startup must not migrate storage.
- A newer unsupported storage format blocks an older binary before mutation.
- Automatic downgrade is not supported.
- Post-commit downgrade requires a verified backup compatible with the target binary and the backup-based rollback procedure.
- Canonical Knowledge Objects and transition evidence are not semantically rewritten by the current format migration.

## 14. Required operator evidence

For an actual release upgrade, preserve at minimum:

- old and new binary digests;
- effective configuration and systemd unit snapshots;
- `inspect`, `plan`, `backup`, `apply`, `verify`, `commit`, or interruption outputs;
- migration backup path and evidence digest;
- post-upgrade storage `doctor` and `verify` output;
- index verification output;
- relay readiness and canonical read/write validation;
- rollback disposition.

The migration journal is durable implementation state, not a complete release evidence bundle.
