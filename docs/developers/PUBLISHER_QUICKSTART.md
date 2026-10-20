# Publisher Quickstart / 公開クライアントの入門ガイド

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は同期翻訳です。内容に差異がある場合は英語版を優先します。

## English

**Execution hold (2026-10-20):** Reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred at the maintainer’s request. Resume only after an explicit maintainer request; v1.0.0 remains unpublished.

**Status: v1.0 pre-release developer guide** | **Last updated: 2026-10-20**

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

**実機試験の保留（2026-10-20）:** 基準ホストでの事前検証、ディスク圧迫試験、正式72時間連続稼働試験は、管理者の指示により保留しています。明示的な再開指示を受けてから再開します。v1.0.0は未公開です。

**状態: v1.0公開前の開発者ガイド** | **最終更新: 2026-10-20**

このガイドは、知識オブジェクト（Knowledge Object）をLingonberryのリレーへ送信するアプリケーション、サービス、CLI、コネクター、プロトコルアダプターの開発者向けです。

### 1. 公開処理の流れ

```text
知識オブジェクトを作成
→ HTTP公開要求のデータ構造に組み込む
→ 正規化された署名対象データを構築
→ Ed25519秘密鍵で署名
→ POST /v1/objects
→ HTTPステータスと取込結果の応答本文を確認
→ 保存成功時は返されたcanonicalIdで取得
```

正規化や署名を独自に実装し直すのではなく、リポジトリ付属のJavaScript生成ツール（参照実装）を参照してください。

### 2. リリース状況と署名検証の適用範囲

- 最新の公開版: `v0.9.0`
- 固定したv1.0公開候補: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`
- 公開候補の検証／文書の手順に沿った動作確認: 合格
- 基準ホストでの管理者権限による事前検証: 次の検証項目
- 正式な72時間連続稼働試験／v1.0.0公開: 未開始

正本となる署名規則はEd25519と`lb.canonical.json.v1`を使用する`lb.http.publish.signature.v1`です。

現在の取込処理では、JSONの解析直後に公開者の署名を検証します。署名を検証できなければ、受入方針の適用、隔離、重複・競合の判定、受信データの追記、正本となるストレージへの保存のいずれにも進まず、要求を拒否します。

`stored`または`duplicate`は必須の署名検証を通過した要求です。ただし公開者への権限付与、鍵の発行、権限委譲、失効、再送攻撃の防止まで保証しません。

### 3. 準備と公開候補の取得

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
git checkout 8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

公開候補の決定後に追加する検証記録や文書のコミットは公開候補を変更しません。

### 4. 署名済み要求の生成

```bash
node conformance/minimal-producer.mjs > /tmp/lingonberry-publish-request.json
```

カスタマイズ例:

```bash
node conformance/minimal-producer.mjs \
  --id 'lb:obj:publisher-example-0001' \
  --created-at '2026-07-27T00:00:00Z' \
  --text 'Can an external publisher submit this Knowledge Object?' \
  --language 'en' \
  > /tmp/lingonberry-publish-request.json
```

参照実装の生成ツールは一時的なEd25519鍵ペアを生成し、正規化された署名対象データに署名します。秘密鍵の管理機能ではありません。

### 5. リレーの起動と送信

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

別の端末で送信します。

```bash
curl -sS \
  -H 'content-type: application/json' \
  --data-binary @/tmp/lingonberry-publish-request.json \
  http://127.0.0.1:8787/v1/objects
```

- `stored`: 新しい署名検証済みオブジェクトを保存
- `duplicate`: 同じ署名検証済みの公開データが既に存在する
- `deferred`: 隔離領域へ移し、正本となるストレージには未保存
- `rejected`: スキーマ、識別情報、署名、方針による拒否
- `conflict`: 同じ正規化された識別情報（canonical identity）に、異なる内容が対応している
- `failed`: 署名検証基盤、操作、またはストレージでの失敗

署名に関するエラーコード:

- `LB_PUBLISH_SIGNATURE_MALFORMED`
- `LB_PUBLISH_SIGNATURE_INVALID`
- `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`

### 6. 取得と結果の処理

```bash
curl -sS 'http://127.0.0.1:8787/v1/objects/<canonical-id>'
```

署名後の要求を書き換えず、`duplicate`は再送しても重複を増やさない成功、`deferred`は未保存、`conflict`は一時的な障害ではない結果として扱ってください。秘密鍵をログ、サンプルデータ、リポジトリ、検証記録一式へ保存しないでください。

### 7. このガイドで保証しない事項

このガイドは公開者への権限付与／鍵のライフサイクル、ノード間連携、本番環境のTLS、DoS対策、正式な72時間連続稼働試験完了、v1.0.0公開を保証しません。
