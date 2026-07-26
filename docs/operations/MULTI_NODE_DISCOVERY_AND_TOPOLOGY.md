# Multi-node Discovery and Topology

**Status: non-normative future design note** | **v1.0 support: not implemented**

## Purpose

This document records constraints for a possible future multi-node discovery and topology design. It is not an operational contract for Lingonberry v1.0.0 and does not describe a checked-in discovery protocol, cluster manager, service registry, or failover implementation.

The v1.0 release is a production-oriented single-node system. Current normative behavior is defined by the checked-in schemas, runtime, conformance fixtures, HTTP and file/archive carrier contracts, and operator documentation.

## Current v1.0 boundary

Lingonberry v1.0 does not provide:

- automatic node discovery
- a signed node-manifest format
- a well-known discovery endpoint
- topology membership or health propagation
- leader election or consensus
- automatic failover between relays or storage nodes
- a distributed capability registry

Operators may configure independent nodes manually, but independently deployed nodes do not thereby form a supported Lingonberry cluster.

## Design invariants for future work

Any future discovery design must preserve these boundaries:

1. Discovery metadata is not knowledge-object semantics.
2. Reachability does not establish authenticity, authorization, or semantic correctness.
3. A node role must be verified from a versioned capability statement, not inferred from a hostname or label.
4. Stale, contradictory, unsigned, or unsupported discovery data must fail closed.
5. Discovery must not silently alter canonical bytes, canonical identifiers, provenance, or accepted evidence.
6. Protocol and carrier remain separate. Discovery may locate a carrier endpoint but does not redefine the protocol.
7. Cached discovery data is derived state and must not become the source of truth.

## Candidate role vocabulary

The following names are planning vocabulary only:

- **relay endpoint**: accepts or serves supported carrier requests
- **storage node**: owns a supported canonical durable state
- **archive endpoint**: exports or imports supported archive bundles
- **gateway**: connects explicitly versioned adapters or carriers

These labels do not imply replication, redundancy, consistency, or failover guarantees.

## Minimum proposal requirements

A future implementation proposal must define and test:

- node identifier and key lifecycle
- signed and versioned capability representation
- freshness, expiry, revocation, and replay handling
- authentication and authorization boundaries
- conflicting-manifest behavior
- endpoint and role validation
- downgrade and unknown-version behavior
- cache invalidation and recovery
- privacy implications of topology disclosure
- conformance fixtures and operator diagnostics

A proposal that omits these items is not sufficient for adoption.

## Topology non-guarantees

Lingonberry does not currently guarantee:

- a global node list
- a single authoritative topology
- globally ordered membership changes
- quorum membership
- automatic routing to an equivalent node
- cross-node readiness aggregation
- transparent replacement of an unavailable node

Multi-node topology must not be described as available until implementation, conformance evidence, security review, and operator qualification exist.

## Relationship to future synchronization

Discovery answers only where a candidate endpoint may be found and what it claims to support. It does not answer:

- whether two nodes contain equivalent evidence
- whether synchronization is complete
- which conflicting object should win
- whether durable state is safe to promote
- whether a node is authorized to publish or administer data

Those questions require separate, versioned contracts.

## References

- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
