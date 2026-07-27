# Issue #323: HTTP Publish Signature Enforcement

**Status: implementation under review**  
**Rule:** `lb.http.publish.signature.v1`

The active Rust publish-ingestion path now verifies the Ed25519 publisher signature immediately after JSON parsing and before acceptance-policy evaluation, quarantine, duplicate or conflict classification, raw-request append, or canonical storage.

## Stable result codes

- `LB_PUBLISH_SIGNATURE_MALFORMED`: missing, structurally invalid, wrong-length, uppercase, or otherwise non-lowercase-hex key/signature encoding.
- `LB_PUBLISH_SIGNATURE_INVALID`: the correctly encoded signature does not verify the canonical request bytes.
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`: the verifier workspace, artifact creation, or OpenSSL execution fails. This is a fail-closed operational failure.

## Enforced order

```text
parse JSON
→ validate signature envelope and encoding
→ reconstruct canonical signature target
→ verify Ed25519 signature
→ validate object and acceptance policy
→ finalize
→ duplicate/conflict classification
→ append raw request and canonical object
```

Duplicate requests do not bypass signature verification. Invalid signatures cannot be deferred to quarantine as otherwise acceptable objects.

## Test coverage

The Rust ingestion module executes the checked-in valid HTTP publish signature vector and a tampered-signature negative case. The JavaScript external conformance runner already executes the registered valid, invalid, and malformed signature vectors.

## Release consequence

This is a runtime-affecting security correction. The previous fixed v1.0.0 qualification candidate must not be reused. After this change is reviewed and merged, Lingonberry requires a new candidate designation and rerun of all affected candidate-bound qualification, security, documentation, and release evidence.
