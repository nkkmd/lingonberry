# 運用準備バックログ

**Status: active** | **Last updated: 2026-06-19**

この文書は、[運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md) のうち、フェーズ 1 から 3 を issue 単位に分解したものです。  
実作業では、依存の薄い issue から並行に進めても構いません。  
ただし、最初の優先順位は **フェーズ 1 と 2** です。

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
12. Issue 3.4: 起動方式を決める
13. Issue 3.3: 再起動後の整合性確認を定義する
14. Issue 3.5: readiness / liveness と失敗時の戻り方を揃える

## ラベル案

- `phase-1`
- `phase-2`
- `phase-3`
- `relay`
- `storage`
- `ops`
- `config`
- `lifecycle`
- `runbook`
- `deployment`

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

### Issue 1.2: relay の永続化依存を切り離す

- 目的: relay が storage の内部実装に直接依存しないようにする
- 依存: 1.1
- 完了条件:
  - relay から storage の内部構造への直接参照がなくなる
  - relay が受け口として成立する
  - relay 側の最小 API が文書化されている

### Issue 1.4: ローカル開発時の接続面を決める

- 目的: 開発時に relay と storage node をどうつなぐかを固定する
- 依存: 1.2, 1.3
- 完了条件:
  - ローカルでの接続先と役割が説明できる
  - relay / storage を別プロセスで起動したときの接続方法がある
  - 実装メモまたは運用メモに残っている

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

### Issue 2.3: storage node の永続化レイアウトを固定する

- 目的: 保存先の構造を実運用に耐える形にする
- 依存: 2.1, 2.2
- 完了条件:
  - data / backup / temp のような領域分離が説明できる
  - raw log と canonical store の扱いが分かる
  - 退役時に残すものが説明できる

### Issue 2.4: storage node の health / status 出力を用意する（完了済み）

- 目的: 起動確認と最低限の状態確認を可能にする
- 依存: 2.1, 2.3
- 完了条件:
  - health の判定基準がある
  - status の出力項目がある
  - 障害時に見ればよい最小情報がある

### Issue 2.5: relay と storage を別プロセスで運用できることを確認する

- 目的: 同梱起動前提をなくす
- 依存: 2.2, 2.4
- 完了条件:
  - relay と storage node を別プロセスで起動できる
  - 片方を差し替えてももう片方の運用が壊れない
  - 同梱起動を前提としない運用メモがある

## Epic 3: 起動・停止・再起動の運用整備

### Issue 3.1: 手動起動の runbook を作る

- 目的: 最初に再現可能な運用手順を固定する
- 依存: 2.1, 2.2
- 完了条件:
  - relay の起動手順がある
  - storage node の起動手順がある
  - 確認コマンドがある

### Issue 3.2: graceful shutdown を定義する

- 目的: 終了時の安全停止を決める
- 依存: 2.1, 2.3
- 完了条件:
  - 終了シグナル受信時の動作が決まっている
  - 保存途中データの扱いが決まっている
  - 強制終了時のリスクが説明できる

### Issue 3.4: 起動方式を決める（完了済み）

- 目的: どのデプロイ方式を primary にするかを固定する
- 依存: 3.1, 3.2
- 完了条件:
  - systemd / container / 手動起動の優先順位が決まっている
  - 運用環境ごとの差分が説明できる
  - 既定の起動方式が 1 つ決まっている

### Issue 3.3: 再起動後の整合性確認を定義する

- 目的: 再起動後に壊れていないことを機械的に確認する
- 依存: 3.2
- 完了条件:
  - 再起動後の最小チェックがある
  - 失敗時の切り分け手順がある
  - replay と保存状態の確認観点がある

### Issue 3.5: readiness / liveness と失敗時の戻り方を揃える

- 目的: 起動失敗や異常時の扱いを統一する
- 依存: 3.3, 3.4
- 完了条件:
  - readiness / liveness の判定条件がある
  - 起動失敗時の exit code またはエラー分類がある
  - ログの見方が手順に含まれている

## 参照文書

- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md)
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md)
- [DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [HTTP carrier contract](../operations/HTTP_CARRIER_CONTRACT.md)
- [File / Archive carrier contract](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](../operations/CARRIER_CAPABILITY_NEGOTIATION.md)
