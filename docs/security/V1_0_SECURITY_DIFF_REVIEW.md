# v1.0.0 Candidate-Diff Security Review

**Status: redesigned-candidate code review complete; artifact-bound disposition pending** | **Release target: v1.0.0** | **Superseded candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`** | **Candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`** | **Tracking issues: #332, #335** | **Last updated: 2026-07-27**

## 1. Review boundary

The previous PASS disposition applied only to candidate `f9543019f2c219aea3b085ff90f2da201b268a48`. PR #331 introduced a runtime-affecting publisher-authentication control, so the earlier candidate review, binaries, qualification artifact, walkthrough, and soak evidence are historical and cannot authorize the redesigned candidate.

This review examines the security-relevant runtime delta introduced before candidate `8c6b48082205a3af555130eec1f3e7d2ac8811fe`.

## 2. Runtime delta

The intentional production change is in `packages/core/src/ingestion.rs`:

- reconstruct and verify the canonical `lb.http.publish.signature.v1` target immediately after JSON parsing;
- reject malformed lowercase-hex key/signature encodings with `LB_PUBLISH_SIGNATURE_MALFORMED`;
- reject cryptographically invalid signatures with `LB_PUBLISH_SIGNATURE_INVALID`;
- fail closed on verifier workspace or OpenSSL execution failure with `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`;
- perform verification before acceptance policy, quarantine, duplicate/conflict classification, raw-request append, or canonical storage.

Two legacy fixtures were replaced with valid signed equivalents so their intended schema-validation and identity-conflict terminal states remain reachable after authentication enforcement.

No intentional change was made to canonical object serialization, durable storage format, migration semantics, index authority, backup/restore format, recovery procedure, or public Rust API.

## 3. Threat analysis

### 3.1 Authentication bypass

**Disposition: mitigated.**

The verification gate precedes every successful, duplicate, conflict, quarantine, and persistence path. A duplicate request cannot obtain an idempotent success result without first passing signature verification.

### 3.2 Malformed key or signature input

**Disposition: fail closed.**

Shape and decoding failures are externally distinguishable from cryptographic verification failure through stable machine-readable codes. Malformed data does not fall through to quarantine or storage.

### 3.3 Verifier process failure

**Disposition: fail closed with operational residual risk.**

Workspace, process-spawn, and OpenSSL failures reject the request rather than treating it as authenticated. The remaining risk is availability: verifier infrastructure failure can reject legitimate publishes. This is acceptable for the v1 security boundary and must be observed during real-host execution.

### 3.4 Canonical-target mismatch

**Disposition: covered by checked-in vectors and conformance.**

The valid golden request passes through Rust ingestion. Changed signature bytes fail before ingestion. JavaScript external conformance covers positive, tampered, and malformed cases.

### 3.5 Storage and quarantine side effects

**Disposition: no bypass identified.**

Invalid or unverifiable requests cannot append raw request data, enter canonical storage, or be deferred to quarantine. This preserves the rule that quarantine is not an authentication fallback.

### 3.6 Secret handling

**Disposition: acceptable.**

The verifier consumes public keys and signatures only. No publisher private key is introduced into the server ingestion path or retained evidence.

## 4. Repository evidence

The following redesigned-candidate PR checks passed:

- standard CI;
- Rust workspace tests and Clippy;
- JavaScript tests and external conformance;
- v1 candidate qualification dry runs;
- Rust public API audit;
- documentation inventory, bilingual, and freeze checks.

The exact main-push candidate artifact has not yet been independently retrieved through the available connector. Therefore its run ID, artifact ID/digest, binary SHA-256 values, and bundle checksum verification remain pending and must not be inferred from PR runs or the superseded candidate.

## 5. Findings

| ID | Area | Severity | Status | Disposition |
|---|---|---:|---|---|
| V1-DIFF-006 | Publisher authentication absent before ingestion | Critical | closed | Ed25519 verification now precedes all terminal and storage paths. |
| V1-DIFF-007 | Duplicate/quarantine authentication bypass | High | closed | Both paths require successful verification first. |
| V1-DIFF-008 | Verifier infrastructure availability | Low | accepted for preflight | Fail-closed behavior is correct; observe rejection and recovery on the real host. |
| V1-DIFF-009 | Exact candidate artifact identity | Medium if omitted | open | Retrieve and independently verify main-push artifact before real-host testing. |
| V1-DIFF-010 | Candidate documentation walkthrough | Medium if stale | open | Rerun using candidate binaries and newly recorded hashes. |

Open release-blocking counts before real-host entry:

| Severity | Open | Release-blocking |
|---|---:|---:|
| Critical | 0 | 0 |
| High | 0 | 0 |
| Medium | 2 evidence requirements | 2 |
| Low | 1 accepted operational observation | 0 |

## 6. Preflight disposition

**Security code-review disposition: PASS.**

**Artifact-bound candidate security disposition: PENDING.**

No code-level security blocker was identified in the publisher-signature enforcement delta. Real-host qualification must not begin until the exact candidate qualification artifact and binary hashes are independently recorded and the documentation walkthrough is pinned to those identities.
