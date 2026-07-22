# Lingonberry v1.0.0 Documentation Walkthrough

**Status: pre-candidate static review active; executable walkthrough pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #124** | **Last updated: 2026-07-23**

## 1. Purpose

This document is the working classification record required by `V1_0_DOCUMENTATION_FREEZE_PLAN.md`.

It separates evidence that can be established before candidate designation from commands and procedures that must be executed against release-built binaries from the designated v1.0.0 candidate commit.

This document does not pass the documentation gate by itself. Final `EXECUTED` classifications, observed outputs, exit statuses, candidate SHA, and binary digests must be recorded after candidate designation.

## 2. Review subject

Pre-candidate static review subject:

- branch basis: `main`
- reference release: `v0.9.0`
- runtime delta from `v0.9.0`: none as recorded in `V1_0_SECURITY_DIFF_REVIEW.md`
- reference platform contract: Ubuntu Server 24.04 LTS, x86_64, systemd
- final candidate SHA: `Pending`
- `lingonberry-storage` SHA-256: `Pending`
- `lingonberry-relay` SHA-256: `Pending`
- review environment: static repository review; executable reference-host walkthrough pending

## 3. Classification rules

- `EXECUTED`: procedure was run using candidate-built binaries on the reference environment.
- `INSPECTED`: static file, configuration, implementation-facing contract, or unit was checked directly.
- `CROSS_REFERENCED`: normative claim was reconciled with another canonical document and available test evidence.
- `NOT_APPLICABLE`: item does not apply to v1.0.0 and the reason is recorded.
- `BLOCKED`: item is stale, ambiguous, unsafe, incomplete, contradictory, or not reproducible.
- `PENDING_EXECUTION`: temporary pre-candidate state used here for work that must become `EXECUTED`, `NOT_APPLICABLE`, or `BLOCKED` before freeze. It is not a final classification permitted by the freeze plan.

## 4. File and contract review

| Family | Source | Current classification | Pre-candidate finding | Candidate action |
|---|---|---|---|---|
| Repository entry point | root `README.md` | `INSPECTED` | Active v1 sources are separated from historical v0.9 evidence; dry-run boundary is explicit. | Confirm all documented operator commands match candidate help and observed behavior. |
| Roadmap index | `docs/roadmap/README.md` | `INSPECTED` | Active v1 execution documents precede historical release records. | Confirm status and remaining sequence after candidate designation. |
| Operations index | `docs/operations/README.md` | `INSPECTED` | Active v1 qualification sources and canonical operator path are exposed without replacing the v0.8 baseline. | Confirm operator path against candidate-built binaries. |
| Reference platform | `docs/operations/SUPPORTED_PLATFORMS.md` | `CROSS_REFERENCED` | Ubuntu Server 24.04 LTS, x86_64, systemd agrees with operator acceptance and qualification workflow runners. | Record exact host image, kernel, systemd, filesystem, and resource limits. |
| Systemd units | `deploy/systemd/lingonberry-storage-ready.service`, `deploy/systemd/lingonberry-relay.service` | `INSPECTED` | Required files exist and are included in documentation-freeze integrity checks. | Run `systemd-analyze verify`; install and start with documented binary paths and permissions. |
| Operator runbook | `docs/operations/V0_8_OPERATOR_RUNBOOK.md` | `CROSS_REFERENCED` | Remains the single-node operational baseline and is referenced by README and qualification plans. | Execute every mandatory procedure from a fresh workspace without source-tree-only knowledge. |
| CLI contract | `docs/operations/OPERATOR_CLI_CONTRACT.md` | `CROSS_REFERENCED` | Command names, machine-readable outputs, diagnostic codes, and exit status are protected by the v1 compatibility policy. | Compare candidate `--help`, success output, and representative failures with the contract. |
| Upgrade and rollback | `docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md` | `CROSS_REFERENCED` | Explicit migration, verified backup, unknown-newer-format rejection, and rollback boundaries agree with the v1 policy. | Execute supported prior-state upgrade and verify documented rollback boundary. |
| Storage migration | `docs/operations/STORAGE_MIGRATION_AND_UPGRADE.md` | `CROSS_REFERENCED` | Ordinary startup must not migrate; mutation requires inspect, plan, verified backup, authorization, verification, and commit. | Run supported migration matrix and record output, status, resume, rollback, and unsupported downgrade behavior. |
| Backup and restore | operator runbook and storage contract | `CROSS_REFERENCED` | Active, non-empty, partial, corrupt, and symlink targets must fail closed; isolated restore must be independently verified. | Execute backup create/verify, restore plan/apply, unsafe-target rejection, read/write/index/cleanup verification. |
| Index lifecycle | `packages/indexer/INDEX_LIFECYCLE.md` and operator runbook | `CROSS_REFERENCED` | Canonical storage remains authoritative; derived state is verifiable and rebuildable. | Verify intact state, inject stale/invalid derived state, rebuild, and reverify. |
| HTTP contracts | `docs/operations/HTTP_CARRIER_CONTRACT.md`, protocol HTTP documents | `CROSS_REFERENCED` | Documented statuses and machine-readable codes are stable where classified by the compatibility policy. | Exercise representative success, invalid input, authorization, conflict, not-found, and internal failure paths. |
| Quarantine administration | quarantine admin, status, annotation, dismissal, rejection docs | `CROSS_REFERENCED` | Authorization and subject resolution must precede mutation; contradictory state fails closed. | Execute read-only inspection and representative authorized/unauthorized operations. |
| Replacement | replacement policy, preview, transaction, generation, recovery docs | `CROSS_REFERENCED` | Preview/proof/subject binding and generation publication rules agree with crash-matrix evidence. | Execute normal replacement plus resume/rollback at documented crash points. |
| Cleanup | retention policy and cleanup runbook | `CROSS_REFERENCED` | Cleanup is proof-bound and must not rewrite archive segments or immutable evidence ledgers. | Execute eligible, ineligible, stale-proof, contradictory-state, interruption, resume, and rollback cases. |
| Recovery and diagnostics | operator runbook, recovery docs, observability docs | `CROSS_REFERENCED` | Troubleshooting must not recommend proof bypass, direct canonical mutation, or ambiguous success. | Record `status`, `health`, `ready`, `doctor`, `verify`, `metrics`, journal output, diagnostic codes, and exit statuses. |
| Compatibility | `docs/architecture/V1_COMPATIBILITY_POLICY.md` | `CROSS_REFERENCED` | Normative v1.x policy covers protocol, Rust API, HTTP/CLI, durable state, migration, error compatibility, and deprecation. | Confirm candidate diff contains no undisposed compatibility change. |
| Rust API | `docs/architecture/V1_0_RUST_API_AUDIT.md` | `CROSS_REFERENCED` | Exported surfaces are classified; no removal or rename is required for v1.0.0. | Regenerate inventory if candidate changes Rust public exports. |
| Security | `docs/security/V1_0_SECURITY_DIFF_REVIEW.md` | `CROSS_REFERENCED` | Pre-candidate review found no production runtime delta and no Critical, High, or release-blocking Medium finding. | Recompare final candidate, inspect qualification logs, operator evidence, and soak results. |
| Qualification | qualification plan, status, workflow, release evidence | `CROSS_REFERENCED` | Dry-run evidence is explicitly not final release evidence. | Record candidate-bound workflow run, artifact digest, all gate results, and binary hashes. |
| Soak | `docs/roadmap/V1_0_SOAK_PLAN.md` | `CROSS_REFERENCED` | Duration, workload floors, telemetry, stop conditions, and evidence bundle are fixed before candidate execution. | Complete 72 hours and minimum workload with no blocker and retained checksummed evidence. |

