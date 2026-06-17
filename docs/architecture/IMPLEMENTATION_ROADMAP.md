# 実装ロードマップ

**Status: draft** | **Last updated: 2026-06-17**

## 目的

この文書は、Lingonberry を実装して、最終的に次の状態へ到達するためのロードマップをまとめます。

- 誰でも `relay` を立てられる
- 誰でも `storage node` を立てられる
- 誰でも `knowledge object` を publish できる
- Toitoi の基盤として利用できる

このロードマップは、core protocol を Toitoi に寄せすぎないことを前提にします。  
Toitoi は最初の重要な application profile ですが、core の設計は分野非依存のまま保ちます。

## 実装の前提

- `wire` と `canonical` は別プロトコルではなく、同じ protocol object の別表現として扱う
- `carrier` は protocol object を wire 上で運ぶ正規の実装として扱う
- `canonical id` は opaque に保つ
- `identity key` は deterministic / content-rule-derived にする
- `relay` は semantic truth を決めない
- `index` は派生構造であり、semantic source にはしない
- `append-only` と `replayable` を壊さない

## フェーズ 0: 仕様の固定点を作る

Phase 0 は仕様固定のための文書作業として完了しています。  
以後の変更は、Phase 1 以降の実装や新しい仕様判断に伴う更新として扱います。

### 目的

実装を始める前に、最小限の仕様境界を固定します。

### やること

- `knowledge object` の最小必須フィールドを確定する
- `identity claim` と `provenance` の責務分離を明文化する
- `rawRef` の役割を固定する
- `relation` と `lineage` の使い分けを整理する
- protocol-native wire format の validate / normalize / finalize の境界を決める

### 固定する境界

Phase 0 では、core の固定点を次のように扱います。

- 最小必須フィールドは `id`, `schemaVersion`, `type`, `createdAt`, `body`, `provenance`, `rawRef`
- `identityClaims` は core では任意で、Phase 3 で実用化する
- `contexts`, `relations`, `status`, `lineage`, `attachments`, `labels`, `meta` は拡張可能な任意フィールド
- `rawRef` は provenance と別責務で、carrier 上の raw object / payload への参照として扱う
- `relation` は意味的なつながり、`lineage` は派生・修正の履歴として扱う
- `validate / normalize / finalize` は別プロトコル変換ではなく、同じ protocol object を canonical object として確定する手順として扱う

### 成果物

- 更新済みの概念文書
- 更新済みの schema
- 最低限の wire format 仕様

### 完了条件

- 仕様を読めば、publish された object の canonicalization 規則を再実装できる
- Toitoi 固有の語彙を core に入れずに説明できる

## フェーズ 1: 単一オブジェクトの publish 経路を作る

### 目的

1 件の `knowledge object` を、草稿から canonical storage まで通せるようにします。

### やること

- ローカル authoring 入口を作る
- wire object を validate する
- normalize する
- canonical object として finalize する
- append-only で保存する
- canonical id で取得できるようにする

### 最小実装の形

- 入力: JSON か protocol-native wire 表現
- 処理: validate → normalize → finalize
- 出力: canonical object の保存と取得

### 成果物

- publish API または publish CLI
- 単一ノードの永続化層
- canonical object 取得 API

### 完了条件

- 1 つの object を publish できる
- 再取得時に canonical object が安定して返る
- 不正な wire object は reject される

## フェーズ 2: relay と storage node を分離して成立させる

### 目的

「publish できる」だけでなく、「relay を立てられる」「storage node を立てられる」を別責務として実装します。

### やること

- relay の append-only log を実装する
- storage backend を抽象化する
- replay を実装する
- duplicate detection を carrier identity / identity key で扱う
- subscription の最小形を実装する

### relay の責務

- wire object を受け取る
- wire-level の整合性を確認する
- append-only で保存する
- replay と export を返す

### storage node の責務

- 長期保管
- replay 可能性の維持
- 取り込み済み object の検索補助

### 成果物

- relay daemon
- storage backend 実装
- replay/export 経路

### 完了条件

- publish した object が relay に保存される
- relay の log から state を再構成できる
- storage node を独立に立てても replay が壊れない

## フェーズ 3: identity と provenance を実用化する

### 目的

carrier をまたいでも同じ semantic object を追跡できるようにします。

### やること

- identity key の導出規則を実装する
- canonical id と identity key の対応を claim として表現する
- provenance に source / author / time / transform chain を載せる
- rawRef を保持して再解析可能にする

### 成果物

- identity resolution ロジック
- identity claim の生成・検証
- provenance の保存と取得

### 完了条件

- 複数 carrier 由来の object を同じ semantic object として扱える
- provenance を見れば来歴を辿れる
- raw payload を再取得して再 canonicalize できる

## フェーズ 4: indexer と API を分離する

### 目的

検索しやすさを追加しつつ、semantic source を canonical store に残します。

### やること

- type index を作る
- relation graph を作る
- lineage graph を作る
- provenance graph を作る
- text search と facet search を作る
- canonical view API を設計する

### 成果物

- indexer
- query API
- canonical view / list / graph 系エンドポイント

### 完了条件

- canonical store とは独立に index を再構築できる
- search 結果から provenance と lineage を辿れる
- index の破損が semantic data の破損にならない

## フェーズ 5: Toitoi 用の application profile を載せる

### 目的

Lingonberry を Toitoi の基盤として使えるようにします。

### やること

- Toitoi 用の application profile を分離する
- inquiry / observation / evidence / synthesis / annotation の使い方を profile 側で定義する
- Toitoi 固有の context 語彙を core から切り離す
- Toitoi が必要とする API だけを profile として公開する

### 成果物

- application profile 文書
- Toitoi 接続用の最小 API 契約
- profile 固有の validation / vocabulary 追加点

### 完了条件

- Toitoi が core protocol を fork せずに載る
- core は Toitoi 固有語彙に縛られない

## フェーズ 6: carrier 拡張と運用性を高める

### 目的

公開運用に必要な carrier の選択肢と運用機能を増やします。

### やること

- HTTP carrier を固める
- file / archive carrier を固める
- 将来の federated sync carrier に備える
- capability negotiation を整える
- access policy と retention policy を整理する
- migration / schema versioning を整える

### 成果物

- carrier 別実装
- capability 文書
- versioning / migration 方針

### 完了条件

- carrier が増えても semantic model が変わらない
- replay と canonicalization が carrier 非依存で動く

## 推奨する実装順

1. フェーズ 0
2. フェーズ 1
3. フェーズ 2
4. フェーズ 3
5. フェーズ 4
6. フェーズ 5
7. フェーズ 6

## 最初の MVP の定義

最初の MVP は、次の 3 点が満たせる状態です。

- 1 つの `knowledge object` を publish できる
- relay が append-only で保存できる
- canonical id で再取得できる

この MVP ができたら、次に identity / provenance / replay / index を順に固めます。
