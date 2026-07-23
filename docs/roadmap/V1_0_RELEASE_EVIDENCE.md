# Lingonberry v1.0.0 Release Evidence

**Status: candidate qualification, security, compatibility, and documentation walkthrough recorded; soak pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Last updated: 2026-07-23**

## 1. Evidence policy

This document is the final commit-bound evidence record for the Lingonberry v1.0.0 release.

A prior release result, workflow dry run, or documentation claim may be referenced as historical context, but it does not satisfy a mandatory v1.0.0 gate unless the qualification plan explicitly allows review-only reuse and the applicability review is recorded here.

Do not mark a gate passed unless its evidence identifies the exact candidate or merged release commit, execution method, environment, pass criteria, retained artifact, and deviation disposition.

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
| Candidate record | `docs/roadmap/V1_0_CANDIDATE.md` |
| Security and compatibility review | `docs/security/V1_0_SECURITY_DIFF_REVIEW.md` |
| Documentation walkthrough | `docs/roadmap/V1_0_DOCUMENTATION_WALKTHROUGH.md` |
| Soak plan | `docs/roadmap/V1_0_SOAK_PLAN.md` |

## 3. Candidate build provenance

| Evidence | Value |
|---|---|
| Repository | `nkkmd/lingonberry` |
| Candidate SHA | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Git status at qualification | Clean |
| Ubuntu version | Ubuntu 24.04.4 LTS |
| Architecture | `x86_64` |
| systemd version | 255 |
| Rust compiler | `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| Cargo version | `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Node.js version | `v22.23.1` |
| `lingonberry-storage` SHA-256 | `22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0` |
| `lingonberry-relay` SHA-256 | `9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d` |
| Qualification workflow | run 6, run ID `29971797941` |
| Qualification artifact | ID `8549953270`; `sha256:cc216536a29acbc65ba7b25e74f1e2198c7050605019ea3a09c1ddab0fb18b7b` |
| Walkthrough workflow | run 3, run ID `29974169660` |
| Walkthrough artifact | ID `8550809328`; `sha256:75adb9ce95b69307632705aa82d89ede1cf413779e11ab29e18e2a47cca56904` |
| Artifact retention | Both GitHub Actions artifacts retained through 2026-10-21; permanent identities and binary hashes recorded here |
| Independent inspection | Qualification bundle: 12 gates and 32 checksums verified. Walkthrough bundle: 16 procedures and 34 checksums verified. |

## 4. Mandatory qualification gates

| Gate | Status | Evidence | Deviations / disposition |
|---|---|---|---|
| Object lifecycle end-to-end | Passed | qualification run `29971797941`, `logs/core-lifecycle.log` | None |
| External protocol conformance | Passed | qualification run and walkthrough DOC-08 | None |
| Supported legacy-state migration | Passed | qualification migration/recovery log; walkthrough DOC-13 | None |
| Backup verification and isolated restore | Passed | qualification operator acceptance; walkthrough DOC-09/10/11 | None |
| Index verify and rebuild | Passed | qualification index log; walkthrough DOC-12 | None |
| Replacement and cleanup crash matrix | Passed | qualification crash matrix; walkthrough DOC-15 | None |
| Standard Rust validation | Passed | qualification run; CI run 1198 | None |
| JavaScript validation | Passed | qualification run; CI run 1198 | None |
| Security regressions | Passed | qualification logs and candidate security review | No panic, abort, OOM, or credential leakage detected |
| Security release-blocker review | Passed | issue #130; `V1_0_SECURITY_DIFF_REVIEW.md` | Critical 0; High 0; release-blocking Medium 0 |
| Reference-platform operator acceptance | Passed | qualification operator acceptance; walkthrough DOC-01 through DOC-12 | Ubuntu 24.04.4, x86_64, systemd 255 |
| Installation/configuration/operations review | Passed | walkthrough run `29974169660`, artifact `8550809328` | Service-user env loading corrected in docs and rerun |
| Upgrade/rollback/recovery review | Passed | walkthrough DOC-13/15/16 and candidate tests | Cross-referenced where no integrated public CLI exists |
| v1.0 qualification soak | Pending | Issue #114 | Not started |

## 5. Compatibility confirmation

