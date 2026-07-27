# Lingonberry v1.0.0 Pre-Real-Host Checklist

**Status: repository preflight in progress** | **Candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`** | **Tracking issue: #335** | **Last updated: 2026-07-27**

## 1. Purpose

This checklist is the mandatory go/no-go boundary before privileged reference-host qualification or the formal 72-hour soak begins.

The fixed candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

Documentation-only evidence commits after that SHA do not move the candidate.

## 2. Repository-side gates

| Gate | State | Evidence / action |
|---|---|---|
| Candidate designation | PASS | PR #333; candidate SHA recorded by PR #334 |
| Standard CI | PASS | PR #331, #333, and #334 checks |
| Rust and JavaScript regression suites | PASS | Candidate-bound PR qualification and CI |
| External conformance | PASS | Signature positive, tampered, and malformed cases included |
| Documentation inventory and bilingual checks | PASS | PR #333 and #334 checks |
| Documentation freeze check | PASS | PR #333 and #334 checks |
| Candidate-delta security code review | PASS with evidence follow-up | `V1_0_SECURITY_DIFF_REVIEW.md` |
| Exact main-push qualification artifact inspection | PENDING | Record run ID, artifact ID/digest, binary hashes, and checksum verification |
| Candidate documentation walkthrough rerun | PENDING | Must use candidate-built binaries and new recorded hashes |

## 3. Security delta acceptance

The runtime delta from the superseded candidate is limited to:

- Ed25519 publisher-signature verification at the start of Rust ingestion;
- stable malformed, invalid-signature, and verifier-error result codes;
- fail-closed behavior before policy, quarantine, duplicate/conflict classification, raw append, or canonical storage;
- valid signed fixture replacement for existing schema/conflict contract tests.

No storage format, migration rule, canonical object format, public Rust API, or recovery procedure is intentionally changed.

## 4. Artifact identity requirements

Before real-host execution, independently obtain and record all of the following for the exact candidate:

- main-push qualification workflow run ID;
- qualification artifact ID and GitHub artifact digest;
- candidate SHA embedded in the bundle;
- `lingonberry-storage` SHA-256;
- `lingonberry-relay` SHA-256;
- successful verification of every `SHA256SUMS` entry;
- aggregate and per-gate pass results.

Values from candidate `f9543019f2c219aea3b085ff90f2da201b268a48` are historical and prohibited for reuse.

## 5. Real-host preparation inputs

Freeze these values before provisioning or execution:

- Ubuntu Server 24.04 LTS, x86_64, systemd reference platform;
- exact candidate checkout and binary hashes;
- service user, directories, ownership, and environment files;
- command map and threshold files under `deploy/soak/`;
- evidence output location with sufficient disk and inode capacity;
- test start/end timestamps in UTC;
- operator identity and deviation log location;
- stop conditions for panic, OOM, corruption, divergence, unsafe recovery, or evidence loss.

## 6. Execution order after GO

1. rerun the candidate documentation walkthrough;
2. inspect and record its artifact and binary identities;
3. execute privileged reference-host preflight and disk-pressure rehearsal;
4. freeze host-specific command-map and threshold inputs;
5. start the formal 72-hour soak from the fixed candidate;
6. verify evidence completeness and record residual-risk disposition.

## 7. Go/no-go decision

**Current decision: NO-GO for real-host execution.**

Repository-side implementation and review are ready, but the exact main-push qualification artifact and new candidate binary digests have not yet been independently recorded. Real-host execution may begin only after those identities are verified and the documentation walkthrough workflow is pinned to them.
