# 運用準備ロードマップ

**Status: active** | **Last updated: 2026-06-19**

## 目的

この文書は、Lingonberry を実運用可能な状態へ持っていくための正本ロードマップです。

このロードマップでは、次の到達点を段階的に固めます。

- `relay` を独立して立てられる
- `storage node` を独立して立てられる
- `knowledge object` を継続運用できる
- 障害時の復旧と再構成ができる
- 複数ノード構成へ拡張できる

## 前提

- core protocol と application profile は分けて考える
- `wire` と `canonical` は同じ protocol object の別表現として扱う
- `carrier` は protocol object を運ぶ正規の実装として扱う
- 運用機能は、仕様文書・operations 文書・roadmap の三層で整理する

## 現在の到達点

ここには、現時点で既に終わっているものを短く記録します。

- 仕様の固定点
- 単一オブジェクトの publish 経路
- relay / storage の分離方針
- archive / capability / access / migration 系の運用文書

## 実運用化の原則

- まず最小の単独運用を成立させる
- その後に複数ノード運用を足す
- relay は semantic truth を決めない
- storage node は長期保管と再構成を担う
- 運用上の判断は、core 仕様に押し込まず profile / policy に分ける
- readiness は `ready` コマンドまたは readiness endpoint で確認できる形に寄せる

## 目標状態

バージョン 0.1.0 として OSS 公開・配布し、だれでも `relay` と `storage node` を独立に立てて運用できる状態にする。

## フェーズ 0: 運用前提の固定（完了済み）

### 目的

運用に入る前に、責務境界と前提条件を固定します。

### ここで決めること

- `relay` と `storage node` の責務分離
- public / private の扱い
- 監視対象としないもの
- どこまでを core、どこからを profile とするか

### 決定済み前提

- [運用前提メモ](../operations/OPERATIONAL_PREMISES_MEMO.md) に Phase 0 の決定を集約する
- `relay` と `storage node` の責務境界を、core 文書と operations 文書の両方で同じ言葉にそろえる
- public / private の扱いを core から外し、profile / policy 側に寄せる
- 監視対象としないものを明記して、後続フェーズの観測範囲を絞る
- Phase 1 以降の前提として再利用できる形に固定する

### レビュー観点

- [ ] 責務境界が明確か
- [ ] public / private の扱いが core から外れているか
- [ ] 監視対象外が明示されているか
- [ ] core と profile の境界が説明できるか
- [ ] Phase 1 で再利用できるか

### 関連文書

- [運用前提メモ](../operations/OPERATIONAL_PREMISES_MEMO.md)
- [技術決定 ADR](../operations/TECH_DECISION_ADR.md)
- [Carrier Decision Memo](../operations/CARRIER_DECISION_MEMO.md)
- [概念モデル](../concepts/CONCEPT_MODEL.md)
- [Carrier](../concepts/CARRIER.md)

### 完了条件

- 運用前提を読めば、何を core に入れないか説明できる
- 以後のフェーズがこの前提に依存して書ける

## フェーズ 1: relay と storage の完全分離（完了済み）

### 目的

`relay` を単独の入口として動かし、`storage node` を別責務として切り出します。

### ここで決めること

- `relay` は ingress / validation / routing に寄せる
- `storage node` は persistence / replay / export に寄せる
- 同一リポジトリ内で別 binary として切るか
- 共有する最小 API は何か
- raw log と canonical store の境界
- relay 側が持たない責務を明示する

### やること

1. relay と storage の責務境界を 1 枚の図か箇条書きで固定する
2. relay から永続化の実装詳細を切り離す
3. storage 側に append / replay / retrieve の最小面を残す
4. ローカル開発時の接続先を明示する
5. 既存の HTTP publish 経路がどちらに属するかを固定する

### 最初の着手順

- `packages/relay/src/main.rs` の `StorageBackend` 依存を洗い出す
- `packages/core/src/lib.rs` と `packages/core/src/sqlite.rs` の保存責務を洗い出す
- `packages/storage/src/lib.rs` と `packages/storage/src/main.rs` の runtime 入口を確認する
- `packages/relay/README.md`、`packages/storage/README.md`、`packages/core/README.md` の責務記述を照合する
- relay が storage の内部構造を直接参照している箇所を洗い出す
- storage が relay の HTTP / carrier 実装を参照している箇所を洗い出す
- 最小の疎結合インターフェースを文書化する

### 完了条件

- relay だけで受け口として成立する
- storage node だけで保存責務を持てる
- 相互依存を最小化できる
- どちらかを差し替えても、他方の責務が崩れない

