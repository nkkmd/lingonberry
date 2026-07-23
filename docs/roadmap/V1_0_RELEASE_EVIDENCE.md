# Lingonberry v1.0.0 Release Evidence

**Status: candidate qualification recorded; remaining gates pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Last updated: 2026-07-23**

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
| Candidate commit | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Candidate designation time | 2026-07-23 01:03:26 UTC, merge of PR #127 |
| Reviewed release PR | Pending |
| Merged release commit | Pending |
| Annotated tag | Pending |
| GitHub Release | Pending |
| Protocol/schema contract | `docs/architecture/V1_COMPATIBILITY_POLICY.md` |
| Qualification plan | `docs/roadmap/V1_0_QUALIFICATION_PLAN.md` |
| Qualification status | `docs/roadmap/V1_0_QUALIFICATION_STATUS.md` |
| Candidate record | `docs/roadmap/V1_0_CANDIDATE.md` |
| Soak plan | `docs/roadmap/V1_0_SOAK_PLAN.md` |

## 3. Candidate build provenance

| Evidence | Value |
|---|---|
| Repository | `nkkmd/lingonberry` |
| Candidate SHA | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Git status at qualification | Clean; artifact manifest `manifests/git-status.txt` is empty |
| Ubuntu version | Ubuntu 24.04.4 LTS |
| Architecture | `x86_64` |
| systemd version | Reference-runner verification passed; exact output retained in workflow log |
| Rust compiler | `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| Cargo version | `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Node.js version | `v22.23.1` |
| `lingonberry-storage` SHA-256 | `22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0` |
| `lingonberry-relay` SHA-256 | `9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d` |
| Qualification workflow | `v1 candidate qualification` run 6, run ID `29971797941` |
| Qualification artifact ID | `8549953270` |
| Qualification artifact digest | `sha256:cc216536a29acbc65ba7b25e74f1e2198c7050605019ea3a09c1ddab0fb18b7b` |
| Artifact retention | GitHub Actions through 2026-10-21; permanent digest and binary hashes recorded here |
| Independent inspection | ZIP downloaded; candidate SHA, 12 gates, binary hashes, and all 32 `SHA256SUMS` entries verified |

The exact candidate was qualified through PR #128, whose head was the designated candidate commit and whose base was the immediately preceding main commit. PR #128 was intentionally closed without merge after evidence inspection.

## 4. Mandatory qualification gates

| Gate | Required evidence | Status | Evidence reference | Deviations / disposition |
|---|---|---|---|---|
| Object lifecycle end-to-end | Candidate-bound workflow result and log | Passed | run `29971797941`, `logs/core-lifecycle.log` | None |
| External protocol conformance | Fixture/version-bound result | Passed | run `29971797941`, `logs/external-conformance.log` | None |
| Supported legacy-state migration | Supported-state matrix result | Passed | run `29971797941`, `logs/storage-migration-recovery.log` | Candidate workflow coverage; final docs walkthrough still pending |
| Backup verification and isolated restore | Backup, verify, restore, and restored-state validation | Passed | run `29971797941`, `logs/operator-acceptance.log` | Candidate workflow coverage; final docs walkthrough still pending |
| Index verify and rebuild | Intact verification, stale-state detection, rebuild, and reverify | Passed | run `29971797941`, `logs/index-consistency.log` and operator acceptance | None |
| Replacement and cleanup crash matrix | Candidate-bound crash-point matrix | Passed | run `29971797941`, `logs/replacement-cleanup-crash-matrix.log` | None |
| Standard Rust validation | Formatting, clippy, and workspace tests | Passed | run `29971797941`; CI run 1198 | None |
| JavaScript validation | Node test suite result | Passed | run `29971797941`; CI run 1198 | None |
| Security regressions | Parser, signature workspace, path, authorization, and fail-closed regressions | Passed for automated coverage | run `29971797941`, workspace tests and crash matrix | Final findings disposition remains pending |
| Security release-blocker review | Candidate diff and findings ledger disposition | Pending | `docs/security/V1_0_SECURITY_DIFF_REVIEW.md` | Requires final candidate review |
| Reference-platform operator acceptance | Ubuntu Server 24.04 LTS x86_64 systemd acceptance | Passed for automated installed-binary scenario | run `29971797941`, `logs/operator-acceptance.log` | Final published-document walkthrough remains pending |
| Installation/configuration/operations review | Frozen documentation walkthrough | Pending | `V1_0_DOCUMENTATION_WALKTHROUGH.md` | Candidate execution rows remain pending |
| Upgrade/rollback/recovery review | Supported upgrade and rollback boundary validation | Pending final walkthrough | Automated migration/recovery tests passed | Documentation execution evidence required |
| v1.0 qualification soak | 72-hour plan-compliant run and retained telemetry | Pending | Issue #114 | Not started |

