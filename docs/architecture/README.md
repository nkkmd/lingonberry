# Architecture Documentation

**Status: active** | **Language: English normative**

This directory contains architecture-level documentation for Lingonberry. It explains system boundaries, checked-in contracts, compatibility constraints, audit records, and explicitly non-normative future directions.

Architecture documents do not override concrete schemas, runtime behavior, conformance fixtures, or protocol contracts. When a summary conflicts with those artifacts, the more specific checked-in contract is authoritative.

## Release boundary

The fixed v1.0.0 release candidate is:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Post-candidate documentation and tooling commits do not redefine that candidate. Formal 72-hour soak, privileged reference-host qualification, version preparation, release PR, tag, and GitHub Release remain separate release gates.

## Recommended reading order

1. [Distributed Knowledge Commons Architecture](./DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md) — current single-node architecture boundary and constrained distributed direction.
2. [Duplicate and Conflict Contract](./DUPLICATE_AND_CONFLICT_CONTRACT.md) — deterministic ingest classification and fail-closed identity conflict behavior.
3. [v1 Compatibility Policy](./V1_COMPATIBILITY_POLICY.md) — compatibility expectations for the v1 line.
4. [Protocol Evolution Proposal](./LINGONBERRY_PROTOCOL_EVOLUTION_PROPOSAL.md) — non-normative post-v1 evolution process and candidate capabilities.

Protocol-level details are indexed in [Protocol Documentation](../protocols/README.md). Operational procedures are indexed in [Operations Documentation](../operations/README.md).

## Document index

| Document | Role | Normative status |
|---|---|---|
| [Distributed Knowledge Commons Architecture](./DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md) | Defines the current architecture boundary, authority layers, replay model, and explicit v1 non-guarantees. | Architecture guidance aligned to checked-in contracts. |
| [Duplicate and Conflict Contract](./DUPLICATE_AND_CONFLICT_CONTRACT.md) | Describes storage-ingest duplicate and conflict classification. | Mirrors the checked-in contract-version-1 classifier. |
| [Protocol Evolution Proposal](./LINGONBERRY_PROTOCOL_EVOLUTION_PROPOSAL.md) | Provides a framework for evaluating future protocol changes. | Non-normative until a proposal is implemented, tested, and adopted. |
| [Toitoi Reference Checklist](./TOITOI_REFERENCE_CHECKLIST.md) | Records integration questions and references relevant to the Toitoi application profile. | Review aid; not a protocol contract. |
| [v0.9 Public API Freeze Candidate](./V0_9_PUBLIC_API_FREEZE_CANDIDATE.md) | Historical v0.9 API-freeze candidate record. | Historical audit evidence. |
| [v0.9 Rust API Inventory](./V0_9_RUST_API_INVENTORY.md) | Historical inventory of exported Rust API surfaces. | Historical audit evidence. |
| [v1.0 Rust API Audit](./V1_0_RUST_API_AUDIT.md) | Audits the candidate Rust API surface for v1.0 readiness. | Release qualification evidence; does not publish v1.0.0. |
| [v1 Compatibility Policy](./V1_COMPATIBILITY_POLICY.md) | Defines compatibility expectations and change classification for the v1 line. | Compatibility policy, subordinate to concrete contracts and release artifacts. |

## Related documentation

- [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md)
- [Protocol Documentation](../protocols/README.md)
- [Operations Documentation](../operations/README.md)
- [Roadmaps and Release Evidence](../roadmap/README.md)
- [Documentation Policy](../DOCUMENTATION_POLICY.md)
- [Generated Documentation Inventory](../DOCUMENTATION_INVENTORY.md)

## Maintenance rules

- Keep this index synchronized when architecture documents are added, renamed, archived, or reclassified.
- Label proposals, historical evidence, release evidence, and active contracts distinctly.
- Do not describe an unimplemented design as a v1 guarantee.
- Do not use documentation-only changes to move the fixed release candidate.
- Link to the specific protocol, schema, test, or runtime artifact for normative technical details.
