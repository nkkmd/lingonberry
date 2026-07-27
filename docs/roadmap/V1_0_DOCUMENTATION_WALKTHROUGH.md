# Lingonberry v1.0.0 Documentation Walkthrough

**Status: superseded; redesigned-candidate rerun pending** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issues: #132, #332, #335** | **Last updated: 2026-07-27**

## 1. Evidence boundary

The prior walkthrough passed for candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Its workflow run `29974169660`, artifact `8550809328`, artifact digest, and binary hashes remain valid historical evidence for that exact commit only.

PR #331 introduced runtime-affecting Ed25519 publisher-signature enforcement. The active candidate is now:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

The old PASS result must not be represented as candidate-bound documentation evidence for the active runtime.

## 2. Prior walkthrough coverage

The historical walkthrough executed or cross-referenced 16 procedures covering:

- platform and candidate binary verification;
- systemd units and lifecycle;
- configuration precedence;
- diagnostics and machine-readable exit behavior;
- publish and restart persistence;
- invalid, boundary, duplicate, and conflict handling;
- backup, isolated restore, DR, unsafe targets, and partial archives;
- index verify/rebuild;
- migration, quarantine, replacement, cleanup, and failure diagnosis.

No procedure result or binary digest from the superseded candidate may be copied into the redesigned-candidate record.

## 3. Required redesigned-candidate rerun

Before privileged reference-host qualification begins:

1. retrieve and independently inspect the exact main-push qualification artifact for candidate `8c6b4808…`;
2. record the candidate `lingonberry-storage` and `lingonberry-relay` SHA-256 values;
3. pin `.github/workflows/v1-documentation-walkthrough.yml` and the harness to the active candidate and recorded hashes;
4. execute all 16 procedures without `BLOCKED` or `PENDING_EXECUTION` results;
5. verify the complete walkthrough `SHA256SUMS` manifest;
6. record workflow run ID, artifact ID/digest, platform, binary hashes, contradictions, and deviations;
7. update `V1_0_RELEASE_EVIDENCE.md` only after independent inspection.

## 4. Signature-enforcement-specific checks

The rerun must explicitly prove:

- the checked-in valid signed publish request is accepted and persists across restart;
- malformed publisher key/signature encoding returns `LB_PUBLISH_SIGNATURE_MALFORMED`;
- a cryptographically invalid signature returns `LB_PUBLISH_SIGNATURE_INVALID`;
- verifier infrastructure failure returns `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR` and does not write or quarantine data;
- duplicate and conflict paths do not bypass signature verification;
- published operator documentation does not describe successful publish responses as proof beyond the verified publisher signature boundary.

## 5. Current disposition

**Documentation walkthrough decision for active candidate: PENDING.**

Repository-side preparation is tracked in `V1_0_PRE_REAL_HOST_CHECKLIST.md`. The previous 16-procedure result is historical and may be used to plan the rerun, but it does not satisfy the active candidate gate.
