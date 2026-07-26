# Multi-node Synchronization

**Status: non-normative future design note** | **v1.0 support: not implemented**

## Purpose

This document records constraints for possible future synchronization between independently operated Lingonberry nodes. It is not a v1.0 synchronization contract and does not claim that relay replication, storage replication, continuous subscription replication, or automatic archive exchange exists.

Lingonberry v1.0 supports deterministic single-node storage, replay, archive export/import, and derived-index reconstruction. Those capabilities may become building blocks for future synchronization, but they do not constitute a distributed consistency protocol.

## Current v1.0 boundary

Lingonberry v1.0 does not provide:

- automatic relay-to-relay replication
- storage-node replication
- cluster membership or peer discovery
- replication cursors shared across nodes
- global ordering or causal ordering
- consensus, quorum writes, or leader election
- automatic conflict resolution
- exactly-once cross-node delivery
- cross-node transactional atomicity

Manual archive transfer is an operator action, not a live replication guarantee.

## Design invariants

Any future synchronization mechanism must preserve these rules:

1. Accepted canonical evidence remains append-only.
2. Canonical bytes and canonical identifiers are not rewritten to match another node.
3. Duplicate, conflict, transition, and identity decisions use the checked-in protocol and storage contracts.
4. Derived indexes and caches are rebuildable and are not replicated as semantic authority.
5. Sync transport success does not establish semantic acceptance.
6. Unknown versions, unsupported capabilities, contradictory checkpoints, and incomplete evidence fail closed.
7. A synchronization mechanism must not weaken local validation, signature, authorization, or quarantine rules.
8. Recovery must be deterministic from accepted durable evidence.

## Candidate transfer modes

The following are planning categories, not implemented v1.0 APIs.

### Incremental delivery

A future carrier may deliver newly observed evidence using an explicit, durable cursor. The contract must define cursor ownership, replay, expiry, retention, duplication, and gap detection.

### Deterministic replay

A node may reconstruct local state from an explicitly bounded evidence set. Replay must use the same validation, canonicalization, identity, duplicate/conflict, and transition rules as ordinary ingestion.

### Archive transfer

Versioned archive bundles may be moved between operators. Import must verify the manifest and contents before mutation, preserve immutable evidence, and report partial or rejected records without silently completing an incomplete transfer.

## Required synchronization state

A future implementation must make the following machine-readable:

- source node or evidence-set identity
- protocol, schema, archive, and capability versions
- bounded source range
- durable cursor or checkpoint
- accepted, duplicate, rejected, deferred, and conflicting counts
- detected gaps and unresolved dependencies
- final digest or proof binding the transferred set
- restart and resume state

A process exit code alone is not sufficient evidence of synchronization completeness.

## Failure and recovery requirements

Future synchronization must define behavior for:

- interruption before or after durable append
- duplicate redelivery
- source truncation or retention expiry
- cursor rollback or reuse
- missing predecessor evidence
- incompatible protocol or archive versions
- invalid signatures or authorization
- identity and canonical-ID conflicts
- local storage failure
- partial archive or manifest corruption

Retry must be idempotent. Ambiguous completion must not be reported as success.

## Consistency non-guarantees

Unless a later versioned contract explicitly provides them, synchronization does not imply:

- linearizability
- serializability across nodes
- globally consistent effective views
- globally ordered transitions
- automatic winner selection
- identical wall-clock timestamps
- immediate convergence

Convergence, when claimed, must be defined over a bounded evidence set and verified by deterministic digests or equivalent proofs.

## Adoption gate

A multi-node synchronization feature requires:

- versioned schemas and wire contract
- threat model and security review
- deterministic conformance fixtures
- duplicate/conflict and interrupted-operation tests
- upgrade, downgrade, backup, restore, and rollback guidance
- operator-visible diagnostics and metrics
- reference-platform qualification

Until those gates are complete, this document remains design input only.

## References

- [Duplicate and Conflict Contract](../architecture/DUPLICATE_AND_CONFLICT_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
