# 実装ロードマップ

**Status: draft** | **Last updated: 2026-06-18**

## 目的

この文書は、Lingonberry を実装して、最終的に次の状態へ到達するためのロードマップをまとめます。

- 誰でも `relay` を立てられる
- 誰でも `storage node` を立てられる
- 誰でも `knowledge object` を publish できる
- Toitoi の基盤として利用できる

このロードマップは、core protocol を Toitoi に寄せすぎないことを前提にします。  
Toitoi は最初の重要な application profile ですが、core の設計は分野非依存のまま保ちます。

実装の本命は [技術決定 ADR](../operations/TECH_DECISION_ADR.md) に合わせて **Rust + SQLite** です。
Phase 1 で進めている JavaScript 実装は、その本命実装に移行する前の検証用ブートストラップとして扱います。

## 実装の前提

- `wire` と `canonical` は別プロトコルではなく、同じ protocol object の別表現として扱う
- `carrier` は protocol object を wire 上で運ぶ正規の実装として扱う
- `canonical id` は opaque に保つ
- `identity key` は deterministic / content-rule-derived にする
- `relay` は semantic truth を決めない
- `index` は派生構造であり、semantic source にはしない
- `append-only` と `replayable` を壊さない
- publish 時の author identity は password ではなく公開鍵署名ベースで扱う
- author / actor の同定は provenance と identity claim に分けて扱う

## フェーズ 0: 仕様の固定点を作る（完了済み）

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

## フェーズ 1: 単一オブジェクトの publish 経路を作る（完了済み）

### 目的

1 件の `knowledge object` を、草稿から canonical storage まで通せるようにします。

### 実装の進め方

Phase 1 は、まず HTTP publish を最小の carrier として固定して進めます。

1. `http-publish-request` を入口にする
2. request envelope を validate する
3. `object` 本体を `knowledge-object` schema で validate する
4. normalize を pure transform として実装する
5. finalize で canonical object を確定する
6. append-only storage に保存する
7. canonical id で再取得できるようにする

この段階では、canonical id の新規発番ルールを追加しません。
validate 済みの object が持つ `id` を、そのまま canonical object の参照軸として保持します。
`id` の再設計が必要になった場合は、Phase 3 以降の identity 作業として切り出します。

### やること

- ローカル authoring 入口を作る
- publish 主体を公開鍵で識別し、署名付き request を受けられるようにする
- wire object を validate する
- normalize する
- canonical object として finalize する
- append-only で保存する
- canonical id で取得できるようにする

### 最小実装の形

- 入力: JSON か protocol-native wire 表現
- 処理: validate → normalize → finalize
- 出力: canonical object の保存と取得

### 最初の着手点

- `schemas/http-publish-request.schema.json` を publish 入力の入口として扱う
- `schemas/knowledge-object.schema.json` を object 本体の検証基準にする
- `fixtures/http-publish-request/*.json` を request 検証の fixtures として使う
- `fixtures/knowledge-object/*.json` を object 検証の fixtures として使う

### 成果物

- publish API または publish CLI
- 単一ノードの永続化層
- canonical object 取得 API

Phase 1 の最初のスキャフォールドは、依存を増やさずに fixture を直接回しやすい `publish CLI` を優先します。
CLI 実装は、推奨リポジトリ構成に合わせて `packages/cli/` に置き、validate / normalize / finalize の共通処理は `packages/codecs/` に寄せます。
単一ノード保存と再取得は `packages/core/` に置き、CLI から `publish` と `get` で触れる最小形から始めます。
canonical view の組み立ては `packages/api/` に置き、`get` の返却形をそこへ寄せます。
この JavaScript 実装は、Rust 版へ移植するための挙動確認・境界固定の段階であることを前提にします。

### 完了条件

- 1 つの object を publish できる
- 再取得時に canonical object が安定して返る
- 不正な wire object は reject される

### 完了した実装

- `packages/codecs/` に validate / normalize / finalize の最小実装を配置
- `packages/core/` に append-only JSONL ストアと再取得を配置
- `packages/api/` に canonical view の組み立てを配置
- `packages/cli/` に `validate` / `publish` / `get` / `list` を配置
- `fixtures/` に検証手順を追加
- `.lingonberry/` をローカル保存先として無視対象に追加

## フェーズ 2: relay と storage node を分離して成立させる

### 着手条件

Phase 2 は、次の前提がそろった時点で着手できます。

- Phase 0 と Phase 1 の仕様境界が固定済みである
- JS ブートストラップで validate / normalize / finalize の挙動が確認済みである
- 本命実装は `Rust + SQLite` で進める方針が確定している
- relay / storage の責務分離を先に進める方針が確定している
- carrier の第一候補は未確定でも、Phase 2 の開始自体は妨げない

### Rust + SQLite の着手点

ここが本命実装の開始点です。  
Phase 1 の JavaScript 実装は挙動確認用のブートストラップとしてここまでで役割を終え、Phase 2 から `Rust + SQLite` による本実装へ移行します。

最初に着手するのは、relay / storage の分離境界です。  
具体的には、`packages/core/` の永続化を SQLite 前提で組み直しつつ、`packages/relay/` を Rust で立ち上げ、raw log と canonical catalog を分けた最小構成を作ります。

### 目的

「publish できる」だけでなく、「relay を立てられる」「storage node を立てられる」を別責務として実装します。

### やること

- relay の append-only log を Rust で実装する
- storage backend を SQLite 前提で抽象化する
- replay を Rust で実装する
- duplicate detection を carrier identity / identity key で扱う
- subscription の最小形を実装する

### 最初の着手順

Phase 2 では、次の順で切り出すと進めやすいです。

1. Rust の workspace と `packages/relay/` の最小バイナリ骨格を立てる
2. wire object の append-only 保存経路を作る
3. raw log と SQLite catalog を分けて保存する
4. replay/export の最小経路を作る
5. subscription の最小形を足す

この順で進めると、最初の 3 issue だけで relay の骨格、保存経路、storage 分離までがひと通り揃います。

### relay の責務

- wire object を受け取る
- wire-level の整合性を確認する
- append-only で保存する
- replay と export を返す

### storage node の責務

- 長期保管
- replay 可能性の維持
- 取り込み済み object の検索補助
- raw log と canonical catalog を分離して持つ

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
- author identity を公開鍵ベースで検証する
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
