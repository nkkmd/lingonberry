# Supported Platforms

**Status: active** | **Reference platform since: v0.8.0** | **Last updated: 2026-07-23**

## Purpose

This document defines the platform support boundary used for Lingonberry release qualification and operator documentation. It does not claim that `v1.0.0` has been released. The latest published release remains `v0.9.0`.

## Reference platform

The formal Linux reference platform is:

- Ubuntu Server 24.04 LTS;
- x86_64 (`amd64`);
- systemd;
- ext4 on a local block device for qualification and recovery exercises;
- release-mode Lingonberry binaries built from the designated source revision.

The designated pre-version `v1.0.0` qualification candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Evidence and tooling commits made after that revision do not redefine the candidate.

## Reference-platform acceptance scope

Release qualification on the reference platform covers, as applicable to the release gate:

- workspace formatting, linting, build, and tests;
- storage configuration, diagnostics, health, readiness, status, and metrics;
- checked-in systemd unit syntax and service contracts;
- publish, retrieve, query, replay, and persistence behavior;
- backup creation and verification;
- isolated restore planning and application;
- index verification and rebuild;
- isolated disaster-recovery exercises;
- explicit storage-format migration and rollback procedures;
- candidate-bound operator acceptance and formal soak evidence.

Hosted CI, dry runs, and virtual-time rehearsals validate tooling and evidence formats. They do not replace privileged reference-host qualification or the formal 72-hour soak.

## Support levels

### Reference support

Ubuntu Server 24.04 LTS on x86_64 with systemd is the platform for which release procedures, checked-in service units, operator runbooks, and acceptance workflows are maintained.

A platform is not promoted to reference support merely because the Rust workspace compiles or basic commands run successfully.

### Best-effort support

The following environments may work but are not part of the complete release qualification matrix unless a release document explicitly says otherwise:

- Debian 12 or later;
- newer Ubuntu LTS releases;
- Fedora, Rocky Linux, AlmaLinux, and other systemd-based Linux distributions;
- Linux on `arm64`;
- local Linux filesystems other than ext4 with equivalent durability and permission semantics.

For best-effort environments, maintainers do not promise per-release validation of systemd behavior, permissions, backup, restore, migration, disaster recovery, disk-pressure handling, or formal soak execution.

## Outside the reference contract

The following are outside the formal reference-platform release contract:

- Linux systems without systemd;
- production-node operation on Windows or macOS;
- network filesystems used as active canonical storage;
- container-only deployment as the sole supported production model;
- 32-bit architectures;
- distributed or multi-host locking guarantees;
- cloud-provider-specific durability guarantees.

These configurations are not prohibited, but successful use does not constitute reference-platform qualification.

## Filesystem and deployment boundaries

- Canonical state, backups, restore targets, temporary data, journals, and qualification evidence must follow the separation rules in the applicable runbook.
- Network filesystems are not qualified as active canonical storage.
- Containers may be used as development or integration tools, but the v1.0 reference service contract is systemd-based.
- The checked-in files under `deploy/systemd/` are the source of truth for maintained service templates.
- Storage migration remains explicit and operator-controlled; ordinary startup must not perform implicit migration.

## Related documents

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Operations Index](./README.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 Qualification Status](../roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0 Soak Plan](../roadmap/V1_0_SOAK_PLAN.md)

## Platform changes

The reference platform must not change implicitly as part of a routine dependency update. A change to the Ubuntu LTS release, CPU architecture, init system, qualified filesystem, or service model requires an explicit decision and coordinated updates to:

- the roadmap and compatibility policy;
- operator runbooks;
- CI and acceptance workflows;
- qualification contracts;
- release checklists and release notes.
