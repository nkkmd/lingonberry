# Publisher Quickstart / Publisher クイックスタート

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は同期翻訳です。内容に差異がある場合は英語版を優先します。

## English

**Status: v1.0 pre-release developer guide** | **Last updated: 2026-07-27**

This guide is for developers building an application, service, CLI, connector, or protocol adapter that submits Knowledge Objects to a Lingonberry relay.

### 1. Publisher flow

```text
create a Knowledge Object
→ build an HTTP publish-request envelope
→ construct the canonical signing target
→ sign with an Ed25519 private key
→ POST /v1/objects
→ inspect both HTTP status and ingestion-result body
→ retrieve the returned canonicalId when storage succeeded
```

Use the checked-in JavaScript reference producer instead of inventing a second canonicalization or signing implementation.

### 2. Release and security boundary

- latest published release: `v0.9.0`
- fixed v1.0 candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`
- v1.0.0: not released
- candidate qualification and documentation walkthrough: passed
- privileged reference-host preflight: next gate
- formal 72-hour soak: not started

The normative signature rule is `lb.http.publish.signature.v1` using Ed25519 and `lb.canonical.json.v1`.

The active ingestion path verifies the publisher signature immediately after parsing and before acceptance policy, quarantine, duplicate/conflict classification, raw append, or canonical storage. It fails closed with stable result codes for malformed encoding, invalid signatures, and verifier execution failures.

A successful `stored` or `duplicate` result therefore occurs only after the request passed the enforced publisher-signature boundary. It does not establish publisher authorization, key issuance, delegation, revocation, or replay prevention.

See [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md) for the exact contract.

### 3. Prerequisites

Required:

- Git;
- Node.js 22 or a compatible current runtime;
- a Rust toolchain with Cargo;
- `curl` or another HTTP client.

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
git checkout 8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

Later evidence and documentation commits do not redefine the fixed candidate.

### 4. Generate a signed request

```bash
node conformance/minimal-producer.mjs > /tmp/lingonberry-publish-request.json
```

Custom example:

```bash
node conformance/minimal-producer.mjs \
  --id 'lb:obj:publisher-example-0001' \
  --created-at '2026-07-27T00:00:00Z' \
  --text 'Can an external publisher submit this Knowledge Object?' \
  --language 'en' \
  > /tmp/lingonberry-publish-request.json
```

The producer generates an ephemeral Ed25519 key pair, constructs the canonical signing target, signs its UTF-8 bytes, and emits the complete envelope. It is a conformance reference, not a private-key management system.

### 5. Start the relay and submit

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

In another terminal:

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @/tmp/lingonberry-publish-request.json \
  http://127.0.0.1:8787/v1/objects
```

Inspect both the HTTP status and JSON ingestion result.

| Ingestion result | Meaning |
|---|---|
| `stored` | a new authenticated request was canonically stored |
| `duplicate` | the same authenticated publication already exists; idempotent success |
| `deferred` | acceptance policy placed it in quarantine; not canonically stored |
| `rejected` | schema, identity, signature, or policy validation rejected it |
| `conflict` | the canonical identity conflicts with different stored content |
| `failed` | verifier infrastructure, operational, or storage failure |

Relevant signature codes include:

