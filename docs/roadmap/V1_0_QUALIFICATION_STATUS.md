# Lingonberry v1.0.0 Qualification Status

**Status: candidate qualification, security review, compatibility review, and documentation walkthrough complete; formal soak and publication pending** | **Target release: v1.0.0** | **Parent issue: #109**

## 1. Fixed candidate

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

The candidate was designated by the merge of PR #127 on 2026-07-23. Later documentation and inventory commits do not move it.

## 2. Completed candidate-bound work

| Work item | Status | Evidence |
|---|---|---|
| Candidate designation | Complete | PR #127 |
| Candidate qualification | Passed | run `29971797941`, artifact `8549953270` |
| Candidate binaries and digests | Recorded | `V1_0_RELEASE_EVIDENCE.md` |
| Rust public API audit | Complete | `V1_0_RUST_API_AUDIT.md` |
| Normative v1 compatibility policy | Complete | `V1_COMPATIBILITY_POLICY.md` |
| Candidate security and compatibility review | Passed | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Candidate documentation walkthrough | Passed | run `29974169660`, artifact `8550809328`, 16 procedures |
| Candidate documentation freeze | Passed for candidate execution | `V1_0_RELEASE_EVIDENCE.md` |

The candidate review records Critical 0, High 0, and release-blocking Medium 0. Candidate qualification and walkthrough evidence are checksummed and commit-bound.

## 3. Remaining release-blocking work

| Gate | State | Required completion |
|---|---|---|
| Formal 72-hour qualification soak | Not started / not completed | Execute the real-time systemd scheduler for at least 259,200 seconds and satisfy every workload, telemetry, crash-matrix, disk-pressure, and stop-threshold requirement. |
| Privileged reference-host qualification | Pending | Complete the dedicated Ubuntu Server 24.04 x86_64 systemd host preparation and disk-pressure rehearsal without undocumented workarounds. |
| Final residual-risk disposition | Pending | Review soak and reference-host evidence and record a release decision. |
| Version preparation | Pending | Set version `1.0.0` and synchronize release checklist, notes, implementation status, and CHANGELOG. |
| Reviewed release PR | Pending | Review the exact version-preparation diff and required checks. |
| Merged-commit validation | Pending | Revalidate the merged release commit and preserve evidence. |
| Annotated tag and GitHub Release | Pending | Publish only after all prior gates pass. |
| Final publication evidence | Pending | Record tag, release, artifact, and digest identities. |

## 4. Formal-soak readiness boundary

The scheduler, command map, crash-matrix driver, disk-pressure driver, and reference-host preparation documents exist and have rehearsal coverage. This is infrastructure readiness only.

The following do not satisfy the formal gate:

- mock-adapter runs;
- virtual-time scheduler rehearsals;
- CI-only disk-pressure or crash-matrix rehearsal;
- bounded v0.9.0 hardening soak evidence;
- candidate qualification runs shorter than the formal duration;
- documentation statements without retained real-host evidence.

## 5. Current execution order

1. complete privileged reference-host preparation and rehearsal;
2. freeze host-specific command-map and threshold inputs;
3. execute the formal 72-hour soak against the fixed candidate binaries;
4. verify the complete evidence bundle and workload floors;
5. record deviations, residual risks, and final pass/fail disposition;
6. prepare version `1.0.0` and release-specific documents;
7. open and review the release PR;
8. validate the merged release commit;
9. create the annotated tag and GitHub Release;
10. record final publication evidence.

## 6. Release boundary

v1.0.0 remains unpublished. Candidate qualification success does not authorize version preparation, tagging, or publication while the formal soak and privileged reference-host qualification remain incomplete.

The authoritative detailed record is [`V1_0_RELEASE_EVIDENCE.md`](./V1_0_RELEASE_EVIDENCE.md).