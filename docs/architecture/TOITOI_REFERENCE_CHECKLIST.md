# Toitoi 参照チェックリスト

**Status: draft** | **Last updated: 2026-06-16**

## 目的

この文書は、新しい **分散知識コモンズ・プロトコル** を設計・実装するときに、Toitoi 側のどの情報を確認すべきかを整理した実務用チェックリストです。

狙いは次の通りです。

- Toitoi 固有の概念と、新プロトコルの共通基盤を混同しない
- 既存の正本を見失わずに、必要な情報だけを移植する
- 後から設計の根拠を辿れるようにする

この文書は Toitoi の仕様そのものではありません。  
新プロトコル開発のために、Toitoi から何を参照すべきかをまとめた案内です。

---

## まず確認する正本

新プロトコルの中核設計に入る前に、まず次の文書と schema を確認します。

### 1. canonical event schema

- [schemas/canonical-event.schema.json](/home/oruorane/github/Toitoi/schemas/canonical-event.schema.json)
- 目的: 正規化された知識オブジェクトの最小構造を確認する
- 見るべき点:
  - `id` の形式
  - `schemaVersion`
  - `type`
  - `body.text` と `body.language`
  - `contexts`
  - `relationships`
  - `lineage`
  - `provenance`
  - `rawRef`
  - `identityClaims`
  - `labels` と `meta`

### 2. identity claim schema

- [schemas/identity-claim.schema.json](/home/oruorane/github/Toitoi/schemas/identity-claim.schema.json)
- 目的: identity key と canonical id の対応をどう検証するかを確認する
- 見るべき点:
  - `identityKey`
  - `canonicalId`
  - `issuer`
  - `issuedAt`
  - `verification`
  - `payloadHash`
  - `signature`
  - `status`

### 3. canonical identity と provenance の正本

- [docs/concepts/CANONICAL_IDENTITY_AND_PROVENANCE.md](/home/oruorane/github/Toitoi/docs/concepts/CANONICAL_IDENTITY_AND_PROVENANCE.md)
- 目的: identity 周りの責務分離を確認する
- 見るべき点:
  - carrier identity
  - canonical identity
  - identity key
  - identity claim
  - provenance
  - rawRef
  - 同一性の根拠として何を使い、何を使わないか

### 4. canonical event の正本

- [docs/protocols/CANONICAL_EVENT.md](/home/oruorane/github/Toitoi/docs/protocols/CANONICAL_EVENT.md)
- 目的: append-only、raw / canonical 分離、index の位置づけを確認する
- 見るべき点:
  - raw wire event
  - normalized event
  - canonicalized event
  - derived index
  - storage snapshot
  - protocol-independent representation
  - carrier 由来の判断を semantic layer に持ち込まない方針

### 5. 用語集

- [docs/concepts/GLOSSARY.md](/home/oruorane/github/Toitoi/docs/concepts/GLOSSARY.md)
- 目的: `inquiry`、`boundary object`、`commons`、`relay`、`indexer` の意味を揃える
- 見るべき点:
  - `inquiry`
  - `question`
  - `canonical event`
  - `provenance`
  - `lineage`
  - `boundary object`
  - `commons`
  - `relay`
  - `indexer`
  - `canonical view`
  - `protocol`
  - `carrier`

---

## 移植すべき情報

新プロトコルへ移すべきなのは、Toitoi 固有の「農業の内容」ではなく、**設計原理** です。

### A. そのまま持っていくもの

- append-only
- replayable
- provenance-aware
- canonical identity first
- raw と canonical の分離
- lineage の保持
- index は派生構造であること
- protocol と carrier を同一視すること

### B. 一般化して持っていくもの

- `canonical event` の考え方
- `identity claim` の考え方
- `rawRef` の考え方
- `provenance` の考え方
- `boundary object` の考え方

### C. Toitoi 固有として外すもの

- 農業前提の `contexts`
- `inquiry` を唯一の中心型にすること
- Nostr 固有の `kind` やタグ表現
- Toitoi 固有語彙を protocol core に入れること

---

## 具体的に確認する項目

この章は、設計会議や実装着手の前に順番に見るとよい項目です。

### 1. オブジェクトの中心型

確認すること:

- 中心型を `inquiry` に限定するか
- `observation`、`claim`、`evidence` まで最初から同格にするか
- knowledge object を 1 つの共通型として扱うか

