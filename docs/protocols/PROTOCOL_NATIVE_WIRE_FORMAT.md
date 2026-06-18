# Protocol-Native Wire Format

**Status: draft** | **Last updated: 2026-06-17**

## 目的

この文書は、Lingonberry の protocol object を wire 上でどう表現するかを定義します。

ここでの前提は次の通りです。

- transport と protocol を分けない
- carrier は protocol の正規実装である
- wire object は protocol object そのものである

## 基本方針

wire format は、canonical knowledge object をそのまま carrier に載せられるように設計します。

そのため、converter や semantic adapter は原則として不要です。
必要なのは、次の最小限です。

- serialize / deserialize
- validate
- normalize
- finalize

HTTP publish のような carrier では、`publisher` のような request envelope を別に持てます。  
ただし、それは wire object 本体の一部ではなく、publish する protocol object を運ぶ carrier 側のメタデータです。

## Wire Object

wire object は、carrier 上で受け渡される protocol object です。

wire object は、追加の意味変換を前提にしません。

### 必須性質

- deterministic であること
- append-only の replay に耐えること
- provenance を保持できること
- raw reference を保持できること
- versioned であること

### 参照 schema で固定すること

現在の参照 schema では、`id`, `schemaVersion`, `type`, `createdAt`, `body`, `provenance`, `rawRef` を core の最小必須として扱います。
`identityClaims` は任意で、identity resolution の実用化は後続フェーズで扱います。

### Identity key と identity claim

Phase 3 では、identity key を versioned な導出規則で決定的に計算し、その対応を identity claim で表します。

詳細は [Identity and Provenance](./IDENTITY_AND_PROVENANCE.md) を参照してください。

identity claim を含む具体例は [fixtures/knowledge-object/with-identity-claim.json](../../fixtures/knowledge-object/with-identity-claim.json) と [fixtures/http-publish-request/with-identity-claim.json](../../fixtures/http-publish-request/with-identity-claim.json) に切り出します。

### 最小例

次は、Phase 0 で想定する最小の wire object 例です。

具体例は [fixtures/knowledge-object/minimal-wire-object.json](../../fixtures/knowledge-object/minimal-wire-object.json) に切り出しています。

この例は、`identityClaims` を含まない最小形です。  
`contexts`, `relations`, `status`, `lineage`, `attachments`, `labels`, `meta` は必要に応じて追加できます。

### 不正例

`rawRef` を欠く例は、schema validation で reject されるべきです。具体例は [fixtures/knowledge-object/invalid-missing-rawref.json](../../fixtures/knowledge-object/invalid-missing-rawref.json) に置いています。

## 参照 schema

現在の参照 schema は次のものです。

- [knowledge-object.schema.json](../../schemas/knowledge-object.schema.json)
- [http-publish-request.schema.json](../../schemas/http-publish-request.schema.json)

これらの schema は、protocol-native な knowledge object と HTTP publish request envelope の構造を定義します。

## 代表的な処理

### 1. serialize

protocol object を carrier に載せるための表現へ変換します。

### 2. deserialize

carrier から protocol object を復元します。

### 3. validate

次を確認します。

- schema が正しいこと
- 必須 field が揃っていること
- identity と provenance の基本整合性があること
- carrier が許容する framing に収まっていること

HTTP publish の場合は、まず request envelope を validate し、その後に `object` 本体を validate します。
`publisher` は carrier 側メタデータとして扱い、wire object 本体の必須 field とは分離します。

### 4. normalize

次を揃えます。

- field の順序
- language tag
- timestamp
- relation / lineage の構造
- 省略可能 field の default 扱い
- identityClaims の順序や表現の揺れ

### 5. finalize

wire object を canonical knowledge object として確定します。

この段階で行うこと:

- canonical id を解決する
- provenance を付与する
- raw/wire reference を保持する
- identityClaims があれば canonical id との対応を保持する
- deterministic な object 表現を得る

Phase 1 では canonical id の新規発番ロジックを増やしません。
validate 済み object が持つ `id` を canonical id の参照軸として保持し、以後の保存・再取得で安定して扱えるようにします。

## ルール

- carrier 固有の意味を object に埋め込まない
- wire object と canonical object を別系統にしない
- 変換のための中間 semantic layer を増やしすぎない
- 不確かな identity は強制的に統合しない

## Compatibility

wire format を変える場合は、protocol version と carrier capability を同時に考慮します。

互換性の判断基準:

- 既存 object を replay できるか
- 既存 identity が壊れないか
- provenance と raw reference が失われないか
- index の再構築が可能か

## 関連

- [Carrier](../concepts/CARRIER.md)
- [概念モデル](../concepts/CONCEPT_MODEL.md)
- [Knowledge Object schema](../../schemas/knowledge-object.schema.json)