## 5. Candidate execution matrix

Every row below is `PENDING_EXECUTION` until candidate-built binaries are available.

| ID | Procedure | Command or action | Expected result | Evidence to retain | Current state |
|---|---|---|---|---|---|
| DOC-01 | Verify platform | `uname -s`, `uname -m`, `/etc/os-release`, `systemctl --version` | Linux, x86_64, Ubuntu 24.04, systemd available | command output | `PENDING_EXECUTION` |
| DOC-02 | Verify binaries | install release-built storage and relay binaries; record `sha256sum` | binaries executable and digests match candidate manifest | paths, modes, digests | `PENDING_EXECUTION` |
| DOC-03 | Verify units | `systemd-analyze verify deploy/systemd/*.service` | no unit verification error | full output | `PENDING_EXECUTION` |
| DOC-04 | Configuration | `lingonberry-storage config` plus file/env/CLI precedence cases | deterministic effective configuration; secrets redacted; invalid contradictions fail | output and exit status | `PENDING_EXECUTION` |
| DOC-05 | Read-only diagnostics | `health`, `ready`, `status`, `doctor`, `verify`, `metrics` | documented machine-readable status; no mutation | output and exit status | `PENDING_EXECUTION` |
| DOC-06 | Service lifecycle | install units, daemon-reload, start, restart, stop, inspect journal | stable startup and shutdown without undocumented step | unit state and journal | `PENDING_EXECUTION` |
| DOC-07 | Publish and restart | publish minimal valid object; list/retrieve before and after restart | persisted object remains identical across process restart | request, response, comparison | `PENDING_EXECUTION` |
| DOC-08 | Invalid and conflict handling | malformed, oversized, deeply nested, invalid signature, duplicate, conflict inputs | bounded deterministic failure; no invalid canonical write or conflict overwrite | output, code, storage check | `PENDING_EXECUTION` |
| DOC-09 | Backup | create and verify backup | archive verifies and is bound to source state | manifest, verify output, digest | `PENDING_EXECUTION` |
| DOC-10 | Isolated restore | plan/apply restore to empty isolated target | restored status and independent read/write/index/cleanup verification pass | output and target verification | `PENDING_EXECUTION` |
| DOC-11 | Unsafe restore targets | active target, non-empty target, symlink target, partial/corrupt archive | fail closed; no target mutation or partial accepted state | output, exit status, filesystem check | `PENDING_EXECUTION` |
| DOC-12 | Index verify/rebuild | verify intact index; create stale derived state; rebuild; reverify | stale state detected; deterministic rebuild agrees with canonical storage | outputs and digests | `PENDING_EXECUTION` |
| DOC-13 | Migration | inspect, plan, backup verify, apply, status, resume/rollback as applicable | only supported states migrate; no implicit mutation; newer format rejected | source fixture, outputs, post-state | `PENDING_EXECUTION` |
| DOC-14 | Quarantine operations | inspect and representative admin actions | authorization, proof, subject, and state checks match docs | requests, responses, codes | `PENDING_EXECUTION` |
| DOC-15 | Replacement and cleanup | preview, prepare, publish, interruption, resume, rollback, cleanup | only documented recoverable states; no proof bypass or unauthorized destructive action | logs, proof, state snapshots | `PENDING_EXECUTION` |
| DOC-16 | Failure diagnosis | induce documented configuration, storage, index, archive, and recovery failures | diagnostic code, exit status, log detail, and remediation match docs | commands, outputs, remediation result | `PENDING_EXECUTION` |

