# Publisher Quickstart / Publisher クイックスタート

[English](#english) | [日本語](#日本語)

> English is the normative version of this document. The Japanese section is a synchronized translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は同期翻訳です。内容に差異がある場合は英語版を優先します。

## English

**Status: v1.0 pre-release developer guide**

This guide is for developers building an application, service, CLI, connector, or protocol adapter that submits Knowledge Objects to a Lingonberry relay.

### 1. What this guide does

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

### 2. Current release and security boundary

- latest published release: `v0.9.0`
- fixed v1.0 candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`
- v1.0.0: not released
- formal 72-hour soak: not completed
- privileged reference-host qualification: incomplete

The normative signature rule is `lb.http.publish.signature.v1` using Ed25519 and `lb.canonical.json.v1`.

**Important:** the current checked-in publish-ingestion path validates the lexical shape of `publisher.publicKey` and `publisher.signature`, but does not yet perform Ed25519 verification before acceptance or storage. A successful response therefore does not currently prove publisher authentication. Do not advertise authenticated publishing until the active ingestion path enforces the normative signature rule.

See [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md) for the exact contract and implementation gap.

### 3. Prerequisites

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

### 4. Run the reference publisher

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

### 5. Understand the request envelope

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

### 6. Construct the signing target correctly

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

### 7. Start a local relay

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

### 8. Publish the request

Send the exact generated bytes:

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @/tmp/lingonberry-publish-request.json \
  http://127.0.0.1:8787/v1/objects
```

Do not rewrite or pretty-print a signed request unless your implementation reconstructs and regenerates the signature correctly.

### 9. Interpret the result

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

### 10. Verify retrieval

For `stored` or `duplicate`, retrieve the exact returned identifier:

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

Preserve URL encoding and do not derive a replacement identifier from display data.

### 11. Conformance checks for a publisher

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

### 12. Private-key handling

The relay does not issue keys and does not possess the publisher private key.

A production publisher must provide its own key lifecycle, including:

- secure generation;
- protected storage;
- access control;
- backup and recovery policy;
- rotation or revocation policy outside the current signature contract.

Do not log private keys or include them in request JSON, URLs, diagnostics, fixtures, or issue reports.

### 13. Non-guarantees

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

### References

- [Developer Documentation](./README.md)
- [Repository Publish Walkthrough](./REPOSITORY_PUBLISH_WALKTHROUGH.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)
- [`conformance/minimal-producer.mjs`](../../conformance/minimal-producer.mjs)

---

## 日本語

**Status: v1.0 pre-release developer guide**

このガイドは、Knowledge ObjectをLingonberry relayへ送信するapplication、service、CLI、connector、またはprotocol adapterを構築する開発者を対象とします。

### 1. このガイドで行うこと

publisherは次の流れを実行します。

```text
Knowledge Objectを作成する
→ HTTP publish-request envelopeを構築する
→ canonical signing targetを構築する
→ Ed25519 private keyで署名する
→ POST /v1/objects
→ HTTP statusとingestion-result bodyの両方を確認する
→ storage成功時に返されたcanonicalIdを取得する
```

このガイドでは、canonicalizationや署名実装を別に作るのではなく、repositoryに含まれるJavaScript reference producerを使用します。

### 2. 現在のreleaseとsecurity boundary

- 最新の公開済みrelease: `v0.9.0`
- 固定v1.0 candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`
- v1.0.0: 未公開
- formal 72-hour soak: 未完了
- privileged reference-host qualification: 未完了

正本となる署名規則は、Ed25519と`lb.canonical.json.v1`を使用する`lb.http.publish.signature.v1`です。

**重要:** 現在repositoryに含まれるpublish-ingestion pathは、`publisher.publicKey`と`publisher.signature`の字句上の形式を検査しますが、受理または保存の前にEd25519検証をまだ実行していません。したがって、現在の成功responseはpublisher authenticationを証明しません。active ingestion pathが正本の署名規則を強制するまで、認証済みpublicationとして案内してはいけません。

正確なcontractと実装上の不足については、[HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)を参照してください。

### 3. 前提条件

必要なもの:

- Git
- Node.js 22、または互換性のある現行Node.js runtime
- local relayを起動するためのCargoを含むRust toolchain
- `curl`または別のHTTP client

repositoryをcloneします。

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

candidateと完全に一致するwalkthroughを実行する場合:

```bash
git checkout f9543019f2c219aea3b085ff90f2da201b268a48
```

後続のdocumentation commitは固定candidateを再定義しません。

### 4. Reference publisherを実行する

実行可能なJavaScript reference implementationは次です。

```text
conformance/minimal-producer.mjs
```

署名済みrequestを生成します。

```bash
node conformance/minimal-producer.mjs > /tmp/lingonberry-publish-request.json
```

変更せずに内容を確認します。

```bash
cat /tmp/lingonberry-publish-request.json
```

producerは次を実行します。

1. 最小のprotocol-native Knowledge Objectを作成する。
2. memory上でEd25519 key pairを生成する。
3. raw 32-byte public keyを64文字のlowercase hexadecimalとしてencodeする。
4. `publisher.signature`を含まないrequestを構築する。
5. repositoryに含まれるJavaScript reference behaviorでrequestをcanonicalizeする。
6. canonical UTF-8 bytesをEd25519で直接署名する。
7. 完全なpublish requestを出力する。

独自の最小objectを生成します。

```bash
node conformance/minimal-producer.mjs \
  --id 'lb:obj:publisher-example-0001' \
  --created-at '2026-07-27T00:00:00Z' \
  --text 'Can an external publisher submit this Knowledge Object?' \
  --language 'en' \
  > /tmp/lingonberry-publish-request.json
```

この例は実行ごとに新しいephemeral key pairを生成します。これはconformanceおよびintegration用referenceであり、private-key management solutionではありません。

### 5. Request envelopeを理解する

HTTP requestは正確に次の構造を含みます。

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

schemaとprotocol contractを正本としてください。上記の例は説明用であり、repository内のschemaを上書きするものではありません。

### 6. Signing targetを正しく構築する

`lb.http.publish.signature.v1`では次を行います。

1. 完全なrequestをJSONとしてparseする。
2. `publisher.signature`だけを削除する。
3. `publisher.publicKey`とその他すべてのfieldを保持する。
4. `lb.canonical.json.v1`でcanonicalizeする。
5. byte-order markおよび末尾newlineなしでUTF-8 encodeする。
6. その正確なbytesをEd25519で直接署名する。

次を行ってはいけません。

- nested `object`だけを署名する。
- `publisher` object全体を削除する。
- canonical bytesではなくSHA-256 digestを署名する。
- source text内のmember順序へ依存する。
- newlineを追加する。
- 署名後にrequestを変更する。

reference producerは`createSignedPublishRequest`でこの手順を実装しています。

### 7. Local relayを起動する

local capabilityを確認します。

```bash
cargo run -p lingonberry-relay -- capabilities
```

loopbackでHTTP carrierを起動します。

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

別のterminalで確認します。

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
```

capability discoveryは記述情報です。runtime negotiation、automatic downgrade、または署名検証が強制されていることの証明ではありません。

### 8. Requestをpublishする

生成された正確なbytesを送信します。

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @/tmp/lingonberry-publish-request.json \
  http://127.0.0.1:8787/v1/objects
```

実装が署名を正しく再構築・再生成する場合を除き、署名済みrequestを書き換えたりpretty-printしたりしてはいけません。

### 9. 結果を解釈する

versioned ingestion resultには、次のような共通fieldが含まれます。

- `contractVersion`
- `status`
- `code`
- `stored`
- `duplicate`
- `errors`

結果に応じて、次が含まれる場合があります。

- `canonicalId`
- `identityKey`
- `carrierIdentity`
- `storedAt`
- `object`
- `quarantineId`

| Status | 意味 | Publisherの動作 |
|---|---|---|
| `stored` | 新しいcanonical objectが保存された | `canonicalId`を記録し、成功として扱う |
| `duplicate` | 同一publicationが既に存在する | idempotent successとして扱う |
| `deferred` | acceptance policyによりquarantineへ配置された | `quarantineId`を記録し、保存済みとして扱わない |
| `rejected` | request、object、identity、またはpolicy validationに失敗した | requestを修正してから再試行する |
| `conflict` | 同一canonical identityが異なるcontentへ対応する | 自動retryしない |
| `failed` | operationまたはstorage failureが発生した | 適切な場合にbounded backoffでretryする |

clientはHTTP statusとJSON bodyの両方を確認しなければなりません。

代表的なHTTP mapping:

| Ingestion status | HTTP status |
|---|---:|
| `stored` | `201 Created` |
| `duplicate` | `200 OK` |
| `deferred` | `202 Accepted` |
| 多くの`rejected`結果 | `400 Bad Request` |
| 未対応identity rule | `422 Unprocessable Entity` |
| `conflict` | `409 Conflict` |
| `failed` | `500 Internal Server Error` |

`202 Accepted` responseはcanonical storageの成功を意味しません。

### 10. Retrievalを検証する

`stored`または`duplicate`の場合、返されたidentifierを正確に取得します。

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

URL encodingを保持し、表示用dataから代替identifierを導出してはいけません。

### 11. Publisherのconformance check

publisher実装をcompatibleと呼ぶ前に、少なくとも次を確認します。

- reference producerの出力が現在のimplementation boundaryに従って受理される。
- canonical signing targetがrepository内のvectorと一致する。
- object fieldを変更するとtargetとsignatureが変わる。
- uppercaseまたは不正なhexadecimal encodingが拒否される。
- 長さが不正なkeyとsignatureが拒否される。
- duplicate publicationが2つ目のcanonical objectを作成しない。
- conflictをduplicateまたはtransient failureとして扱わない。
- `deferred`をcanonical storageとして扱わない。
- oversizedおよびdeeply nestedなuntrusted JSONが文書化された上限内で拒否される。
- response fieldへ依存する前にclientが`contractVersion`を確認する。

JavaScript conformance checkを実行します。

```bash
node --test conformance/minimal-producer.test.mjs
node conformance/run.mjs
```

activeなRustおよびJavaScript test suiteを、repository内compatibility behaviorの正本とします。

### 12. Private keyの取り扱い

relayはkeyを発行せず、publisher private keyを保持しません。

production publisherは、次を含む独自のkey lifecycleを提供しなければなりません。

- 安全な生成
- 保護された保存
- access control
- backupおよびrecovery policy
- 現在のsignature contract外でのrotationまたはrevocation policy

private keyをlogへ出力したり、request JSON、URL、diagnostic、fixture、issue reportへ含めたりしてはいけません。

### 13. 非保証事項

このQuickstartは次を確立または保証しません。

- 現在のingestion pathにおけるpublisher authenticationの強制
- publisher authorization、delegation、またはkey revocation
- replay preventionまたはtrusted timestamp
- network pub/sub delivery
- runtime capability negotiation
- automatic carrier fallback
- federatedまたはmulti-node synchronization
- production TLS terminationまたはdenial-of-service protection
- formal release qualificationまたはv1.0.0公開

### 参考文書

- [Developer Documentation](./README.md)
- [Repository Publish Walkthrough](./REPOSITORY_PUBLISH_WALKTHROUGH.md)
- [HTTP Publish Signature](../protocols/HTTP_PUBLISH_SIGNATURE.md)
- [Canonicalization](../protocols/CANONICALIZATION.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [Acceptance Policy](../operations/ACCEPTANCE_POLICY.md)
- [`conformance/minimal-producer.mjs`](../../conformance/minimal-producer.mjs)
