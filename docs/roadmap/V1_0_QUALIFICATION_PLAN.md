# Lingonberry v1.0.0 Qualification Plan

**Status: active; candidate-bound qualification complete, formal soak and publication gates pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #110**

## 1. Purpose

This document defines the mandatory qualification sequence for the Lingonberry v1.0.0 stable single-node release. It does not authorize publication by itself.

The fixed pre-version candidate is:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Later documentation and inventory commits do not redefine that candidate.

## 2. Qualification principles

1. Executable evidence must identify the exact candidate or final merged release commit.
2. Documentation claims do not replace executable tests, drills, or retained evidence.
3. Canonical storage is authoritative; indexes and effective views are derived.
4. Unknown, corrupt, contradictory, partial, or unsupported state fails closed.
5. Candidate qualification does not replace the formal soak, privileged reference-host qualification, version preparation, or publication validation.
6. Runtime-affecting changes require explicit candidate reconsideration and evidence reruns.
7. Release publication requires all mandatory evidence to identify a coherent commit and contract set.

## 3. Current gate inventory

| Gate family | Current state | Evidence / remaining work |
|---|---|---|
| Candidate designation | Passed | PR #127; fixed candidate `f9543019…` |
| Candidate build and executable qualification | Passed | workflow run `29971797941`, artifact `8549953270` |
| External protocol conformance | Passed | candidate qualification and retained logs |
| Lifecycle, migration, recovery, backup/restore, and index | Passed | candidate qualification bundle |
| Replacement and cleanup crash matrix | Passed for candidate qualification | formal soak still requires distributed cycles |
| Rust public API audit | Passed | `V1_0_RUST_API_AUDIT.md` |
| Normative v1 compatibility policy | Passed | `V1_COMPATIBILITY_POLICY.md` |
| Candidate security and compatibility review | Passed | `V1_0_SECURITY_DIFF_REVIEW.md`; Critical 0, High 0, release-blocking Medium 0 |
| Candidate documentation walkthrough | Passed | workflow run `29974169660`, artifact `8550809328`, 16 procedures |
| Documentation freeze for candidate execution | Passed | `V1_0_RELEASE_EVIDENCE.md` |
| Formal 72-hour qualification soak | Pending | Issue #114; must use frozen scheduler, workload floors, telemetry, crash matrix, and disk-pressure evidence |
| Privileged reference-host qualification | Pending | dedicated Ubuntu Server 24.04 x86_64 systemd host; disk-pressure rehearsal and host-specific evidence |
| Final residual-risk disposition | Pending | record after soak and reference-host evidence |
| Version and release-document preparation | Pending | version `1.0.0`, checklist, notes, implementation-status and CHANGELOG synchronization |
| Reviewed release PR and merged-commit validation | Pending | all required checks and final evidence must identify the merged release commit |
| Annotated tag and GitHub Release | Pending | only after release authorization |

## 4. Formal soak requirements

The formal soak must follow:

- [`V1_0_SOAK_PLAN.md`](./V1_0_SOAK_PLAN.md)
- [`V1_0_FORMAL_SOAK_COMMAND_MAP.md`](./V1_0_FORMAL_SOAK_COMMAND_MAP.md)
- [`V1_0_FORMAL_SOAK_SCHEDULER.md`](./V1_0_FORMAL_SOAK_SCHEDULER.md)
- [`V1_0_CRASH_MATRIX_DRIVER.md`](./V1_0_CRASH_MATRIX_DRIVER.md)
- [`V1_0_DISK_PRESSURE_DRIVER.md`](./V1_0_DISK_PRESSURE_DRIVER.md)
- [`V1_0_REFERENCE_HOST_REHEARSAL.md`](./V1_0_REFERENCE_HOST_REHEARSAL.md)

Mock runs, virtual-time rehearsals, CI dry runs, and bounded pre-release soaks cannot satisfy the formal 72-hour gate.

## 5. Pass criteria

Release authorization requires all of the following:

- the formal soak completes continuously for at least 72 hours;
- every workload minimum and disruptive-scenario distribution requirement is met;
- candidate binaries and evidence inputs remain digest-bound;
- no panic, abort, OOM, canonical corruption, unrecoverable injected failure, or unexplained object/index divergence occurs;
- disk and inode growth, journals, proofs, archives, and workspaces have an explicit acceptable disposition;
- privileged reference-host procedures pass without undocumented workarounds;
- no Critical, High, or release-blocking Medium security finding remains;
- release documents and version metadata are synchronized;
- the reviewed release PR and merged release commit pass required validation;
- the annotated tag, GitHub Release, and published artifacts match recorded identities and digests.

## 6. Failure and change control

Any unexplained evidence gap, incomplete workload family, digest mismatch, unsupported host divergence, unsafe recovery instruction, or release-blocking defect fails the relevant gate.

A runtime-affecting change after candidate designation invalidates affected candidate evidence. Documentation-only corrections must still be reviewed for their effect on walkthrough and operator acceptance evidence.

## 7. Evidence authority

The current evidence record is [`V1_0_RELEASE_EVIDENCE.md`](./V1_0_RELEASE_EVIDENCE.md). Execution status is summarized in [`V1_0_QUALIFICATION_STATUS.md`](./V1_0_QUALIFICATION_STATUS.md).

This plan does not claim that the formal soak, privileged reference-host qualification, version preparation, release PR, tag, GitHub Release, or final publication evidence is complete.