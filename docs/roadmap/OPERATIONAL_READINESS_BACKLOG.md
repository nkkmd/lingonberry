# 運用準備バックログ

**Status: active** | **Last updated: 2026-06-19**

この文書は、[運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md) のうち、フェーズ 1 から 7 を issue 単位に分解したものです。  
フェーズ 0 は [運用前提メモ](../operations/OPERATIONAL_PREMISES_MEMO.md) に集約し、この backlog では issue 化しません。  
実作業では、依存の薄い issue から並行に進めても構いません。  
Phase 2 と Phase 3 は完了済みです。  
Phase 4 も完了済みで、設定・環境変数・シークレット管理の完了記録として残します。
Phase 5 も完了済みで、観測の正本は [Observability](../operations/OBSERVABILITY.md) にあります。
Phase 6 も完了済みで、backup / restore / retirement の正本は [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) にあります。  
Phase 7 は完了済みで、HTTP carrier の公開運用に関する前提を固めました。


## Phase 0 完了確認

Phase 0 は完了済みですが、Phase 1 へ進む前に次の確認を通しておきます。

- [ ] 責務境界が一致しているか
- [ ] storage node の責務が固定されているか
- [ ] public / private の扱いが core から外れているか
- [ ] 監視対象外が明文化されているか
- [ ] core に入れないものが説明できるか
- [ ] Phase 1 に流用できるか

## Phase 1 完了メモ

Phase 1 は、既存の `relay` 実装に残っていた保存責務を棚卸しし、`storage node` 側へ切り出す前提を固めました。

1. `Issue 1.1` と `Issue 1.3` の境界定義を再確認する
1. `Issue 1.2` のために `relay` から永続化依存を切り離す対象を列挙する
1. `Issue 1.4` のためにローカル開発時の接続面を決める
1. `Issue 2.1` 以降へ進むため、`storage node` の binary 入口と設定面の前提を確認する

この段階では、`packages/relay/src/main.rs`、`packages/core/src/lib.rs`、`packages/core/src/sqlite.rs`、`packages/storage/src/main.rs` の責務差分を最初の棚卸し対象にします。

## 推奨実装順

1. Issue 1.1: relay / storage の責務境界を固定する
2. Issue 1.3: storage node の最小 API を定義する
3. Issue 1.2: relay の永続化依存を切り離す
4. Issue 1.4: ローカル開発時の接続面を決める
5. Issue 2.1: storage node の binary と entrypoint を決める
6. Issue 2.2: storage node の設定形式を決める
7. Issue 2.3: storage node の永続化レイアウトを固定する
8. Issue 2.4: storage node の health / status 出力を用意する
9. Issue 2.5: relay と storage を別プロセスで運用できることを確認する
10. Issue 3.1: 手動起動の runbook を作る
11. Issue 3.2: graceful shutdown を定義する
12. Issue 3.3: 再起動後の整合性確認を定義する
13. Issue 3.5: readiness / liveness と失敗時の戻り方を揃える

Issue 3.4 は完了済みで、container を primary、systemd を併設とする方針が確定しています。
Issue 6 系は backup / restore / retirement の運用化が完了済みです。

## ラベル案

- `phase-1`
- `phase-2`
- `phase-3`
- `phase-4`
- `relay`
- `storage`
- `ops`
- `config`
- `secret`
- `profile`
- `lifecycle`
- `observability`
- `runbook`
- `deployment`

## Phase 7 完了メモ

Phase 7 は、HTTP carrier を公開運用するための前提を issue 単位に分解し、完了しました。  
HTTP carrier contract、capability negotiation、access / retention policy、observability を正本として、公開 endpoint、authn/authz、rate limit、公開時の contract を揃えました。

## Epic 4: 設定・環境変数・シークレット管理（完了済み）

### Issue 4.1: 設定ファイル形式と precedence を固定する（完了済み）

