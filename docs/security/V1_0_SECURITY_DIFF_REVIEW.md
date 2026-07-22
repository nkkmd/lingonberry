# v1.0.0 Candidate-Diff Security Review

**Status: pre-candidate review complete; final candidate verification pending** | **Release target: v1.0.0** | **Baseline: v0.9.0 (`971155340603afdc0c9c5bd37e596f49c260d15e`)** | **Reviewed head: `57ad22bad0d85091b2e19856a6f224aa089682a0`** | **Last updated: 2026-07-23**

## 1. Purpose

This record reviews the security-relevant difference between the published v0.9.0 baseline and the current v1.0 qualification head before a final candidate is designated.

It supplements, but does not replace:

- `docs/security/V0_9_SECURITY_REVIEW.md`
- `docs/security/V0_9_SECURITY_FINDINGS.md`
- `docs/architecture/V1_COMPATIBILITY_POLICY.md`
- `docs/roadmap/V1_0_QUALIFICATION_PLAN.md`
- `docs/roadmap/V1_0_SOAK_PLAN.md`

A final candidate-bound review is still required after the candidate commit is fixed. Any runtime-affecting change after that point invalidates the candidate-bound disposition.

## 2. Diff scope

Comparison:

```text
base: v0.9.0 / 971155340603afdc0c9c5bd37e596f49c260d15e
head: 57ad22bad0d85091b2e19856a6f224aa089682a0
commits ahead: 35
```

The compared files are limited to:

- documentation
- GitHub Actions workflows
- qualification and public-API inventory shell scripts

No file under `packages/**`, `conformance/**`, `tests/**`, protocol fixtures, storage format implementation, migration implementation, relay implementation, or canonicalization implementation changed in this interval.

Therefore, this diff does not introduce a new runtime parser, signature, authorization, storage, migration, recovery, index, or operator mutation path. Existing v0.9 runtime security evidence remains applicable subject to final candidate reruns.

## 3. New trust boundaries introduced by qualification infrastructure

| Boundary | Untrusted or variable input | Protected asset | Required behavior |
|---|---|---|---|
| Pull-request workflow checkout | PR head and GitHub event metadata | candidate identity and evidence validity | Explicitly checkout the PR head SHA; never substitute the temporary merge SHA as the qualified candidate. |
| Shell qualification orchestrator | repository paths, environment variables, command exit status | host workspace, evidence bundle, release decision | Fixed commands, strict shell mode, clean checkout requirement, quoted paths, immediate failure propagation. |
| GitHub Actions token | workflow execution context | repository contents and release state | `contents: read` only; no tag, release, issue, package, or deployment write permission. |
| Artifact creation | logs, manifests, binaries, generated JSON | evidence integrity and confidentiality | No secrets; candidate and binary digest binding; checksummed self-contained bundle; bounded retention. |
| Tool installation | pinned Actions references and Rust/Node toolchains | build reproducibility and supply-chain integrity | Use reviewed Actions references; record toolchain versions; final evidence must retain workflow run and artifact digest. |

## 4. Review findings

### 4.1 Runtime implementation delta

**Disposition: no runtime security delta detected.**

The v0.9.0-to-reviewed-head comparison contains no modifications to production Rust or JavaScript implementation. Runtime security invariants defined by the v0.9 review are neither weakened nor bypassed by this diff.

This is not a substitute for candidate-bound regression execution. Parser limits, canonicalization, identity/signature verification, authorization ordering, path handling, migration, recovery, index, and destructive-operation regressions remain mandatory in final qualification.

### 4.2 Workflow permissions

**Disposition: acceptable.**

The added workflows use read-only repository permissions:

```yaml
permissions:
  contents: read
```

They do not request write permission for contents, Actions, packages, deployments, pull requests, issues, attestations, or releases. Qualification cannot publish a tag or release through the provided token.

### 4.3 Candidate commit binding

**Disposition: acceptable after dry-run correction.**

The qualification workflow explicitly checks out the pull-request head SHA and verifies that the generated `summary.json` identifies that same SHA. This prevents GitHub's synthetic pull-request merge commit from being misrepresented as the candidate.

Final-candidate execution must be dispatched or triggered against the exact candidate commit. The release evidence must record:

- candidate SHA
- workflow run ID and attempt
- artifact ID and digest
- binary SHA-256 values

### 4.4 Command and path injection

**Disposition: acceptable for the reviewed implementation.**

