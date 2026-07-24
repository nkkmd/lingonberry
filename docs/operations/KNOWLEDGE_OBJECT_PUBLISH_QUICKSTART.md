# Knowledge Object Publish Quickstart

**Status: v1.0 pre-release normative**

This quickstart exercises the implemented Lingonberry publication path. English is normative.

## 1. Release boundary

This document describes the checked-in v1.0 candidate behavior. It does not publish or redefine the candidate.

- latest public release: `v0.9.0`
- fixed v1.0 candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`
- v1.0.0: not released
- formal 72-hour soak: not performed
- privileged reference-host qualification: incomplete
- version update, tag, and GitHub Release: not performed

## 2. Prerequisites

Required:

- Git;
- a Rust toolchain with Cargo;
- `curl` or an equivalent HTTP client for the HTTP example.

Clone the repository and enter the workspace:

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

For an exact candidate walkthrough, check out the fixed candidate explicitly:

```bash
git checkout f9543019f2c219aea3b085ff90f2da201b268a48
```

Documentation or tooling commits after that SHA do not redefine the candidate.

Verify the toolchain and workspace:

```bash
rustc --version
cargo --version
cargo metadata --no-deps
```

## 3. Understand the request boundary

Publication accepts a versioned HTTP publish-request envelope containing:

- `object`: a protocol-native knowledge object;
- `publisher.publicKey`: the required lowercase hexadecimal publisher public key;
- `publisher.signature`: the required lowercase hexadecimal signature.

The checked-in fixture is:

```text
fixtures/http-publish-request/minimal-request.json
```

Its object uses knowledge-object schema version `0.1.0`. The HTTP publish-request schema version is also `0.1.0`.

Do not edit a signed fixture without generating a matching signature. The signature covers the canonical signing payload defined by [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md). The relay does not issue publisher keys and does not possess the publisher private key.

The v1.0 request validator requires the checked-in hexadecimal key representation. This quickstart does not promise `npub` decoding, alternate key encodings, remote key enrollment, or an authentication handshake.

## 4. Inspect local capabilities

Run the relay capability command:

```bash
cargo run -p lingonberry-relay -- capabilities
```

This prints the generated local capability manifest. Capability discovery is descriptive. It does not perform runtime negotiation, automatic fallback, dynamic downgrade, or a remote handshake.

## 5. Publish through the local relay command

Use the signed fixture:

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

The command routes the request through the same core stages used by HTTP publication:

1. parse the request JSON;
2. validate the publish-request envelope and knowledge object;
3. verify publisher signature and identity rules;
4. evaluate the configured acceptance policy;
5. defer eligible requests to quarantine, reject invalid requests, or continue;
6. finalize the knowledge object;
7. append through the configured storage backend;
8. classify the result.

A successful command is not limited to a single generic `ok` outcome. Inspect the versioned ingestion result.

## 6. Interpret the ingestion result

The ingestion-result contract version is `1`. The response contains the common fields:

- `contractVersion`;
- `status`;
- `code`;
- `stored`;
- `duplicate`;
- `errors`.

Depending on the outcome, it may also contain:

- `canonicalId`;
- `identityKey`;
- `carrierIdentity`;
- `storedAt`;
- `object`;
- `quarantineId`.

The implemented statuses are:

| Status | Meaning |
|---|---|
| `stored` | a new canonical object was stored |
| `duplicate` | the same publication was already stored; this is idempotent success |
| `deferred` | acceptance policy placed the request in quarantine; canonical storage did not occur |
| `rejected` | validation, signature, identity, or acceptance requirements rejected the request |
| `conflict` | the canonical identity conflicts with different stored content |
| `failed` | an operational or storage failure occurred |

Do not infer successful canonical storage from process completion alone. Check `status`, `stored`, `duplicate`, and any `quarantineId` or `errors`.

## 7. Publish through HTTP

Start the public HTTP carrier on a loopback address:

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

In another terminal, inspect capabilities and readiness:

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
```

Publish the signed fixture without rewriting its bytes:

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @fixtures/http-publish-request/minimal-request.json \
  http://127.0.0.1:8787/v1/objects
```

The implemented HTTP status mapping is:

| Ingestion status | HTTP status |
|---|---:|
| `stored` | `201 Created` |
| `duplicate` | `200 OK` |
| `deferred` | `202 Accepted` |
| most `rejected` outcomes | `400 Bad Request` |
| unsupported identity rule | `422 Unprocessable Entity` |
| `conflict` | `409 Conflict` |
| `failed` | `500 Internal Server Error` |

Clients must inspect both the HTTP status and the versioned JSON body. A `202` response means deferred quarantine, not canonical storage.

## 8. Verify retrieval and persistence

For a stored or duplicate result, use the returned `canonicalId` with the checked-in retrieval route:

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

Replace `<canonical-id>` with the exact returned value and preserve URL encoding as required by the client.

Readiness is a service-level signal. It does not prove backup integrity, migration readiness, privileged disk-pressure behavior, archive qualification, or reference-host qualification.

For deeper storage and replay checks, use:

- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md);
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md);
- [File/Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md).

## 9. Common failure modes

### Invalid or modified fixture

A modified signed fixture normally fails signature validation unless the signature is regenerated from the exact canonical signing payload.

### `rejected`

Read `code` and `errors`. Rejection is distinct from conflict and operational failure.

### `deferred`

Read `quarantineId`. The request entered quarantine and was not canonically stored.

### `duplicate`

This is idempotent success. It does not create a second canonical object.

### `conflict`

The same canonical identity is associated with different content. Do not retry it as though it were a transient server failure.

### Bind failure

Choose an unused local address and port. Internet-facing deployment requires the reverse-proxy and operational controls described by [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md).

## 10. Non-guarantees

This quickstart does not establish or guarantee:

- network pub/sub delivery;
- remote subscription handshake or acknowledgement;
- runtime capability negotiation;
- dynamic carrier fallback or version downgrade;
- federated synchronization;
- production TLS termination or denial-of-service protection;
- formal 72-hour soak completion;
- privileged reference-host qualification;
- v1.0.0 publication.

## References

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [Acceptance Policy](./ACCEPTANCE_POLICY.md)
- [Carrier Capability Discovery and Compatibility](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
