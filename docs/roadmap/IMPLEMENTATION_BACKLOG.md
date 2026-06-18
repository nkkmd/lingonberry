# 実装バックログ

**Status: draft** | **Last updated: 2026-06-18**

この文書は、[実装ロードマップ](./IMPLEMENTATION_ROADMAP.md) を issue 単位に分解したものです。  
実作業では、上から順に切るよりも、依存が薄いところから並行に進めても構いません。  
ただし、最初の優先順位は **フェーズ 0 と 1** です。

## ラベル案

- `phase-0`
- `phase-1`
- `phase-2`
- `phase-3`
- `phase-4`
- `phase-5`
- `phase-6`
- `spec`
- `schema`
- `protocol`
- `relay`
- `storage`
- `identity`
- `provenance`
- `index`
- `toitoi`

## Epic 0: 仕様の固定点

Phase 0 の各 issue は、文書上の固定点として完了済みです。  
完了済みの issue は、見出しに `（完了済み）` を付けて統一します。

### Issue 0.1: knowledge object の必須フィールドを確定する（完了済み）

- 目的: publish 可能な最小構造を確定する
- 依存: なし
- 完了条件:
  - 必須フィールドが文書化されている
  - `id`, `schemaVersion`, `type`, `createdAt`, `body`, `provenance`, `rawRef` が core の最小必須として一致している
  - optional / extension フィールドが明示されている
  - 最小構造の JSON 例がある
  - fixture に切り出された最小構造の JSON 例がある
  - schema と概念文書の間で矛盾がない

### Issue 0.2: identity claim と provenance の責務分離を明文化する（完了済み）

- 目的: 同一性の根拠と来歴の責務を分ける
- 依存: なし
- 完了条件:
  - identity claim が identity 用であることが明確
  - provenance が origin / history 用であることが明確
  - identity claim が Phase 0 / 1 では任意であることが明確

### Issue 0.3: rawRef の役割を固定する（完了済み）

- 目的: 再解析と監査のための参照先を確定する
- 依存: なし
- 完了条件:
  - rawRef が provenance と別責務である
  - carrier/wire payload への参照として説明できる

### Issue 0.4: relation と lineage の使い分けを整理する（完了済み）

- 目的: semantic relation と派生履歴を混同しない
- 依存: なし
- 完了条件:
  - relation の用途が明確
  - lineage の用途が明確

### Issue 0.5: validate / normalize / finalize の境界を定義する（完了済み）

- 目的: wire から canonical への変換手順を固定する
- 依存: 0.1, 0.2, 0.3
- 完了条件:
  - 各段階の責務が説明できる
  - 決定性の要件が明確

### Issue 0.6: publish 時の author identity / signing model を固定する（完了済み）

- 目的: 作者同定を password ではなく公開鍵署名ベースで扱う
- 依存: 0.2
- 完了条件:
  - publish 主体が public key で識別される前提が明確
  - author / actor の同定が provenance に記録されることが明確
  - identity claim との役割分担が説明できる

## Epic 1: 単一オブジェクト publish（完了済み）

### Issue 1.1: 単一 object の入力形式を決める（完了済み）

- 目的: 最初の publish 入力を固定する
- 依存: 0.1, 0.5, 0.6
- 実装メモ:
  - 入口は `http-publish-request` を第一候補にする
  - request envelope と object 本体を分けて validate する
  - `publisher` は carrier 側メタデータとして扱う
- 完了条件:
  - JSON か protocol-native wire 表現のどちらを受けるか決まっている
  - 入力例がある
  - 最小構造の具体例が `protocol-native wire format` と fixture にある
  - publish request に署名情報をどう載せるか決まっている

### Issue 1.2: wire object の validate を実装する（完了済み）

- 目的: 不正な object を早い段階で reject する
- 依存: 1.1
- 実装メモ:
  - `knowledge-object.schema.json` を検証の基準にする
  - `rawRef` 欠落と type / format 不整合を最初に落とす
  - 正常系と不正系の fixture を使って validate を固定する
- 完了条件:
  - schema validation が動く
  - 必須項目欠落や型不整合を検出できる
  - `rawRef` 欠落の不正例が fixture としてある

### Issue 1.3: normalize を実装する（完了済み）

- 目的: canonicalization の前処理を決定的にする
- 依存: 1.2
- 実装メモ:
  - field 順序を安定化する
  - 省略可能 field の default 扱いを固定する
  - 文字列表現の揺れは validate 後に吸収する
- 完了条件:
  - field 順序や default の扱いが安定している
  - 同じ input から同じ normalized object が得られる

### Issue 1.4: finalize を実装する（完了済み）

- 目的: canonical object を確定する
- 依存: 1.3, 0.2, 0.3
- 実装メモ:
  - canonical id は validate 済み object の `id` を軸に保持する
  - provenance と rawRef は finalize で失わない
  - identityClaims がある場合だけ対応関係を保持する
- 完了条件:
  - canonical id を付与できる
  - provenance と rawRef を保持できる

### Issue 1.5: 単一ノード保存を実装する（完了済み）

- 目的: publish された object を永続化する
- 依存: 1.4
- 実装メモ:
  - 最小の append-only JSONL ストアを `packages/core/` に置く
  - `publish` 時に canonical object を追記する
  - 同一 `id` の再 publish は idempotent に扱い、内容が異なる場合は conflict にする
  - `get` で canonical id から再取得できるようにする
- 完了条件:
  - append-only で保存される
  - 再起動後も再取得できる

### Issue 1.6: canonical object の取得 API を実装する（完了済み）

