# Node Lifecycle Runbook

**Status: active** | **Last updated: 2026-06-19**

## 目的

この文書は、`relay` と `storage node` の起動・停止・再起動を、container-first の運用前提で扱うための runbook です。  
Phase 3 では、まず手動で再現できる手順を固め、その上で container を primary、systemd を併設とする方針を前提化します。

## 1. 運用の優先順位

- primary は container
- systemd は併設手段
- 手動起動は確認・検証用の基準手順

この文書では、container 化の最終形よりも、同じ起動コマンドと同じ環境変数を使えることを優先します。

## 2. 起動順

- 同じ `stateDir` を共有する構成では、まず `storage node` の設定を確認し、その後に `relay` を起動する
- `stateDir` を分ける構成では、各 binary を独立に確認できる
- `relay` と `storage node` を同梱起動前提にしない

## 3. 手動起動

### 3.1 `storage node`

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage
cargo run -p lingonberry-storage -- capabilities
cargo run -p lingonberry-storage -- config
cargo run -p lingonberry-storage -- run
```

`config` は、`LINGONBERRY_STORAGE_CONFIG` と `LINGONBERRY_STATE_DIR` を含めた解決済み設定を確認するために使います。  
`run` は最低限の status 出力を確認するために使います。  
`stateDir` を config file で上書きする運用では、`config` の `stateDir`、`dataDir`、`backupDir`、`tempDir` が意図どおりかを先に確認します。

### 3.2 `relay`

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

HTTP carrier を確認する場合は、別端末から次を実行します。

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
```

## 4. container / systemd

### 4.1 container

- container は、手動起動と同じ binary と同じ引数を使う
- container は、同じ環境変数を受け取る
- container は、起動後に `ready` または HTTP の readiness endpoint で確認できる
- container は、起動後に `capabilities` または HTTP の capability endpoint で機能面を確認できる
- container は、`storage node` と `relay` を別コンテナとして扱う

container の具体例は [Container Execution Templates](./CONTAINER_EXECUTION_TEMPLATES.md) にまとめます。

### 4.2 systemd

- systemd は container の代替ではなく併設手段として扱う
- `ExecStart` は手動起動と同じコマンドに合わせる
- `KillSignal` は `SIGTERM` を基本とする
- `Restart` は失敗時の再起動を前提にする
- systemd unit は `storage` と `relay` で分ける

unit の具体例は [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md) にまとめます。

## 5. graceful shutdown

- 終了時は新規受付を止める
- 進行中の処理は可能な限り完了させる
- 保存途中のデータは、次回起動時に replay 可能であることを優先する
- 強制終了は、最後の 1 件の処理が不完全になるリスクを持つ

現状の実装では、専用の signal hook を明示していません。  
そのため、ここでの graceful shutdown は、container / systemd / 手動運用に対する期待動作として定義しています。

## 6. 再起動後の確認

再起動後は、次の順で確認します。

1. `storage node` の `ready` または `run` が `status: ok` を返す
2. `storage node` の `config` で `configPath`、`stateDir`、`dataDir`、`backupDir`、`tempDir` が意図どおりか確認する
3. `storage node` の `replay` または `list` で保存件数が想定どおりであることを確認する
4. `relay` の `ready` が返る
5. `relay` の `capabilities` が返る
6. HTTP carrier を使う場合は `GET /v1/ready` と `GET /v1/capabilities` が返る
7. 必要なら対象 object を `retrieve` または `GET /v1/objects/<id>` で確認する

切り分けの順番は次の通りです。

- `storage node` 側の `ready` が失敗するなら、設定解決か保存先の初期化を疑う
- `storage node` 側の `replay` / `list` が崩れるなら、保存状態か raw log を疑う
- `relay` 側の `ready` が失敗するなら、bind 失敗か環境変数を疑う
- `GET /v1/capabilities` が失敗するなら、HTTP carrier の起動状態を疑う

## 7. readiness / liveness

### 7.1 storage node

- readiness: `ready` または `run` が `status: ok` を返し、解決済みの保存先を表示できる
- liveness: プロセスが継続して動作し、`replay` / `list` を受け付けられる

### 7.2 relay

- readiness: `ready` が返り、HTTP listener が bind できる
- liveness: `serve-http` が継続動作し、HTTP リクエストを受け付けられる

### 7.3 失敗時の扱い

