# Lingonberry v1.0.0 Documentation Walkthrough

**Status: PASS** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #132** | **Last updated: 2026-07-23**

## 1. Purpose and evidence identity

This record completes the executable documentation-freeze walkthrough required by `V1_0_DOCUMENTATION_FREEZE_PLAN.md`.

| Field | Value |
|---|---|
| Candidate SHA | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Platform | Ubuntu 24.04.4 LTS, x86_64, systemd 255 |
| Storage SHA-256 | `22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0` |
| Relay SHA-256 | `9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d` |
| Workflow | `v1 documentation walkthrough` run 3, run ID `29974169660` |
| Artifact ID | `8550809328` |
| Artifact digest | `sha256:75adb9ce95b69307632705aa82d89ede1cf413779e11ab29e18e2a47cca56904` |
| Artifact retention | GitHub Actions through 2026-10-21 |
| Independent inspection | Candidate identity, all 16 results, binary hashes, and all 34 `SHA256SUMS` entries verified |

The walkthrough harness was reviewed in PR #133. It checked out the designated candidate separately from the harness branch, rebuilt the release binaries, verified the recorded hashes, installed them at the published paths, and retained per-procedure logs and JSON results.

## 2. Classification rules

- `EXECUTED`: the published procedure was run using candidate-built binaries on the reference environment.
- `INSPECTED`: the static document, configuration, or unit was checked directly.
- `CROSS_REFERENCED`: the normative claim was reconciled with candidate-bound automated evidence.
- `NOT_APPLICABLE`: the item does not apply and the reason is recorded.
- `BLOCKED`: the item is stale, unsafe, contradictory, or not reproducible.

No final item is `PENDING_EXECUTION` or `BLOCKED`.

## 3. File and contract review

| Family | Source | Final classification | Disposition |
|---|---|---|---|
| Repository, roadmap, and operations indexes | root and documentation indexes | `INSPECTED` | Active v1 sources remain distinct from historical v0.9 evidence. |
| Reference platform | `SUPPORTED_PLATFORMS.md` | `EXECUTED` | Ubuntu 24.04.4, x86_64, systemd 255 observed. |
| Systemd units | `deploy/systemd/*.service` | `EXECUTED` | `systemd-analyze verify`, enable, start, restart, stop, and journal inspection passed. |
| Operator runbook | `V0_8_OPERATOR_RUNBOOK.md` | `EXECUTED` | Installation, service lifecycle, publish, persistence, backup, restore, index, and diagnostics passed after correcting service-user environment loading. |
| CLI contract | `OPERATOR_CLI_CONTRACT.md` | `EXECUTED` | Machine-readable success and warning outputs, codes, and exit statuses were observed. |
| Upgrade, rollback, and migration | upgrade and storage migration documents | `CROSS_REFERENCED` | Candidate storage test suite passed; no implicit migration or unknown-newer acceptance was introduced. |
| Backup and restore | runbook and storage contract | `EXECUTED` | Create, verify, plan, apply, DR drill, non-empty, symlink, and partial archive cases passed. |
| Index lifecycle | index lifecycle and runbook | `EXECUTED` | Verify, rebuild, and reverify agreed with canonical storage. |
| HTTP and invalid input | carrier and protocol documents | `CROSS_REFERENCED` | Protocol, validation, and external conformance suites passed. |
| Quarantine administration | quarantine HTTP/RBAC documents | `CROSS_REFERENCED` | Candidate quarantine tests passed. |
| Replacement and cleanup | replacement and cleanup runbooks | `CROSS_REFERENCED` | Candidate crash-matrix suite passed. |
| Recovery and diagnostics | operator and recovery documents | `EXECUTED` | Read-only diagnostics and representative failure outputs matched documented fail-closed behavior. |
| Compatibility and Rust API | v1 compatibility policy and API audit | `CROSS_REFERENCED` | Candidate-bound compatibility review is PASS. |
| Security | `V1_0_SECURITY_DIFF_REVIEW.md` | `CROSS_REFERENCED` | Candidate-bound security review is PASS. |
| Qualification | qualification workflow and release evidence | `CROSS_REFERENCED` | Exact-SHA qualification artifact was independently verified. |
| Soak | `V1_0_SOAK_PLAN.md` | `CROSS_REFERENCED` | Plan is frozen; execution remains a separate release gate. |

## 4. Candidate execution matrix

