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
- candidate qualification and documentation walkthrough: passed
- privileged reference-host preflight: next gate
- formal 72-hour soak and v1.0.0 publication: not started

The normative signature rule is `lb.http.publish.signature.v1` using Ed25519 and `lb.canonical.json.v1`.

The active ingestion path verifies the publisher signature immediately after parsing and before acceptance policy, quarantine, duplicate/conflict classification, raw append, or canonical storage. Malformed encoding, invalid signatures, and verifier execution failures fail closed.

A `stored` or `duplicate` result occurs only after the enforced signature boundary. It does not establish publisher authorization, key issuance, delegation, revocation, or replay prevention.

See [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md).

### 3. Prerequisites and candidate checkout

Required: Git, Node.js 22 or compatible, Rust/Cargo, and `curl` or another HTTP client.

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

The producer generates an ephemeral Ed25519 key pair, constructs the canonical signing target, signs its UTF-8 bytes, and emits the complete envelope. It is not a private-key management system.

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

Inspect both the HTTP status and JSON result.

| Result | Meaning |
|---|---|
| `stored` | new signature-verified canonical object stored |
| `duplicate` | same signature-verified publication already exists |
| `deferred` | quarantined; not canonically stored |
| `rejected` | schema, identity, signature, or policy rejection |
| `conflict` | canonical identity conflicts with different content |
| `failed` | verifier infrastructure, operational, or storage failure |

Signature codes:

- `LB_PUBLISH_SIGNATURE_MALFORMED`
- `LB_PUBLISH_SIGNATURE_INVALID`
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`

### 6. Retrieve and handle results

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

Use the exact returned identifier and preserve URL encoding.

Client requirements:

- never rewrite a signed request after signing;
- treat `duplicate` as idempotent success;
- do not treat `deferred` as canonical storage;
- correct `rejected` requests before retrying;
- do not retry `conflict` as transient;
- use bounded backoff only for genuinely transient `failed` outcomes;
- never place private keys in logs, fixtures, repository files, or evidence bundles.

### 7. Non-guarantees

This guide does not guarantee publisher authorization/key lifecycle, replay prevention beyond current ingestion semantics, federation, production TLS, denial-of-service protection, formal soak completion, or v1.0.0 publication.

---

<a id="japanese"></a>

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
- candidate qualification／documentation walkthrough: PASS
- privileged reference-host preflight: 次のgate
- 正式72時間soak／v1.0.0公開: 未開始

正本となる署名規則はEd25519と`lb.canonical.json.v1`を使用する`lb.http.publish.signature.v1`です。

active ingestion pathはparse直後にpublisher signatureを検証し、acceptance policy、quarantine、duplicate／conflict判定、raw append、canonical storageより前にfail closedします。

`stored`または`duplicate`は強制された署名境界を通過したrequestです。ただしpublisher authorization、key発行、delegation、revocation、replay preventionまで保証しません。

### 3. 準備とcandidate checkout

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

reference producerはephemeral Ed25519 key pairを生成し、canonical signing targetへ署名します。private-key management solutionではありません。

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

- `stored`: 新しい署名検証済みobjectを保存
- `duplicate`: 同じ署名検証済みpublicationが存在
- `deferred`: quarantineへ移動し未保存
- `rejected`: schema、identity、signature、policyによる拒否
- `conflict`: 同じcanonical identityに異なるcontent
- `failed`: verifier infrastructure、operation、storage failure

署名関連code:

- `LB_PUBLISH_SIGNATURE_MALFORMED`
- `LB_PUBLISH_SIGNATURE_INVALID`
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`

### 6. 取得とresult処理

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

署名後のrequestを書き換えず、`duplicate`はidempotent success、`deferred`は未保存、`conflict`は非一時障害として扱ってください。private keyをlog、fixture、repository、evidence bundleへ保存しないでください。

### 7. 非保証

このガイドはpublisher authorization／key lifecycle、federation、production TLS、DoS protection、正式72時間soak完了、v1.0.0公開を保証しません。
