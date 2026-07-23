# v1.0.0 Candidate-Diff Security Review

**Status: final candidate-bound review complete** | **Release target: v1.0.0** | **Baseline: v0.9.0 (`971155340603afdc0c9c5bd37e596f49c260d15e`)** | **Candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`** | **Tracking issue: #130** | **Last updated: 2026-07-23**

## 1. Purpose

This record is the final security and compatibility disposition for the designated pre-version v1.0.0 candidate. It supplements:

- `docs/security/V0_9_SECURITY_REVIEW.md`
- `docs/security/V0_9_SECURITY_FINDINGS.md`
- `docs/architecture/V1_COMPATIBILITY_POLICY.md`
- `docs/architecture/V1_0_RUST_API_AUDIT.md`
- `docs/roadmap/V1_0_RELEASE_EVIDENCE.md`

The disposition is valid only for the exact candidate SHA above. Any runtime-affecting, protocol, durable-format, CLI/HTTP contract, migration, recovery, or security-control change requires a new candidate review and new executable evidence.

## 2. Final diff scope

Comparison:

```text
base: v0.9.0 / 971155340603afdc0c9c5bd37e596f49c260d15e
candidate: f9543019f2c219aea3b085ff90f2da201b268a48
commits ahead: 53
changed files: 23
```

The changed files are limited to:

- documentation and documentation indexes;
- GitHub Actions qualification and documentation-integrity workflows;
- qualification, documentation-check, and Rust public-API inventory scripts.

No file under `packages/**`, `conformance/**`, `tests/**`, protocol fixtures, canonicalization implementation, storage implementation, migration runtime, relay runtime, HTTP handlers, or operator CLI implementation changed between v0.9.0 and the candidate.

**Disposition: no production runtime, protocol, storage, migration, CLI, or HTTP implementation delta.**

The v0.9.0 runtime security review remains applicable, while candidate-bound regression and operator evidence are supplied by qualification run `29971797941`.

## 3. Candidate-bound qualification evidence

| Evidence | Value |
|---|---|
| Workflow | `v1 candidate qualification` run 6 |
| Run ID | `29971797941` |
| Artifact ID | `8549953270` |
| Artifact digest | `sha256:cc216536a29acbc65ba7b25e74f1e2198c7050605019ea3a09c1ddab0fb18b7b` |
| Candidate SHA in artifact | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Recorded gates | 12 passed, 0 failed |
| Bundle checksums | all 32 `SHA256SUMS` entries independently verified |
| Relay binary SHA-256 | `9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d` |
| Storage binary SHA-256 | `22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0` |

The qualification covered Rust formatting and clippy, workspace tests, storage migration and recovery, index consistency, core lifecycle, JavaScript tests, external conformance, replacement and cleanup crash points, release builds, and installed-binary operator acceptance.

## 4. Security review findings

### 4.1 Runtime security delta

**Disposition: acceptable; no delta detected.**

No new parser, canonicalization, signature, authorization, path mutation, storage, migration, recovery, indexing, or destructive-operation implementation entered after v0.9.0.

### 4.2 Workflow permissions

**Disposition: acceptable.**

Qualification workflows use:

```yaml
permissions:
  contents: read
```

They cannot modify repository contents, publish tags or releases, write packages, update deployments, or mutate issues and pull requests through the workflow token.

### 4.3 Candidate identity binding

**Disposition: passed.**

The workflow checks out the explicit candidate SHA, records it in `summary.json`, and fails unless the recorded SHA equals the workflow candidate. Qualification-only PR #128 used the designated candidate itself as `pull_request.head.sha` and was closed without merge after artifact inspection.

### 4.4 Command and path injection

**Disposition: acceptable.**

The orchestrator uses a fixed command list, quoted paths, `set -euo pipefail`, and no `eval` or dynamically constructed shell program. Pull-request titles, bodies, branch names, labels, and commit messages are not executed as shell input.

### 4.5 Failure propagation and ambiguous success

**Disposition: passed.**

A non-zero gate stops qualification. The workflow separately requires:

- aggregate status `passed`;
- every gate status `passed`;
- candidate SHA equality;
- a non-empty checksum manifest;
- successful verification of the entire evidence bundle.

Partial evidence is retained on failure through `if: always()` rather than silently omitted.

### 4.6 Artifact integrity and retention

**Disposition: passed with accepted process residual risk.**

The artifact is self-contained and checksum-verifiable. Its GitHub retention is finite, but the artifact ID, artifact digest, binary digests, candidate SHA, toolchain, and gate references are retained in repository release evidence.

### 4.7 Log and sensitive-data review

**Disposition: passed.**

All retained qualification logs were scanned for panic, abort, OOM, credential-shaped output, secret-bearing configuration, and unexplained failure markers.

Results:

- panic matches: 0;
- abort matches: 0;
- OOM matches: 0;
- credential or bearer-token output: 0;
- secret-pattern matches: 4 benign references only.

The four benign matches were test names or outputs asserting secret-free behavior: `appends_canonical_secret_free_events`, `containsSecrets:false`, and `report_schema_never_contains_secret_fields`. No credential or sensitive value was present.

Temporary runner paths and expected negative-test failure text appear in logs. They are not credentials and do not reveal operator production paths.

### 4.8 Supply-chain residual risk

**Disposition: accepted Low risk; not release-blocking.**

GitHub-hosted runners, Rust and Node distributions, and third-party Actions are referenced through reviewed version tags rather than immutable commit SHAs. This matches repository-wide practice but is weaker than complete immutable pinning.

Risk controls for v1.0.0:

1. qualification workflow source is frozen for the candidate;
2. runner OS and toolchain versions are retained;
3. release binaries are independently hashed;
4. artifact-level and bundle-level digests are retained;
5. the final merged release commit must be revalidated before publication.

Changing all Actions to immutable SHAs is suitable follow-up hardening, but introducing that repository-wide change after candidate designation is not required for v1.0.0.

## 5. Compatibility disposition

The candidate introduces no implementation change in any approved v1 compatibility family.

| Contract family | Candidate delta | Disposition | Evidence |
|---|---|---|---|
| Protocol and schema | No implementation or fixture change | Compatible | v0.9.0 comparison; external conformance passed |
| Canonical serialization and identifiers | No implementation change | Compatible | core lifecycle and workspace tests passed |
| Digest and signature payload | No implementation change | Compatible | workspace security regressions passed |
| Public Rust API | No runtime API source change after audit | Compatible | `V1_0_RUST_API_AUDIT.md`; Rust gates passed |
| HTTP and operator CLI | No handler or CLI implementation change | Compatible | installed-binary operator acceptance passed |
| Diagnostics and machine-readable errors | No implementation change | Compatible | operator acceptance and workspace tests passed |
| Configuration | No implementation or default change | Compatible | operator acceptance passed |
| Storage and durable artifacts | No storage-format implementation change | Compatible | migration/recovery, backup/restore, and index gates passed |
| Migration and rollback | No migration implementation change | Compatible | storage migration/recovery gate passed |

No compatibility exception, waiver, or deprecation is required for the candidate.

## 6. Findings summary

| ID | Area | Severity | Status | Disposition |
|---|---|---:|---|---|
| V1-DIFF-001 | Runtime implementation delta | Informational | closed | No production implementation changed after v0.9.0. |
| V1-DIFF-002 | Candidate SHA binding | Medium if absent | closed | Exact candidate qualified and artifact identity independently verified. |
| V1-DIFF-003 | Evidence checksum working directory | Low | closed | Initial dry-run defect corrected before candidate designation. |
| V1-DIFF-004 | Action references use version tags | Low | accepted | Controlled through provenance, artifact digest, binary hashes, and final revalidation. |
| V1-DIFF-005 | Expiring Actions artifact | Low | accepted-process-risk | Permanent identifying digests and evidence references are retained in-repository. |

Final open finding counts:

| Severity | Open | Release-blocking |
|---|---:|---:|
| Critical | 0 | 0 |
| High | 0 | 0 |
| Medium | 0 | 0 |
| Low | 2 accepted process residual risks | 0 |

## 7. Final disposition

**Security disposition: PASS for candidate-bound review.**

**Compatibility disposition: PASS for candidate-bound review.**

There is no identified security or compatibility blocker preventing the candidate from proceeding to the documentation walkthrough and 72-hour soak.

This is not the final release decision. The release remains blocked until the documentation walkthrough, soak, residual-risk review, version preparation, merged-release-commit validation, and publication evidence are complete.