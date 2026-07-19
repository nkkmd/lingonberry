# v0.5.0 Phase 2: Duplicate and Conflict Plan

**Status: in progress** | **Parent: #76**

## Goal

Apply one deterministic duplicate/conflict contract to every canonical storage entry path.

## Completed

- [x] Define contract version `1`
- [x] Define `new`, `exact duplicate`, `canonical ID conflict`, `carrier identity conflict`, and `cross-identity conflict`
- [x] Add a pure core classifier
- [x] Add public contract tests
- [x] Document identity inputs, invariants, and decision order
- [x] Add file/SQLite backend parity tests
- [x] Verify duplicate and conflict paths do not append to the raw wire log
- [x] Add archive import duplicate/conflict parity tests
- [x] Add live retry parity tests
- [x] Route the v0.5 live ingestion path through a classified storage adapter

## Remaining

- [ ] Replace file backend handwritten classification with the core classifier
- [ ] Replace SQLite backend handwritten classification with the core classifier
- [ ] Apply the classifier explicitly to replay-derived restore
- [ ] Add quarantine promotion parity tests
- [ ] Update `RELEASE_0_5_0_ROADMAP.md`

## Safety gates

- No conflict path may mutate canonical storage or raw wire log.
- Exact duplicate remains idempotent success.
- Cross-identity aliasing is always conflict, even when canonical content matches.
- Corruption and I/O errors remain failures and are not collapsed into classification results.
- CLI and HTTP live ingestion classify before backend-specific append logic.
