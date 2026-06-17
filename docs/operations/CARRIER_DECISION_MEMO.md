# Carrier Decision Memo

**Status: draft** | **Last updated: 2026-06-17**

## 目的

この文書は、Lingonberry の最初の正規 carrier をどう選ぶかを整理します。  
ここでの目的は、複数 carrier を否定することではなく、MVP で最初に実装すべき carrier を決めることです。  
あわせて、最初の HTTP carrier schema の基本方針も含めます。

## 決定

現時点の第一候補は **HTTP publish API** です。

その上で、後続 carrier として次を位置づけます。

- `relay-based pub/sub`
- `file/archive ingest`
- 将来の `federated sync` / `offline sync`

つまり、最初は **HTTP を入口にして、core の wire semantics を固定する** のがよいです。

## なぜ HTTP を最初にするか

### 1. 実装と検証がしやすい

HTTP は、publish と retrieve の最小ループを作りやすいです。  
最初の `knowledge object` を受け取り、validate / normalize / finalize し、canonical object を返す流れを構成しやすいです。

### 2. relay と API を分けやすい

HTTP publish は、`relay` の外側にある単なる変換層ではなく、carrier の 1 形として扱えます。  
そのうえで、relay は append-only log と配信を担い、API は canonical view を返す、という分離を作りやすいです。

### 3. Toitoi との接続がしやすい

Toitoi 側の edge や UI から見ると、HTTP は最も扱いやすい接続点です。  
application profile は Toitoi 側に残しつつ、Lingonberry core への入口をシンプルにできます。

### 4. capability negotiation に進みやすい

HTTP から始めると、後から `carrier capability`、`content negotiation`、`versioning` を足しやすいです。  
最初の wire semantics を固定する入口として扱いやすいです。

## 候補比較

### HTTP publish API

向いている点:

- 実装が分かりやすい
- テストしやすい
- Toitoi から接続しやすい
- 初期の operational friction が低い

注意点:

- push 型の分散配信そのものではない
- pub/sub をやるには別の機構が必要になる

### relay-based pub/sub

向いている点:

- 分散 relay モデルに自然に合う
- subscription と replay を扱いやすい
- push での配信に向く

注意点:

- 最初の実装としては HTTP より重い
- handshake、ordering、delivery semantics を先に詰める必要がある

### file/archive ingest

向いている点:

- 再現性が高い
- export/import に向く
- archive relay と相性がよい

注意点:

- 日常的な publish 入口としてはやや間接的
- interactive な利用には HTTP より向かない

### federated sync / offline sync

向いている点:

- 将来の分散同期に向く
- carrier 間の相互運用性を高められる

注意点:

- 初期版には重い
- identity / provenance / conflict policy を先に固める必要がある

## 採用方針

### 第一候補

- **HTTP publish API**

### 第二候補

- **file/archive ingest**

### 第三候補

- **relay-based pub/sub**

この順にすると、最初の MVP を小さく始めながら、後で分散配信へ拡張しやすくなります。

## HTTP carrier schema

### 前提

- HTTP は carrier であり、protocol の外側にある翻訳層ではない
- request body は protocol-native な knowledge object をそのまま運ぶ
- publish 主体は password ではなく公開鍵署名ベースで識別する
- author / actor の同定は object 本体ではなく request envelope と provenance で扱う
- publish 主体の public key は canonical には lowercase hex で扱う
- `npub` 形式は ingress で受けてもよいが、保存前に hex へ正規化する
- response は canonical view と metadata を返す
- error は carrier 固有の失敗ではなく、できるだけ protocol 的に扱えるようにする

### 入口

#### 1. Publish

- `POST /v1/objects`

##### Request body

```json
{
  "object": {
    "...": "knowledge object"
  },
  "publisher": {
    "publicKey": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "signature": "..."
  }
}
```

##### 使い方

- `object` に protocol-native な `knowledge object` を入れる
- `publisher.publicKey` で publish 主体を識別する
- `publisher.signature` で request の真正性を検証する
- `publisher.publicKey` は canonical には hex で扱う
- HTTP 側で semantic adapter を挟まない
- 受け取る object は `id`, `schemaVersion`, `type`, `createdAt`, `body`, `provenance`, `rawRef` を満たす
- 受け取った object は validate / normalize / finalize に渡す
- `publisher` は wire object そのものではなく request envelope に属する

##### 成功時の応答

```json
{
  "status": "ok",
  "id": "lb:obj:...",
  "identityKey": "lb:key:...",
  "canonical": {
    "...": "canonical knowledge object"
  },
  "rawRef": {
    "...": "raw reference"
  }
}
```

##### 期待する性質

- publish 後すぐ `id` を返せる
- canonical view を返せる
- rawRef を保持できる
- identityKey を必要に応じて返せる
- publisher の public key を provenance に引き渡せる
- `identityClaims` はあれば返してよいが、publish の必須応答にはしない

#### 2. Retrieve

- `GET /v1/objects/{id}`

##### 成功時の応答

```json
{
  "status": "ok",
  "canonical": {
    "...": "canonical knowledge object"
  },
  "rawRef": {
    "...": "raw reference"
  }
}
```

##### 期待する性質

- canonical id で取得できる
- carrier 固有の詳細を response から隠せる
- canonical object を安定して返せる
- rawRef を含む canonical representation を返せる

#### 3. Capability discovery

- `GET /v1/capabilities`

##### 応答の目的

carrier が何を受けられるか、どこまで互換かを公開します。

##### 返すべき情報

- protocol version
- supported object types
- supported schema version
- supported auth mode
- supported content type
- validation / finalize の制約

### 推奨 response 形

HTTP carrier の response は、なるべく次の 3 種類に絞ると扱いやすいです。

#### 1. Success

```json
{
  "status": "ok",
  "data": {}
}
```

#### 2. Validation error

```json
{
  "status": "error",
  "error": {
    "type": "validation_error",
    "message": "..."
  }
}
```

#### 3. Not found / unavailable

```json
{
  "status": "error",
  "error": {
    "type": "not_found",
    "message": "..."
  }
}
```

## carrier に求める条件

最初の carrier は、次を満たすべきです。

- protocol object をそのまま載せられる
- semantic adapter を不要にする
- wire object と canonical object を別プロトコルにしない
- validate / normalize / finalize に接続できる
- rawRef と provenance を保持できる
- replay 可能性を損なわない

## どの carrier を選ばないか

### 今は選ばないもの

- 独自バイナリプロトコル
- Toitoi 固有の transport
- semantic translation を前提にした gateway 専用 carrier

理由:

- core の wire semantics を複雑にしやすい
- 実装コストが上がる
- MVP の検証速度が落ちる

## 実装境界

### HTTP carrier で持つもの

- publish
- retrieve
- validation error
- capability discovery
- request / response schema

### relay / storage が持つもの

- append-only log
- replay
- provenance
- canonical catalog

### profile 側に残すもの

- domain-specific ルーティング
- domain-specific UI
- domain-specific curation rule

## 未決事項

次は別途決めます。

1. publish 成功時の返却形式
2. error model
3. authentication / authorization を初期版に含めるか
4. file/archive ingest の具体フォーマット
5. relay-based pub/sub の handshake と delivery semantics
6. HTTP carrier の response code と `status` のどちらを主にするか

## 見直し条件

この判断は、次のときに見直します。

- push 型配信が MVP の中心要件になったとき
- file/archive ingest が先に必要になったとき
- public relay の trust model が HTTP より別 carrier に向いたとき
- carrier 間同期の要件が早期に必要になったとき
