# Lingonberry v1.0.0 Qualification Status

**Status: pre-candidate qualification active** | **Target release: v1.0.0** | **Parent issue: #109** | **Last updated: 2026-07-23**

## 1. Purpose

This document records the current execution state of the normative qualification plan in `V1_0_QUALIFICATION_PLAN.md`.

It does not replace the qualification plan, compatibility policy, soak plan, or final release evidence. Its purpose is to distinguish completed qualification infrastructure from evidence that must still be rerun against the designated final v1.0.0 candidate commit.

## 2. Completed foundation work

The following pre-candidate work is complete and merged to `main`:

| Work item | Status | Evidence |
|---|---|---|
| Gate inventory and qualification plan | Complete | Issue #110, PR #111, `V1_0_QUALIFICATION_PLAN.md` |
| Rust exported-surface audit | Complete | Issue #112, PR #115, `V1_0_RUST_API_AUDIT.md` |
| Normative v1 compatibility policy | Complete | Issue #113, PR #116, `V1_COMPATIBILITY_POLICY.md` |
| Qualification soak and telemetry contract | Plan complete; execution pending | Issue #114, PR #117, `V1_0_SOAK_PLAN.md` |
| Candidate-bound qualification workflow | Complete and dry-run verified | Issue #118, PR #119, `v1-candidate-qualification.yml` |
| Pre-candidate security diff review | Complete; final candidate verification pending | PR #121, `V1_0_SECURITY_DIFF_REVIEW.md` |

## 3. Dry-run evidence boundary

The candidate qualification workflow was dry-run successfully before final candidate selection.

Dry-run evidence:

- workflow run: `v1 candidate qualification` run 3
- qualified commit: `a4bbc622c8be4c140a0042139660a4053bd39d7f`
- artifact ID: `8535314810`
- artifact digest: `sha256:649dc1a9ef3bab069b4d531e2edbc6d195e47053c2a8ec881798492d3e21e546`
- result: 12 recorded gates passed and artifact checksums verified

This proves that the orchestrator and evidence format work. It is not final v1.0.0 release evidence because the qualified commit predates the final candidate designation and the final soak.

## 4. Current gate state

| Gate family | Current state | Required next evidence |
|---|---|---|
| Stable compatibility declarations | Complete | Reconfirm no candidate diff contradicts the approved policy. |
| Standard Rust, JavaScript, and external conformance | Infrastructure ready | Rerun against the designated final candidate. |
| Lifecycle, migration, recovery, backup/restore, and index | Infrastructure ready | Rerun against the designated final candidate and retain the bundle. |
| Replacement and cleanup crash matrix | Infrastructure ready | Rerun against the designated final candidate. |
| Security regressions and release-blocker review | Pre-candidate diff reviewed; final disposition pending | Recompare the designated candidate, rerun regressions, review retained logs, and record final finding counts. |
| Reference-platform operator acceptance | Infrastructure ready | Run against final candidate-built binaries and frozen documentation. |
| Documentation review | Pending final candidate | Walk installation, configuration, operation, upgrade, rollback, recovery, and troubleshooting instructions. |
| 72-hour qualification soak | Plan ready; not executed | Execute under `V1_0_SOAK_PLAN.md` and retain telemetry and logs. |
| Version and release-document preparation | Blocked by pre-version gates | Begin only after candidate qualification, operator acceptance, security review, and soak are green. |
| Publication | Not started | Requires reviewed release PR, merged-commit validation, tag, release, and final evidence. |

## 5. Candidate designation rule

A commit may be designated as the v1.0.0 qualification candidate only when:

1. all intended runtime and contract changes are merged;
2. no unrelated feature work is included;
3. compatibility documents are frozen for candidate review;
4. the candidate qualification workflow exists on that commit;
5. resource ceilings and soak stop thresholds are fixed before execution;
6. any change after designation is classified as runtime-affecting or evidence-only.

A runtime-affecting change invalidates candidate-bound executable evidence and requires a new candidate designation and rerun.

## 6. Remaining execution order

1. Complete candidate-diff contradiction and security review.
2. Freeze the candidate documentation set.
3. Designate the final candidate commit.
4. Run the candidate-bound qualification workflow.
5. Run reference-platform operator acceptance from the frozen instructions.
6. Execute the 72-hour qualification soak.
7. Record deviations, residual risks, and final pass/fail disposition.
8. Prepare version `1.0.0`, release checklist, release notes, and changelog.
9. Validate the reviewed release PR and merged commit.
10. Tag and publish only after all evidence points to the same commit and contract set.

## 7. Release-blocking state

v1.0.0 remains blocked until all of the following are true:

- final candidate qualification is green;
- final security disposition has no Critical, High, or release-blocking Medium finding;
- reference-platform acceptance is green;
- the 72-hour soak is green;
- documentation review is complete;
- `V1_0_RELEASE_EVIDENCE.md` contains commit-bound evidence for every mandatory gate;
- release version, tag, notes, artifacts, and compatibility declarations identify the same release.