## 5. Compatibility confirmation

Record the final candidate review against the approved v1 compatibility policy.

| Contract family | Candidate change since policy approval | Compatibility disposition | Evidence |
|---|---|---|---|
| Protocol and schema | Pending final review | Pending | Pending |
| Canonical serialization and identifiers | Pending final review | Pending | Pending |
| Digest and signature payload | Pending final review | Pending | Pending |
| Public Rust API | Pending final review | Pending | Pending |
| HTTP and operator CLI | Pending final review | Pending | Pending |
| Diagnostics and machine-readable errors | Pending final review | Pending | Pending |
| Configuration | Pending final review | Pending | Pending |
| Storage and durable artifacts | Pending final review | Pending | Pending |
| Migration and rollback | Pending final review | Pending | Pending |

## 6. Soak result

| Field | Value |
|---|---|
| Qualified commit | `f9543019f2c219aea3b085ff90f2da201b268a48` |
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
| Critical | Pending final confirmation | Pending | `V1_0_SECURITY_DIFF_REVIEW.md` |
| High | Pending final confirmation | Pending | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Medium | Pending final confirmation | Pending | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Low | Pending | N/A | Pending |

The release is blocked unless Critical = 0, High = 0, and release-blocking Medium = 0.

## 8. Documentation freeze

| Document area | Status | Reviewed commit | Notes |
|---|---|---|---|
| Installation | Pending execution | `f9543019f2c219aea3b085ff90f2da201b268a48` | Walkthrough row pending |
| Configuration | Pending execution | `f9543019f2c219aea3b085ff90f2da201b268a48` | Walkthrough row pending |
| Operations | Pending execution | `f9543019f2c219aea3b085ff90f2da201b268a48` | Walkthrough row pending |
| Upgrade and rollback | Pending execution | `f9543019f2c219aea3b085ff90f2da201b268a48` | Automated tests passed; walkthrough pending |
| Recovery and troubleshooting | Pending execution | `f9543019f2c219aea3b085ff90f2da201b268a48` | Automated tests passed; walkthrough pending |
| Compatibility policy | Pending final confirmation | `f9543019f2c219aea3b085ff90f2da201b268a48` | |
| README and documentation indexes | Static review complete | `f9543019f2c219aea3b085ff90f2da201b268a48` | PR #125; documentation check run 8 passed |
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
| Pre-version candidate exact-SHA qualification | Passed | run `29971797941`, artifact `8549953270` |
| Version consistency | Pending | Pending |
| Tag points to merged release commit | Pending | Pending |
| GitHub Release points to annotated tag | Pending | Pending |
| Published artifacts match recorded digests | Pending | Pending |

## 10. Deviations and residual risks

No release-blocking deviation was observed in candidate qualification.

Remaining planned work is not classified as a deviation:

- candidate-bound documentation walkthrough;
- final compatibility and security disposition;
- 72-hour qualification soak;
- version preparation and publication validation.

Every future deviation entry must include:

- affected gate or contract;
- observed condition;
- severity and release-blocking classification;
- mitigation;
- regression or monitoring evidence;
- explicit accept, defer, or block decision;
- approving issue or pull request.

## 11. Final decision

**Decision: Pending**

The exact pre-version candidate qualification is green. A final `PASS` decision may be recorded only when the documentation walkthrough, final security and compatibility confirmation, 72-hour soak, release preparation, merged-commit validation, and publication evidence are complete.