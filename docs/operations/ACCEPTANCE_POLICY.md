# Acceptance Policy

**Status: v1.0 pre-release normative operations contract**

This document defines the implemented acceptance boundary for Lingonberry publish ingestion and archive import. It describes how validation results and local acceptance-policy settings produce `Accept`, `Reject`, or `Defer` decisions.

Acceptance policy is an operator-controlled ingress rule. It does not change protocol semantics, canonical identifiers, signature rules, administrator authorization, or storage recovery behavior.

## 1. Implemented configuration

The policy is resolved from the process environment.

```text
LINGONBERRY_REQUIRE_IDENTITY_CLAIM
LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY
```

Defaults:

```text
require identity claim: false
unsupported identity rule: reject
```

### 1.1 Require an Identity Claim

```bash
export LINGONBERRY_REQUIRE_IDENTITY_CLAIM=true
```

Accepted boolean spellings are:

```text
true:  true, 1, yes, on
false: false, 0, no, off
```

An invalid value is a configuration error. When enabled, a validated object with no Identity Claim is rejected with:

```text
LB_IDENTITY_CLAIM_REQUIRED
```

This switch does not make an invalid claim valid and does not bypass schema, identity, provenance, or signature validation.

### 1.2 Unsupported Identity Key rules

```bash
export LINGONBERRY_UNSUPPORTED_IDENTITY_POLICY=reject
```

Allowed values:

- `reject` — the default; reject with `LB_UNSUPPORTED_IDENTITY_RULE`.
- `defer` — keep the request outside canonical storage and return `LB_IDENTITY_DEFERRED`.

Values are trimmed and compared case-insensitively. Any other value is a configuration error.

## 2. Decision order

Acceptance is evaluated in the following order.

1. Parse and structural validation run.
2. Schema and identity hard errors are collected.
3. Any hard error produces `Reject` with `LB_VALIDATION_FAILED`.
4. If the object has no Identity Claim and the local policy requires one, the result is `Reject` with `LB_IDENTITY_CLAIM_REQUIRED`.
5. If an Identity Claim uses an unsupported identity-key rule, the configured unsupported-rule policy produces either `Reject` or `Defer`.
6. Otherwise the result is `Accept`.

A local policy may only narrow or defer ingress after validation. It does not convert malformed, invalidly signed, or otherwise invalid input into an accepted object.

## 3. Publish-ingestion outcomes

The publish-ingestion contract reports one of these statuses:

```text
stored

duplicate

deferred

rejected

conflict

failed
```

### 3.1 Stored

`Accept` followed by a successful append produces:

```text
status: stored
code: LB_OBJECT_STORED
stored: true
duplicate: false
```

The result may include the canonical identifier, identity key, carrier identity, stored time, and canonical object.

### 3.2 Duplicate

An already-stored equivalent object is an idempotent success:

```text
status: duplicate
code: LB_OBJECT_DUPLICATE
stored: true
duplicate: true
```

Duplicate is not a rejection and does not create another canonical record.

### 3.3 Deferred

`Defer` writes the original publish request and the validation reasons to the quarantine store under the resolved state directory. The result includes a `quarantineId` when the append succeeds.

A deferred request:

- is not written to the canonical catalog,
- has no canonical identifier from that ingestion attempt,
- is not equivalent to successful publication,
- is not a temporary canonical object,
- remains subject to later revalidation or an explicit terminal quarantine resolution.

If writing the quarantine record fails, the publish result is `failed`, not `deferred`.

### 3.4 Rejected

Rejection is terminal for that ingestion attempt and does not write the object to canonical storage. Examples include:

```text
LB_EMPTY_REQUEST
LB_INVALID_JSON
LB_VALIDATION_FAILED
LB_IDENTITY_CLAIM_REQUIRED
LB_UNSUPPORTED_IDENTITY_RULE
LB_PUBLISH_REQUEST_OBJECT_MISSING
```

The exact transport status or process exit code is defined by the relevant HTTP or CLI contract. Error code and semantic status are the stable cross-surface signals.

### 3.5 Conflict

A storage-level identity or canonical-object conflict produces:

```text
status: conflict
code: LB_OBJECT_CONFLICT
```

A conflict is not a policy rejection and is not eligible for policy-based quarantine merely because it occurred after acceptance.

### 3.6 Failed

Operational failures, including quarantine-write or storage failures, produce `failed`. Operators must not reinterpret an operational failure as a policy rejection, successful deferral, or successful storage.

## 4. HTTP, CLI, and archive-import boundaries

### 4.1 HTTP publish

