# Duplicate and Conflict Contract

**Contract version: 1** | **Status: active for v0.5.0**

## Purpose

This document defines how Lingonberry classifies a publish attempt when canonical storage already contains a related record.

The same rules apply to normal publish, retry, replay-derived restore, quarantine promotion, and archive import.

## Identity inputs

Classification uses three inputs:

1. `canonicalId`: the finalized canonical object identifier.
2. `carrierIdentity`: the stable identity derived from the publish request carrier envelope.
3. `canonicalContent`: canonical JSON for the finalized knowledge object.

Raw JSON formatting, key order, whitespace, and publisher signature bytes are not canonical content differences.

## Classifications

| Classification | Condition | Result code | Storage mutation |
|---|---|---|---|
| new | Neither canonical ID nor carrier identity exists | `LB_OBJECT_NEW` internally; publish returns `LB_OBJECT_STORED` | Append raw request and canonical record |
| exact duplicate | Canonical ID, carrier identity, and canonical content all match | `LB_OBJECT_DUPLICATE` | None |
| canonical ID conflict | Canonical ID exists but canonical content differs | `LB_OBJECT_CONFLICT` | None |
| carrier identity conflict | Carrier identity exists but canonical content differs | `LB_OBJECT_CONFLICT` | None |
| cross-identity conflict | A canonical ID or carrier identity points to a different counterpart, even if content matches | `LB_OBJECT_CONFLICT` | None |

## Required invariants

- Exact duplicate is idempotent success.
- Duplicate publish does not append another raw log record.
- Conflict never overwrites or appends a canonical record.
- A carrier identity cannot be rebound to another canonical ID.
- A canonical ID cannot be rebound to another carrier identity.
- Canonical content comparison uses canonical JSON, not source formatting.
- Storage I/O or corruption errors are never classified as duplicate or conflict.
- Archive import and retry use the same storage classification as live publish.

## Decision order

1. Look up the incoming carrier identity.
2. When found, require both canonical ID and canonical content to match.
3. Otherwise, look up the incoming canonical ID.
4. When found, require both carrier identity and canonical content to match.
5. When neither exists, classify as new.

The order is deterministic, but both identity dimensions are authoritative. A contradictory state is conflict, not duplicate.

## Machine-readable API

The core contract is exposed by `packages/core/src/duplicate_conflict.rs`:

- `DUPLICATE_CONFLICT_CONTRACT_VERSION`
- `DuplicateConflictClassification`
- `classify_duplicate_or_conflict`

External publish responses continue to use the publish ingestion contract:

- stored: `LB_OBJECT_STORED`
- duplicate: `LB_OBJECT_DUPLICATE`
- conflict: `LB_OBJECT_CONFLICT`

## Test requirements

The contract must include tests for:

- new object
- exact duplicate
- same canonical ID with different content
- same carrier identity with different content
- cross-identity aliasing with equal content
- file and SQLite backend parity
- retry and archive import parity
