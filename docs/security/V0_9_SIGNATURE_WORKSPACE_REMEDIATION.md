# v0.9.0 Signature Verification Workspace Remediation

**Status: implemented and regression-tested; historical v0.9.0 record** | **Finding: LB-SEC-009-001** | **Last updated: 2026-07-22**

## 1. Purpose

This document preserves the implementation contract used to remediate temporary-workspace handling in `verify_publish_request_signature_with_openssl` under `packages/protocol/src/lib.rs`.

The remediation changed workspace safety only. It did not change the signature algorithm, canonical publish-request payload, OpenSSL verification contract, protocol schema, wire format, or externally visible verification result.

## 2. Implemented workspace contract

### Exclusive creation

The workspace remains beneath the operating-system temporary directory, but no existing candidate path is reused. Candidate names combine process identity, a process-local monotonic counter, and a time-derived value. The directory is created exclusively; path collisions, files, directories, and symlinks are rejected and retried only within a finite bound.

### Permissions and artifact creation

On the Unix reference platform, the workspace uses owner-only permissions. Verification artifacts are created as new files only:

- `public-key.der`
- `signature.bin`
- `message.bin`

Existing files are never truncated or overwritten. Artifact creation uses exclusive new-file semantics.

### Cleanup ownership

A single RAII scope guard owns workspace cleanup. Best-effort removal runs for normal return paths including:

- signature success;
- signature mismatch;
- intermediate artifact-write failure;
- OpenSSL spawn failure;
- OpenSSL non-zero exit;
- path-conversion failure.

The primary verification or setup result is not replaced by a cleanup failure. Diagnostics must remain bounded and must not expose canonical payloads, signatures, public-key material, or sensitive host paths.

Process abort, SIGKILL, kernel termination, and host power loss can bypass `Drop`; stale workspaces from those events remain an operational cleanup risk rather than a signature-verification bypass.

## 3. Concurrency and filesystem safety

Concurrent verification attempts must never share a workspace. Safety depends on exclusive filesystem creation, not merely on candidate-name unpredictability.

The implementation must not:

- follow a pre-existing symlink candidate;
- reuse an existing directory;
- overwrite an existing artifact;
- depend on `create_dir_all` for verification workspace creation;
- expose temporary artifact content through errors.

## 4. Regression evidence

Regression coverage verifies:

1. cleanup after valid-signature verification;
2. cleanup after invalid-signature verification;
3. cleanup after intermediate failures;
4. refusal to reuse a pre-existing candidate path;
5. refusal to follow candidate symlinks;
6. preservation of existing files during artifact collisions;
7. workspace isolation under concurrent verification;
8. owner-only Unix permissions;
9. stable canonical payload and verification semantics;
10. standard formatting, clippy, and workspace-test success.

The findings ledger records source commit `fe23c523f358cfa62aea396ec7481778a0915c2c` and regression-test commit `1083ab0348881aabba924f102151c5d4ed3da292`.

## 5. Final disposition

**Implemented, regression-tested, and closed for v0.9.0.**

The fixed v1.0.0 candidate contains no later production runtime change to this implementation. Candidate applicability is reviewed in [`V1_0_SECURITY_DIFF_REVIEW.md`](./V1_0_SECURITY_DIFF_REVIEW.md).

This historical remediation record does not claim that the formal 72-hour soak, privileged reference-host qualification, release PR, tag, or GitHub Release is complete.