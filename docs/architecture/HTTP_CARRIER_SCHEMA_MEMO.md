# HTTP Carrier Schema Memo

**Status: draft** | **Last updated: 2026-06-17**

## 目的

この文書は、Lingonberry の最初の `HTTP publish API` で使う request / response schema の考え方を整理します。  
ここでの目的は、HTTP を別プロトコル化することではなく、**protocol object をそのまま扱える最小の HTTP 形を決めること**です。

## 前提

- HTTP は carrier であり、protocol の外側にある翻訳層ではない
- request body は protocol object をそのまま運ぶ
- response は canonical view と metadata を返す
- error は carrier 固有の失敗ではなく、できるだけ protocol 的に扱えるようにする

## 入口

### 1. Publish

- `POST /v1/objects`

#### Request body

```json
{
  "object": {
    "...": "knowledge object"
  }
}
```

#### 使い方

- `object` に protocol-native な `knowledge object` を入れる
- HTTP 側で semantic adapter を挟まない
- 受け取った object は validate / normalize / finalize に渡す

#### 成功時の応答

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

#### 期待する性質

- publish 後すぐ `id` を返せる
- canonical view を返せる
- rawRef を保持できる
- identityKey を必要に応じて返せる

### 2. Retrieve

- `GET /v1/objects/{id}`

#### 成功時の応答

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

#### 期待する性質

- canonical id で取得できる
- carrier 固有の詳細を response から隠せる
- canonical object を安定して返せる

### 3. Capability discovery

- `GET /v1/capabilities`

#### 応答の目的

carrier が何を受けられるか、どこまで互換かを公開します。

#### 返すべき情報

- protocol version
- supported object types
- supported schema version
- supported auth mode
- supported content type
- validation / finalize の制約

## 推奨 response 形

HTTP carrier の response は、なるべく次の 3 種類に絞ると扱いやすいです。

### 1. Success

```json
{
  "status": "ok",
  "data": {}
}
```

### 2. Validation error

```json
{
  "status": "error",
  "error": {
    "type": "validation_error",
    "message": "..."
  }
}
```

### 3. Not found / unavailable

```json
{
  "status": "error",
  "error": {
    "type": "not_found",
    "message": "..."
  }
}
```

## エラー方針

### validation error

次のような場合に返します。

- schema が壊れている
- required field がない
- format が合わない
- identity / provenance の最小整合性がない

### conflict / duplicate

次のような場合に返します。

- 同じ identity key に対して重複した publish が来る
- carrier identity で既存 log と衝突する

### not found

次のような場合に返します。

- canonical id が存在しない
- rawRef が解決できない

## schema 設計の方針

### 1. request body は薄くする

request body に余計な carrier 情報を混ぜない方がよいです。  
中身は基本的に `knowledge object` そのものに寄せます。

### 2. response は canonical を返す

HTTP で返すべき主役は raw ではなく canonical view です。  
必要なら rawRef を添えます。

### 3. carrier metadata は header か別 envelope

request id、correlation id、transport debug info などは、object 本体に埋め込まない方がよいです。  
HTTP 固有 metadata は header か別 envelope に閉じます。

### 4. content type は versioned にする

将来の互換性のために、content type か version field は明示します。

## 推奨する最小バージョン

### publish request

- content type: `application/json`
- version: `/v1`

### response

- JSON
- success / error を明示

## 何をやらないか

- HTTP を semantic adapter にしない
- request body で別 protocol へ翻訳しない
- carrier ごとに object schema を変えない
- publish と finalize を別系統の意味にしない

## 未決事項

次は別途決めます。

1. request body を `object` 包み込みにするか、object 直置きにするか
2. `status: ok` 形式に統一するか、HTTP status code を主にするか
3. auth を初期版に入れるか
4. `identityKey` を publish 応答で常に返すか
5. `capabilities` の詳細粒度

## 見直し条件

この schema は次のときに見直します。

- HTTP が publish だけでなく subscription も担うようになったとき
- file/archive ingest と response 形を揃えたくなったとき
- auth / authorization を強く組み込みたくなったとき
- carrier capability をより厳密に公開する必要が出たとき

