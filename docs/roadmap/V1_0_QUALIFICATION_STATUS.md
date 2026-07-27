# Lingonberry v1.0.0 Qualification Status

**Status: repository evidence complete; reference-host preflight next** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issues: #332, #335, #341**

## 1. Active candidate

The fixed v1.0.0 pre-version candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

It was designated by the merge of PR #333 and recorded by PR #334. Documentation-only evidence commits after this SHA do not move the candidate.

The previous candidate `f9543019f2c219aea3b085ff90f2da201b268a48` and all executable evidence bound to it are historical and cannot authorize the active runtime.

## 2. Completed runtime correction

PR #331 completed the release-blocking publisher-authentication correction:

- verifies Ed25519 publisher signatures immediately after JSON parsing;
- verifies before acceptance policy, quarantine, duplicate/conflict classification, raw-request append, and canonical storage;
- rejects malformed encodings with `LB_PUBLISH_SIGNATURE_MALFORMED`;
- rejects cryptographically invalid signatures with `LB_PUBLISH_SIGNATURE_INVALID`;
- fails closed on verifier execution failures with `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`;
- covers the checked-in valid vector and tampered-signature path in Rust ingestion;
- preserves intended validation and conflict coverage with correctly signed fixtures.

## 3. Repository-side qualification gates

| Gate | State | Evidence |
|---|---|---|
| Candidate designation | PASS | PR #333; exact SHA recorded by PR #334 |
| Standard CI | PASS | PR #331, #333, #334, #338, and #340 checks |
| Rust and JavaScript regressions | PASS | Workspace tests, Clippy, conformance, signed vectors |
| Documentation inventory and bilingual checks | PASS | Repository CI |
| Documentation freeze check | PASS | Repository CI and active walkthrough |
| Candidate-delta security code review | PASS | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Exact candidate qualification | PASS | run `30238378797`; artifact `8642393171` |
| Candidate documentation walkthrough | PASS | run `30239602412`; artifact `8642773653` |
| Privileged reference-host preflight | PENDING | Execute frozen host checklist |
| Formal 72-hour soak | PENDING | Start only after reference-host GO |

## 4. Active qualification evidence

| Field | Value |
|---|---|
| Qualification run | `30238378797` |
| Qualification artifact | `8642393171` |
| Artifact name | `v1-qualification-8c6b48082205a3af555130eec1f3e7d2ac8811fe-1` |
| Artifact digest | `sha256:c30a0472f6ea07f3e395c9a27c67d1460b8f35a13a7afd397bd0e5895cb93b3e` |
| Candidate SHA in bundle | `8c6b48082205a3af555130eec1f3e7d2ac8811fe` |
| `lingonberry-storage` SHA-256 | `737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507` |
| `lingonberry-relay` SHA-256 | `23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c` |
| Gate result | 12 of 12 passed |
| Manifest verification | Every `SHA256SUMS` entry verified |
| Independent ZIP digest | Matched GitHub artifact digest |

## 5. Active documentation walkthrough evidence

| Field | Value |
|---|---|
| Walkthrough run | `30239602412` |
| Walkthrough artifact | `8642773653` |
| Artifact name | `v1-documentation-walkthrough-8c6b48082205a3af555130eec1f3e7d2ac8811fe-1` |
| Artifact digest | `sha256:9b954ada86f86e5da4966951039af9dddc2eddb3d49c996d09256e4cad598338` |
| Candidate and binary identities | Matched qualification evidence |
| Procedure result | 16 of 16 passed |
| Manifest verification | All 34 `SHA256SUMS` entries verified |
| Independent ZIP digest | Matched GitHub artifact digest |

The walkthrough covers valid, changed, malformed, and verifier-failure signature behavior and confirms duplicate/conflict processing does not bypass authentication.

## 6. Security delta acceptance

The intentional production delta from the superseded candidate is limited to:

- Ed25519 publisher-signature verification at the beginning of ingestion;
- stable malformed, invalid-signature, and verifier-error result codes;
- fail-closed behavior before every terminal, quarantine, duplicate/conflict, raw append, and storage path;
- valid signed fixture replacement for existing schema/conflict contract tests.

No storage format, migration rule, canonical object format, public Rust API, backup/restore format, or recovery procedure is intentionally changed.

## 7. Reference-host preparation inputs

Freeze these values before provisioning or execution:

- Ubuntu Server 24.04 LTS, x86_64, systemd reference platform;
- exact candidate checkout and the recorded storage and relay hashes;
- service user, directories, ownership, and environment files;
- command map and threshold files under `deploy/soak/`;
- evidence output location with sufficient disk and inode capacity;
- UTC start/end timestamps, operator identity, and deviation log location;
- stop conditions for panic, OOM, corruption, divergence, unsafe recovery, verifier failure, or evidence loss.

## 8. Execution order

1. merge the repository evidence update;
2. execute privileged reference-host preflight and disk-pressure rehearsal;
3. freeze host-specific command-map and threshold inputs;
4. start the formal 72-hour soak from the fixed candidate;
5. verify evidence completeness and record residual-risk disposition;
6. prepare and review version `1.0.0`;
7. validate the merged release commit, tag, publish, and record final evidence.

## 9. Go/no-go decision

**Current decision: GO for privileged reference-host preflight; NO-GO for formal soak until that preflight passes.**

The exact candidate qualification artifact and active-candidate documentation walkthrough have been independently verified and recorded. Formal soak authorization remains conditional on successful privileged reference-host preparation.