- 目的: 設定の置き場所と解決順を一貫させる
- 依存: 2.2, 3.4
- 完了条件:
  - 設定ファイルの形式が JSON object として一貫している
  - `LINGONBERRY_STORAGE_CONFIG`、`$LINGONBERRY_STATE_DIR/storage-config.json`、`LINGONBERRY_STATE_DIR`、既定値の優先順が説明できる
  - `stateDir`、`dataDir`、`backupDir`、`tempDir` の役割が説明できる
  - [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md) が正本になっている

### Issue 4.2: 環境変数の責務を固定する（完了済み）

- 目的: 環境変数を設定の補助入力として位置づけ、責務を混ぜないようにする
- 依存: 4.1
- 完了条件:
  - 環境変数で上書きしてよい項目が説明できる
  - secret と非 secret の役割が分かれている
  - `relay` と `storage node` で共通化する環境変数と、個別に持つ環境変数が分かる
  - 運用メモまたは runbook に責務境界が残っている

### Issue 4.3: secret の保管・注入方法を決める（完了済み）

- 目的: secret を平文前提で扱わず、運用上の注入経路を決める
- 依存: 4.1, 4.2
- 完了条件:
  - secret を設定ファイルに平文で置かない方針が明確
  - secret の保管先または注入経路が運用メモに残っている
  - [Secret Management](../operations/SECRET_MANAGEMENT.md) に正本がある
  - `private / encrypted object` の扱いを core へ持ち込まない方針と整合している
  - access / retention policy と矛盾しない

### Issue 4.4: profile ごとの差分の置き場を決める（完了済み）

- 目的: 分野固有・運用固有の差分を core 設定から切り離す
- 依存: 4.2, 4.3
- 完了条件:
  - profile ごとの差分が core ではなく profile / policy 側にあることが説明できる
  - Toitoi のような application profile の差分を載せる場所が説明できる
  - public / curated / private の扱いと整合している
  - profile 側で差し替える設定が整理されている

## Epic 1: relay と storage の完全分離

### Issue 1.1: relay / storage の責務境界を固定する（完了済み）

- 目的: どの責務を relay に残し、どの責務を storage node に移すかを明確にする
- 依存: なし
- 完了条件:
  - `relay` の責務が ingress / validation / routing に整理されている
  - `storage node` の責務が persistence / replay / export に整理されている
  - 既存の HTTP publish 経路の所属が説明できる
  - どちらに持たせない責務が明示されている

### Issue 1.3: storage node の最小 API を定義する（完了済み）

- 目的: storage node に必要な最小の操作面を固定する
- 依存: 1.1
- 完了条件:
  - append の入口がある
  - replay の入口がある
  - retrieve の入口がある
  - export の扱いが仮でもよいので明示されている

### Issue 1.2: relay の永続化依存を切り離す（完了済み）

- 目的: relay が storage の内部実装に直接依存しないようにする
- 依存: 1.1
- 完了条件:
  - relay から storage の内部構造への直接参照がなくなる
  - relay から `packages/storage` への直接依存がなくなる
  - relay が受け口として成立する
  - relay 側の最小 API が文書化されている

### Issue 1.4: ローカル開発時の接続面を決める（完了済み）

- 目的: 開発時に relay と storage node をどうつなぐかを固定する
- 依存: 1.2, 1.3
- 完了条件:
  - ローカルでの接続先と役割が説明できる
  - relay / storage を別プロセスで起動したときの接続方法がある
  - 実装メモまたは運用メモに残っている
  - `LINGONBERRY_STATE_DIR` を使って同じ state dir を共有する方法が説明できる

## Epic 2: `storage node` の独立バイナリ化

### Issue 2.1: storage node の binary と entrypoint を決める（完了済み）

- 目的: `storage node` を個別デプロイ可能な単位として切り出す
- 依存: 1.1
- 完了条件:
  - binary 名が決まっている
  - 起動コマンドの入口が決まっている
  - 既存の relay とは別実行であることが明確

### Issue 2.2: storage node の設定形式を決める

