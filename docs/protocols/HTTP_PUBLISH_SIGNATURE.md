# HTTP Publish Signature Contract

**Status: normative v1.0 pre-release protocol contract** | **Rule version: `lb.http.publish.signature.v1`** | **Last reviewed: 2026-07-25**

This document defines the byte-exact Ed25519 signature contract for a Lingonberry HTTP publish request. It also records the checked-in relay boundary: the current schema validates the public-key and signature encodings, but the current publish-ingestion path does not yet perform Ed25519 verification before acceptance or storage.

A producer that claims this rule must generate the signature exactly as defined below. A consumer must not advertise cryptographic verification of this rule until its request path actually verifies the signature target and rejects failures before canonical storage.

## 1. Request envelope

The signed value is an `http-publish-request` JSON object containing exactly:

```text
object
publisher
```

The `publisher` object contains exactly:

```text
publicKey
signature
```

The checked-in schema rejects additional top-level or publisher fields.

Encoding constraints:

- `publisher.publicKey` is 32 raw Ed25519 public-key bytes encoded as exactly 64 lowercase hexadecimal characters;
- `publisher.signature` is 64 raw Ed25519 signature bytes encoded as exactly 128 lowercase hexadecimal characters;
- uppercase hexadecimal, prefixes such as `0x`, whitespace, base64, and variable-length encodings are not valid v1 encodings.

Schema acceptance proves only that the values match these lexical constraints. It does not prove that the signature is cryptographically valid.

## 2. Signature target construction

To construct the v1 signature target, an implementation must:

1. parse the complete publish request as JSON;
2. require the top-level request and `publisher` value to be objects;
3. remove only `publisher.signature`;
4. preserve `publisher.publicKey` and every other field and value;
5. canonicalize the resulting JSON value using `lb.canonical.json.v1`;
6. UTF-8 encode the canonical JSON with no byte-order mark and no trailing newline.

The resulting UTF-8 bytes are the Ed25519 message.

Conceptually:

```text
signatureTarget = UTF8(
  canonicalize(
    request with publisher.signature removed
  )
)
```

An implementation must not:

- sign only the nested `object`;
- remove the complete `publisher` object;
- remove `publisher.publicKey`;
- remove unknown fields accepted by a future version;
- normalize timestamps, identifiers, numbers, or strings outside `lb.canonical.json.v1`;
- use source-text member order or runtime map iteration order;
- append a newline;
- sign a SHA-256 digest in place of the canonical bytes.

The canonicalization limitations and cross-runtime requirements in `CANONICALIZATION.md` apply. Producers must not rely on JSON constructs that do not have verified cross-runtime canonical behavior.

## 3. Cryptographic parameters

`lb.http.publish.signature.v1` fixes:

| Parameter | Required value |
|---|---|
| Signature algorithm | Ed25519 |
| Public-key bytes | exactly 32 bytes |
| Public-key transport encoding | 64 lowercase hexadecimal characters |
| Signature bytes | exactly 64 bytes |
| Signature transport encoding | 128 lowercase hexadecimal characters |
| Message | canonical request bytes with only `publisher.signature` removed |
| Pre-hashing | none |
| Canonicalization | `lb.canonical.json.v1` |

Ed25519 verification is performed directly over the signature-target bytes. A diagnostic SHA-256 digest may be recorded in a conformance fixture or evidence bundle, but it is not the Ed25519 message and does not replace signature verification.

## 4. Verification procedure

A verifier implementing this rule must perform the following fail-closed procedure before canonical storage:

1. parse the request;
2. validate the request envelope and required fields;
3. require `publisher.publicKey` and `publisher.signature` to satisfy their lowercase-hex lengths;
4. decode the public key to exactly 32 bytes;
5. decode the signature to exactly 64 bytes;
6. reconstruct the signature target independently from the parsed request;
7. verify the Ed25519 signature over the exact target bytes;
8. continue with object validation, acceptance policy, finalization, duplicate/conflict handling, and storage only after verification succeeds.

The verifier must not trust a caller-supplied signature-target string or digest when it can reconstruct the target from the request.

Changing a request after signing—including `object`, `publisher.publicKey`, member values, or accepted future fields—must cause verification failure.

