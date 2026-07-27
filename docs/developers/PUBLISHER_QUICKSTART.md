# Publisher Quickstart

**Status: v1.0 pre-release developer guide**

This guide is for developers building an application, service, CLI, connector, or protocol adapter that submits Knowledge Objects to a Lingonberry relay. English is normative.

## 1. What this guide does

A publisher performs the following flow:

```text
create a Knowledge Object
→ build an HTTP publish-request envelope
→ construct the canonical signing target
→ sign with an Ed25519 private key
→ POST /v1/objects
→ inspect both HTTP status and ingestion-result body
→ retrieve the returned canonicalId when storage succeeded
```

This guide uses the checked-in JavaScript reference producer rather than inventing a second canonicalization or signing implementation.

## 2. Current release and security boundary

- latest published release: `v0.9.0`
- fixed v1.0 candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`
- v1.0.0: not released
- formal 72-hour soak: not completed
- privileged reference-host qualification: incomplete

The normative signature rule is `lb.http.publish.signature.v1` using Ed25519 and `lb.canonical.json.v1`.

**Important:** the current checked-in publish-ingestion path validates the lexical shape of `publisher.publicKey` and `publisher.signature`, but does not yet perform Ed25519 verification before acceptance or storage. A successful response therefore does not currently prove publisher authentication. Do not advertise authenticated publishing until the active ingestion path enforces the normative signature rule.

See [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md) for the exact contract and implementation gap.

## 3. Prerequisites

Required:

- Git;
- Node.js 22 or a compatible current Node.js runtime;
- a Rust toolchain with Cargo for starting the local relay;
- `curl` or another HTTP client.

Clone the repository:

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

For an exact candidate-level walkthrough:

```bash
git checkout f9543019f2c219aea3b085ff90f2da201b268a48
```

Later documentation commits do not redefine the fixed candidate.

## 4. Run the reference publisher

The executable JavaScript reference implementation is:

```text
conformance/minimal-producer.mjs
```

Generate a signed request:

```bash
node conformance/minimal-producer.mjs > /tmp/lingonberry-publish-request.json
```

Inspect it without modifying it:

```bash
cat /tmp/lingonberry-publish-request.json
```

The producer:

1. creates a minimal protocol-native Knowledge Object;
2. generates an Ed25519 key pair in memory;
3. encodes the raw 32-byte public key as 64 lowercase hexadecimal characters;
4. constructs the request without `publisher.signature`;
5. canonicalizes that request using the checked-in JavaScript reference behavior;
6. signs the canonical UTF-8 bytes directly with Ed25519;
7. emits the complete publish request.

Generate a custom minimal object:

```bash
node conformance/minimal-producer.mjs \
  --id 'lb:obj:publisher-example-0001' \
  --created-at '2026-07-27T00:00:00Z' \
  --text 'Can an external publisher submit this Knowledge Object?' \
  --language 'en' \
  > /tmp/lingonberry-publish-request.json
```

The example generates a new ephemeral key pair each time. It is a conformance and integration reference, not a private-key management solution.

## 5. Understand the request envelope

The HTTP request contains exactly:

```json
{
  "object": {
    "id": "lb:obj:publisher-example-0001",
    "schemaVersion": "0.1.0",
    "type": "inquiry",
    "createdAt": "2026-07-27T00:00:00Z",
    "body": {
      "text": "Can an external publisher submit this Knowledge Object?",
      "language": "en"
    },
    "provenance": {
      "sources": [
        {
          "protocol": "publisher-example",
          "sourceId": "source:publisher-example-0001",
          "observedAt": "2026-07-27T00:00:00Z"
        }
      ]
    },
    "rawRef": {
      "protocol": "publisher-example",
      "sourceId": "source:publisher-example-0001"
    }
  },
  "publisher": {
    "publicKey": "<64 lowercase hexadecimal characters>",
    "signature": "<128 lowercase hexadecimal characters>"
  }
}
```

Use the schemas and protocol contracts as the authority. The example above is illustrative and must not override the checked-in schema.

## 6. Construct the signing target correctly

For `lb.http.publish.signature.v1`:

1. parse the complete request as JSON;
2. remove only `publisher.signature`;
3. preserve `publisher.publicKey` and all other fields;
4. canonicalize with `lb.canonical.json.v1`;
5. UTF-8 encode with no byte-order mark and no trailing newline;
6. sign those exact bytes directly with Ed25519.

Do not:

- sign only the nested `object`;
- remove the complete `publisher` object;
- sign a SHA-256 digest instead of the canonical bytes;
- depend on source-text member order;
- append a newline;
- modify the request after signing.

The reference producer implements this sequence in `createSignedPublishRequest`.

## 7. Start a local relay

Inspect local capabilities:

```bash
cargo run -p lingonberry-relay -- capabilities
```

Start the HTTP carrier on loopback:

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

In another terminal:

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
```