- `LB_PUBLISH_SIGNATURE_MALFORMED`
- `LB_PUBLISH_SIGNATURE_INVALID`
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`

### 6. Retrieve stored content

For `stored` or `duplicate`, use the returned `canonicalId`:

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

Preserve URL encoding and use the exact returned identifier.

### 7. Client requirements

- do not rewrite a signed request after signature generation;
- treat `duplicate` as idempotent success;
- do not treat `deferred` as canonical storage;
- correct `rejected` requests before retrying;
- do not retry `conflict` as a transient failure;
- use bounded backoff only for genuinely transient `failed` outcomes;
- never place private keys in logs, fixtures, repository files, or evidence bundles.

### 8. Non-guarantees

This guide does not provide or guarantee:

- publisher authorization, key issuance, delegation, rotation, or revocation;
- replay prevention beyond existing ingestion and duplicate semantics;
- network pub/sub delivery or federation;
- runtime capability negotiation or dynamic downgrade;
- production TLS termination or denial-of-service protection;
- formal 72-hour soak completion or v1.0.0 publication.

## 日本語

**Status: v1.0 pre-release developer guide** | **最終更新: 2026-07-27**

このガイドは、Knowledge ObjectをLingonberry relayへ送信するapplication、service、CLI、connector、protocol adapterの開発者向けです。

### 1. Publisher flow

```text
Knowledge Objectを作成
→ HTTP publish-request envelopeを構築
→ canonical signing targetを構築
→ Ed25519 private keyで署名
→ POST /v1/objects
→ HTTP statusとingestion-result bodyを確認
→ 保存成功時は返却されたcanonicalIdで取得
```

独自のcanonicalization／署名実装を新設せず、checked-in JavaScript reference producerを参照してください。

### 2. Release／security境界

- 最新公開release: `v0.9.0`
- 固定v1.0 candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`
- v1.0.0: 未公開
- candidate qualification／documentation walkthrough: PASS
- privileged reference-host preflight: 次のgate
- 正式72時間soak: 未開始

正本となる署名規則は、Ed25519と`lb.canonical.json.v1`を使用する`lb.http.publish.signature.v1`です。

active ingestion pathはparse直後にpublisher signatureを検証し、acceptance policy、quarantine、duplicate／conflict判定、raw append、canonical storageより前にfail closedします。malformed encoding、invalid signature、verifier execution failureには安定したresult codeを返します。

`stored`または`duplicate`は、強制されたpublisher-signature境界を通過したrequestです。ただしpublisher authorization、key発行、delegation、revocation、replay preventionまで保証するものではありません。

詳細は[HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)を参照してください。

### 3. 準備

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
git checkout 8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

candidate決定後のevidence／documentation commitはcandidateを変更しません。

### 4. 署名済みrequestの生成

```bash
node conformance/minimal-producer.mjs > /tmp/lingonberry-publish-request.json
```

custom例:

```bash
node conformance/minimal-producer.mjs \
  --id 'lb:obj:publisher-example-0001' \
  --created-at '2026-07-27T00:00:00Z' \
  --text 'Can an external publisher submit this Knowledge Object?' \
  --language 'en' \
  > /tmp/lingonberry-publish-request.json
```

reference producerはephemeral Ed25519 key pairを生成し、canonical signing targetへ署名して完全なenvelopeを出力します。private-key management solutionではありません。

### 5. Relay起動と送信

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

別terminal:

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @/tmp/lingonberry-publish-request.json \
  http://127.0.0.1:8787/v1/objects
```

HTTP statusとJSON ingestion resultの両方を確認してください。

- `stored`: 新しい認証済みrequestをcanonical storageへ保存
- `duplicate`: 同じ認証済みpublicationが存在するidempotent success
- `deferred`: quarantineへ移動し、canonical storage未保存
- `rejected`: schema、identity、signature、policyによる拒否
- `conflict`: 同じcanonical identityに異なるcontent
- `failed`: verifier infrastructure、operation、storage failure

署名関連code:

- `LB_PUBLISH_SIGNATURE_MALFORMED`
- `LB_PUBLISH_SIGNATURE_INVALID`
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`

### 6. 保存済みcontentの取得

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

返却されたidentifierを正確に使用し、URL encodingを保持してください。

### 7. Client要件

- 署名後のrequest bytesを書き換えない
- `duplicate`をidempotent successとして扱う
- `deferred`をcanonical storage成功と扱わない
- `rejected`は修正してから再送する
- `conflict`を一時障害としてretryしない
- private keyをlog、fixture、repository、evidence bundleへ保存しない

### 8. 非保証

このガイドはpublisher authorization、key lifecycle、federation、production TLS、DoS protection、正式72時間soak完了、v1.0.0公開を保証しません。