## 6. Contradiction ledger

| ID | Sources | Question | Current disposition | Blocker |
|---|---|---|---|---|
| CONTR-01 | root README, roadmap index, operations index | Are current v1 sources distinguishable from v0.9 historical evidence? | Resolved by linking active v1 sources first and preserving a separate historical section. | No |
| CONTR-02 | qualification status, workflow dry-run record, release evidence | Can the dry run be interpreted as final qualification? | Resolved: all three explicitly state that final candidate-bound evidence is still required. | No |
| CONTR-03 | compatibility policy, migration docs, runbook | Can ordinary startup migrate storage implicitly? | Resolved: prohibited; migration is explicit, backup-bound, verified, and operator-authorized. | No |
| CONTR-04 | index docs, storage docs, README | Is the index a semantic source of truth? | Resolved: canonical storage is authoritative; index is derived, verifiable, and rebuildable. | No |
| CONTR-05 | troubleshooting, replacement, cleanup, security policy | May an operator bypass proof or mutate canonical state manually during recovery? | Resolved: prohibited. Unknown or contradictory state must fail closed and escalate. | No |
| CONTR-06 | supported platform, workflows, systemd docs | What is the v1.0 reference platform? | Resolved: Ubuntu Server 24.04 LTS, x86_64, systemd. | No |
| CONTR-07 | v1 policy, operator CLI/HTTP docs | Which prose and ordering details are compatibility commitments? | Resolved: machine codes/statuses and documented required structure are protected; free-form prose/debug layout/order are not stable unless explicitly stated. | No |

Any new contradiction discovered during candidate execution must be added here and marked `Blocker: Yes` until one normative source is selected and all dependent instructions are updated.

## 7. Diagnostic and exit-status observation record

This table must be completed from candidate execution rather than copied from prior releases.

| Command or API | Scenario | Expected code/status | Observed code/status | Output contract match | State |
|---|---|---|---|---|---|
| `lingonberry-storage health` | healthy | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| `lingonberry-storage ready` | ready / not ready | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| `lingonberry-storage doctor` | healthy and representative failure | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| `lingonberry-storage backup verify` | valid and corrupt archive | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| `lingonberry-storage restore apply` | valid isolated and unsafe target | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| `lingonberry-storage index verify` | intact and stale/invalid index | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| migration CLI | supported, unsupported, newer format | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| relay publish/API | valid, invalid, duplicate, conflict | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |
| quarantine admin API | authorized and unauthorized | Pending contract lookup | Pending | Pending | `PENDING_EXECUTION` |

## 8. Freeze completion checklist

- [x] Active v1 sources are reachable from root, roadmap, and operations indexes.
- [x] The dry-run evidence boundary is explicit.
- [x] All mandatory documentation families have a review row.
- [x] Pre-candidate contradictions identified so far have a normative disposition.
- [x] Required files and local links are covered by `v1 documentation freeze check`.
- [ ] Final candidate SHA and binary digests are recorded.
- [ ] Every `PENDING_EXECUTION` item is replaced with a final allowed classification.
- [ ] Commands, observed output, diagnostic codes, and exit statuses are recorded.
- [ ] No unresolved contradiction or `BLOCKED` item remains.
- [ ] The final PASS/FAIL disposition is copied into `V1_0_RELEASE_EVIDENCE.md`.

## 9. Current disposition

Pre-candidate documentation indexing, static classification, and contradiction review are complete enough to prepare the executable walkthrough.

The documentation gate remains **not passed**. Candidate designation, candidate-built binary execution, observed diagnostics, and final freeze evidence are still mandatory before the 72-hour soak begins.