HTTP publish exposes the same ingestion result object, including `contractVersion`, `status`, `code`, `stored`, `duplicate`, `errors`, and applicable identifiers.

HTTP status codes are transport mappings. Automation should inspect both the HTTP status and the Lingonberry result `status` and `code`.

### 4.2 CLI publish

CLI publish uses the same acceptance and ingestion pipeline. A nonzero process exit is not, by itself, enough to distinguish validation rejection, deferral, conflict, configuration failure, or storage failure; automation must preserve the classified code and output.

### 4.3 Archive import

Archive import reads and validates the archive manifest and each wire-log record before applying acceptance policy to the contained object.

- `Accept` proceeds to finalization and append.
- `Reject` aborts the import with the classified rejection code.
- `Defer` appends the original request to quarantine, returns `LB_IDENTITY_DEFERRED` with the quarantine identifier in the error detail, and aborts that import operation.
- duplicates are counted separately from newly imported records.

Archive import does not bypass the current local acceptance policy and is not a bulk route around validation.

## 5. Quarantine revalidation

Deferred records can be re-evaluated against the current implementation and current acceptance policy.

Single-record surfaces:

```bash
lingonberry quarantine-promote <quarantine-id>
lingonberry quarantine-resolutions
```

```text
POST /v1/quarantine/<quarantine-id>/promote
GET /v1/quarantine-resolutions
```

Promotion is allowed only when revalidation now produces `Accept` and the subsequent canonical append succeeds or resolves as an idempotent duplicate.

Successful and duplicate promotions are recorded in the append-only quarantine resolution ledger. The original quarantine record is retained for auditability. Repeated promotion requests return the existing resolution instead of appending another resolution.

## 6. Bounded batch revalidation

Unresolved quarantine records may be evaluated in bounded batches.

```bash
lingonberry quarantine-promote-batch [limit] [--dry-run]
```

```text
POST /v1/quarantine/promote-batch
```

The implemented default limit is `100`; the maximum is `1000`.

`--dry-run` evaluates records without writing canonical storage or the resolution ledger. Only records without an existing resolution are selected. A batch result is an aggregate maintenance result, not a claim that every scanned record was promoted.

A scheduler may invoke the bounded operation, but this document does not define or guarantee an automatic promotion scheduler.

## 7. Administrator authorization is separate

Object acceptance and administrator authorization are different controls.

- Public or carrier publish acceptance is determined by validation and this acceptance policy.
- Quarantine inspection, annotation, promotion, dismissal, or other administrator operations are governed by the administrator role-token contract.
- Possessing an administrator token does not make an invalid object acceptable.
- An accepted object does not grant access to administrator surfaces.

See [Secret Management](./SECRET_MANAGEMENT.md) and the quarantine administrator contracts for token and role boundaries.

## 8. Signature and provenance boundary

Acceptance policy does not replace cryptographic or provenance validation.

- Invalid schema or identity evidence remains a hard rejection.
- A required Identity Claim is an additional local requirement, not a substitute for validating a present claim.
- `defer` applies only to an otherwise non-hard-error object whose identity-key rule is unsupported.
- Operators must not use `defer` as a general catch-all for malformed, unsigned, invalidly signed, conflicting, or operationally failed input.

## 9. Configuration changes and operations

Acceptance policy is loaded from the process environment. Changing the environment file does not retroactively reclassify canonical objects or unresolved quarantine records.

After changing policy:

1. validate the environment value before restart;
2. restart the affected process or command execution context;
3. run a controlled accepted, rejected, and—when configured—deferred test;
4. verify canonical storage and quarantine outcomes separately;
5. record the active policy and tested commit in operational evidence;
6. use explicit revalidation for existing unresolved quarantine records.

Do not edit quarantine or canonical storage files manually to simulate a policy change.

## 10. Not implemented by this contract

This contract does not provide:

- per-tenant acceptance policy,
- per-object dynamic policy expressions,
- automatic acceptance-policy distribution between nodes,
- a general moderation or content-truth decision,
- private-object encryption,
- automatic expiry or deletion,
- policy-based acceptance of hard validation failures,
- automatic quarantine promotion,
- an administrator bypass of validation.

## 11. Audit checklist

For a policy change or release qualification, record at least:

```text
Audited commit:
Environment:
Require identity claim:
Unsupported identity policy:
Accepted fixture result:
Rejected fixture result:
Deferred fixture result, if applicable:
Canonical storage verified:
Quarantine storage verified:
Archive import verified, if applicable:
Administrator authorization boundary verified:
Open findings:
Operator:
Completed at:
```

## Related documents

- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Access and Retention Audit Checklist](./ACCESS_RETENTION_AUDIT_CHECKLIST.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
