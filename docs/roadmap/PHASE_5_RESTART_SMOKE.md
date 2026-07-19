# Phase 5: Restart smoke scenario

The v0.5.0 smoke test uses the real relay binary and a single durable state directory.

## Covered sequence

1. Publish a canonical object.
2. Query before restart.
3. Start and stop the HTTP process.
4. Start a new HTTP process against the same state directory.
5. Retrieve the object after restart.
6. Query after restart.
7. Run `rebuild-index`.
8. Verify `LB_INDEX_CONSISTENT` and checkpoint persistence.

The scenario proves process-local memory is not required for retrieval or query continuity. Canonical storage remains the source of truth and index consistency is checked through the versioned lifecycle contract.