- 目的: 起動時に必要な設定をコードから切り離す
- 依存: 2.1
- 完了条件:
  - 設定ファイルの形式が決まっている
  - 環境変数に寄せる項目が決まっている
  - データディレクトリとバックアップ先の役割が分かれている
  - [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md) に正本がある

### Issue 2.3: storage node の永続化レイアウトを固定する

- 目的: 保存先の構造を実運用に耐える形にする
- 依存: 2.1, 2.2
- 完了条件:
  - data / backup / temp のような領域分離が説明できる
  - raw log と canonical catalog の実パスが説明できる
  - raw log と canonical store の扱いが分かる
  - 退役時に残すものが説明できる
  - [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md) に保存レイアウトの正本がある

### Issue 2.4: storage node の health / status 出力を用意する（完了済み）

- 目的: 起動確認と最低限の状態確認を可能にする
- 依存: 2.1, 2.3
- 完了条件:
  - health の判定基準がある
  - status の出力項目がある
  - 障害時に見ればよい最小情報がある

### Issue 2.5: relay と storage を別プロセスで運用できることを確認する（完了済み）

- 目的: 同梱起動前提をなくす
- 依存: 2.2, 2.4
- 完了条件:
  - relay と storage node を別プロセスで起動できる
  - 片方を差し替えてももう片方の運用が壊れない
  - 同梱起動を前提としない運用メモがある
  - [relay / storage separation](../operations/RELAY_STORAGE_SEPARATION.md) に正本がある

## Epic 3: 起動・停止・再起動の運用整備（完了済み）

### Issue 3.1: 手動起動の runbook を作る（完了済み）

- 目的: 最初に再現可能な運用手順を固定する
- 依存: 2.1, 2.2
- 完了条件:
  - relay の起動手順がある
  - storage node の起動手順がある
  - 確認コマンドがある
  - 正本 runbook が [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) にある

### Issue 3.2: graceful shutdown を定義する（完了済み）

- 目的: 終了時の安全停止を決める
- 依存: 2.1, 2.3
- 完了条件:
  - 終了シグナル受信時の動作が決まっている
  - 保存途中データの扱いが決まっている
  - 強制終了時のリスクが説明できる
  - graceful shutdown の正本が runbook に反映されている

### Issue 3.4: 起動方式を決める（完了済み）

- 目的: container を primary にし、systemd を併設する方針を固定する
- 依存: 3.1, 3.2
- 完了条件:
  - container が primary である
  - systemd は併設手段として定義されている
  - 手動起動は確認・検証用として位置づけられている
  - 運用環境ごとの差分が説明できる
  - 既定の起動方式が container である
  - 既定方針が runbook と一致している

### Issue 3.3: 再起動後の整合性確認を定義する（完了済み）

- 目的: 再起動後に壊れていないことを機械的に確認する
- 依存: 3.2
- 完了条件:
  - `storage node` の `ready` または `run` を確認できる
  - `storage node` の `config`、`replay`、`list` を使った確認順がある
  - `relay` の `ready` と `capabilities` を確認できる
  - HTTP carrier の `GET /v1/ready` と `GET /v1/capabilities` を確認できる
  - 失敗時の切り分け手順が runbook にある
  - 再起動後チェックが runbook で参照できる

### Issue 3.5: readiness / liveness と失敗時の戻り方を揃える（完了済み）

- 目的: 起動失敗や異常時の扱いを統一する
- 依存: 3.3, 3.4
- 完了条件:
  - readiness / liveness の判定条件がある
  - 起動失敗時の exit code またはエラー分類がある
  - ログの見方が手順に含まれている
  - readiness / liveness の正本が runbook に反映されている
  - `ready` コマンドまたは readiness endpoint が利用できる
  - `storage node` と `relay` の失敗時メッセージが識別できる

## Epic 5: 監視・ログ・メトリクス

### Issue 5.1: 構造化ログの形式を固定する（完了済み）

- 目的: 失敗時に機械的に追跡できるログ形をそろえる
- 依存: Phase 0, 3.5
- 完了条件:
  - `relay` と `storage node` で共通の field がある
  - `event` と `errorType` の使い分けが説明できる
  - 1 event 1 line の方針がある
  - [Observability](../operations/OBSERVABILITY.md) に正本がある