| ID | Procedure | Final classification | Result | Evidence |
|---|---|---|---|---|
| DOC-01 | Platform verification | `EXECUTED` | Passed | `logs/DOC-01.log` |
| DOC-02 | Candidate binary build, install, and digest verification | `EXECUTED` | Passed | `logs/DOC-02.log` |
| DOC-03 | Unit verification | `EXECUTED` | Passed | `logs/DOC-03.log` |
| DOC-04 | Configuration and precedence | `EXECUTED` | Passed | `logs/DOC-04.log` |
| DOC-05 | Read-only diagnostics | `EXECUTED` | Passed | `logs/DOC-05.log` |
| DOC-06 | Systemd lifecycle and journal | `EXECUTED` | Passed | `logs/DOC-06.log` |
| DOC-07 | Publish and restart persistence | `EXECUTED` | Passed | `logs/DOC-07.log` |
| DOC-08 | Invalid, boundary, duplicate, and conflict handling | `CROSS_REFERENCED` | Passed | `logs/DOC-08.log` |
| DOC-09 | Backup creation and verification | `EXECUTED` | Passed | `logs/DOC-09.log` |
| DOC-10 | Isolated restore and DR drill | `EXECUTED` | Passed | `logs/DOC-10.log` |
| DOC-11 | Unsafe restore targets | `EXECUTED` | Passed | `logs/DOC-11.log` |
| DOC-12 | Index verify, rebuild, and reverify | `EXECUTED` | Passed | `logs/DOC-12.log` |
| DOC-13 | Migration, recovery, resume, and rollback contracts | `CROSS_REFERENCED` | Passed | `logs/DOC-13.log` |
| DOC-14 | Quarantine administration contracts | `CROSS_REFERENCED` | Passed | `logs/DOC-14.log` |
| DOC-15 | Replacement and cleanup crash recovery | `CROSS_REFERENCED` | Passed | `logs/DOC-15.log` |
| DOC-16 | Failure diagnosis and workspace regressions | `CROSS_REFERENCED` | Passed | `logs/DOC-16.log` |

## 5. Contradiction ledger

| ID | Question | Final disposition | Blocker |
|---|---|---|---|
| CONTR-01 | Are active v1 sources distinct from v0.9 history? | Resolved in documentation indexes. | No |
| CONTR-02 | Can a dry run be treated as final qualification? | No; exact candidate evidence is separately recorded. | No |
| CONTR-03 | Can startup migrate implicitly? | No; migration remains explicit and backup-bound. | No |
| CONTR-04 | Is the index authoritative? | No; canonical storage is authoritative. | No |
| CONTR-05 | May recovery bypass proofs or mutate canonical state manually? | No; contradictory state fails closed. | No |
| CONTR-06 | What is the reference platform? | Ubuntu Server 24.04 LTS, x86_64, systemd. | No |
| CONTR-07 | Which output details are stable? | Documented codes, statuses, required structure, and exit behavior are protected; free prose is not. | No |
| CONTR-08 | How may an operator load a `0640 root:lingonberry` environment file? | Resolved in PR #133: source the file inside `sudo -u lingonberry sh -c`; pre-sudo command substitution is prohibited. | No |

## 6. Diagnostic and exit-status observations

| Command or API | Observed result | Contract match |
|---|---|---|
| `health` | exit 0, `status:"ok"` | Yes |
| `ready` | exit 0, `ready:true`, warning diagnostic status on empty storage | Yes |
| `status` / `doctor` / `verify` | read-only warning result with stable `LB_DOCTOR_*` codes | Yes |
| `metrics` | exit 0, bounded cardinality, `ready:1` | Yes |
| relay publish | exit 0, `LB_OBJECT_STORED` | Yes |
| backup create / verify | exit 0, `status:"verified"` | Yes |
| restore plan / apply | exit 0, `planned` then `restored`, `readVerified:true` | Yes |
| DR drill | exit 0, read/write/cleanup verification all true | Yes |
| non-empty restore target | non-zero, target sentinel preserved | Yes |
| symbolic-link restore target | non-zero, `refusing symbolic link path` | Yes |
| partial archive | non-zero, `LB_ARCHIVE_IMPORT` | Yes |
| index verify / rebuild | exit 0, `LB_INDEX_CONSISTENT` | Yes |

## 7. Freeze completion checklist

- [x] Active v1 sources are reachable from the documentation indexes.
- [x] Exact candidate SHA and both binary digests are recorded.
- [x] All 16 execution rows have final allowed classifications.
- [x] Commands, outputs, diagnostic codes, and exit behavior are retained.
- [x] The service-user environment-loading contradiction was corrected and rerun.
- [x] No unresolved contradiction or `BLOCKED` item remains.
- [x] The evidence bundle is independently checksum-verified.
- [x] The PASS disposition is copied into `V1_0_RELEASE_EVIDENCE.md`.

## 8. Final disposition

**Documentation freeze decision: PASS**

The frozen published operator path is reproducible against candidate `f9543019f2c219aea3b085ff90f2da201b268a48` on the reference environment. No runtime-affecting change was required. The remaining release-critical execution gate is the 72-hour qualification soak.
