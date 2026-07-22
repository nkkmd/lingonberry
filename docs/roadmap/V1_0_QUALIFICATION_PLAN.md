# Lingonberry v1.0.0 Qualification Plan

**Status: active** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #110** | **Last updated: 2026-07-22**

## 1. Purpose

This document is the canonical gate inventory and evidence plan for the Lingonberry v1.0.0 stable single-node release.

v1.0.0 is not a feature-expansion release. It promotes the v0.9.0 freeze candidates and operational baseline into explicit v1.x compatibility commitments only after every release gate has evidence tied to the final candidate commit.

A checked item in an earlier release document is prior evidence, not automatic proof for v1.0.0. Every gate below must be reviewed or rerun according to its classification.

## 2. Qualification principles

1. The final candidate commit is the qualification subject.
2. Test and drill evidence must identify the commit, command or workflow, pass criteria, and result.
3. Documentation-only claims cannot replace executable evidence where a test or drill is required.
4. Passing tests cannot replace a normative compatibility declaration where stability is promised.
5. Unknown, corrupt, contradictory, partial, or incompletely verified state fails closed.
6. A release blocker remains open until its disposition and regression evidence are recorded.
7. Package version updates and publication occur only after the pre-version qualification gates are understood and scheduled.
8. The bounded v0.9.0 hardening soak is prior evidence, not the final v1.0 qualification soak.

## 3. Classification

| Classification | Meaning |
|---|---|
| `SATISFIED_REVIEW_ONLY` | Implementation is frozen and the existing evidence may be reused after review against the final candidate commit. |
| `RERUN_REQUIRED` | An executable test, matrix, acceptance run, or drill must run against the v1.0.0 candidate. |
| `STABLE_DECLARATION_REQUIRED` | A freeze candidate exists, but a normative v1.x compatibility declaration is still required. |
| `EVIDENCE_GAP` | Implementation may exist, but evidence is incomplete, ambiguous, non-reproducible, or not mapped to the requirement. |
| `BLOCKED` | A known defect, contradiction, or unresolved decision prevents qualification. |

## 4. Gate inventory