### Issue 5.2: メトリクスの種類と最小セットを決める（完了済み）

- 目的: 劣化傾向を数値で追えるようにする
- 依存: 5.1
- 完了条件:
  - counter / gauge / histogram の使い分けがある
  - 起動、publish、append、replay、retrieve の最小メトリクスがある
  - 高カーディナリティを避ける方針がある
  - [Observability](../operations/OBSERVABILITY.md) に正本がある

### Issue 5.3: alert の閾値と調査順を決める（完了済み）

- 目的: 何が起きたときに運用側が動くかを固定する
- 依存: 5.1, 5.2
- 完了条件:
  - 起動失敗、readiness 失敗、replay 失敗、publish 失敗の扱いがある
  - alert の閾値の考え方がある
  - まず見る場所の順番がある
  - runbook か運用メモに調査順が反映されている

### Issue 5.4: 最低限の観測項目を runbook に反映する（完了済み）

- 目的: 障害時にどこを見るかを 1 本化する
- 依存: 5.1, 5.2, 5.3
- 完了条件:
  - `ready` / `run` / config / replay / retrieve の確認順がある
  - `relay` と `storage node` の切り分け順がある
  - 観測対象外が説明できる
  - Node Lifecycle Runbook に反映されている

## Epic 6: バックアップ・リストア・退役手順（完了済み）

### Issue 6.1: backup の単位を固定する（完了済み）

- 目的: 何を 1 つの backup と見なすかを明確にする
- 依存: 2.3, 3.3
- 完了条件:
  - backup に含める最小要素が説明できる
  - `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` の役割が分かる
  - `resolved-config.json` を残すかどうかの方針がある
  - archive / export と backup の関係が説明できる
  - [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md) に正本がある

### Issue 6.2: restore の手順を定義する（完了済み）

- 目的: 障害復旧時に backup から再構成できるようにする
- 依存: 6.1, 3.3
- 完了条件:
  - restore の入力と出力が説明できる
  - restore 前後で `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` を確認する順がある
  - restore 前後で `stateDir` と保存先を確認する項目がある
  - replay と canonical 再構成の順番がある
  - [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) に正本がある

### Issue 6.3: 退役時に残すものを定義する（完了済み）

- 目的: storage node や relay の退役時に、何を保持して何を廃棄するかを決める
- 依存: 6.1, 4.3, 2.5
- 完了条件:
  - 退役時に残すファイルとして `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json` が説明できる
  - `tempDir` 配下で削除してよいものが説明できる
  - 物理削除と retention に基づく削除が区別されている
  - export 可能性を壊さない方針がある
  - [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md) に正本がある

### Issue 6.4: 再投入時の整合性確認を定義する（完了済み）

- 目的: 退役後に同じ backup / archive を再投入したときの確認を固定する
- 依存: 6.2, 6.3, 5.4
- 完了条件:
  - version / manifest / replay の確認手順がある
  - 再投入後に見るべき最小チェックがある
  - 失敗時に `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` を順に切り分ける手順がある
  - [File / Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md) と整合している

### Issue 6.5: Phase 6 の runbook 反映を行う（完了済み）

- 目的: backup / restore / retirement の正本を運用文書に反映する
- 依存: 6.1, 6.2, 6.3, 6.4
- 完了条件:
  - Node Lifecycle Runbook に backup / restore / retirement の順がある
  - Node Lifecycle Runbook に通常運用、backup からの restore、退役の 3 つの実行例がある
  - Node Lifecycle Runbook に backup / restore / retirement の前後確認表がある
  - retention policy と矛盾しない
  - archive carrier と backup の使い分けが説明できる

### 完了メモ

- Phase 6 の正本は [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) に集約した
- backup / restore / retirement の file 単位は `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json` に整理した
- 退役時の保持対象と削除対象は [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md) で固定した
- archive と backup の関係は [File / Archive Carrier Contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md) に反映した