The candidate was reviewed against the approved v1 compatibility policy. The v0.9.0-to-candidate comparison contains no production implementation, protocol fixture, storage-format, migration-runtime, HTTP-handler, or operator-CLI change.

| Contract family | Candidate change | Disposition | Evidence |
|---|---|---|---|
| Protocol and schema | None | Compatible | external conformance passed |
| Canonical serialization and identifiers | None | Compatible | core lifecycle and workspace tests passed |
| Digest and signature payload | None | Compatible | security regressions passed |
| Public Rust API | No runtime API source change after audit | Compatible | Rust API audit and Rust gates |
| HTTP and operator CLI | None | Compatible | installed-binary acceptance and walkthrough |
| Diagnostics and machine-readable errors | None | Compatible | walkthrough observations |
| Configuration | Documentation invocation correction only | Compatible | precedence and service-user loading executed |
| Storage and durable artifacts | None | Compatible | migration, backup/restore, and index gates |
| Migration and rollback | None | Compatible | storage tests and walkthrough cross-reference |

No compatibility exception, waiver, or deprecation is required.

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
| Critical | 0 | 0 | `V1_0_SECURITY_DIFF_REVIEW.md` |
| High | 0 | 0 | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Medium | 0 | 0 | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Low | 2 accepted process residual risks | N/A | version-tagged Actions and finite artifact retention |

The accepted Low risks are controlled by retained toolchain provenance, candidate and binary digests, bundle checksums, repository-recorded artifact identity, and mandatory final merged-commit revalidation.

## 8. Documentation freeze

| Document area | Status | Reviewed commit | Evidence |
|---|---|---|---|
| Installation | Passed | candidate `f9543019…` | walkthrough DOC-02/03/06 |
| Configuration | Passed | candidate `f9543019…` | walkthrough DOC-04; corrected runbook |
| Operations | Passed | candidate `f9543019…` | walkthrough DOC-05 through DOC-12 |
| Upgrade and rollback | Passed | candidate `f9543019…` | walkthrough DOC-13 |
| Recovery and troubleshooting | Passed | candidate `f9543019…` | walkthrough DOC-11/15/16 |
| Compatibility policy | Passed | candidate `f9543019…` | issue #130 |
| README and documentation indexes | Passed | candidate `f9543019…` | PR #125 and freeze checks |
| Current implementation status | Pending release sync | Pending release PR | Non-runtime release-document task |
| Release checklist | Not created | Pending release PR | Required before publication |
| Release notes | Not created | Pending release PR | Required before publication |
| CHANGELOG | Pending | Pending release PR | Required before publication |

**Documentation freeze gate: PASS** for candidate execution. Release-specific checklist, notes, implementation-status, and CHANGELOG synchronization remain version-preparation tasks.

## 9. Final release validation

| Validation | Status | Evidence |
|---|---|---|
| Reviewed release PR checks | Pending | Pending |
| Merged-commit standard CI | Pending | Pending |
| Merged-commit candidate qualification | Pending | Pending |
| Pre-version candidate exact-SHA qualification | Passed | run `29971797941`, artifact `8549953270` |
| Candidate security and compatibility review | Passed | issue #130 |
| Candidate documentation walkthrough | Passed | run `29974169660`, artifact `8550809328` |
| Version consistency | Pending | Pending |
| Tag points to merged release commit | Pending | Pending |
| GitHub Release points to annotated tag | Pending | Pending |
| Published artifacts match recorded digests | Pending | Pending |

## 10. Deviations and residual risks

No release-blocking deviation was observed in candidate qualification, security/compatibility review, or documentation walkthrough.

Resolved documentation defect:

- The original runbook used pre-`sudo` command substitution to read a protected environment file and invoked direct publish as the operator user. PR #133 corrected both commands without changing runtime behavior, and the complete walkthrough was rerun successfully.

Accepted process residual risks:

- third-party GitHub Actions use reviewed version tags rather than immutable commit SHAs;
- GitHub Actions artifact retention is finite, while permanent artifact identity and binary digests are retained in this repository.

Remaining planned work is not classified as a deviation:

- 72-hour qualification soak;
- version preparation and publication validation.

## 11. Final decision

**Decision: Pending**

Candidate qualification, security, compatibility, and documentation freeze are green. A final `PASS` decision may be recorded only when the 72-hour soak, version preparation, merged-commit validation, tag, GitHub Release, and publication evidence are complete.