## 5. Failure classification

A cryptographically enforcing implementation must reject the request when:

- the request or publisher envelope is structurally invalid;
- the public key or signature is absent;
- either hexadecimal value is malformed;
- decoded lengths are not exactly 32 and 64 bytes;
- the public-key bytes cannot be interpreted as an Ed25519 verification key;
- signature-target construction fails;
- Ed25519 verification fails;
- the declared signature rule is unsupported.

A malformed, unsupported, or invalid signature must not be normalized into validity, deferred to quarantine as an otherwise acceptable object, or written to the canonical catalog or raw publish-request log.

Stable externally visible error codes must distinguish at least unsupported rule, malformed encoding, and failed verification. Reusing an unrelated schema or object-validation code obscures the security boundary.

## 6. Checked-in implementation boundary

The current checked-in publish path performs these relevant steps:

1. rejects an empty request;
2. parses JSON;
3. validates the publish-request schema and object/identity rules;
4. applies the acceptance policy;
5. finalizes the nested Knowledge Object;
6. applies duplicate/conflict classification;
7. stores the original request JSON and finalized object when accepted.

The current schema enforces the lowercase hexadecimal lengths for `publisher.publicKey` and `publisher.signature`.

The checked-in validation and ingestion path does **not** currently:

- decode the public key or signature;
- construct the v1 signature target;
- call an Ed25519 verifier;
- reject a correctly shaped but cryptographically invalid signature;
- expose a signature-verification-specific ingestion result.

Accordingly:

- the presence of a 64-character public key and 128-character signature in stored request JSON is not proof of publisher authentication;
- a successful current publish response is not proof that `lb.http.publish.signature.v1` was verified;
- capability or deployment documentation must not claim enforced publish authentication until verification is wired into the active ingestion path and covered by tests;
- this documentation change does not itself implement cryptographic enforcement.

This is a security-critical pre-release gap, not permission to weaken the normative signature contract.

## 7. Storage and replay boundary

The storage backend persists the original request JSON in its raw request record and persists the finalized nested object in the canonical catalog.

Once verification is implemented:

- verification must occur before either new record is appended;
- duplicate handling may return an existing record only after the incoming request has passed the required verification policy;
- archive import and quarantine promotion must apply an explicitly defined signature policy rather than silently bypassing the HTTP rule;
- replaying stored request JSON does not retroactively prove that it was verified at original ingestion time unless verification evidence is separately retained.

This rule authenticates the signed request bytes. It does not by itself provide authorization, replay prevention, key revocation, trusted timestamps, transport confidentiality, or proof that the signing key controls a real-world identity.

## 8. Compatibility and versioning

Any change to one of the following requires a new signature rule version:

- covered request fields;
- removal of `publisher.signature`;
- canonicalization behavior;
- message encoding;
- signature algorithm;
- key or signature encoding;
- pre-hashing behavior;
- treatment of additional fields.

Implementations must not silently reinterpret a v1 signature under a future rule.

## 9. Conformance requirements

Conformance coverage for this rule must fix:

- the complete request;
- the exact request after removing only `publisher.signature`;
- exact canonical target bytes;
- optional diagnostic SHA-256 of those bytes;
- public key;
- signature;
- expected verification result.

Negative vectors must include:

- one changed object field;
- one changed public-key character;
- one changed signature character;
- uppercase hexadecimal;
- wrong-length key and signature;
- malformed hexadecimal;
- signing only the nested object;
- signing a digest instead of the canonical request bytes;
- including `publisher.signature` in the signed target;
- removing `publisher.publicKey` from the signed target.

A fixture manifest entry or documentation reference alone is not sufficient. The active Rust and JavaScript verification implementations must execute the same vectors before enforcement can be claimed.

## 10. Non-goals

This contract does not define:

- publisher authorization policy;
- delegation or key rotation;
- nonce or timestamp replay protection;
- certificate or DID resolution;
- private-key storage;
- TLS configuration;
- transport-level authentication;
- archive trust policy;
- release qualification or release authorization.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation normalization and ordinary walkthrough checks do not redefine that candidate or satisfy the outstanding formal release gates.
