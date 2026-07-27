# Lingonberry v1.0.0 Qualification Status

**Status: redesigned candidate fixed; repository preflight in progress; real-host execution NO-GO** | **Target release: v1.0.0** | **Parent issue: #109** | **Tracking issues: #332, #335**

## 1. Active candidate

The fixed v1.0.0 pre-version candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

It was designated by the merge of PR #333. PR #334 recorded the exact SHA. Documentation-only evidence commits after this SHA do not move the candidate.

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

## 3. Repository-side preflight gates

| Gate | State | Evidence / action |
|---|---|---|
| Candidate designation | PASS | PR #333; exact SHA recorded by PR #334 |
| Standard CI | PASS | PR #331, #333, and #334 checks |
| Rust and JavaScript regressions | PASS | Workspace tests, Clippy, conformance, signed vectors |
| Documentation inventory and bilingual checks | PASS before preflight update | Must remain green on issue #335 PR |
| Documentation freeze check | PASS before preflight update | Must remain green on issue #335 PR |
| Candidate-delta security code review | PASS with evidence follow-up | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Exact main-push qualification artifact inspection | PENDING | Record run ID, artifact identity/digest, binary hashes, and bundle checksums |
| Candidate documentation walkthrough rerun | PENDING | Pin workflow to active candidate and newly recorded hashes |

## 4. Security delta acceptance

The intentional production delta from the superseded candidate is limited to:

- Ed25519 publisher-signature verification at the beginning of ingestion;
- stable malformed, invalid-signature, and verifier-error result codes;
- fail-closed behavior before every terminal, quarantine, duplicate/conflict, raw append, and storage path;
- valid signed fixture replacement for existing schema/conflict contract tests.

No storage format, migration rule, canonical object format, public Rust API, backup/restore format, or recovery procedure is intentionally changed.

## 5. Required artifact identities before real-host execution

Independently obtain and record all of the following for the exact candidate:

- main-push qualification workflow run ID;
- qualification artifact ID and GitHub artifact digest;
- candidate SHA embedded in the bundle;
- `lingonberry-storage` SHA-256;
- `lingonberry-relay` SHA-256;
- successful verification of every `SHA256SUMS` entry;
- aggregate and per-gate PASS results.

Values from the superseded candidate are prohibited for reuse. Unavailable values must remain `Pending`; they must not be inferred from PR runs.

## 6. Documentation walkthrough replacement

The prior 16-procedure walkthrough is historical. Before entering privileged reference-host qualification:

1. pin `.github/workflows/v1-documentation-walkthrough.yml` and its harness to candidate `8c6b4808…` and the newly recorded binary hashes;
2. rerun all 16 procedures;
3. explicitly test valid, malformed, invalid, and verifier-error signature paths;
4. verify duplicate and conflict handling cannot bypass authentication;
5. independently verify the walkthrough artifact digest and complete `SHA256SUMS` manifest;
6. update `V1_0_DOCUMENTATION_WALKTHROUGH.md` and `V1_0_RELEASE_EVIDENCE.md`.

## 7. Real-host preparation inputs

Freeze these values before provisioning or execution:

- Ubuntu Server 24.04 LTS, x86_64, systemd reference platform;
- exact candidate checkout and binary hashes;
- service user, directories, ownership, and environment files;
- command map and threshold files under `deploy/soak/`;
- evidence output location with sufficient disk and inode capacity;
- UTC start/end timestamps, operator identity, and deviation log location;
- stop conditions for panic, OOM, corruption, divergence, unsafe recovery, verifier failure, or evidence loss.

## 8. Execution order after GO

1. independently inspect and record the exact candidate qualification artifact;
2. update and rerun the candidate documentation walkthrough;
3. inspect and record the walkthrough artifact and binary identities;
4. execute privileged reference-host preflight and disk-pressure rehearsal;
5. freeze host-specific command-map and threshold inputs;
6. start the formal 72-hour soak from the fixed candidate;
7. verify evidence completeness and record residual-risk disposition;
8. prepare and review version `1.0.0`;
9. validate the merged release commit, tag, publish, and record final evidence.

## 9. Go/no-go decision

**Current decision: NO-GO for privileged reference-host and formal-soak execution.**

Repository-side implementation and code review are ready. Entry remains blocked until the exact main-push qualification artifact and active-candidate binary hashes are independently recorded and the documentation walkthrough is pinned to those identities.

The authoritative detailed record is [`V1_0_RELEASE_EVIDENCE.md`](./V1_0_RELEASE_EVIDENCE.md).