参照先:

- [schemas/canonical-event.schema.json](/home/oruorane/github/Toitoi/schemas/canonical-event.schema.json)
- [docs/concepts/GLOSSARY.md](/home/oruorane/github/Toitoi/docs/concepts/GLOSSARY.md)

### 2. identity の扱い

確認すること:

- canonical id を opaque にするか
- identity key をどう定義するか
- identity claim を必須にするか任意にするか
- 同一性の根拠をどこまで明示的に要求するか

参照先:

- [schemas/identity-claim.schema.json](/home/oruorane/github/Toitoi/schemas/identity-claim.schema.json)
- [docs/concepts/CANONICAL_IDENTITY_AND_PROVENANCE.md](/home/oruorane/github/Toitoi/docs/concepts/CANONICAL_IDENTITY_AND_PROVENANCE.md)

### 3. provenance と raw reference

確認すること:

- provenance に何を含めるか
- rawRef を何のために持つか
- provenance と rawRef をどう分離するか

参照先:

- [schemas/canonical-event.schema.json](/home/oruorane/github/Toitoi/schemas/canonical-event.schema.json)
- [docs/concepts/CANONICAL_IDENTITY_AND_PROVENANCE.md](/home/oruorane/github/Toitoi/docs/concepts/CANONICAL_IDENTITY_AND_PROVENANCE.md)

### 4. 変更と履歴

確認すること:

- 修正を上書きで扱うか
- revision と tombstone を許すか
- `lineage` をどう使うか
- `revises`、`supersedes`、`derived_from` をどう扱うか

参照先:

- [docs/protocols/CANONICAL_EVENT.md](/home/oruorane/github/Toitoi/docs/protocols/CANONICAL_EVENT.md)
- [schemas/canonical-event.schema.json](/home/oruorane/github/Toitoi/schemas/canonical-event.schema.json)

### 5. carrier と relay

確認すること:

- relay は保存と配信だけを担うのか
- 最小限の検証や重複排除を標準責務に入れるか
- どの carrier を最初の運搬路にするか

参照先:

- [docs/protocols/CANONICAL_EVENT.md](/home/oruorane/github/Toitoi/docs/protocols/CANONICAL_EVENT.md)
- [docs/architecture/MULTI_PROTOCOL_INDEXER.md](/home/oruorane/github/Toitoi/docs/architecture/MULTI_PROTOCOL_INDEXER.md)

### 6. indexer と API

確認すること:

- indexer を semantic source にしない方針を守るか
- canonical view を API の標準にするか
- relation、lineage、provenance の検索をどう公開するか

参照先:

- [docs/architecture/MULTI_PROTOCOL_INDEXER.md](/home/oruorane/github/Toitoi/docs/architecture/MULTI_PROTOCOL_INDEXER.md)
- [docs/architecture/DIRECTORY_BOUNDARIES.md](/home/oruorane/github/Toitoi/docs/architecture/DIRECTORY_BOUNDARIES.md)

### 7. 語彙とアプリケーション・プロファイル

確認すること:

- domain vocabulary を core から分離するか
- アプリケーション・プロファイルをどう定義するか
- Toitoi を 1 つの profile として扱うか

参照先:

- [docs/concepts/GLOSSARY.md](/home/oruorane/github/Toitoi/docs/concepts/GLOSSARY.md)
- [README.md](/home/oruorane/github/Toitoi/README.md)

---

## Toitoi から抽出したい設計情報の要約

### 正本

- canonical event schema
- identity claim schema
- canonical identity と provenance の仕様
- canonical event の責務分離
- wire object と canonical object の責務分離

### 語彙

- inquiry
- question
- boundary object
- commons
- relay
- indexer
- canonical view

### 実装境界

- raw / normalized / canonical の段階分離
- codec / validator / indexer / API の責務分離
- carrier と protocol の同一化

### 将来の一般化対象

- knowledge object の多型化
- domain vocabulary の外部化
- multi-carrier 化
- profile 駆動化

---

## この文書の使い方

1. まず正本を読む
2. 次に移植すべき情報と外す情報を分ける
3. その後に新プロトコルの schema と core model を書く
4. 実装を始める前に、identity と provenance の方針を固定する

この順番で進めると、Toitoi 固有の都合に引きずられずに、分野非依存のプロトコルへ拡張しやすくなります。
