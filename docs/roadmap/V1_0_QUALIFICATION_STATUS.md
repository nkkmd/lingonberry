# Lingonberry v1.0.0 Qualification Status

**Status: candidate redesignation pending; prior candidate-bound evidence superseded by runtime security change** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #332**

## 1. Candidate state

The previous fixed candidate was:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

It was designated by PR #127 on 2026-07-23. PR #331 subsequently introduced runtime-affecting Ed25519 publisher-signature enforcement before acceptance and storage. Under the candidate change-control policy, the previous candidate and its executable evidence are now historical and cannot authorize v1.0.0 publication.

The new candidate will be the `main` merge commit produced by the redesignation PR tracked in issue #332. Until that merge commit passes main-push qualification and its artifact is inspected and recorded, there is no active qualified v1.0.0 candidate.

## 2. Superseded candidate-bound work

| Work item | Current status | Historical evidence | Required replacement |
|---|---|---|---|
| Candidate designation | Superseded | PR #127 | Merge the issue #332 redesignation PR |
| Candidate qualification | Superseded | run `29971797941`, artifact `8549953270` | Qualify the new exact `main` merge commit |
| Candidate binaries and digests | Superseded | `V1_0_RELEASE_EVIDENCE.md` | Record new candidate-built binary digests |
| Rust public API audit | Remains applicable, rerun required by workflow | `V1_0_RUST_API_AUDIT.md` | Pass audit on the new candidate |
| Normative v1 compatibility policy | Remains normative | `V1_COMPATIBILITY_POLICY.md` | Review signature-enforcement delta against policy |
| Candidate security and compatibility review | Superseded | `V1_0_SECURITY_DIFF_REVIEW.md` | Rerun for the new candidate and PR #331 delta |
| Candidate documentation walkthrough | Superseded | run `29974169660`, artifact `8550809328` | Rerun with new candidate-built binaries |
| Candidate documentation freeze | Reopened | prior candidate evidence | Revalidate affected developer/operator procedures |
| Privileged reference-host qualification | Pending | none complete | Execute against the new candidate |
| Formal 72-hour soak | Not started | none | Start only from the new fixed candidate |

Historical evidence remains retained for auditability. It must not be relabeled, copied, or cited as evidence for the redesigned candidate.

## 3. Completed runtime correction

PR #331 completed the release-blocking publisher-authentication correction:

- verifies Ed25519 publisher signatures immediately after JSON parsing;
- verifies before acceptance policy, quarantine, duplicate/conflict classification, raw-request append, and canonical storage;
- rejects malformed encodings with `LB_PUBLISH_SIGNATURE_MALFORMED`;
- rejects cryptographically invalid signatures with `LB_PUBLISH_SIGNATURE_INVALID`;
- fails closed on verifier execution failures with `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`;
- covers the checked-in valid vector and tampered-signature path in Rust ingestion;
- preserves intended validation and conflict test coverage with correctly signed fixtures.

PR #331 passed standard CI, Rust public API audit, JavaScript external conformance, documentation inventory, and PR candidate qualification before merge.

## 4. Redesignation gates

| Gate | State | Required completion |
|---|---|---|
| Redesignation PR standard CI | Pending | All required checks pass on the PR head. |
| PR candidate qualification dry run | Pending | Qualification passes without treating the PR head as the fixed candidate. |
| New candidate designation | Pending | Merge the redesignation PR; its `main` merge commit becomes the fixed candidate. |
| Main-push candidate qualification | Pending | Qualify the exact pushed merge commit and retain the artifact. |
| Independent artifact inspection | Pending | Verify candidate identity, all gates, checksums, and binary digests. |
| Release evidence update | Pending | Record new SHA, run ID, artifact ID/digest, binary digests, and disposition. |
| Security and compatibility review | Pending rerun | Review the runtime delta through the new candidate. |
| Documentation walkthrough | Pending rerun | Execute all required procedures using candidate-built binaries. |

## 5. Remaining release-blocking work

| Gate | State | Required completion |
|---|---|---|
| Formal 72-hour qualification soak | Blocked on redesignation | Execute for at least 259,200 seconds and satisfy every workload, telemetry, crash-matrix, disk-pressure, and stop-threshold requirement. |
| Privileged reference-host qualification | Blocked on redesignation | Complete Ubuntu Server 24.04 x86_64 systemd host preparation and disk-pressure rehearsal without undocumented workarounds. |
| Final residual-risk disposition | Pending | Review all replacement evidence and record a release decision. |
| Version preparation | Pending | Set version `1.0.0` and synchronize release checklist, notes, implementation status, and CHANGELOG. |
| Reviewed release PR | Pending | Review the exact version-preparation diff and required checks. |
| Merged-commit validation | Pending | Revalidate the merged release commit and preserve evidence. |
| Annotated tag and GitHub Release | Pending | Publish only after all prior gates pass. |
| Final publication evidence | Pending | Record tag, release, artifact, and digest identities. |

## 6. Current execution order

1. merge the candidate-redesignation PR tracked by issue #332;
2. qualify the exact resulting `main` merge commit;
3. inspect and record the qualification artifact and binary digests;
4. rerun security/compatibility review and the documentation walkthrough;
5. complete privileged reference-host qualification;
6. execute the formal 72-hour soak against the new candidate binaries;
7. verify all evidence, deviations, workload floors, and residual risks;
8. prepare and review version `1.0.0`;
9. validate the merged release commit, create the annotated tag and GitHub Release, and record publication evidence.

## 7. Release boundary

v1.0.0 remains unpublished. There is currently no active qualified candidate because the previous candidate was invalidated by a runtime security correction. Version preparation, tagging, and publication remain prohibited until the redesigned candidate and all subsequent release gates pass.

The authoritative detailed record is [`V1_0_RELEASE_EVIDENCE.md`](./V1_0_RELEASE_EVIDENCE.md), which must be updated after the new main-push qualification artifact is available.
