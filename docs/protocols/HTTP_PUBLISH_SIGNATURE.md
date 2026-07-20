# HTTP Publish Signature Rule

**Status: draft for v0.6.0** | **Rule version: `lb.http.publish.signature.v1`** | **Last updated: 2026-07-20**

## 1. Purpose

This document defines the byte-exact Ed25519 signature contract for the Lingonberry HTTP publish request.

## 2. Input envelope

The input is an `http-publish-request` JSON object containing:

- `object`: the protocol object being published;
- `publisher.publicKey`: the publisher Ed25519 public key;
- `publisher.signature`: the signature encoded as lowercase hexadecimal.

## 3. Signature target

To construct the signature target, an implementation MUST:

1. parse the complete request as JSON;
2. remove only `publisher.signature`;
3. preserve every other field and value, including `publisher.publicKey`;
4. canonicalize the resulting JSON value using `lb.canonical.json.v1`;
5. UTF-8 encode the canonical JSON without a trailing newline.

The resulting bytes are the signature target.

An implementation MUST NOT sign only the nested `object`, remove additional publisher fields, normalize timestamps, or use runtime map iteration order.

## 4. Cryptographic parameters

`lb.http.publish.signature.v1` uses:

| Parameter | Value |
|---|---|
| Signature algorithm | Ed25519 |
| Public key encoding | 32 raw bytes represented by 64 lowercase hexadecimal characters |
| Signature encoding | 64 raw bytes represented by 128 lowercase hexadecimal characters |
| Canonicalization | `lb.canonical.json.v1` |
| Pre-hashing | none |

Ed25519 verification is performed directly over the signature target bytes. The SHA-256 digest included in the conformance vector is diagnostic and MUST NOT replace the Ed25519 message bytes.

## 5. Verification result

A consumer MUST classify the request as cryptographically invalid when:

- the public key or signature is malformed;
- hexadecimal decoding fails;
- the decoded lengths are not exactly 32 and 64 bytes;
- Ed25519 verification fails;
- the declared signature rule is unsupported.

Malformed or unsupported signatures MUST NOT be normalized into valid signatures and MUST NOT enter canonical storage.

## 6. Compatibility

Any change to the covered fields, removal rule, canonicalization rule, algorithm, key encoding, signature encoding, or pre-hashing behavior requires a new signature rule version.

## 7. Conformance vector

The initial golden vector is rooted at:

```text
conformance/http-publish-signature/
```

It fixes the complete request, exact signature target bytes, SHA-256 diagnostic digest, public key, signature, and expected verification result.
