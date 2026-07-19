# Lingonberry v0.5.0 Release Notes

v0.5.0 integrates the normal knowledge-object lifecycle across publish, storage, retrieval, query, restart, and index consistency operations.

## Added

- Versioned publish, retrieval, and basic-query contracts.
- Shared CLI and HTTP ingestion orchestration.
- Stable machine-code, HTTP-status, and CLI-exit mappings.
- Deterministic index generations, ID/content digests, atomic checkpoints, rebuild, verification, and catch-up.
- Fail-closed handling for corrupt, unsupported, stale, and ambiguous index state.
- Real-binary smoke coverage through restart and recovery.

## Compatibility and safety

Canonical storage remains the source of truth and index state remains derived and rebuildable. Invalid objects are not stored, conflicts do not overwrite canonical records, and inconsistent index results cannot replace an existing checkpoint.

## Known limitations

v0.5.0 does not introduce a separately persisted searchable index database, multi-node consistency, vector search, or AI integration. Publication requires successful post-merge main CI, an annotated `v0.5.0` tag, and a GitHub Release.
