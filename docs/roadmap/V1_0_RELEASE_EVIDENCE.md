# Lingonberry v1.0.0 Release Evidence

**Status: active candidate qualification and documentation walkthrough PASS; reference-host preflight and soak pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Last updated: 2026-07-27**

## 1. Evidence policy

This document is the authoritative commit-bound evidence record for the Lingonberry v1.0.0 release.

A prior candidate result may remain as historical context, but it does not satisfy an active release gate. A gate is passed only when its evidence identifies the exact candidate, execution method, environment, retained artifact, checksums, pass criteria, and deviation disposition.

## 2. Release identity

| Field | Value |
|---|---|
| Release version | `1.0.0` |
| Active candidate commit | `8c6b48082205a3af555130eec1f3e7d2ac8811fe` |
| Candidate designation | PR #333; exact SHA recorded by PR #334 |
| Superseded candidate | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Reviewed release PR | Pending |
| Merged release commit | Pending |
| Annotated tag | Pending |
| GitHub Release | Pending |
| Qualification plan | `docs/roadmap/V1_0_QUALIFICATION_PLAN.md` |
| Candidate record | `docs/roadmap/V1_0_CANDIDATE.md` |
| Security review | `docs/security/V1_0_SECURITY_DIFF_REVIEW.md` |
| Documentation walkthrough | `docs/roadmap/V1_0_DOCUMENTATION_WALKTHROUGH.md` |
| Soak plan | `docs/roadmap/V1_0_SOAK_PLAN.md` |

Documentation-only evidence commits after the active candidate do not move the candidate.

## 3. Runtime security delta

PR #331 added release-blocking Ed25519 publisher-signature enforcement before acceptance policy, quarantine, duplicate/conflict classification, raw append, and canonical storage.

The active candidate provides stable outcomes for:

- malformed public-key or signature encoding;
- cryptographically invalid signatures;
- verifier operational failures;
- valid checked-in signed vectors;
- duplicate and conflict processing after authentication.

No storage format, migration rule, canonical object format, public Rust API, backup/restore format, or recovery procedure was intentionally changed.

## 4. Candidate qualification provenance

| Evidence | Value |
|---|---|
| Repository | `nkkmd/lingonberry` |
| Candidate SHA | `8c6b48082205a3af555130eec1f3e7d2ac8811fe` |
| Qualification workflow run | `30238378797` |
| Qualification artifact ID | `8642393171` |
| Artifact name | `v1-qualification-8c6b48082205a3af555130eec1f3e7d2ac8811fe-1` |
| Artifact digest | `sha256:c30a0472f6ea07f3e395c9a27c67d1460b8f35a13a7afd397bd0e5895cb93b3e` |
| `lingonberry-storage` SHA-256 | `737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507` |
| `lingonberry-relay` SHA-256 | `23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c` |
| Aggregate result | `passed` |
| Gate result | 12 of 12 passed |
| Bundle checksum verification | Every `SHA256SUMS` entry verified |
| Independent artifact verification | Downloaded ZIP digest matched GitHub artifact digest |
| Artifact expiration | 2026-10-25 |

The artifact manifest embedded the exact candidate SHA and the binary digests matched direct calculations.

## 5. Documentation walkthrough provenance

| Evidence | Value |
|---|---|
| Walkthrough workflow run | `30239602412` |
| Walkthrough job | `89893906819` |
| Walkthrough artifact ID | `8642773653` |
| Artifact name | `v1-documentation-walkthrough-8c6b48082205a3af555130eec1f3e7d2ac8811fe-1` |
| Artifact digest | `sha256:9b954ada86f86e5da4966951039af9dddc2eddb3d49c996d09256e4cad598338` |
| Candidate identity | Matched active candidate |
| Binary identities | Matched candidate qualification evidence |
| Procedure result | 16 of 16 passed |
| Bundle checksum verification | All 34 `SHA256SUMS` entries verified |
| Independent artifact verification | Downloaded ZIP digest matched GitHub artifact digest |
| Artifact expiration | 2026-10-25 |

The walkthrough executed or cross-referenced `DOC-01` through `DOC-16` without failed, blocked, or pending procedures.

## 6. Mandatory qualification gates

