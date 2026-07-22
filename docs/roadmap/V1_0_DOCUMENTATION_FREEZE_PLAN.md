# Lingonberry v1.0.0 Documentation Freeze Plan

**Status: active pre-candidate review** | **Target release: v1.0.0** | **Parent issue: #109** | **Last updated: 2026-07-23**

## 1. Purpose

This plan defines the documentation set that must be reviewed and frozen before a final v1.0.0 candidate is designated.

The objective is not editorial polish alone. The review must prove that a new operator can install, configure, operate, upgrade, recover, and diagnose Lingonberry using published documentation without undocumented repository knowledge or unsafe shortcuts.

A documentation-only change after candidate designation does not automatically invalidate runtime qualification, but it must be classified. Any change that alters a command, default, compatibility claim, migration boundary, recovery procedure, or release contract requires re-review and may invalidate operator-acceptance evidence.

## 2. Canonical documentation set

The freeze review covers the following families.

### 2.1 Installation and platform

- root `README.md`
- `docs/operations/SUPPORTED_PLATFORMS.md`
- systemd unit documentation and files under `deploy/systemd/**`
- binary installation paths and required filesystem permissions

### 2.2 Configuration

- configuration file, environment, and CLI precedence
- defaults and required values
- sensitive value handling and redaction
- state, data, backup, temporary, and runtime directory requirements
- invalid and contradictory configuration behavior

### 2.3 Normal operation

- `docs/operations/V0_8_OPERATOR_RUNBOOK.md`
- start, stop, restart, status, health, readiness, doctor, and metrics
- publish, retrieve, list, query, and consistency verification
- expected exit status and machine-readable output

### 2.4 Backup, restore, and disaster recovery

- backup creation and verification
- isolated restore planning and application
- active, non-empty, and symlink target rejection
- restored-state read, write, index, and cleanup verification
- disaster-recovery drill and retained evidence

### 2.5 Index operation

- index verification
- stale or invalid derived-state detection
- deterministic rebuild from canonical storage
- last-known-good and incomplete catch-up behavior

### 2.6 Migration, upgrade, and rollback

- supported source states
- inspect, plan, verified backup, apply, verify, commit, resume, and rollback
- unknown-newer-format rejection
- explicit rollback limits and unsupported downgrade behavior
- prohibition on implicit migration during ordinary startup

### 2.7 Quarantine, replacement, and cleanup

- inspection and proof requirements
- authorization and subject binding
- prepare, publish, resume, rollback, and cleanup
- contradictory-state rejection
- crash-point recovery outcomes
- terminal workspace retention responsibility

### 2.8 Recovery and troubleshooting

- diagnostic codes and exit status
- fail-closed classifications
- safe operator remediation
- prohibited manual mutation or proof bypass
- log and metric interpretation
- escalation conditions

### 2.9 Compatibility and release qualification

- `docs/architecture/V1_COMPATIBILITY_POLICY.md`
- `docs/architecture/V1_0_RUST_API_AUDIT.md`
- `docs/security/V1_0_SECURITY_DIFF_REVIEW.md`
- `docs/roadmap/V1_0_QUALIFICATION_PLAN.md`
- `docs/roadmap/V1_0_QUALIFICATION_STATUS.md`
- `docs/roadmap/V1_0_SOAK_PLAN.md`
- `docs/roadmap/V1_0_RELEASE_EVIDENCE.md`

## 3. Review method

Each instruction or contract claim must be classified as one of:

| Classification | Meaning |
|---|---|
| `EXECUTED` | Command or procedure was run against candidate-built binaries in the reference environment. |
| `INSPECTED` | Static configuration, path, unit, or contract was checked directly against implementation. |
| `CROSS_REFERENCED` | Normative claim was reconciled with another canonical document and test evidence. |
| `NOT_APPLICABLE` | The item does not apply to v1.0.0; the reason is recorded. |
| `BLOCKED` | The instruction is stale, ambiguous, unsafe, incomplete, or cannot be reproduced. |

No mandatory item may remain unclassified.

## 4. Walkthrough requirements

The final walkthrough must use:

- Ubuntu Server 24.04 LTS
- x86_64
- systemd
- release-built `lingonberry-storage` and `lingonberry-relay` binaries
- a fresh operator workspace
- only published documentation and explicitly listed prerequisites

The walkthrough must demonstrate:

1. prerequisites can be identified without source-tree knowledge;
2. binaries can be installed at documented paths;
3. systemd units validate and reference the documented binaries and directories;
4. configuration precedence and effective configuration are deterministic;
5. service startup, health, readiness, status, doctor, and metrics behave as documented;
6. publish and persisted-state retrieval survive a process restart boundary;
7. backup, verification, isolated restore, and restored-state verification succeed;
8. unsafe restore targets fail closed without mutation;
9. index verify and rebuild behave as documented;
10. migration and rollback instructions match the supported-state policy;
11. quarantine, replacement, cleanup, and recovery instructions never bypass proof or authorization;
12. every documented diagnostic and exit status used in procedures matches observed behavior.

## 5. Contradiction checks

The reviewer must explicitly reconcile:

- README examples versus operator runbook commands
- CLI help and actual required arguments versus documentation
- HTTP status and machine-readable error codes versus compatibility policy
- storage and migration version claims across architecture and operations documents
- backup and restore safety preconditions across all runbooks
- canonical-storage authority versus index documentation
- security fail-closed requirements versus troubleshooting advice
- qualification status versus release evidence and issue checklists
- reference platform claims versus workflow runner and systemd units

A contradiction is a release blocker until one normative source is selected and all dependent text is updated.

## 6. Freeze record

The final freeze record must contain:

```text
reviewed candidate SHA
review date
reviewer
reference environment
candidate binary digests
files reviewed
classification per file or procedure
commands executed
observed deviations
resolved contradictions
open blockers
residual risks
final PASS / FAIL disposition
```

The record belongs in `docs/roadmap/V1_0_RELEASE_EVIDENCE.md` or a referenced checksummed artifact.

## 7. Change control after freeze

After documentation freeze:

- typo or formatting-only changes require review but normally do not invalidate executable evidence;
- command, path, default, required setting, status code, compatibility, migration, recovery, or security changes invalidate the affected walkthrough evidence;
- runtime changes invalidate candidate-bound qualification according to `V1_0_QUALIFICATION_STATUS.md`;
- all post-freeze changes must be listed in final release evidence with their disposition.

## 8. Pass criteria

The documentation gate passes only when:

- every mandatory family has been reviewed;
- the reference-platform walkthrough succeeds without undocumented steps;
- no instruction recommends bypassing validation, authorization, proof, backup verification, or fail-closed handling;
- all commands, paths, defaults, outputs, diagnostic codes, and exit statuses used operationally match the candidate;
- compatibility and version claims are internally consistent;
- no `BLOCKED` item remains;
- the freeze record identifies the exact candidate and binary digests.

## 9. Current disposition

The documentation set and review method are defined, but the gate is **not yet passed**.

Final walkthrough, contradiction disposition, and freeze evidence must be completed against the designated candidate-built binaries before the 72-hour soak begins.