- 目的: canonical id で再取得できるようにする
- 依存: 1.5
- 実装メモ:
  - CLI の `get` を最初の取得 API として扱う
  - まずは単一ノードのローカル再取得に限定する
  - canonical view の組み立ては `packages/api/` に寄せる
- 完了条件:
  - `id` で取得できる
  - 安定した canonical view を返せる

## Epic 2: relay と storage node の分離

### Issue 2.1: relay の append-only log を実装する

- 目的: relay を保存ノードとして成立させる
- 依存: 1.5
- 完了条件:
  - wire object の追記ログがある
  - 上書きではなく追記になる

### Issue 2.2: storage backend を抽象化する

- 目的: storage 実装を差し替え可能にする
- 依存: 2.1
- 完了条件:
  - 実装差し替えの境界がある
  - replay に必要なデータが失われない

### Issue 2.3: replay を実装する

- 目的: log から state を再構成できるようにする
- 依存: 2.1, 2.2
- 完了条件:
  - 保存済み object を再構築できる
  - canonical state の再現ができる

### Issue 2.4: duplicate detection を実装する

- 目的: carrier identity と identity key で重複を扱う
- 依存: 2.1, 3.1
- 完了条件:
  - 同一 carrier の重複を検出できる
  - carrier をまたぐ照合の入口がある

### Issue 2.5: subscription の最小形を実装する

- 目的: relay から必要な object を受け取れるようにする
- 依存: 2.1
- 完了条件:
  - object type などの最小 filter がある
  - 冪等な配信を想定できる

## Epic 3: identity と provenance

### Issue 3.1: identity key の導出規則を実装する

- 目的: carrier-neutral な照合キーを作る
- 依存: 0.5
- 完了条件:
  - versioned rule で決定的に導出できる
  - relay URL に依存しない

### Issue 3.2: identity claim の生成と検証を実装する

- 目的: identity key と canonical id の対応を検証可能にする
- 依存: 3.1, 1.4
- 完了条件:
  - claim を生成できる
  - claim を検証できる

### Issue 3.3: provenance の保存と取得を実装する

- 目的: 来歴を追跡できるようにする
- 依存: 1.4, 0.6
- 完了条件:
  - source / author / time / transform chain を保持できる
  - provenance を API から参照できる

### Issue 3.4: raw payload の再取得経路を実装する

- 目的: 再 canonicalize と監査を可能にする
- 依存: 0.3, 3.3
- 完了条件:
  - rawRef から payload を参照できる
  - 再解析の入口がある

## Epic 4: indexer と API

### Issue 4.1: type index を実装する

- 目的: type で検索できるようにする
- 依存: 1.5
- 完了条件:
  - type ごとの一覧が出せる

### Issue 4.2: relation graph を実装する

- 目的: object 間の意味関係を辿れるようにする
- 依存: 1.5, 0.4
- 完了条件:
  - relation 辿りができる

### Issue 4.3: lineage graph を実装する

- 目的: 派生と revision を辿れるようにする
- 依存: 1.5, 0.4
- 完了条件:
  - derived_from / revises / supersedes を辿れる

### Issue 4.4: provenance graph を実装する

- 目的: 来歴の検索面を作る
- 依存: 3.3
- 完了条件:
  - source ごとに辿れる

### Issue 4.5: canonical view API を設計する

- 目的: UI と外部利用の共通参照面を作る
- 依存: 4.1, 4.2, 4.3, 4.4
- 完了条件:
  - canonical view の取得・一覧・検索ができる

### Issue 4.6: index 再構築手順を定義する

- 目的: index を派生構造として保つ
- 依存: 4.1, 4.2, 4.3, 4.4
- 完了条件:
  - canonical store から再構築できる

## Epic 5: Toitoi application profile

### Issue 5.1: Toitoi profile の境界を定義する

- 目的: core と profile の境界を固定する
- 依存: 4.5
- 完了条件:
  - core に入れない語彙が明確

### Issue 5.2: inquiry / observation / evidence / synthesis / annotation の使い方を定義する

- 目的: Toitoi が必要とする型を profile 側で扱う
- 依存: 5.1
- 完了条件:
  - profile の語彙と validation がある

### Issue 5.3: Toitoi 用 API 契約を定義する

- 目的: Toitoi が core を fork せずに接続できるようにする
- 依存: 5.1, 5.2
- 完了条件:
  - 必要な API が整理されている

## Epic 6: carrier 拡張と運用性

### Issue 6.1: HTTP carrier を固める

- 目的: 実運用で扱いやすい carrier を持つ
- 依存: 1.1, 1.6
- 完了条件:
  - publish と retrieval ができる

### Issue 6.2: file / archive carrier を固める

- 目的: 長期保管と持ち運びを支える
- 依存: 2.3
- 完了条件:
  - archive から replay できる

### Issue 6.3: capability negotiation を定義する

- 目的: carrier 差分を扱えるようにする
- 依存: 6.1, 6.2
- 完了条件:
  - carrier capability を公開できる

### Issue 6.4: access policy と retention policy を整理する

- 目的: 公開運用に必要な制御を整える
- 依存: 2.1, 6.1
- 完了条件:
  - 公開 / 限定 / private の扱いがある

## 推奨順序

1. 0.1 - 0.5
2. 1.1 - 1.6
3. 2.1 - 2.5
4. 3.1 - 3.4
5. 4.1 - 4.6
6. 5.1 - 5.3
7. 6.1 - 6.4

## まず切るべき issue

最初に切るなら次の 6 つです。

- 0.1
- 0.2
- 0.3
- 0.5
- 1.2
- 1.4

このセットで、最初の publish 可能条件をかなり早く固められます。