| Gate | Classification | Existing basis | v1.0.0 requirement | Pass criteria | Blocker conditions | Final evidence location |
|---|---|---|---|---|---|---|
| Object lifecycle end-to-end test | `RERUN_REQUIRED` | v0.8 operator acceptance and v0.9 regression evidence cover publish, storage, restart, retrieval, query, and verification paths. | Run the complete lifecycle scenario against the candidate commit, including restart and consistency verification. | Publish, validate, store, retrieve, query, restart, query, and consistency verification all succeed without ambiguous success. | Any accepted invalid object, lost object, conflict overwrite, non-deterministic query result, or inconsistent index state. | `docs/roadmap/V1_0_RELEASE_EVIDENCE.md` plus workflow/run identifier. |
| External protocol conformance suite | `RERUN_REQUIRED` | v0.9.0 CI and release-preparation workflow passed the external conformance suite. | Run the suite against the candidate commit and record fixture/version inputs. | All valid, invalid, boundary, digest, signature, conflict, and legacy fixtures produce expected results. | Canonical bytes, digest, signature payload, identifier, or error classification differs without explicit compatibility disposition. | v1.0 release evidence and conformance workflow result. |
| Supported legacy-state migration test | `RERUN_REQUIRED` | v0.7 and v0.8 established inspect, plan, verified backup, migrate, verify, commit, resume, rollback, and unknown-newer-format rejection. | Execute migration from every state explicitly listed as supported for v1.0.0. | Read, write, index verification, backup, recovery, and compatible rollback behavior satisfy the documented policy. | Implicit migration, migration without verified backup, premature format publication, data loss, or unsupported downgrade presented as safe. | v1.0 release evidence and migration integration result. |
| Backup and isolated restore drill | `RERUN_REQUIRED` | v0.8 operator acceptance and v0.9 regression evidence preserve verified backup and isolated restore. | Run a fresh backup, verification, isolated restore, and restored-state validation against the candidate. | Archive binding verifies; active, non-empty, or symlink targets are rejected; restored data and index can be verified. | Restore overwrites active state, verification is bypassed, archive corruption is accepted, or restored state cannot be independently verified. | v1.0 release evidence and operator-acceptance artifact. |
| Index rebuild and consistency verification | `RERUN_REQUIRED` | Deterministic verify/rebuild and storage-authoritative index semantics are freeze candidates. | Verify an intact index, detect a deliberately stale/invalid derived state, rebuild from canonical storage, and reverify. | Rebuilt index is deterministic and agrees with canonical storage; incomplete catch-up does not replace last-known-good state. | Index is treated as semantic source, stale state passes verification, or rebuild changes canonical data. | v1.0 release evidence and index test/drill result. |
| Replacement and cleanup crash matrix | `RERUN_REQUIRED` | v0.9 bounded soak repeatedly passed the quarantine replacement crash-point matrix; earlier releases established proof-bound cleanup. | Run the complete replacement and cleanup crash matrix against the candidate, including resume, rollback, and contradictory-state rejection. | Every crash point resolves only to a documented recoverable state; proof and subject binding remain exact. | Ambiguous success, unbound proof, unauthorized destructive action, contradictory state accepted, or unrecoverable partial publication. | v1.0 release evidence and crash-matrix result. |
| Security release-blocker review | `RERUN_REQUIRED` | v0.9.0 closed all Critical, High, and release-blocking Medium findings; parser and signature-workspace regressions were added. | Review candidate diff from v0.9.0, rerun security regressions, and update the findings ledger. | Open Critical = 0; Open High = 0; release-blocking Medium = 0; all relevant fixes have regression coverage. | Any unresolved Critical/High, accepted release-blocking Medium, verification bypass, information leak, unsafe path handling, or unbounded untrusted input. | v1.0 security review/findings and release evidence. |
| Protocol compatibility policy | `STABLE_DECLARATION_REQUIRED` | v0.9 public API freeze candidate identifies protocol envelope, canonicalization, identifiers, digest/signature payload, validation, fixtures, and version axes. | Publish a normative protocol v1 compatibility declaration and identify the exact protocol/schema versions covered. | Breaking protocol behavior is prohibited through v1.x; accepted compatibility exceptions and deprecation rules are explicit. | Candidate documents disagree, version axes are ambiguous, or undocumented behavior is claimed stable. | `docs/architecture/V1_COMPATIBILITY_POLICY.md` or equivalent normative document. |
| Public Rust API compatibility policy | `EVIDENCE_GAP` | v0.9 Rust API inventory classifies exported surfaces, but remaining audit work is explicitly incomplete. | Complete machine-generated export inventory, documentation-consumer mapping, error-ordering review, deprecated-candidate review, and final supported-entry-point declaration. | Every exported item is classified; supported entry points map to tests/docs; accidental public surface is reduced or explicitly unstable. | Unclassified exported surface, documentation recommending unstable internals, or unresolved accidental public API. | Final Rust API inventory and v1 compatibility declaration. |
| Operator CLI and HTTP compatibility policy | `STABLE_DECLARATION_REQUIRED` | v0.8 operator CLI contract and v0.9 freeze candidate classify commands, diagnostics, exit codes, HTTP behavior, health/readiness, and metrics. | Promote the candidate to a normative v1 operator/API contract after acceptance coverage is mapped. | Command names, required arguments, machine-readable outputs, diagnostic codes, exit statuses, and documented HTTP behavior are explicitly classified. | Acceptance tests and documented behavior disagree, or compatibility-relevant output remains unspecified. | v1 compatibility declaration and operator acceptance evidence. |
| Storage format compatibility policy | `STABLE_DECLARATION_REQUIRED` | Storage manifest, unknown-newer-format rejection, migration, journal, archive, proof, generation pointer, backup, and isolated restore are freeze candidates. | Declare storage format v1 read compatibility, migration authorization, rollback limits, and durable artifact version axes. | v1.x preserves documented read compatibility; breaking migration requires plan, verified backup, and operator authorization. | Implicit destructive migration, unknown-newer-format acceptance, ambiguous durable format version, or undocumented rollback boundary. | v1 compatibility policy and storage/migration documentation. |
| Installation documentation | `SATISFIED_REVIEW_ONLY` | v0.8 established Ubuntu Server 24.04 LTS, x86_64, systemd as the formal reference platform with release-built binaries. | Review all installation instructions against the candidate package layout and commands. | A new operator can install without repository-specific tribal knowledge. | Missing prerequisite, stale binary path, undocumented permission requirement, or instructions that require source-tree assumptions. | Final installation/runbook review record. |
| Configuration documentation | `SATISFIED_REVIEW_ONLY` | v0.8 fixed configuration file, environment, and CLI precedence and exposed read-only diagnostics. | Review defaults, precedence, sensitive values, filesystem permissions, and invalid-configuration behavior against candidate. | Effective configuration is deterministic, inspectable, and fail closed on unsafe or contradictory settings. | Conflicting precedence, secret leakage, silent fallback from invalid security settings, or undocumented required setting. | Final configuration document review record. |
| Operations documentation | `SATISFIED_REVIEW_ONLY` | v0.8 operator runbook covers start, status, doctor, verify, backup, restore, index, quarantine, replacement, cleanup, diagnostics, health, readiness, and metrics. | Walk every documented command against the candidate and resolve stale examples or output contracts. | Operator can perform normal operation and diagnosis using only published documentation. | Command drift, undocumented destructive precondition, non-reproducible runbook step, or read-only command mutating state. | Final operator acceptance and documentation review. |
| Upgrade and rollback documentation | `RERUN_REQUIRED` | v0.7/v0.8 defined explicit migration and compatible rollback, and v0.9 added no storage migration. | Verify the documented upgrade path from the supported prior release and the documented rollback boundary. | Upgrade is explicit, backup-bound, verifiable, resumable/rollback-capable where promised, and unknown newer state is rejected. | Data mutation before authorization/backup, unsupported rollback presented as supported, or stale version examples. | v1 upgrade/rollback document and migration evidence. |
| Recovery and troubleshooting documentation | `SATISFIED_REVIEW_ONLY` | Existing doctor, verify, recovery classification, DR drill, crash matrices, and runbooks establish the baseline. | Review every documented failure classification and remediation against final diagnostics and exit codes. | Documented recovery never asks operators to bypass proof, mutate canonical state manually, or treat contradictory state as success. | Unsafe manual repair, missing fail-closed path, stale diagnostic code, or contradictory recovery instructions. | Final troubleshooting/recovery review record. |
| Reference-platform operator acceptance | `RERUN_REQUIRED` | Operator acceptance run 74 succeeded for v0.9.0 on the formal reference platform. | Run the complete acceptance scenario on Ubuntu Server 24.04 LTS, x86_64, systemd against candidate-built binaries. | Install, configure, start, publish, retrieve, backup, restore, verify, index rebuild, quarantine inspection, and diagnosis succeed from documentation. | Source-tree dependency, privileged workaround, undocumented step, unstable service startup, or acceptance environment divergence. | v1 operator acceptance workflow and artifact. |
| v1.0 qualification soak | `EVIDENCE_GAP` | v0.9 bounded hardening soak covered parser limits, signature workspace, and replacement crash matrix for five iterations; long-duration telemetry and disk-pressure/power-loss injection remained residual risks. | Define duration, workload, telemetry, resource ceilings, failure injection scope, and explicit pass criteria before execution. | No crash, panic, corruption, unbounded growth, resource-limit regression, or unrecoverable state; all injected failures have documented recovery outcomes. | Undefined pass criteria, missing telemetry, unexplained workspace accumulation, data/index divergence, or any release-blocking defect. | `docs/roadmap/V1_0_SOAK_PLAN.md` and final soak evidence. |

