# v0.9.0 Security Review

**Status: historical review contract; completed for v0.9.0** | **Release target: v0.9.0** | **Last updated: 2026-07-22**

## 1. Purpose

This document preserves the security-review scope and invariants used for v0.9.0. It is historical evidence for the runtime baseline inherited by the fixed v1.0.0 candidate. The candidate-specific disposition is recorded in [`V1_0_SECURITY_DIFF_REVIEW.md`](./V1_0_SECURITY_DIFF_REVIEW.md).

## 2. Security invariants

1. An object that has not passed validation must never enter canonical storage.
2. Identity, signature, and digest verification must not be bypassable.
3. Duplicate and conflict handling must remain distinct; a conflict must not overwrite an existing object.
4. Unknown, corrupt, contradictory, or partial state must not be treated as success.
5. Ordinary startup must not perform implicit migration, repair, or destructive operation.
6. Backup, restore, replacement, cleanup, and migration must bind the exact subject and evidence.
7. Restore must reject active state/data directories, non-empty targets, and symlink targets.
8. Canonical storage is authoritative; indexes are derived and must not become semantic authority.
9. Authorization must not permit side effects before target resolution, validation, and precondition checks are complete.
10. Attacker-controlled values must not expose secrets, full filesystem paths, or unbounded metric labels.

## 3. Trust boundaries

| Boundary | Untrusted input | Protected asset | Required behavior |
|---|---|---|---|
| Relay ingress | HTTP body, headers, object envelope | canonical storage, CPU, memory | size/depth limits, deterministic validation, fail closed |
| Identity verification | public key, signature, digest subject | authenticity contract | exact binding to canonical bytes; no bypass |
| Filesystem operations | configured paths, archive entries, workspaces | data directory, backup, host filesystem | traversal and symlink rejection; revalidation before mutation |
| Administrative operations | token, role, request ordering | quarantine, replacement, cleanup, restore | authentication and authorization before mutation; proof binding |
| Journal and recovery | partial or malformed durable records | last-known-good state | contradiction rejection; idempotent resume/rollback |
| Index reader | malformed or stale derived state | query correctness and availability | storage authority; no panic, OOB access, or unbounded allocation |
| Diagnostics | configuration, errors, object metadata | secrets, privacy, availability | redaction, bounded cardinality, safe remote detail |

## 4. Review matrix

The v0.9.0 review covered:

- path traversal and root-bound path validation;
- symlink handling, including parent-component and dangling links;
- oversized and deeply nested untrusted input;
- malformed serialization and parser acceptance consistency;
- signature-verification bypass, algorithm confusion, and canonicalization mismatch;
- authorization ordering and mutation-before-authorization risks;
- information leakage through logs, diagnostics, HTTP errors, and metrics;
- TOCTOU between plan/apply, verify/commit, and authorize/mutate phases;
- disk-full and partial-I/O failures across create, write, flush, fsync, rename, directory fsync, and delete operations.

The required behavior was fail-closed rejection, no partial canonical publication, deterministic recovery, and retained evidence for every security-relevant fix.

## 5. Finding requirements

Each finding record had to include:

- identifier and affected component;
- severity and state;
- attack precondition and impact;
- reproduction and root cause;
- fix and regression test;
- compatibility impact, owner, and release disposition.

Critical and High findings could not be risk-accepted for release.

## 6. Final review disposition

The resulting findings ledger is [`V0_9_SECURITY_FINDINGS.md`](./V0_9_SECURITY_FINDINGS.md). v0.9.0 closed all identified release-blocking findings and retained regression coverage for the parser and signature-workspace remediations.

This historical record does not authorize v1.0.0 publication and does not satisfy the pending formal 72-hour soak or privileged reference-host qualification.