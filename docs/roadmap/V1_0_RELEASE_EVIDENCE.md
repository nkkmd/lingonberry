# Lingonberry v1.0.0 Release Evidence

**Status: evidence collection not started** | **Target release: v1.0.0** | **Parent issue: #109** | **Last updated: 2026-07-23**

## 1. Evidence policy

This document is the final commit-bound evidence record for the Lingonberry v1.0.0 release.

A prior release result, workflow dry run, or documentation claim may be referenced as historical context, but it does not satisfy a mandatory v1.0.0 gate unless the qualification plan explicitly allows review-only reuse and the applicability review is recorded here.

Do not mark a gate passed unless its evidence identifies:

- the exact candidate or merged release commit;
- the command, workflow, or drill;
- the environment and relevant tool versions;
- the pass criteria;
- the result and retained artifact location;
- any deviation or residual-risk disposition.

## 2. Release identity

| Field | Value |
|---|---|
| Release version | `1.0.0` |
| Candidate commit | Pending |
| Candidate designation time | Pending |
| Reviewed release PR | Pending |
| Merged release commit | Pending |
| Annotated tag | Pending |
| GitHub Release | Pending |
| Protocol/schema contract | `docs/architecture/V1_COMPATIBILITY_POLICY.md` |
| Qualification plan | `docs/roadmap/V1_0_QUALIFICATION_PLAN.md` |
| Qualification status | `docs/roadmap/V1_0_QUALIFICATION_STATUS.md` |
| Soak plan | `docs/roadmap/V1_0_SOAK_PLAN.md` |

## 3. Candidate build provenance

| Evidence | Value |
|---|---|
| Repository | `nkkmd/lingonberry` |
| Candidate SHA | Pending |
| Git status at qualification | Pending |
| Ubuntu version | Pending |
| Architecture | Pending |
| systemd version | Pending |
| Rust compiler | Pending |
| Cargo version | Pending |
| Node.js version | Pending |
| `lingonberry-storage` SHA-256 | Pending |
| `lingonberry-relay` SHA-256 | Pending |
| Qualification artifact ID | Pending |
| Qualification artifact digest | Pending |
| Artifact retention/archival location | Pending |

## 4. Mandatory qualification gates

| Gate | Required evidence | Status | Evidence reference | Deviations / disposition |
|---|---|---|---|---|
| Object lifecycle end-to-end | Candidate-bound workflow result and log | Pending | Pending | None recorded |
| External protocol conformance | Fixture/version-bound result | Pending | Pending | None recorded |
| Supported legacy-state migration | Supported-state matrix result | Pending | Pending | None recorded |
| Backup verification and isolated restore | Backup, verify, restore, and restored-state validation | Pending | Pending | None recorded |
| Index verify and rebuild | Intact verification, stale-state detection, rebuild, and reverify | Pending | Pending | None recorded |
| Replacement and cleanup crash matrix | Candidate-bound crash-point matrix | Pending | Pending | None recorded |
| Standard Rust validation | Formatting, clippy, and workspace tests | Pending | Pending | None recorded |
| JavaScript validation | Node test suite result | Pending | Pending | None recorded |
| Security regressions | Parser, signature workspace, path, authorization, and fail-closed regressions | Pending | Pending | None recorded |
| Security release-blocker review | Candidate diff and findings ledger disposition | Pending | Pending | None recorded |
| Reference-platform operator acceptance | Ubuntu Server 24.04 LTS x86_64 systemd acceptance | Pending | Pending | None recorded |
| Installation/configuration/operations review | Frozen documentation walkthrough | Pending | Pending | None recorded |
| Upgrade/rollback/recovery review | Supported upgrade and rollback boundary validation | Pending | Pending | None recorded |
| v1.0 qualification soak | 72-hour plan-compliant run and retained telemetry | Pending | Pending | None recorded |

## 5. Compatibility confirmation

Record the final candidate review against the approved v1 compatibility policy.

| Contract family | Candidate change since policy approval | Compatibility disposition | Evidence |
|---|---|---|---|
| Protocol and schema | Pending review | Pending | Pending |
| Canonical serialization and identifiers | Pending review | Pending | Pending |
| Digest and signature payload | Pending review | Pending | Pending |
| Public Rust API | Pending review | Pending | Pending |
| HTTP and operator CLI | Pending review | Pending | Pending |
| Diagnostics and machine-readable errors | Pending review | Pending | Pending |
| Configuration | Pending review | Pending | Pending |
| Storage and durable artifacts | Pending review | Pending | Pending |
| Migration and rollback | Pending review | Pending | Pending |

## 6. Soak result

| Field | Value |
|---|---|
| Qualified commit | Pending |
| Start time | Pending |
| End time | Pending |
| Continuous duration | Pending |
| Workload minimums met | Pending |
| Abrupt termination scenarios | Pending |
| Disk-pressure scenarios | Pending |
| Backup/restore cadence | Pending |
| Index verification/rebuild cadence | Pending |
| Maximum RSS | Pending |
| Maximum file descriptors | Pending |
| Disk/inode growth disposition | Pending |
| Journal/proof/archive/workspace growth disposition | Pending |
| Panic/abort/OOM count | Pending |
| Canonical corruption count | Pending |
| Object/index divergence count | Pending |
| Unrecoverable injected failures | Pending |
| Soak artifact location and digest | Pending |
| Final soak decision | Pending |

## 7. Security findings disposition

| Severity | Open count | Release-blocking count | Evidence |
|---|---:|---:|---|
| Critical | Pending | Pending | Pending |
| High | Pending | Pending | Pending |
| Medium | Pending | Pending | Pending |
| Low | Pending | N/A | Pending |

The release is blocked unless Critical = 0, High = 0, and release-blocking Medium = 0.

## 8. Documentation freeze

| Document area | Status | Reviewed commit | Notes |
|---|---|---|---|
| Installation | Pending | Pending | |
| Configuration | Pending | Pending | |
| Operations | Pending | Pending | |
| Upgrade and rollback | Pending | Pending | |
| Recovery and troubleshooting | Pending | Pending | |
| Compatibility policy | Pending final confirmation | Pending | |
| README and documentation indexes | Pending | Pending | |
| Current implementation status | Pending | Pending | |
| Release checklist | Not created | Pending | |
| Release notes | Not created | Pending | |
| CHANGELOG | Pending | Pending | |

## 9. Final release validation

| Validation | Status | Evidence |
|---|---|---|
| Reviewed release PR checks | Pending | Pending |
| Merged-commit standard CI | Pending | Pending |
| Merged-commit candidate qualification | Pending | Pending |
| Version consistency | Pending | Pending |
| Tag points to merged release commit | Pending | Pending |
| GitHub Release points to annotated tag | Pending | Pending |
| Published artifacts match recorded digests | Pending | Pending |

## 10. Deviations and residual risks

No v1.0.0 deviation or residual-risk acceptance is recorded yet.

Every entry must include:

- affected gate or contract;
- observed condition;
- severity and release-blocking classification;
- mitigation;
- regression or monitoring evidence;
- explicit accept, defer, or block decision;
- approving issue or pull request.

## 11. Final decision

**Decision: Pending**

A final `PASS` decision may be recorded only when all mandatory gates are passed, all artifacts are retained and checksum-verifiable, all release identities agree, and no release blocker remains.
