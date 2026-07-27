# Lingonberry v1.0.0 Documentation Walkthrough

**Status: PASS for active candidate; privileged reference-host execution pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issues: #332, #335, #341** | **Last updated: 2026-07-27**

## 1. Evidence boundary

The active fixed candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

The previous candidate `f9543019f2c219aea3b085ff90f2da201b268a48` and its walkthrough run `29974169660` remain historical evidence for that exact commit only. They do not satisfy the active-candidate gate.

## 2. Active-candidate inputs

| Field | Value |
|---|---|
| Candidate commit | `8c6b48082205a3af555130eec1f3e7d2ac8811fe` |
| `lingonberry-storage` SHA-256 | `737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507` |
| `lingonberry-relay` SHA-256 | `23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c` |
| Qualification run | `30238378797` |
| Qualification artifact | `8642393171` |
| Qualification artifact digest | `sha256:c30a0472f6ea07f3e395c9a27c67d1460b8f35a13a7afd397bd0e5895cb93b3e` |

The walkthrough workflow required all three identities explicitly and rejected any candidate other than the fixed candidate before checkout.

## 3. Walkthrough execution

| Field | Value |
|---|---|
| Workflow run | `30239602412` |
| Job | `89893906819` |
| Artifact | `8642773653` |
| Artifact name | `v1-documentation-walkthrough-8c6b48082205a3af555130eec1f3e7d2ac8811fe-1` |
| Artifact digest | `sha256:9b954ada86f86e5da4966951039af9dddc2eddb3d49c996d09256e4cad598338` |
| Candidate identity | Matched |
| Binary identities | Matched qualification evidence |
| Procedures | 16 passed; 0 failed; 0 blocked |
| Manifest verification | All 34 `SHA256SUMS` entries verified |
| Result | PASS |

The independently downloaded ZIP SHA-256 matched the GitHub artifact digest exactly.

## 4. Procedure coverage

The walkthrough executed or cross-referenced all 16 frozen procedures covering:

- platform and candidate binary verification;
- systemd units and lifecycle;
- configuration precedence;
- diagnostics and machine-readable exit behavior;
- signed publish and restart persistence;
- invalid, boundary, duplicate, and conflict handling;
- backup, isolated restore, disaster recovery, unsafe targets, and partial archives;
- index verification and rebuild;
- migration, quarantine, replacement, cleanup, and failure diagnosis.

Procedures `DOC-01` through `DOC-16` all reported `passed`. Cross-referenced procedures remained candidate-bound through the exact checkout and qualification evidence.

## 5. Signature-enforcement checks

The evidence confirms:

- the checked-in valid signed request passes the ingestion guard;
- changed signature bytes fail before ingestion;
- malformed signature material returns the stable malformed-signature security path;
- verifier operational errors fail closed;
- duplicate and conflict classifications retain their expected semantics after authentication;
- conformance vectors cover valid, tampered, malformed-key, and malformed-signature requests;
- no signature failure is treated as a successful stored, duplicate, or conflict result.

Relevant evidence is retained in `DOC-08` and `DOC-16` logs and their checksummed result records.

## 6. Deviations and contradictions

No release-blocking deviation or documentation/runtime contradiction was found. No superseded-candidate digest or procedure result was reused as active evidence.

## 7. Current disposition

**Documentation walkthrough decision for active candidate: PASS.**

The documentation gate no longer blocks reference-host preparation. Formal release authorization still requires privileged reference-host preflight, the 72-hour qualification soak, version preparation, merged-release validation, tag, GitHub Release, and publication evidence.