- 設定解決や bind の失敗は起動失敗として扱う
- 起動失敗は non-zero exit code として運用側に返す
- `64` は usage / 引数不備、`65` は validation 失敗、`66` は not found、`70` は runtime/storage エラー、`78` は config / bind 失敗の目安として扱う
- HTTP request の検証失敗は 4xx として返す
- storage 由来の実行時エラーは 5xx または CLI error として返す

## 8. Observability の見方

障害時は、次の順で最小確認を行います。

1. `ready` / `run` の結果を見る
2. `service` と `event` を見て、`relay` か `storage node` かを分ける
3. `requestId`、`errorType`、`errorCode` を拾う
4. `startup` / `validation_failed` / `runtime_error` の event を見る
5. `publish` / `append` / `replay` / `retrieve` の件数と失敗率を見る
6. `stateDir` と保存先を見直す
7. 必要なら [Observability](./OBSERVABILITY.md) の alert / metric / log の対応を参照する

障害時は、`relay` と `storage node` のどちらで失敗しているかを先に分けます。  
`relay` の failure は受け口と routing を、`storage node` の failure は保存先と replay を優先して確認します。
`publish` が `429` で返る場合は、rate limit の閾値、対象 carrier、直近のアクセス集中を優先して確認します。

alert を受けたときは、`service`、`event`、`requestId` の 3 つを先に拾うと切り分けが早くなります。  
`errorType` と `errorCode` は、その後の再現や問い合わせ時の手掛かりとして残します。

### 8.1 `relay` の最小確認

- `startup` と `readiness_checked` が `ok` か確認する
- `publish_received` と `validation_failed` のログを確認する
- `rate_limited` が増えていないか確認する
- `lingonberry_startup_total`、`lingonberry_publish_total`、`lingonberry_publish_failure_total` を確認する
- `lingonberry_rate_limited_total` を確認する
- `lingonberry_runtime_error_total` と `lingonberry_operation_duration_ms` を確認する

### 8.2 `storage node` の最小確認

- `startup` と `readiness_checked` が `ok` か確認する
- `config_resolved` で `stateDir` と保存先が意図どおりか確認する
- `append_completed`、`replay_completed`、`retrieve_completed` のログを確認する
- `lingonberry_startup_total`、`lingonberry_config_resolved_total`、`lingonberry_append_total`、`lingonberry_replay_total`、`lingonberry_retrieve_total` を確認する
- `lingonberry_runtime_error_total` と `lingonberry_operation_duration_ms` を確認する

### 8.3 観測対象外

- 内容の真偽は確認対象にしない
- profile 固有の trust rule はここでは追わない
- UI や表示順序の問題はここでは追わない
- carrier 変換の細部は carrier contract 側で扱う

## 9. backup / restore / retirement

Quick reference:

- backup: `dataDir` から `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、必要なら `resolved-config.json` を固める
- restore: `tempDir` を分けてから、`config`、`run`、`replay`、`list` を順に確認する
- retirement: `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json` を残し、`tempDir` の一時物を消す

### 9.1 backup

- backup の単位は、`storage node` の `dataDir` を基本とし、少なくとも `rawLogPath`、`catalogPath`、replay metadata、manifest を含める
- backup bundle では、`manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` を基本要素として扱う
- `resolved-config.json` がある場合は、復旧時の比較用に残す
- backup は replay 可能性を壊さないことを優先する
- 物理コピーの粒度よりも、再投入時に同じ canonical state を再構成できることを優先する
- archive 形式に載せる場合は [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md) の `manifest.json` と `wire-log.jsonl` の考え方に合わせる

backup 実行時の確認は次の順で行います。

1. `config` で `stateDir`、`dataDir`、`backupDir`、`tempDir` を確認する
2. `backupDir` の空き容量と書き込み権限を確認する
3. `dataDir` から `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、必要なら `resolved-config.json` を束ねる
4. archive を作る場合は `manifest.json` と `wire-log.jsonl` が一致しているか確認する
5. backup 後に、次回 restore に使う保管先を記録する

| 段階 | 確認 |
| --- | --- |
| 実行前 | `stateDir`、`dataDir`、`backupDir`、`tempDir`、空き容量、権限 |
| 実行中 | `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json` |
| 実行後 | 保管先、archive の有無、次回 restore 先 |

### 9.2 restore

