# Phase 2: Duplicate / Conflict Contract Plan

## Objective

Apply one deterministic duplicate/conflict contract across every storage entry path.

## Contract

The classifier evaluates three axes:

1. canonical ID
2. carrier identity
3. canonical content

Classifications:

- `new`
- `exact duplicate`
- `canonical ID conflict`
- `carrier identity conflict`
- `cross-identity conflict`

Canonical ID and carrier identity rebinding is prohibited, even when canonical content is equivalent.

## Completed

- [x] Define duplicate/conflict contract version `1`
- [x] Add pure classifier and public result types
- [x] Add canonical JSON equivalence tests
- [x] Add cross-identity rebinding tests
- [x] Add File / SQLite backend parity tests
- [x] Verify duplicate/conflict do not append the raw wire log
- [x] Verify conflict preserves the existing canonical object
- [x] Verify live retry idempotency for File / SQLite
- [x] Verify archive re-import duplicate accounting
- [x] Verify archive conflict preserves existing object and identity binding
- [x] Route live CLI / HTTP ingestion through classified append
- [x] Add classified quarantine promotion API
- [x] Add File / SQLite quarantine promotion parity tests
- [x] Route active `quarantine-promote` CLI through classified promotion
- [x] Route active `quarantine-promote-batch` CLI through classified promotion
- [x] Preserve legacy dry-run batch behavior
- [x] Route archive import explicitly through classified append
- [x] Route active `import-archive` CLI through classified import
- [x] Confirm `StorageBackend::replay()` is read-only and no replay-derived restore write path currently exists

## Remaining

- [ ] Replace File backend handwritten classification with the shared classifier
- [ ] Replace SQLite backend handwritten classification with the shared classifier
- [ ] Synchronize `RELEASE_0_5_0_ROADMAP.md`

## Replay scope

The current `replay()` contract only reconstructs and returns stored records. It does not append records to canonical storage. Therefore no duplicate/conflict classification is required for replay itself. Any future mutating restore command built from replay data must call classified append.

## Safety boundaries

- Exact duplicates are idempotent success.
- Conflicts never overwrite canonical storage.
- Duplicate/conflict outcomes never append the raw wire log.
- Storage I/O and corruption errors are never collapsed into duplicate/conflict results.
- File and SQLite backends must expose identical externally visible behavior.