| Gate | Status | Evidence | Deviations / disposition |
|---|---|---|---|
| Object lifecycle end-to-end | Passed | qualification run `30238378797` | None |
| External protocol conformance | Passed | qualification run and walkthrough `DOC-08` | None |
| Supported legacy-state migration | Passed | qualification migration gate; walkthrough `DOC-13` | None |
| Backup verification and isolated restore | Passed | qualification and walkthrough `DOC-09` through `DOC-11` | None |
| Index verify and rebuild | Passed | qualification and walkthrough `DOC-12` | None |
| Replacement and cleanup crash matrix | Passed | qualification and walkthrough `DOC-15` | None |
| Standard Rust validation | Passed | candidate qualification and repository CI | None |
| JavaScript validation | Passed | candidate qualification and repository CI | None |
| Publisher signature regressions | Passed | walkthrough `DOC-08` and `DOC-16` | No bypass found |
| Security release-blocker review | Passed | `V1_0_SECURITY_DIFF_REVIEW.md` | Artifact-bound follow-up complete |
| Installation/configuration/operations review | Passed | walkthrough run `30239602412` | None |
| Upgrade/rollback/recovery review | Passed | walkthrough `DOC-13`, `DOC-15`, `DOC-16` | Cross-referenced where appropriate |
| Privileged reference-host preflight | Pending | Issue #335 | Not yet executed |
| v1.0 qualification soak | Pending | soak tracking issue | Not started |

## 7. Signature-enforcement evidence

The active walkthrough confirms:

- the checked-in valid signature vector passes ingestion;
- changed signature bytes fail before ingestion;
- malformed signature material follows the stable malformed-security path;
- verifier operational errors fail closed;
- conformance vectors include valid, tampered, malformed-public-key, and malformed-signature requests;
- duplicate and conflict behavior remains explicit and cannot be reached by bypassing signature verification.

No successful stored, duplicate, conflict, quarantine, or raw-append result was observed for a signature failure.

## 8. Compatibility confirmation

| Contract family | Active candidate disposition | Evidence |
|---|---|---|
| Protocol and schema | Compatible with intentional signature enforcement | conformance and signed vectors |
| Canonical serialization and identifiers | Compatible | lifecycle and workspace tests |
| Digest and signature payload | Intentional security correction | security review and walkthrough |
| Public Rust API | No intentional incompatible change | Rust validation |
| HTTP and operator CLI | Compatible | walkthrough |
| Diagnostics and machine-readable errors | Stable security codes added | walkthrough and tests |
| Configuration | Compatible | walkthrough |
| Storage and durable artifacts | Unchanged | migration, backup/restore, index gates |
| Migration and rollback | Compatible | qualification and walkthrough |

No compatibility exception or waiver is required.

## 9. Documentation freeze

| Document area | Status | Evidence |
|---|---|---|
| Installation | Passed | `DOC-02`, `DOC-03`, `DOC-06` |
| Configuration | Passed | `DOC-04` |
| Operations | Passed | `DOC-05` through `DOC-12` |
| Upgrade and rollback | Passed | `DOC-13` |
| Recovery and troubleshooting | Passed | `DOC-11`, `DOC-15`, `DOC-16` |
| Compatibility policy | Passed | security/compatibility review |
| README and documentation indexes | Passed | repository CI |
| Current implementation status | Pending release sync | release PR |
| Release checklist | Pending | release PR |
| Release notes | Pending | release PR |
| CHANGELOG | Pending | release PR |

**Documentation freeze gate: PASS for the active candidate.**

## 10. Soak result

| Field | Value |
|---|---|
| Qualified commit | `8c6b48082205a3af555130eec1f3e7d2ac8811fe` |
| Reference-host preflight | Pending |
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
| Panic/abort/OOM count | Pending |
| Canonical corruption count | Pending |
| Object/index divergence count | Pending |
| Unrecoverable injected failures | Pending |
| Soak artifact identity | Pending |
| Final soak decision | Pending |

## 11. Final release validation

| Validation | Status | Evidence |
|---|---|---|
| Active candidate exact-SHA qualification | Passed | run `30238378797`; artifact `8642393171` |
| Active candidate security review | Passed | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Active candidate documentation walkthrough | Passed | run `30239602412`; artifact `8642773653` |
| Privileged reference-host preflight | Pending | Pending |
| 72-hour soak | Pending | Pending |
| Reviewed release PR checks | Pending | Pending |
| Merged-commit standard CI | Pending | Pending |
| Merged-commit candidate qualification | Pending | Pending |
| Version consistency | Pending | Pending |
| Tag points to merged release commit | Pending | Pending |
| GitHub Release points to annotated tag | Pending | Pending |
| Published artifacts match recorded digests | Pending | Pending |

## 12. Deviations and residual risks

No release-blocking deviation was observed in active-candidate qualification or documentation walkthrough.

Accepted process residual risks remain:

- third-party GitHub Actions use reviewed version tags rather than immutable commit SHAs;
- GitHub Actions artifact retention is finite, while artifact identities, bundle checksums, and binary digests are recorded permanently in this repository.

Remaining planned work is not classified as a deviation:

- privileged reference-host preflight and disk-pressure rehearsal;
- formal 72-hour qualification soak;
- version preparation and publication validation.

## 13. Final decision

**Decision: Pending**

The active candidate qualification, security review, compatibility review, and documentation freeze are green. The repository is **GO for privileged reference-host preflight** and remains **NO-GO for formal soak** until that preflight passes. A final release `PASS` requires the soak, version preparation, merged-commit validation, tag, GitHub Release, and publication evidence.