# Repository Publish Walkthrough

**Status: v1.0 pre-release repository-integration guide**

This quickstart verifies the checked-in Lingonberry publication path with a local relay and repository fixtures. English is normative.

External application, service, CLI, connector, and protocol-adapter developers should use [Publisher Quickstart](../developers/PUBLISHER_QUICKSTART.md) as their primary entry point.

## 1. Release boundary

- latest public release: `v0.9.0`
- fixed v1.0 candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`
- v1.0.0: not released
- formal 72-hour soak: not performed
- privileged reference-host qualification: incomplete
- version update, tag, and GitHub Release: not performed

This document does not publish or redefine the candidate.

## 2. Purpose and audience

Use this document when you need to:

- start the checked-in relay from the repository;
- submit the checked-in publish-request fixture;
- inspect ingestion classification;
- confirm retrieval and persistence behavior;
- exercise repository integration paths.

Use [Publisher Quickstart](../developers/PUBLISHER_QUICKSTART.md) when implementing a new external publisher, generating custom Knowledge Objects, constructing canonical signing targets, managing keys, or defining client retry behavior.

## 3. Prerequisites

Required:

- Git;
- a Rust toolchain with Cargo;
- `curl` or an equivalent HTTP client.

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

For an exact candidate walkthrough:

```bash
git checkout f9543019f2c219aea3b085ff90f2da201b268a48
```

Verify the workspace:

```bash
rustc --version
cargo --version
cargo metadata --no-deps
```

## 4. Signature implementation boundary

The normative request-signature rule is `lb.http.publish.signature.v1` using Ed25519 and `lb.canonical.json.v1`.

The checked-in schema validates that:

- `publisher.publicKey` contains exactly 64 lowercase hexadecimal characters;
- `publisher.signature` contains exactly 128 lowercase hexadecimal characters.

The current checked-in publish-ingestion path does **not** yet perform Ed25519 verification before acceptance or storage. It does not decode the key and signature, reconstruct the canonical signing target, or reject a correctly shaped but cryptographically invalid signature.

Therefore:

- successful publication is not currently proof of publisher authentication;
- this quickstart must not be used to claim enforced signature verification;
- the normative future verification behavior remains defined by [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md).

## 5. Inspect local capabilities

```bash
cargo run -p lingonberry-relay -- capabilities
```

Capability discovery is descriptive. It does not perform runtime negotiation, automatic fallback, dynamic downgrade, or a remote authentication handshake.

## 6. Publish through the local relay command

The checked-in fixture is:

```text
fixtures/http-publish-request/minimal-request.json
```

Do not edit a signed fixture unless you regenerate the request and signature according to the normative contract.

Run:

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

The current command path:

1. parses the request JSON;
2. validates the publish-request schema and nested Knowledge Object;
3. validates identity and acceptance-policy requirements;
4. defers eligible requests to quarantine, rejects invalid requests, or continues;
5. finalizes the Knowledge Object;
6. applies duplicate and conflict classification;
7. appends accepted data through the configured storage backend;
8. returns a versioned ingestion result.

Cryptographic Ed25519 verification is not currently one of these active stages.

## 7. Interpret the ingestion result

Common fields include:

- `contractVersion`;
- `status`;
- `code`;
- `stored`;
- `duplicate`;
- `errors`.

Depending on the result, it may also include:

- `canonicalId`;
- `identityKey`;
- `carrierIdentity`;
- `storedAt`;
- `object`;
- `quarantineId`.

| Status | Meaning |
|---|---|
| `stored` | a new canonical object was stored |
| `duplicate` | the same publication was already stored; idempotent success |
| `deferred` | acceptance policy placed the request in quarantine; canonical storage did not occur |
| `rejected` | request, object, identity, or policy validation rejected the request |
| `conflict` | the canonical identity conflicts with different stored content |
| `failed` | an operational or storage failure occurred |

Do not infer canonical storage from process completion alone. Inspect `status`, `stored`, `duplicate`, `quarantineId`, and `errors`.

## 8. Publish through HTTP

Start the public HTTP carrier on loopback:

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

In another terminal:

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
```

Publish the fixture without rewriting its bytes:

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @fixtures/http-publish-request/minimal-request.json \
  http://127.0.0.1:8787/v1/objects
```

Typical mapping:

| Ingestion status | HTTP status |
|---|---:|
| `stored` | `201 Created` |
| `duplicate` | `200 OK` |
| `deferred` | `202 Accepted` |
| most `rejected` outcomes | `400 Bad Request` |
| unsupported identity rule | `422 Unprocessable Entity` |
| `conflict` | `409 Conflict` |
| `failed` | `500 Internal Server Error` |

Clients must inspect both the HTTP status and JSON body. `202 Accepted` means deferred quarantine, not canonical storage.

## 9. Verify retrieval

For a stored or duplicate result, use the returned `canonicalId`:

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

Replace the placeholder with the exact returned identifier and preserve URL encoding.

For deeper storage checks, see:

- [Storage Node Quickstart](../operations/STORAGE_NODE_QUICKSTART.md);
- [Storage Node Runtime](../operations/STORAGE_NODE_RUNTIME.md);
- [File/Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md).

## 10. Common outcomes

### `duplicate`

Treat it as idempotent success. It does not create a second canonical object.

### `deferred`

Record `quarantineId`. The request was not canonically stored.

### `rejected`

Read `code` and `errors`. Correct the request before retrying.

### `conflict`

Do not retry it as though it were a transient server failure.

### `failed`

Investigate the operational or storage failure. A client may use bounded retry with backoff when the failure is genuinely transient.

## 11. Non-guarantees

This quickstart does not establish or guarantee:

- enforced publisher authentication in the current ingestion path;
- publisher authorization, key issuance, delegation, or revocation;
- replay prevention;
- network pub/sub delivery;
- runtime capability negotiation;
- dynamic carrier fallback or version downgrade;
- federated synchronization;
- production TLS termination or denial-of-service protection;
- formal 72-hour soak completion;
- privileged reference-host qualification;
- v1.0.0 publication.

## References

- [Publisher Quickstart](../developers/PUBLISHER_QUICKSTART.md)
- [Developer Documentation](../developers/README.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)
- [Storage Node Quickstart](../operations/STORAGE_NODE_QUICKSTART.md)
