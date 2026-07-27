# Roadmap to Lingonberry v1.0.0

**Status: active release roadmap; repository qualification complete, reference-host qualification next**  
**Latest published release: v0.9.0**  
**Fixed pre-version qualification candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`**  
**Last reviewed: 2026-07-27**

## 1. Purpose and authority

This document summarizes the release-level path from the completed v0.x milestones to Lingonberry v1.0.0.

The v1.0.0 target is a stable, production-oriented **single-node** release in which an independent operator can receive, validate, store, index, retrieve, quarantine, back up, restore, migrate, verify, and recover canonical knowledge objects by following checked-in documentation.

This roadmap is not the normative source for protocol, storage, API, security, or operational behavior. When statements differ, use the following authority order:

1. checked-in schemas, conformance fixtures, and executable tests;
2. normative protocol, storage, compatibility, security, and operations contracts;
3. candidate-bound qualification evidence;
4. this roadmap and historical milestone records.

## 2. Current release state

The published v0.5.0 through v0.9.0 milestones completed the implementation, contract, migration, operations, and hardening work originally described in earlier revisions of this roadmap.

The latest published release is **v0.9.0**. After the release-blocking publisher-authentication correction, the repository designated and qualified the following immutable pre-version candidate for v1.0.0:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

The previous candidate `f9543019f2c219aea3b085ff90f2da201b268a48` and evidence bound to it are historical and cannot authorize the active runtime. Documentation, inventory, and evidence-maintenance commits after the active candidate SHA do not redefine the candidate. Any runtime-affecting, protocol, durable-format, CLI or HTTP contract, migration, recovery, or default-behavior change would require an explicit new candidate and new candidate-bound evidence.

### 2.1 Completed foundations

The checked-in implementation and evidence cover the following single-node foundations:

- deterministic protocol parsing, validation, identity, canonicalization, digest, and signature handling;
- durable canonical storage with append-only evidence and replay;
- duplicate, conflict, transition, supersession, and effective-view behavior;
- persistent quarantine and operator-controlled maintenance;
- index verification, rebuild, catch-up, generation replacement, and recovery;
- verified backup, isolated restore, storage migration, and rollback boundaries;
- HTTP and file/archive carrier contracts;
- operator CLI, readiness, diagnostics, observability, systemd templates, and reference-platform procedures;
- normative v1 compatibility policy and completed Rust public API audit;
- candidate qualification, crash-matrix, disk-pressure, documentation-walkthrough, and security-review infrastructure.

### 2.2 Completed candidate-bound checks

The active candidate has passed standard CI, Rust and JavaScript regressions, external conformance, documentation inventory and freeze checks, candidate-delta security review, exact-candidate executable qualification, and the candidate documentation walkthrough. The qualification bundle passed all 12 gates, and the walkthrough passed all 16 procedures with independently verified checksums and binary identities.

These checks establish reproducible repository-side evidence for the candidate but do not by themselves authorize release publication. The next gate is privileged reference-host qualification on the designated Ubuntu Server 24.04 LTS x86_64 systemd host. The formal 72-hour soak remains blocked until that gate passes.

## 3. v1.0.0 product boundary

### 3.1 Included guarantees

The intended v1.0.0 support boundary includes:

- canonical knowledge-object ingestion and deterministic validation;
- identity, digest, provenance, and signature verification;
- durable canonical storage and replay;
- retrieval and supported index/query surfaces;
- explicit duplicate and conflict classification;
- persistent quarantine and documented operator actions;
- verified backup and isolated restore;
- index verification and deterministic rebuild;
- verified generation replacement and proof-bound cleanup;
- crash recovery and contradictory-state rejection;
- explicit, versioned storage migration;
- documented installation, configuration, operation, upgrade, rollback, and recovery;
- protocol v1, storage-format v1, CLI, HTTP, and supported Rust API compatibility rules.

### 3.2 Explicitly excluded from the v1.0.0 release gate

The following are future work and are not v1.0.0 guarantees:

- multi-node discovery, replication, synchronization, failover, placement, or consensus;
- distributed locking or strong consistency;
- Kubernetes operators or managed orchestration;
- complete OAuth/OIDC integration or per-record ACLs;
- remote backup services or secure erase guarantees;
- vector search or AI integration;
- universal support for arbitrary carriers or transports;
- automatic truth judgment, trust scoring, semantic merging, or domain-specific conflict resolution.

Running multiple independent Lingonberry processes does not create a supported cluster.

## 4. Historical milestone completion

The following milestones are historical and complete as published releases:

| Milestone | Primary outcome | Status |
|---|---|---|
| v0.5.0 | End-to-end object lifecycle | Published |
| v0.6.0 | Protocol contract and external conformance | Published |
| v0.7.0 | Storage migration and upgrade guarantees | Published |
| v0.8.0 | Operational readiness | Published |
| v0.9.0 | Public-contract hardening and release preparation | Published |

Historical release checklists and release notes remain evidence for their respective releases. They are not current v1.0.0 release authorization.

## 5. Remaining v1.0.0 gates

The following work remains mandatory before publication.

### 5.1 Privileged reference-host qualification

The reference-platform procedure must be completed on the designated Ubuntu Server 24.04 LTS x86_64 systemd host using candidate-built binaries and the required privileged installation, service-lifecycle, backup, restore, migration, recovery, disk-pressure, and public-endpoint checks.

Container, local-development, or unprivileged rehearsals do not replace this gate. The host-specific command map, resource thresholds, identities, deviations, and evidence locations must be frozen and independently inspected before the formal soak is authorized.

### 5.2 Formal 72-hour soak

The formal candidate-bound soak has not been performed. It may start only after privileged reference-host qualification receives a GO decision. It must satisfy the checked-in duration and workload floors, evidence requirements, interruption rules, stop conditions, and disposition criteria. Rehearsals do not count as the formal soak.

### 5.3 Final security and residual-risk disposition

Candidate-bound security findings, accepted deviations, residual risks, soak results, and any environment-specific limitations must receive final disposition. No unresolved release-blocking security or operational finding may remain.

### 5.4 Version and publication sequence

Only after all prior gates pass:

1. prepare version `1.0.0` without changing the qualified runtime behavior;
2. freeze release documents and checksums;
3. open and review the release PR;
4. validate the merged release commit;
5. create the `v1.0.0` tag;
6. publish the GitHub Release and final evidence record.

A passing CI run, candidate qualification run, documentation walkthrough, or rehearsal alone does not authorize these steps.

## 6. Release acceptance criteria

v1.0.0 may be published only when all of the following are true:

- standard CI and external conformance pass;
- candidate-bound executable qualification evidence is valid;
- object lifecycle, migration, backup/restore, index rebuild, crash recovery, and disk-pressure gates pass;
- the formal 72-hour soak passes;
- privileged reference-host qualification passes;
- documentation inventory has no release-blocking entries;
- the security and residual-risk disposition contains no release blocker;
- protocol, storage, CLI, HTTP, and public Rust API compatibility declarations are final;
- release versioning, tag, GitHub Release, checksums, and evidence point to the same reviewed release commit.

## 7. v1.x compatibility direction

The approved compatibility policy governs the exact contract. At roadmap level:

- protocol-v1 breaking changes require a later major version;
- v1.x must preserve the documented storage read and migration boundary;
- destructive or breaking migration requires an explicit plan, verified backup, and operator authorization;
- deprecation requires a documented replacement and removal window;
- unknown-newer, corrupt, contradictory, or incomplete durable state fails closed;
- canonical durable evidence is authoritative, while indexes and effective views remain derived and rebuildable.

## 8. Post-v1.0 candidates

Potential post-v1.0 work includes:

- additional carrier adapters;
- remote or encrypted backup integrations;
- richer query and relation traversal;
- alternative storage backends;
- multi-node experiments with explicit schemas, security review, conformance, recovery, and operator qualification;
- AI-assisted or vector-search profiles that do not alter canonical protocol semantics.

None of these items may be inferred as a current v1.0.0 guarantee.

## 9. Safety boundaries that must not be weakened

1. Never place unvalidated input in canonical storage.
2. Never silently ignore corruption, contradictory state, or I/O failure.
3. Never rewrite accepted canonical evidence in place.
4. Never treat derived indexes or effective views as the canonical source of truth.
5. Never perform implicit migration during ordinary startup.
6. Never restore over active state or active data.
7. Never reuse evidence from a different candidate after a candidate-invalidating change.
8. Never present an unfinished qualification gate as complete.

## 10. Related documents

- [Current v1.0 qualification status](./V1_0_QUALIFICATION_STATUS.md)
- [Pre-version candidate record](./V1_0_CANDIDATE.md)
- [Qualification plan](./V1_0_QUALIFICATION_PLAN.md)
- [Formal soak plan](./V1_0_SOAK_PLAN.md)
- [Reference-host rehearsal](./V1_0_REFERENCE_HOST_REHEARSAL.md)
- [Release evidence](./V1_0_RELEASE_EVIDENCE.md)
- [Documentation freeze plan](./V1_0_DOCUMENTATION_FREEZE_PLAN.md)
- [v1 compatibility policy](../architecture/V1_COMPATIBILITY_POLICY.md)
- [v1 operator runbook](../operations/V1_0_OPERATOR_RUNBOOK.md)
- [v1 upgrade and rollback](../operations/V1_0_UPGRADE_AND_ROLLBACK.md)
