# Lingonberry v1.0.0 Pre-Version Candidate

**Status: designated and candidate-qualified; release authorization pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issue: #126** | **Designation date: 2026-07-23**

## 1. Designated candidate

The fixed Lingonberry v1.0.0 pre-version qualification candidate is:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

This is the `main` merge commit produced by PR #127. It is not a later documentation, evidence, or inventory commit. Documentation-only work after designation does not silently move or redefine the candidate.

## 2. Candidate scope

The candidate contains:

- the v0.9.0 production implementation and single-node operator baseline;
- the approved v1 compatibility policy;
- the completed Rust public API audit;
- qualification, security-diff, documentation-freeze, and soak contracts;
- candidate-bound qualification and documentation-integrity workflows;
- the active v1 indexes and pre-candidate documentation walkthrough record.

The reviewed interval from v0.9.0 through the candidate contains no production runtime implementation change under `packages/**`. Contract finalization, review records, and qualification infrastructure do not expand the supported runtime surface.

## 3. Recorded candidate qualification

Candidate qualification is recorded in [`V1_0_RELEASE_EVIDENCE.md`](./V1_0_RELEASE_EVIDENCE.md).

| Evidence | Recorded value |
|---|---|
| Candidate SHA | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Designation | PR #127 merge, 2026-07-23 01:03:26 UTC |
| Qualification workflow | run ID `29971797941` |
| Qualification artifact | ID `8549953270` |
| Qualification artifact digest | `sha256:cc216536a29acbc65ba7b25e74f1e2198c7050605019ea3a09c1ddab0fb18b7b` |
| `lingonberry-storage` SHA-256 | `22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0` |
| `lingonberry-relay` SHA-256 | `9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d` |
| Documentation walkthrough | run ID `29974169660`, artifact ID `8550809328` |
| Walkthrough artifact digest | `sha256:75adb9ce95b69307632705aa82d89ede1cf413779e11ab29e18e2a47cca56904` |

The qualification bundle passed its recorded gates, and the documentation walkthrough passed all 16 procedures. These results establish candidate-bound evidence only; they do not publish v1.0.0.

## 4. Remaining release gates

The following work remains mandatory:

1. complete the formal 72-hour qualification soak and workload floors;
2. complete the privileged reference-host qualification required by the release process;
3. record the final residual-risk and release disposition;
4. prepare version `1.0.0` and synchronize release-specific documents;
5. open and review the release PR;
6. validate the merged release commit;
7. create the annotated tag and GitHub Release;
8. record final publication evidence and artifact digests.

Candidate qualification, security review, compatibility review, and documentation walkthrough must not be interpreted as authorization to skip any remaining gate.

## 5. Change control

After designation:

- a runtime-affecting protocol, durable-format, CLI/HTTP contract, default, migration, or recovery behavior change invalidates candidate-bound executable evidence and requires explicit candidate reconsideration;
- an evidence-only or documentation-only correction must be reviewed for effect on operator acceptance and documentation freeze;
- a command, path, required setting, diagnostic code, or recovery-instruction change invalidates the affected walkthrough evidence;
- no change may reuse old binary or soak evidence while silently moving the qualified SHA.

## 6. Authority and non-guarantees

This record identifies the fixed candidate. Detailed gate status and evidence belong to:

- [`V1_0_RELEASE_EVIDENCE.md`](./V1_0_RELEASE_EVIDENCE.md)
- [`V1_0_QUALIFICATION_STATUS.md`](./V1_0_QUALIFICATION_STATUS.md)
- [`V1_0_QUALIFICATION_PLAN.md`](./V1_0_QUALIFICATION_PLAN.md)
- [`V1_0_SOAK_PLAN.md`](./V1_0_SOAK_PLAN.md)

This document does not claim that the formal soak, privileged reference-host qualification, version preparation, release PR, tag, GitHub Release, or final publication evidence is complete.