- restore は、backup または archive から `storage node` を再構成する手順として扱う
- restore の前に、対象 backup の `archive version`、`protocol version`、`item count`、`manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` の所在を確認する
- restore の後に、`config`、`run`、`replay`、`list` の順で整合性を確認し、件数と保存先が想定どおりかを見る
- restore は canonical catalog を再生成できることを前提にする
- restore 中に不整合が見つかった場合は、replay を止めて backup の内容を優先して調査する

restore 実行時の確認は次の順で行います。

1. まず `tempDir` を空にし、restore 中の作業領域を分ける
2. `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` の有無と更新日時を確認する
3. `archive version`、`protocol version`、`item count` を確認する
4. `config` で `stateDir` と保存先が期待値と一致するか確認する
5. `run`、`replay`、`list` を順に実行する
6. 必要なら `relay` を再起動し、`ready` と `capabilities` を確認する

| 段階 | 確認 |
| --- | --- |
| 実行前 | `tempDir` の空状態、`manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`archive version`、`protocol version`、`item count` |
| 実行中 | `config` の `stateDir` と保存先、`run`、`replay`、`list` |
| 実行後 | `relay` の `ready`、`capabilities`、canonical state の再構成結果 |

### 9.3 retirement

- retirement は、`storage node` や `relay` を operator の判断で退役させる操作として扱う
- 退役前に、必要な archive を作成し、再投入に必要な `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` を残す
- 退役対象に残すものは、少なくとも `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json`、`backupDir` 配下の未使用退避物とする
- 退役対象からは、`tempDir` 配下の一時ファイル、未完了の作業領域、再生成可能なキャッシュを除外してよい
- 物理削除は retention policy と operator policy に従い、protocol semantic とは分けて扱う
- 退役完了後は、archive の保管先、保持期間、再投入担当を記録する
- 退役後も、必要なら archive から replay できることを確認する

retirement 実行時の確認は次の順で行います。

1. 退役対象の `stateDir`、`dataDir`、`backupDir`、`tempDir` を確認する
2. 必要な archive が作成済みか確認する
3. `tempDir` 配下の未完了作業と再生成可能キャッシュを削除する
4. `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json` を退役保管へ移す
5. retention policy と operator policy の記録を残す
6. 必要なら archive から replay できるかを、退役前に最終確認する

| 段階 | 確認 |
| --- | --- |
| 実行前 | `stateDir`、`dataDir`、`backupDir`、`tempDir`、archive 作成済みか |
| 実行中 | `tempDir` の削除、退役保管への移送、`manifest.json` と `wire-log.jsonl` の保持 |
| 実行後 | retention / operator policy の記録、必要なら archive replay の最終確認 |

### 9.4 再投入時の整合性

- 再投入時は、version と manifest が期待値と一致するかを先に見る
- `resolved-config.json` がある場合は、再投入先の `stateDir` と差分がないかを確認する
- その後で replay により canonical state が再構成できるかを確認する
- 失敗した場合は、`manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` のどこで崩れているかを順に切り分ける
- そのうえで、raw log の欠落、catalog の不整合、設定差分の順で切り分ける
- 再投入後は、`storage node` と `relay` を別々に再確認する

## 10. 運用例

### 10.1 通常運用

1. `storage node` の `config` を確認する
2. `storage node` の `run` で status を確認する
3. `relay` の `capabilities` を確認する
4. `relay` の HTTP listener を起動する
5. publish / retrieve を 1 件だけ通して、再起動後に同じ object を再確認する

### 10.2 backup からの restore

1. `backupDir` か archive 保管先から、`manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、必要なら `resolved-config.json` を取り出す
2. `archive version`、`protocol version`、`item count` を確認する
3. `storage node` の `dataDir` と `tempDir` を分けて restore を実行する
4. `storage node` の `config`、`run`、`replay`、`list` を順に確認する
5. `relay` を再起動し、`capabilities` と `ready` を確認する
6. 必要なら対象 object を `retrieve` して、再構成後の canonical state を確認する

### 10.3 退役

1. 退役対象の `storage node` または `relay` について、必要な archive を作成する
2. `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json` を退役保管に残す
3. `tempDir` 配下の一時ファイルと再生成可能なキャッシュを削除する
4. retention policy と operator policy の記録を残す
5. 必要なら archive から replay できることを確認してから、退役を完了する

## 参照

- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](../roadmap/OPERATIONAL_READINESS_BACKLOG.md)
- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
- [relay / storage separation](./RELAY_STORAGE_SEPARATION.md)
