# Repository Publish Walkthrough

**Status: v1.0 pre-release repository-integration guide** | **Last updated: 2026-07-27**

This walkthrough verifies the checked-in Lingonberry publication path with a local relay and repository fixtures. English is normative. External publisher developers should use [Publisher Quickstart](./PUBLISHER_QUICKSTART.md) as their primary guide.

## 1. Release boundary

- latest public release: `v0.9.0`
- fixed v1.0 candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`
- candidate qualification and documentation walkthrough: passed
- privileged reference-host preflight: next gate
- formal 72-hour soak: not started
- version update, tag, and GitHub Release: not performed

This document does not publish or redefine the candidate.

## 2. Purpose

Use this document to:

- start the checked-in relay;
- submit the checked-in signed publish-request fixture;
- inspect signature enforcement and ingestion classification;
- confirm retrieval and restart persistence;
- exercise repository integration paths.

Use [Publisher Quickstart](./PUBLISHER_QUICKSTART.md) for custom object generation, canonical signing targets, key handling, and client retry behavior.

## 3. Prerequisites

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
git checkout 8c6b48082205a3af555130eec1f3e7d2ac8811fe
rustc --version
cargo --version
cargo metadata --no-deps
```

## 4. Enforced signature boundary

The normative request-signature rule is `lb.http.publish.signature.v1` using Ed25519 and `lb.canonical.json.v1`.

The active ingestion path:

1. parses bounded JSON;
2. validates the request envelope;
3. decodes the publisher public key and signature;
4. reconstructs the canonical signing target;
5. verifies Ed25519 before acceptance, quarantine, duplicate/conflict classification, raw append, or canonical storage;
6. fails closed for malformed encoding, invalid signatures, or verifier execution failures.

Stable signature result codes include:

- `LB_PUBLISH_SIGNATURE_MALFORMED`
- `LB_PUBLISH_SIGNATURE_INVALID`
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`

Duplicate and conflict paths do not bypass signature verification.

## 5. Inspect capabilities

```bash
cargo run -p lingonberry-relay -- capabilities
```

Capability discovery is descriptive. It does not perform runtime negotiation, dynamic downgrade, or a remote authorization handshake.

## 6. Publish through the local relay command

The checked-in signed fixture is:

```text
fixtures/http-publish-request/minimal-request.json
```

Do not edit it unless the request and signature are regenerated according to the normative contract.

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

The command verifies the signature, applies identity and acceptance policy, performs duplicate/conflict classification, appends accepted data through the configured backend, and returns a versioned ingestion result.

## 7. Interpret the ingestion result

Common fields include:

- `contractVersion`
- `status`
- `code`
- `stored`
- `duplicate`
- `errors`
- `canonicalId`
- `identityKey`
- `carrierIdentity`
- `storedAt`
- `quarantineId`

| Status | Meaning |
|---|---|
| `stored` | a new signature-verified canonical object was stored |
| `duplicate` | the same signature-verified publication already exists; idempotent success |
| `deferred` | acceptance policy placed it in quarantine; not canonically stored |
| `rejected` | request, identity, signature, or policy validation rejected it |
| `conflict` | canonical identity conflicts with different stored content |
| `failed` | verifier infrastructure, operational, or storage failure |

Do not infer canonical storage from process completion alone. Inspect the structured result.

## 8. Publish through HTTP

Start the HTTP carrier:

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

In another terminal:

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
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

## 9. Verify retrieval and persistence

For `stored` or `duplicate`, use the returned `canonicalId`:

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

Restart the relay and retrieve the same identifier again to verify persistence.

## 10. Negative signature checks

Create copies outside the repository fixture tree and verify that:

- malformed public-key or signature encoding is rejected with `LB_PUBLISH_SIGNATURE_MALFORMED`;
- changing any signed request field without resigning is rejected with `LB_PUBLISH_SIGNATURE_INVALID`;
- a verifier execution failure is returned as `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR` and does not store or quarantine the request;
- tampered requests matching an existing canonical identity cannot reach `duplicate` or `conflict` classification.

The candidate documentation-walkthrough artifact records successful execution of these cases. Do not modify the checked-in signed fixture in place.

## 11. Common outcomes

- `duplicate`: idempotent success; no second canonical object.
- `deferred`: retain `quarantineId`; no canonical storage.
- `rejected`: inspect `code` and `errors`; correct before retry.
- `conflict`: do not treat as transient.
- `failed`: investigate verifier infrastructure, operation, or storage; use bounded retry only when genuinely transient.

## 12. Non-guarantees

This walkthrough does not establish or guarantee:

- publisher authorization, key issuance, delegation, rotation, or revocation;
- replay prevention beyond current ingestion and duplicate semantics;
- network pub/sub delivery or federation;
- runtime capability negotiation or dynamic carrier fallback;
- production TLS termination or denial-of-service protection;
- formal 72-hour soak completion;
- reference-host preflight completion;
- v1.0.0 publication.

## References

- [Publisher Quickstart](./PUBLISHER_QUICKSTART.md)
- [Developer Documentation](./README.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)
- [Storage Node Quickstart](../operations/STORAGE_NODE_QUICKSTART.md)