Capability discovery is descriptive. It is not runtime negotiation, automatic downgrade, or proof that signature verification is enforced.

## 8. Publish the request

Send the exact generated bytes:

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @/tmp/lingonberry-publish-request.json \
  http://127.0.0.1:8787/v1/objects
```

Do not rewrite or pretty-print a signed request unless your implementation reconstructs and regenerates the signature correctly.

## 9. Interpret the result

The versioned ingestion result includes common fields such as:

- `contractVersion`;
- `status`;
- `code`;
- `stored`;
- `duplicate`;
- `errors`.

Depending on the result, it may include:

- `canonicalId`;
- `identityKey`;
- `carrierIdentity`;
- `storedAt`;
- `object`;
- `quarantineId`.

| Status | Meaning | Publisher action |
|---|---|---|
| `stored` | a new canonical object was stored | record `canonicalId`; success |
| `duplicate` | the same publication already exists | treat as idempotent success |
| `deferred` | acceptance policy placed it in quarantine | record `quarantineId`; do not treat as stored |
| `rejected` | request, object, identity, or policy validation failed | correct the request before retrying |
| `conflict` | the same canonical identity maps to different content | do not automatically retry |
| `failed` | an operational or storage failure occurred | use bounded retry with backoff where appropriate |

Clients must inspect both the HTTP status and JSON body.

Typical HTTP mapping:

| Ingestion status | HTTP status |
|---|---:|
| `stored` | `201 Created` |
| `duplicate` | `200 OK` |
| `deferred` | `202 Accepted` |
| most `rejected` outcomes | `400 Bad Request` |
| unsupported identity rule | `422 Unprocessable Entity` |
| `conflict` | `409 Conflict` |
| `failed` | `500 Internal Server Error` |

A `202 Accepted` response does not mean canonical storage succeeded.

## 10. Verify retrieval

For `stored` or `duplicate`, retrieve the exact returned identifier:

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

Preserve URL encoding and do not derive a replacement identifier from display data.

## 11. Conformance checks for a publisher

Before calling a publisher implementation compatible, verify at least:

- the reference producer output is accepted according to the current implementation boundary;
- the canonical signing target matches the checked-in vectors;
- a changed object field changes the target and signature;
- uppercase or malformed hexadecimal encodings are rejected;
- wrong-length keys and signatures are rejected;
- duplicate publication does not create a second canonical object;
- conflict is not treated as duplicate or transient failure;
- `deferred` is not treated as canonical storage;
- oversized and deeply nested untrusted JSON is rejected within documented bounds;
- the client checks `contractVersion` before relying on response fields.

Run the JavaScript conformance checks:

```bash
node --test conformance/minimal-producer.test.mjs
node conformance/run.mjs
```

The active Rust and JavaScript suites remain the authority for checked-in compatibility behavior.

## 12. Private-key handling

The relay does not issue keys and does not possess the publisher private key.

A production publisher must provide its own key lifecycle, including:

- secure generation;
- protected storage;
- access control;
- backup and recovery policy;
- rotation or revocation policy outside the current signature contract.

Do not log private keys or include them in request JSON, URLs, diagnostics, fixtures, or issue reports.

## 13. Non-guarantees

This quickstart does not establish or guarantee:

- enforced publisher authentication in the current ingestion path;
- publisher authorization, delegation, or key revocation;
- replay prevention or trusted timestamps;
- network pub/sub delivery;
- runtime capability negotiation;
- automatic carrier fallback;
- federated or multi-node synchronization;
- production TLS termination or denial-of-service protection;
- formal release qualification or v1.0.0 publication.

## References

- [Developer Documentation](./README.md)
- [Knowledge Object Publish Quickstart](../operations/KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)
- [`conformance/minimal-producer.mjs`](../../conformance/minimal-producer.mjs)
