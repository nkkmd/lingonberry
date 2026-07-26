# Multi-node Conflict Handling

**Status: non-normative future design note** | **v1.0 support: not implemented**

## Purpose

This document records constraints for handling evidence received from multiple independently operated nodes. It is not a separate v1.0 conflict-resolution contract and does not authorize automatic merge, winner selection, replication conflict repair, or cross-node canonical-state reconciliation.

The current duplicate and conflict behavior is defined by the checked-in single-node implementation and [Duplicate and Conflict Contract](../architecture/DUPLICATE_AND_CONFLICT_CONTRACT.md). Multi-node transport does not replace or weaken that contract.

## Current v1.0 boundary

Lingonberry v1.0 does not provide:

- node-to-node conflict arbitration
- last-writer-wins resolution
- vector clocks or causal histories
- quorum-based winner selection
- automatic merge of canonical objects
- globally authoritative identity claims
- distributed quarantine resolution
- globally consistent effective views

Operators may import evidence from another source, but each record is evaluated locally under the same validation and storage rules as any other ingestion.

## Required distinctions

Future multi-node work must keep these cases separate:

### Exact duplicate

The canonical identifier, carrier binding, and canonical bytes match the already accepted record. This is an idempotent non-mutating success. It does not create a second semantic object or change the original `storedAt` value.

### Canonical-ID conflict

The same canonical identifier is presented with different canonical bytes or incompatible identity binding. The existing record is not overwritten. The new input fails under the stable conflict contract.

### Cross-identity conflict

A carrier identity, canonical identifier, or identity key attempts to bind inconsistently to an existing relationship. Equal content does not make an inconsistent identity rebinding safe.

### Transition disagreement

Nodes possess different accepted transition evidence or different bounded evidence sets. This is not resolved by rewriting the original object. Transition authority, supersession, orphan handling, and effective-view contracts remain controlling.

### Domain disagreement

Two valid records make incompatible real-world claims. Lingonberry authenticity or provenance evidence does not establish which claim is true. Domain policy belongs to an application profile or external adjudication process.

## Prohibited resolution rules

A future implementation must not choose a winner solely by:

- arrival order
- wall-clock timestamp
- node hostname
- higher sequence number without a versioned authority rule
- operator convenience
- signature presence alone
- majority count without an adopted consensus contract

No hidden overwrite or silent merge is permitted.

## Evidence preservation

When conflicting input is retained for diagnosis or quarantine, the system must preserve enough immutable evidence to determine:

- source and carrier identity
- canonical identifier and canonical bytes
- protocol and schema versions
- signatures and verification result
- local decision and machine-readable code
- related transition or identity evidence
- bounded evidence-set digest when applicable

Diagnostic retention must not cause rejected evidence to appear as accepted canonical state.

## Future resolution proposal requirements

Any proposal for cross-node resolution must define:

- conflict classes and authority boundaries
- deterministic inputs and ordering
- identity and key lifecycle assumptions
- behavior under incomplete or contradictory evidence
- authorization for manual decisions
- audit log and rollback semantics
- impact on canonical identifiers and signatures
- conformance fixtures and security review

A domain-specific merge policy must be implemented as an explicit application profile and must not silently alter the core protocol contract.

## Non-guarantees

Lingonberry does not currently guarantee:

- global conflict visibility
- automatic convergence after partition
- a globally preferred canonical record
- cross-node quarantine synchronization
- distributed operator decisions
- deletion or erasure of conflicting remote evidence

## References

- [Duplicate and Conflict Contract](../architecture/DUPLICATE_AND_CONFLICT_CONTRACT.md)
- [Identity and Provenance](../protocols/IDENTITY_AND_PROVENANCE.md)
- [Transition Authority](../protocols/TRANSITION_AUTHORITY.md)
- [Transition Supersession](../protocols/TRANSITION_SUPERSESSION.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
