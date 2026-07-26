# v0.9.0 Security Findings

**Status: historical release record; closed for v0.9.0** | **Release target: v0.9.0** | **Last updated: 2026-07-22**

## 1. Purpose and authority

This document preserves the findings ledger used for the v0.9.0 security review. It is historical evidence, not the final security disposition for the fixed v1.0.0 candidate.

The candidate-bound v1 review is [`V1_0_SECURITY_DIFF_REVIEW.md`](./V1_0_SECURITY_DIFF_REVIEW.md). The v0.9.0 findings remain relevant because no production runtime implementation changed between v0.9.0 and the fixed candidate.

## 2. Severity and release policy

- **Critical:** realistic compromise of a core security or durability boundary; immediate release stop.
- **High:** major confidentiality, integrity, availability, authorization, or durability impact; remediation required before release.
- **Medium:** incomplete defense layer, bounded resource exhaustion, residual data, or operational safety reduction; remediation or explicit release disposition required.
- **Low:** defense-in-depth or hardening improvement; explicit disposition required when unresolved.

v0.9.0 shipped with no unresolved Critical or High finding and no unresolved release-blocking Medium finding.

## 3. Finding LB-SEC-009-001

### Signature verification temporary artifacts are not removed

| Field | Value |
|---|---|
| Severity | Medium |
| State | Closed |
| Release blocker | Resolved |
| Component | `packages/protocol/src/lib.rs` |
| Function | `verify_publish_request_signature_with_openssl` |
| Source commit | `fe23c523f358cfa62aea396ec7481778a0915c2c` |
| Regression-test commit | `1083ab0348881aabba924f102151c5d4ed3da292` |

The earlier implementation wrote the public key, signature, and canonical payload beneath the operating-system temporary directory and did not reliably remove the workspace after success or failure.

The remediation:

- creates a collision-resistant candidate name from process ID, time, and a process-local atomic counter;
- creates the workspace exclusively rather than reusing an existing path;
- applies owner-only permissions on Unix;
- creates artifacts with `create_new(true)`;
- owns cleanup through an RAII guard covering normal return paths;
- avoids placing payload, signature, or host paths in error messages.

Regression tests cover workspace removal, Unix permissions, collision refusal, concurrent isolation, and cleanup after concurrent use.

Residual risk: a process crash, SIGKILL, kernel termination, or host power loss can bypass Rust `Drop` and leave a stale workspace. This is an operational cleanup risk, not evidence of signature bypass.

**Disposition: fixed and regression-tested; closed for v0.9.0.**

## 4. Finding LB-SEC-009-002

### Protocol JSON parsing lacked explicit input-size and nesting-depth limits

| Field | Value |
|---|---|
| Severity | Medium |
| State | Closed |
| Release blocker | Resolved |
| Component | `packages/protocol/src/lib.rs` |
| Function | `parse_json` |
| Source and test commit | `fe23c523f358cfa62aea396ec7481778a0915c2c` |

The remediation introduced bounded parsing behavior for oversized and deeply nested untrusted JSON and retained deterministic failure semantics without partial canonical writes.

Regression coverage verifies rejection of oversized and excessive-depth inputs and preserves the accepted canonical representation for valid inputs.

**Disposition: fixed and regression-tested; closed for v0.9.0.**

## 5. Final v0.9.0 disposition

| Severity | Open | Release-blocking |
|---|---:|---:|
| Critical | 0 | 0 |
| High | 0 | 0 |
| Medium | 0 | 0 |

This ledger does not claim that the v1.0.0 formal soak, privileged reference-host qualification, release PR, tag, or GitHub Release is complete.