## 5. Evidence gaps requiring follow-up work

### 5.1 Rust public API audit completion

The v0.9 Rust API inventory still lists the following unfinished work:

- machine-generate the rustdoc/exported-item inventory
- map README and operator documentation references to supported Rust entry points
- determine whether external fixtures depend on error ordering
- identify deprecated candidates
- record the final approval in a v1 freeze document

Until these items are complete, the Rust API gate remains `EVIDENCE_GAP`.

### 5.2 Final v1.0 soak definition

The previous bounded hardening soak is intentionally narrower than the final stable-release soak. A separate plan must define:

- candidate commit and build provenance
- reference host and service configuration
- duration and iteration count
- publish/read/query/restart workload
- backup/restore and index verification cadence
- replacement/cleanup crash-injection cadence
- parser-boundary and malformed-input workload
- disk usage, memory, file-descriptor, workspace, and journal telemetry
- disk-pressure and abrupt-termination scenarios that are safe and reproducible
- stop conditions, blocker thresholds, and retained artifacts

### 5.3 Stable compatibility declaration

The existing freeze-candidate documents must be reconciled into one normative v1.x policy covering:

- protocol and schema
- canonical serialization, identifiers, digest, and signature payload
- public Rust entry points versus unstable/internal exports
- HTTP and operator CLI behavior
- diagnostic and error codes
- storage format and durable artifact versions
- migration, rollback, deprecation, and unknown-newer-format behavior