## フェーズ 2: `storage node` の独立バイナリ化（完了済み）

### 目的

`storage node` を個別デプロイ可能な単位にします。

### ここで決めること

- 起動コマンドの形
- 設定ファイルの場所
- 永続化先のレイアウト
- `relay` からの接続方法
- 退役時の扱い
- export / import をここで公開するか

### やること

1. storage node の binary 名を決める
2. 起動時に必要な設定を列挙する
3. データディレクトリとバックアップ先を分ける
4. 最低限の health / status 出力を用意する
5. relay とは別プロセスで動くことを確認する

### 最初の着手順

- `storage node` の起動引数を先に決める
- その引数に合わせて設定ファイルを切る
- 永続化先ディレクトリを固定する
- `relay` から `storage node` へ向ける接続面を最小化する

### 完了条件

- storage node を単体起動できる
- relay と別プロセスで運用できる
- 置き換え手順がある
- storage node の運用に relay の同梱起動を前提としない

### 完了メモ

- `storage node` の設定と保存レイアウトは [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md) に集約した
- `relay` と `storage node` の別プロセス運用は [relay / storage separation](../operations/RELAY_STORAGE_SEPARATION.md) に集約した
- `lingonberry-storage` は `capabilities` / `config` / `run` を出せる
- `relay` と `storage node` は、それぞれ独立に起動確認できる

## フェーズ 3: 起動・停止・再起動の運用整備（完了済み）

### 目的

日常運用で必要な lifecycle を整えます。

### ここで決めること

- container を primary にし、systemd を併設するか
- graceful shutdown の合図
- 再起動時の整合性確認
- readiness / liveness の扱い
- 起動失敗時の戻り方

### やること

1. relay と storage node それぞれの起動手順を分ける
2. 終了シグナル受信時の安全停止を決める
3. 再起動後に確認する最小チェックを定義する
4. 起動失敗時のログと exit code を揃える
5. 運用手順を 1 つの実行例として書く

### 関連文書

- [運用前提メモ](../operations/OPERATIONAL_PREMISES_MEMO.md)
- [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md)
- [relay / storage separation](../operations/RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)

### 最初の着手順

- まず手動起動の手順を固定する
- 次に container を primary とする運用手順を固め、systemd の併設方針を明示する
- 最後に readiness / liveness の判定条件を入れる

### 完了条件

- 起動と停止の手順が再現できる
- 再起動後に壊れない
- 失敗時に何を確認すればよいかが分かる

### 完了メモ

- 手動起動、container 実行、systemd unit の入口を分離した
- `ready` / `capabilities` / `config` / `replay` / `list` の確認手順を runbook に揃えた
- 失敗時の exit code と切り分け手順を runbook に反映した
- container / systemd の具体例はテンプレート文書に分けた
- `Issue 3.4` の container-first 方針を primary として固定した

## フェーズ 4: 設定・環境変数・シークレット管理（完了済み）

### 目的

運用時の設定を、コードと切り離して扱えるようにします。

### ここで決めること

- 設定ファイル形式
- 環境変数の責務
- secret の保管方法
- profile ごとの差分

### 関連文書

- [運用前提メモ](../operations/OPERATIONAL_PREMISES_MEMO.md)
- [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md)
- [Secret Management](../operations/SECRET_MANAGEMENT.md)
- [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md)
- [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md)

### 最初の着手順

- まず `storage node runtime` にある設定解決順と設定ファイル形式を、運用上の正本として再確認する
- 次に `relay` と `storage node` で環境変数の責務を分け、override と secret を混ぜない形にする
- そのうえで secret の保管・注入方法を決め、profile ごとの差分を policy / profile 側へ逃がす
- 最後に runbook と CLI の説明へ反映し、どこに何を置くかを一貫させる

### 完了条件

- 設定の置き場所が一貫している
- secret を平文前提にしない

## フェーズ 5: 監視・ログ・メトリクス（完了済み）

### 目的

障害検知と原因追跡を可能にします。

### ここで決めること

- 構造化ログの形式
- メトリクスの種類
- alert の閾値
- 最低限の観測項目

### 関連文書

- [Observability](../operations/OBSERVABILITY.md)
- [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)
- [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md)
- [運用前提メモ](../operations/OPERATIONAL_PREMISES_MEMO.md)

### 最初の着手順

1. `relay` と `storage node` の共通ログ field を固定する
2. 最低限のメトリクスを counter / gauge / histogram に分ける
3. alert の起点になる failure pattern を 3 つ程度に絞る
4. runbook から見に行く順番を 1 本にする
5. 既存の `ready` / `run` / carrier contract と矛盾がないか確認する

