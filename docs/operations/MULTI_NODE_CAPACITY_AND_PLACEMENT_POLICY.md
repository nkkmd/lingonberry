# Multi-node Capacity and Placement

**Status: non-normative future design note** | **v1.0 support: not implemented**

## Purpose

This document records operational constraints for a possible future multi-node capacity and placement model. It is not a v1.0 placement policy and does not describe a checked-in scheduler, cluster autoscaler, replication controller, or automatic failover system.

Lingonberry v1.0 is qualified as a production-oriented single-node deployment. Current capacity planning is therefore host-local and governed by the supported-platform, observability, storage, backup, recovery, and operator documentation.

## Current v1.0 boundary

Lingonberry v1.0 does not provide:

- automatic placement of relay, storage, index, or archive roles
- capacity-aware routing
- node draining or live migration
- automatic shard allocation
- replicated storage ownership
- cluster-wide admission control
- cluster autoscaling
- automatic failover after host loss
- cross-node capacity metrics aggregation

Running more than one independent process does not create a supported cluster.

## Design invariants

Any future placement system must preserve these rules:

1. Placement is operational policy, not knowledge-object semantics.
2. Moving or copying data must not rewrite canonical bytes, canonical identifiers, provenance, or accepted evidence.
3. Derived indexes may be rebuilt; canonical durable evidence must be explicitly owned, backed up, and verified.
4. Placement decisions must be explainable, versioned, and auditable.
5. Ambiguous ownership, incomplete transfer, unknown capacity, or contradictory topology must fail closed.
6. A healthy process is not sufficient evidence that its durable state is complete or safe to promote.
7. Capacity automation must not bypass validation, authorization, migration, backup, restore, or recovery contracts.

## Candidate role vocabulary

Future planning may distinguish:

- **relay role**: request admission and supported carrier endpoints
- **storage role**: ownership of canonical durable evidence
- **index role**: rebuildable derived views
- **archive role**: versioned portable evidence bundles
- **gateway role**: explicitly versioned carrier or protocol adapters

These roles are not current deployment guarantees. A node label alone does not establish authority or readiness.

## Minimum placement inputs

A future placement decision should be based on bounded, machine-readable inputs such as:

- supported role and capability versions
- CPU, memory, disk, and file-descriptor headroom
- durable evidence size and growth rate
- append, retrieval, replay, verify, and rebuild latency
- backup age and verification status
- recovery-point and recovery-time objectives
- network ingress, egress, and transfer backlog
- current ownership and transfer state
- operator maintenance and failure-domain constraints

Missing or stale inputs must be visible rather than replaced with optimistic defaults.

## Ownership and transfer requirements

Before moving a durable role, a future implementation must define:

1. source and target identity
2. bounded evidence set and digest
3. backup and restore prerequisites
4. transfer, resume, and interruption behavior
5. target verification
6. promotion authority
7. source retirement or rollback boundary
8. proof that no accepted evidence was silently omitted or rewritten

Promotion must not occur solely because bytes were copied or a process started successfully.

## Capacity pressure priorities

A future policy should prioritize, in order:

1. preserving accepted canonical evidence
2. preserving verifiable backup and recovery paths
3. preventing unsafe partial writes or migrations
4. maintaining operator visibility and diagnostics
5. maintaining supported read and publish paths when safe
6. rebuilding disposable derived state after durable safety is restored

Dropping provenance, weakening validation, or silently deleting evidence is not an acceptable capacity response.

## Failure-domain requirements

A future multi-node design must explicitly model:

- shared disks and shared power domains
- correlated network failure
- credential and key compromise
- region or provider dependency
- backup co-location
- stale replicas and incomplete transfers

Two processes on the same host or storage device do not provide meaningful failure-domain redundancy.

## Non-guarantees

Lingonberry does not currently guarantee:

- continued service after loss of a node
- automatic replacement of a failed role
- balanced resource use across hosts
- zero-downtime role migration
- replicated durability
- distributed admission control
- a cluster-wide capacity threshold

## Adoption gate

A future capacity and placement feature requires versioned configuration, implementation, security review, deterministic transfer and failure tests, operator runbooks, upgrade and rollback procedures, metrics, and reference-platform qualification.

## References

- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Observability](./OBSERVABILITY.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