## 6. Rerun matrix

| Order | Qualification activity | Depends on | Expected output |
|---:|---|---|---|
| 1 | Documentation and contract contradiction review | Current main and v0.9 evidence | Contradiction ledger and resolved normative sources |
| 2 | Rust exported-surface audit completion | Rust workspace | Final API classification and supported entry points |
| 3 | Stable v1 compatibility declaration | Steps 1-2 | Normative compatibility policy |
| 4 | Standard Rust, JavaScript, and external conformance validation | Candidate implementation | Commit-bound baseline result |
| 5 | Lifecycle, migration, backup/restore, and index qualification | Step 4 | Integration/drill evidence |
| 6 | Replacement/cleanup crash matrix and security regressions | Step 4 | Matrix and findings disposition |
| 7 | Reference-platform operator acceptance | Steps 3-6 and frozen docs | Acceptance workflow and artifacts |
| 8 | v1.0 qualification soak | Steps 3-7 | Soak telemetry and final disposition |
| 9 | Version preparation and release-document freeze | All pre-version gates green | `1.0.0` candidate commit |
| 10 | Final merged-commit validation and publication | Reviewed release PR | Tag, GitHub Release, and publication evidence |

## 7. Residual-risk ledger

| Risk | Current disposition | Required v1.0 action |
|---|---|---|
| Signature verification workspace may remain after process crash, SIGKILL, or host power loss. | Accepted as a documented v0.9 residual risk; normal returns are RAII-cleaned. | Confirm startup/doctor visibility and bounded operational remediation; include accumulation checks in soak telemetry. |
| Bounded CI soak does not provide long-duration resource telemetry. | Not sufficient for v1.0 final soak. | Define and execute the dedicated qualification soak. |
| Disk-pressure and power-loss behavior is not fully evidenced by v0.9 bounded soak. | Carried into v1.0 qualification. | Add safe, reproducible failure scenarios or explicitly document any untestable boundary and conservative fail-closed policy. |
| Same-host locking is not distributed coordination. | Explicit non-goal for v1.0. | Preserve documentation and ensure no v1 claim implies multi-node safety. |
| Terminal cleanup workspace retention automation is out of scope. | Deferred beyond v1.0. | Confirm docs distinguish operator cleanup responsibility from automated retention. |

## 8. Release-blocker rules

Publication is blocked by any of the following:

1. A mandatory gate is `BLOCKED` or lacks final evidence.
2. A `RERUN_REQUIRED` activity has not passed against the candidate commit.
3. A compatibility-relevant surface is changed without fixture, migration, or explicit disposition.
4. Protocol, API, CLI, storage, migration, or operational documents contradict one another.
5. Any unresolved Critical or High security finding exists.
6. Any Medium finding is classified as release-blocking and remains unresolved.
7. Canonical storage, index, backup, restore, replacement, cleanup, or migration can report success for ambiguous or contradictory state.
8. Final operator acceptance depends on undocumented knowledge, source-tree execution, or unsupported platform behavior.
9. Soak results lack the telemetry or retained artifacts required by the soak plan.
10. Version, tag, release notes, compatibility declarations, and published artifacts identify different commits or contracts.

## 9. Recommended follow-up issues

Create and complete the following work items under #109:

1. Complete the Rust exported-surface audit and supported-entry-point declaration.
2. Write the normative v1.x compatibility policy.
3. Define the v1.0 qualification soak and telemetry contract.
4. Implement or consolidate the candidate-commit qualification workflow.
5. Run reference-platform operator acceptance and the final soak.
6. Prepare versions and release documents only after all pre-version gates are green.

## 10. Completion criteria for this plan

This qualification plan is complete when:

- every mandatory gate has exactly one classification
- pass and blocker criteria are explicit
- all reruns and stable declarations are identified
- evidence gaps have dedicated follow-up work
- the execution order prevents premature versioning or publication
- #109 and #110 reference this document as the canonical qualification plan

The classifications remain provisional until reviewed against the candidate implementation and all canonical documentation.