### 完了条件

- 異常時にどこを見るかが分かる
- 運用中の劣化を把握できる

### 完了メモ

- 構造化ログ、メトリクス、alert、調査順は [Observability](../operations/OBSERVABILITY.md) に集約した
- 障害時の最小確認順は [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) に反映した
- `relay` と `storage node` の観測境界は、Phase 0 の責務分離と整合する形で固定した

## フェーズ 6: バックアップ・リストア・退役手順（完了済み）

### 目的

障害復旧とノード退役を手順化します。

### ここで決めること

- backup の単位
- restore の手順
- 退役時に残すもの
- 再投入時の整合性

### やること

1. backup の単位を `storage node` の保存レイアウトに合わせて固定する
2. restore の入力と確認順を定義する
3. 退役時に残すものと消すものを policy に寄せて固定する
4. 再投入時の整合性確認を runbook に接続する
5. archive を使う場合の manifest / wire-log 前提を明文化する

### 関連文書

- [File / Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md)
- [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)
- [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md)
- [Migration and Schema Versioning](../operations/MIGRATION_AND_SCHEMA_VERSIONING.md)
- [relay / storage separation](../operations/RELAY_STORAGE_SEPARATION.md)

### 最初の着手順

1. backup の単位を `storage node runtime` の保存レイアウトに合わせて固定する
2. restore の入口を archive / export の正本と衝突しない形で書く
3. 退役時に残すものと消すものを access / retention policy に寄せる
4. 再投入時の確認手順を Node Lifecycle Runbook に繋ぐ
5. 必要なら archive carrier の manifest / wire-log 前提を明文化する

### 完了条件

- restore が定義されている
- 退役が安全にできる
- backup の単位と再投入時の検証条件が説明できる
- backup / restore / retirement の入口が関連文書から辿れる

### 完了メモ

- backup / restore / retirement の手順は [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) に集約した
- `storage node runtime` で `dataDir`、`backupDir`、`tempDir` の役割を固定した
- `Access and Retention Policy` で退役時の保持対象と削除対象を固定した
- `File / Archive Carrier Contract` で archive と backup の関係を固定した
- Phase 6 の再投入確認と runbook 反映を backlog の完了条件に接続した

## フェーズ 7: HTTP carrier の公開運用

### 目的

HTTP carrier を公開運用できる形に整えます。

### ここで決めること

- 公開 endpoint
- 認証 / 認可
- rate limit
- 公開時の contract

### 完了条件

- 公開運用に必要な前提が文書化されている

## フェーズ 8: archive export / import の運用化

### 目的

移送・退避・再投入を運用手順にします。

### ここで決めること

- export の粒度
- import の検証手順
- archive version の扱い
- 差分移送の要否

### 完了条件

- export / import を運用手順として説明できる

## フェーズ 9: migration / schema versioning の運用化

### 目的

schema 変更を運用しながら進められるようにします。

### ここで決めること

- version bump の規則
- backward compatibility の範囲
- migration の責務
- rollback の可否

### 完了条件

- schema 変更時の手順がある

## フェーズ 10: access / retention policy の運用化

### 目的

公開範囲と保持期間を運用レベルで制御します。

### ここで決めること

- access scope
- retention hint
- policy の適用点
- 監査時の確認事項

### 完了条件

- 保持と公開のルールが一貫している

## フェーズ 11: 複数ノード運用

### 目的

単一ノード前提から、複数ノード前提へ移行します。

### ここで決めること

- ノード間同期
- 競合解決
- discoverability
- capacity 分散

### 完了条件

- 複数ノードで同じ object 群を扱える

## フェーズ 12: 追加 carrier への拡張準備

### 目的

HTTP 以外の carrier を足せるようにします。

### ここで決めること

- carrier capability negotiation
- carrier ごとの制約
- 共通化する validation
- profile 側で差し替える点

### 完了条件

- 新 carrier の追加手順が説明できる

## 完了条件

このロードマップ全体の完了条件を、ここにまとめます。

- 実運用に必要な経路が揃っている
- 障害復旧が手順化されている
- profile 追加や carrier 追加の道筋がある

## 参照文書

- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md)
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
- [DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [HTTP carrier contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [FILE archive carrier contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [carrier capability negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md)
- [access / retention policy](../operations/ACCESS_RETENTION_POLICY.md)
- [migration / schema versioning](../operations/MIGRATION_AND_SCHEMA_VERSIONING.md)
