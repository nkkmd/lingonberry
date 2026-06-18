# HTTP Carrier Contract

**Status: draft** | **Last updated: 2026-06-18**

## 目的

この文書は、Lingonberry の最初の正規 carrier としての HTTP の契約を定義します。

ここでの HTTP は、protocol の外側にある単なる transport ではありません。  
protocol object を wire 上で成立させる正規の carrier 実装です。

## 範囲

HTTP carrier が扱うのは次の 3 つです。

- publish
- retrieve
- capability discovery

HTTP carrier は、semantic adapter や別 protocol への翻訳層を持ちません。

## 1. Publish

### Endpoint

- `POST /v1/objects`

### Request

request body は `http-publish-request` envelope とします。

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

### 使い方

- `object` に protocol-native な `knowledge object` を入れる
- `publisher` は carrier 側メタデータとして扱う
- `publisher.publicKey` は canonical には lowercase hex で扱う
- `publisher.signature` は request payload あるいは carrier が定義する canonicalized subset を覆う
- 受信後は `validate -> normalize -> finalize` に渡す

### 期待する性質

- 同じ入力は同じ canonical object に着地する
- publish 成功時に canonical view を返せる
- rawRef と provenance を失わない
- carrier 固有の意味を object に埋め込まない

### 成功応答

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

## 2. Retrieve

### Endpoint

- `GET /v1/objects/{id}`

### 返却方針

- canonical view を基本返却にする
- rawRef を含めて返せる
- carrier 固有の詳細はできるだけ隠す

### 成功応答

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

## 3. Capability discovery

### Endpoint

- `GET /v1/capabilities`

### 返すべき情報

- protocol version
- supported object types
- supported schema versions
- supported auth modes
- supported content types
- validation / finalize constraints
- supported access scopes
- supported retention hints

## 4. Response model

HTTP carrier の response は、次の 3 種類に寄せます。

### Success

```json
{
  "status": "ok",
  "data": {}
}
```

### Validation error

```json
{
  "status": "error",
  "error": {
    "type": "validation_error",
    "message": "..."
  }
}
```

### Not found / unavailable

```json
{
  "status": "error",
  "error": {
    "type": "not_found",
    "message": "..."
  }
}
```

## 5. 実装境界

- HTTP carrier は request / response contract を持つ
- relay / storage は append-only log、replay、provenance、canonical catalog を持つ
- profile 側は domain-specific routing や curation rule を持つ
- access / retention の既定値は protocol semantic ではなく運用ポリシーに従う
- HTTP では public を既定、curated は任意、private は初期版では無効を基本とする

## 6. 未決事項

次は運用で詰めます。

1. publish 成功時の返却形式の細部
2. error model の拡張
3. authentication / authorization を初期版に含めるか
4. response code と `status` の主従
5. access / retention hint の wire 表現

## 関連

- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [Knowledge Object schema](../../schemas/knowledge-object.schema.json)
- [HTTP Publish Request schema](../../schemas/http-publish-request.schema.json)