## Epic 7: HTTP carrier の公開運用

### Issue 7.1: 公開 endpoint と readiness / capabilities の公開範囲を固定する

- 目的: 公開運用でどの endpoint を外部に見せるかを明確にする
- 依存: 5.1, 5.4, 6.2
- 関連文書:
  - [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
  - [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)
  - [Observability](../operations/OBSERVABILITY.md)
- 完了条件:
  - `POST /v1/objects` は publish、`GET /v1/objects/{id}` は retrieve、`GET /v1/capabilities` は capability discovery、`GET /v1/ready` は readiness として説明できる
  - public relay の公開面と private な運用面が分かれている
  - readiness と capability discovery の責務が混ざっていない
  - HTTP carrier の公開面を説明する runbook の導線がある
  - [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md) に正本がある

### Issue 7.2: authn/authz を policy と capability に閉じる

- 目的: 認証 / 認可を protocol semantic から切り離して運用層に寄せる
- 依存: 7.1, 4.3
- 関連文書:
  - [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md)
  - [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md)
  - [Secret Management](../operations/SECRET_MANAGEMENT.md)
  - [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
- 完了条件:
  - authn/authz が protocol core の必須要素ではないと説明できる
  - HTTP carrier での supported auth modes が capability に出せる
  - HTTP carrier の公開面で authn/authz を必須にするかどうかが policy として説明できる
  - secret を設定ファイルや protocol object に入れない方針が説明できる
  - secret の保管と注入は [Secret Management](../operations/SECRET_MANAGEMENT.md) と整合している
  - [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md) に正本がある

### Issue 7.3: rate limit と abuse control の運用方針を決める

- 目的: 公開運用時の過負荷や abuse を運用ルールとして扱えるようにする
- 依存: 7.1, 5.3
- 関連文書:
  - [Observability](../operations/OBSERVABILITY.md)
  - [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)
  - [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md)
  - [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md)
- 完了条件:
  - rate limit を protocol semantic に入れない方針が説明できる
  - どの層で制御するかが説明できる
  - HTTP carrier の公開面で拒否される場合に `429` などの運用上の応答が説明できる
  - rate limit のヒット数や拒否数を観測できる
  - 運用上の閾値と観測ポイントがある
  - [Observability](../operations/OBSERVABILITY.md) か運用メモに正本がある

### Issue 7.4: 公開時の response / error / observability contract を固定する

- 目的: 公開運用時に返す情報と追跡方法を揃える
- 依存: 7.1, 7.2, 7.3
- 完了条件:
  - publish 成功時の応答形式が説明できる
  - validation error と not found / unavailable の扱いが説明できる
  - `requestId` が response body と observability で共通して使える
  - HTTP status と body の `status` の使い分けが説明できる
  - `publish_received`、`readiness_checked`、`validation_failed`、`runtime_error` が追える
  - [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md) と [Observability](../operations/OBSERVABILITY.md) に正本がある

### 完了メモ

- Phase 7 の正本は [HTTP Carrier Contract](../operations/HTTP_CARRIER_CONTRACT.md) に集約する
- 認証 / 認可は [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md) と [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md) で扱う
- rate limit と abuse control は protocol core ではなく運用層の制御として扱う
- response model と observability は [Observability](../operations/OBSERVABILITY.md) と [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md) に反映する

## 参照文書

- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md)
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md)
- [DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [HTTP carrier contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [File / Archive carrier contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md)
- [storage node runtime](../operations/STORAGE_NODE_RUNTIME.md)
- [運用前提メモ](../operations/OPERATIONAL_PREMISES_MEMO.md)
- [Secret Management](../operations/SECRET_MANAGEMENT.md)
- [Access and Retention Policy](../operations/ACCESS_RETENTION_POLICY.md)
- [Observability](../operations/OBSERVABILITY.md)
- [Node Lifecycle Runbook](../operations/NODE_LIFECYCLE_RUNBOOK.md)
- [Toitoi Application Profile](../profiles/TOITOI_APPLICATION_PROFILE.md)