The orchestrator executes a fixed command list rather than evaluating repository-controlled command strings. Paths are derived from a fixed output root or quoted variables. It uses `set -euo pipefail` and does not use `eval`, unquoted command substitution as a command, dynamically constructed shell programs, or archive extraction into privileged paths.

The workflow does not interpolate user-supplied pull-request text, branch names, issue content, labels, or commit messages into shell commands.

### 4.5 Failure propagation and ambiguous success

**Disposition: acceptable.**

Each gate records exit status and a machine-readable result. A non-zero gate stops qualification. The workflow separately verifies that:

- `summary.status` is `passed`
- every listed gate is `passed`
- the candidate SHA matches the checked-out head SHA
- the bundle checksum verifies

Artifacts are uploaded with `if: always()` so a failed run can retain partial logs and results instead of silently omitting the failed gate.

### 4.6 Evidence integrity

**Disposition: acceptable for dry-run infrastructure; final retention action pending.**

The evidence bundle contains:

- candidate manifest
- commit and platform provenance
- Rust, Cargo, and Node versions
- release-built binaries
- binary digests using bundle-relative paths
- gate-level JSON results and logs
- aggregate summary
- `SHA256SUMS`

GitHub also records an artifact-level SHA-256 digest. The final release record must preserve the artifact identifier and digest outside the expiring Actions artifact itself.

### 4.7 Secret and sensitive-data exposure

**Disposition: no new secret source identified.**

The qualification workflow does not require repository secrets. Logs can contain source paths, compiler output, test names, and temporary runner paths, but no authentication token is intentionally printed. GitHub masks its token and the workflow passes no credential to tested binaries.

Before final execution, confirm that no newly introduced test or environment configuration injects credentials into command output or retained manifests.

### 4.8 Supply-chain considerations

**Disposition: residual risk retained; no release blocker at this stage.**

The workflow relies on GitHub-hosted runners, Rust and Node distributions, and third-party Actions. Current references are version tags rather than immutable commit SHAs. This matches existing repository practice but is weaker than full action-SHA pinning.

For v1.0.0, this is accepted as a process-level residual risk provided that:

1. workflow source is frozen before final qualification;
2. action versions and toolchain outputs are retained in the run record;
3. release binaries are independently hashed;
4. the merged release commit is revalidated before publication.

Changing to immutable Action SHAs may be considered separately, but it is not required as a late release-blocking change unless repository policy is changed consistently across all workflows.

## 5. Findings summary

| ID | Area | Severity | Status | Disposition |
|---|---|---:|---|---|
| V1-DIFF-001 | Runtime implementation delta | Informational | closed | No production implementation changed after v0.9.0 in the reviewed interval. |
| V1-DIFF-002 | Candidate SHA binding | Medium if absent | closed | Head SHA checkout and summary verification are implemented and dry-run tested. |
| V1-DIFF-003 | Evidence checksum working directory | Low | closed | Initial dry run found a relative-path verification defect; corrected and rerun successfully. |
| V1-DIFF-004 | Action references use version tags | Low | accepted | Existing repository practice; retain run/toolchain provenance and binary digests. |
| V1-DIFF-005 | Expiring Actions artifact | Low | open-process-item | Final release evidence must copy artifact ID and digest into non-expiring repository documentation. |

No Critical, High, or release-blocking Medium finding is open in this pre-candidate diff review.

## 6. Final candidate verification checklist

After the final candidate SHA is designated:

- [ ] Compare `v0.9.0...<candidate>` again and confirm the file set.
- [ ] Confirm no unreviewed `packages/**`, fixture, protocol, storage, migration, or operator-contract change entered after this review.
- [ ] Run the candidate qualification workflow and retain its complete artifact.
- [ ] Confirm parser, signature, authorization, migration, recovery, index, crash-matrix, and operator gates pass.
- [ ] Review all logs for panic, abort, sanitizer-like failure, secret exposure, unexplained warning, or skipped test.
- [ ] Confirm artifact candidate SHA and binary digests match the release evidence.
- [ ] Update `docs/security/V0_9_SECURITY_FINDINGS.md` or create a v1 finding record for any new issue.
- [ ] Record open Critical = 0, open High = 0, release-blocking Medium = 0.
- [ ] Re-evaluate this disposition if the candidate changes.

## 7. Current release disposition

The reviewed v0.9.0-to-main interval introduces no runtime security change and no presently identified security release blocker.

The security gate is **not yet finally passed** because the final candidate has not been designated and candidate-bound regression, log review, reference-platform acceptance, and soak evidence remain pending.
