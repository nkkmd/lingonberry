# Schemas

このディレクトリには、Lingonberry の protocol-native な JSON Schema を置きます。

## 役割

この `schemas/` 配下の schema は、Lingonberry の protocol object を wire 上で扱うための参照実装です。

- `knowledge-object.schema.json` は、protocol-native な `knowledge object` 本体の schema です
- `http-publish-request.schema.json` は、HTTP publish 時の request envelope の schema です
- `http-publish-request.schema.json` の `object` は `knowledge-object.schema.json` を参照します
- HTTP publish では、まず request envelope を validate し、その後に `object` 本体を validate します

## 取り扱い

- `schemaVersion` は payload 側の contract version として扱います
- `knowledge-object.schema.json` は現行 baseline として `0.1.0` を使います
- `http-publish-request.schema.json` は request envelope なので、payload の `schemaVersion` とは別に document として version 管理します
- 変更時は [Migration and Schema Versioning](../docs/operations/MIGRATION_AND_SCHEMA_VERSIONING.md) と fixtures を同時に見直します

## 各 schema の説明

### `knowledge-object.schema.json`

`knowledge-object.schema.json` は、Lingonberry が扱う知識単位そのものを定義します。

この schema は、canonical 化される対象の最小構造と、その拡張点を定めます。現行 baseline では次を core の必須 field とします。

- `id`
- `schemaVersion`
- `type`
- `createdAt`
- `body`
- `provenance`
- `rawRef`

加えて、任意で `contexts`, `relations`, `status`, `lineage`, `identityClaims`, `attachments`, `labels`, `meta` を持てます。  
この schema は、carrier 固有の envelope ではなく、protocol object 本体の contract として扱います。

#### 主要 field の意味

- `id`: protocol 内で扱う canonical identity の参照軸です
- `schemaVersion`: payload 側の contract version です
- `type`: knowledge object の意味的な種別です
- `createdAt`: canonical な作成時刻です
- `body`: 本体の自然言語コンテンツです
- `provenance`: どこから来たか、誰が主張したかを示す来歴です
- `rawRef`: carrier 上の raw object への参照です
- `contexts`: 文脈メタデータです
- `relations`: 他の object との意味的なつながりです
- `lineage`: 派生・修正・翻訳などの履歴です
- `identityClaims`: identity key と canonical identity の対応を表す検証可能な主張です
- `attachments`: 補助的な添付情報です
- `labels`: 検索や分類用のラベルです
- `meta`: 非意味的な実装メタデータです

### `http-publish-request.schema.json`

`http-publish-request.schema.json` は、HTTP publish 用の request envelope を定義します。

この schema の主目的は、`knowledge object` 本体に加えて、publish 主体に関する carrier 側メタデータを一緒に運ぶことです。

- `object` に publish 対象の `knowledge object` を入れます
- `publisher` には publish 主体の公開鍵と署名を入れます
- `publisher` は wire object 本体の一部ではなく、carrier 側の付帯情報です

つまり、この schema は「知識オブジェクトそのもの」ではなく、「知識オブジェクトを HTTP で送るときの外側の入れ物」を表します。

#### 主要 field の意味

- `object`: publish 対象の `knowledge object` 本体です
- `publisher`: publish 主体の署名付きメタデータです
- `publisher.publicKey`: publisher 側で用意した Ed25519 の公開鍵です
- `publisher.signature`: `publisher.signature` を除いた canonicalized request payload に対する署名です

`publisher` は object の意味内容ではなく、carrier 側で publish を成立させるための情報です。

## 読み方

まず `knowledge-object.schema.json` を見て、protocol object の本体を把握します。  
その上で `http-publish-request.schema.json` を読むと、HTTP publish にだけ必要な envelope の責務が分かりやすくなります。

## 文書

- [knowledge-object.schema.json](./knowledge-object.schema.json)
- [http-publish-request.schema.json](./http-publish-request.schema.json)
