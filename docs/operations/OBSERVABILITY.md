# Observability

**Status: active** | **Last updated: 2026-06-19**

## 目的

この文書は、Lingonberry の運用時に必要な監視・ログ・メトリクスの正本メモです。  
Phase 5 では、障害検知と原因追跡に必要な最小セットを先に固定します。

## 1. 基本方針

- 観測は `relay` と `storage node` の運用成立に直結するものに絞る
- 構造化ログは機械的に追える形を優先する
- メトリクスは低カーディナリティで、行動に結びつくものだけにする
- alert は「見るべき場所が分かる」ことを重視し、過剰に増やさない
- domain truth や profile 固有の判断は observability に持ち込まない
- `relay` と `storage node` で同じ語彙を使い、違いは `service` と `component` で表す

## 2. 構造化ログ

### 2.1 共通 field

`relay` と `storage node` で共有する共通 field は次を基本にします。

- `timestamp`
- `level`
- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `objectId`
- `carrier`
- `durationMs`
- `errorType`
- `errorCode`

### 2.1.1 field の意味

- `service`: `relay` か `storage node` を表す
- `component`: `http`、`cli`、`storage`、`runtime` などの内部区分を表す
- `event`: 観測した事象の種類を表す
- `status`: `ok` / `error` / `warn` などの結果を表す
- `requestId`: 1 回の要求や処理の追跡子を表す
- `objectId`: 対象 object が分かる場合に入れる
- `carrier`: HTTP など carrier を経由した場合に入れる
- `durationMs`: 処理時間を表す
- `errorType`: 失敗の分類を表す
- `errorCode`: 実装や運用で参照する短い分類コードを表す

### 2.2 イベント種別

最低限、次の event を追えるようにします。

- startup
- config_resolved
- readiness_checked
- publish_received
- append_completed
- replay_completed
- retrieve_completed
- validation_failed
- rate_limited
- runtime_error
- shutdown_requested
- shutdown_completed

### 2.2.1 service ごとの主イベント

#### `relay`

- `startup`
- `config_resolved`
- `readiness_checked`
- `publish_received`
- `validation_failed`
- `rate_limited`
- `shutdown_requested`
- `shutdown_completed`

#### `storage node`

- `startup`
- `config_resolved`
- `readiness_checked`
- `append_completed`
- `replay_completed`
- `retrieve_completed`
- `validation_failed`
- `runtime_error`
- `shutdown_requested`
- `shutdown_completed`

### 2.2.2 event ごとの必須 field

#### `startup`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`

#### `config_resolved`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `durationMs`

#### `readiness_checked`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `durationMs`

#### `publish_received`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `carrier`
- `durationMs`

#### `append_completed`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `objectId`
- `durationMs`

#### `replay_completed`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `durationMs`

#### `retrieve_completed`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `objectId`
- `durationMs`

#### `validation_failed`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `errorType`
- `errorCode`

#### `rate_limited`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `carrier`
- `errorType`
- `errorCode`
- `durationMs`

#### `runtime_error`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`
- `errorType`
- `errorCode`

#### `shutdown_requested`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`

#### `shutdown_completed`

- `service`
- `component`
- `event`
- `status`
- `message`
- `requestId`

### 2.3 ログの扱い

- 1 行 1 event を基本にする
- 人間向けの説明文は `message` に寄せる
- 追跡に必要な識別子は毎回同じ field 名で出す
- 例外メッセージだけに依存せず、分類用の `errorType` と `errorCode` を残す
- `requestId` は、HTTP request だけでなく CLI でも 1 回の操作を追えるように付ける

### 2.4 ログ例

```json
{
  "timestamp": "2026-06-19T12:34:56Z",
  "level": "info",
  "service": "storage node",
  "component": "runtime",
  "event": "config_resolved",
  "status": "ok",
  "message": "resolved storage node config",
  "requestId": "req_01HZX...",
  "durationMs": 4
}
```

```json
{
  "timestamp": "2026-06-19T12:35:02Z",
  "level": "error",
  "service": "relay",
  "component": "http",
  "event": "publish_received",
  "status": "error",
  "message": "publish validation failed",
  "requestId": "req_01HZY...",
  "objectId": "lb:obj:...",
  "carrier": "http",
  "errorType": "validation_error",
  "errorCode": "HTTP_PUBLISH_INVALID",
  "durationMs": 12
}
```

### 2.5 サービス別のログ要件

#### `relay`

- `startup` で起動対象 binary と mode が分かる
- `config_resolved` で bind 先と受け口の前提が分かる
- `readiness_checked` で listener の状態が分かる
- `publish_received` で carrier と対象 object の追跡ができる
- `validation_failed` で request 側の問題か runtime 側の問題かを区別できる
- `rate_limited` で過負荷や abuse による拒否を追える
- `shutdown_requested` と `shutdown_completed` で停止経路が追える

#### `storage node`

- `startup` で起動対象 binary と mode が分かる
- `config_resolved` で `stateDir` と保存先が分かる
- `readiness_checked` で保存先の状態が分かる
- `append_completed` で append の成功と対象 object が追える
- `replay_completed` で再構成の成功と所要時間が追える
- `retrieve_completed` で取得対象 object が追える
- `validation_failed` で保存前の不備が追える
- `runtime_error` で保存層の障害が追える
- `shutdown_requested` と `shutdown_completed` で停止経路が追える

## 3. メトリクス

### 3.1 最低限の種類

次の 3 種類を基本にします。

- counter
- gauge
- histogram

### 3.1.1 使い分け

- `counter`: 起動回数、成功回数、失敗回数のように増えるだけの値に使う
- `gauge`: 現在値を表すものに使う
- `histogram`: 処理時間や待ち時間の分布に使う

### 3.2 まず見る指標

- 起動成功数 / 失敗数
- publish 受付数 / 成功数 / 失敗数
- append 成功数 / 失敗数
- replay 実行回数 / 失敗回数
- retrieve 成功数 / 失敗数
- readiness 失敗回数
- 直近エラーの件数
- 処理時間の分布

### 3.2.1 共通 metric family

次の metric family を共通に使います。  
個別の実装では、`service` と `component` をラベルにして、`relay` と `storage node` を分けます。

- `lingonberry_startup_total` `counter`
- `lingonberry_config_resolved_total` `counter`
- `lingonberry_readiness_checked_total` `counter`
- `lingonberry_readiness_failure_total` `counter`
- `lingonberry_publish_total` `counter`
- `lingonberry_publish_failure_total` `counter`
- `lingonberry_rate_limited_total` `counter`
- `lingonberry_append_total` `counter`
- `lingonberry_append_failure_total` `counter`
- `lingonberry_replay_total` `counter`
- `lingonberry_replay_failure_total` `counter`
- `lingonberry_retrieve_total` `counter`
- `lingonberry_retrieve_failure_total` `counter`
- `lingonberry_validation_failure_total` `counter`
- `lingonberry_runtime_error_total` `counter`
- `lingonberry_shutdown_total` `counter`
- `lingonberry_operation_duration_ms` `histogram`
- `lingonberry_inflight_requests` `gauge`

### 3.2.2 service 別の重点指標

#### `relay`

- `lingonberry_startup_total`
- `lingonberry_readiness_checked_total`
- `lingonberry_publish_total`
- `lingonberry_publish_failure_total`
- `lingonberry_rate_limited_total`
- `lingonberry_validation_failure_total`
- `lingonberry_runtime_error_total`
- `lingonberry_operation_duration_ms`

#### `storage node`

- `lingonberry_startup_total`
- `lingonberry_config_resolved_total`
- `lingonberry_readiness_checked_total`
- `lingonberry_append_total`
- `lingonberry_replay_total`
- `lingonberry_retrieve_total`
- `lingonberry_validation_failure_total`
- `lingonberry_runtime_error_total`
- `lingonberry_operation_duration_ms`

### 3.2.3 最小ラベル

- `service`
- `component`
- `event`
- `result`
- `carrier`

ラベルは、障害の切り分けに必要な最小限だけに留めます。

`objectId` は label にせず、必要な場合だけログと response body に残します。

### 3.3 メトリクスの扱い

- ラベルは増やしすぎない
- object ごとの高カーディナリティな分割は避ける
- 運用判断に使わない指標は増やさない

## 4. Alert

### 4.1 初期に置く alert

- 起動失敗が連続する
- readiness が継続して失敗する
- publish 失敗率が高い
- rate limit の拒否が急増する
- replay 失敗が続く
- storage 由来の runtime error が継続する

### 4.1.1 目安の起点

- 起動失敗: 直近 3 回連続で失敗
- readiness 失敗: 5 分以上継続
- publish 失敗率: 15 分窓で 5% 超
- rate limit 拒否: 15 分窓で通常時の 2 倍超、または継続増加
- replay 失敗: 連続失敗が 2 回以上
- runtime error: 10 分窓で継続増加

### 4.2 閾値の考え方

- 閾値は固定値よりも、まずは運用判断に十分な期間の継続失敗で置く
- 一時的な揺れで鳴る alert は避ける
- 調査の起点になる alert と、根本原因を示すログ / メトリクスを分ける

### 4.3 alert を受けたときの確認順

1. `service` を確認して、`relay` か `storage node` かを分ける
2. `event` を確認して、startup / readiness / publish / replay / runtime_error のどれかを特定する
3. 直近のログで `requestId` と `errorType` を見る
4. 関連するメトリクスで件数と失敗率を見る
5. `stateDir` と保存先を確認する
6. 必要なら [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md) に戻る

### 4.4 alert の切り分け先

- `startup` と `readiness_checked` の問題は起動や設定解決を疑う
- `publish` と `validation_failed` の問題は request 側か carrier 側を疑う
- `rate_limited` の問題は公開面の閾値、対象 carrier、アクセス集中を疑う
- `append_completed`、`replay_completed`、`retrieve_completed` の問題は保存層を疑う
- `runtime_error` の問題は保存層か周辺処理を疑う
- `shutdown` 関連は停止経路や運用手順の問題として扱う

## 5. 最低限の観測項目

- `relay` と `storage node` の起動可否
- `ready` / `run` の結果
- config 解決結果
- publish / append / replay / retrieve の成否
- raw log と canonical catalog の保存先
- shutdown の開始と完了
- 直近の runtime error

### 5.1 観測の最小セット

#### `relay`

- `startup`
- `config_resolved`
- `readiness_checked`
- `publish_received`
- `validation_failed`
- `rate_limited`
- `shutdown_requested`
- `shutdown_completed`
- `runtime_error`

#### `storage node`

- `startup`
- `config_resolved`
- `readiness_checked`
- `append_completed`
- `replay_completed`
- `retrieve_completed`
- `runtime_error`
- `shutdown_requested`
- `shutdown_completed`
- `validation_failed`

## 6. 運用時の見る順番

1. readiness の結果を見る
2. 構造化ログで失敗 event を見る
3. 直近のメトリクス変化を見る
4. config と保存先を確認する
5. 必要なら runbook に従って replay / retrieve を確認する

### 6.1 切り分けの補助

- `relay` 側の failure なら、publish の受け口と HTTP listener を優先して見る
- `storage node` 側の failure なら、config 解決、保存先、replay を優先して見る
- 両方で failure が出るなら、共有環境変数と `stateDir` を優先して見る

## 7. 境界

- content の真偽は観測対象にしない
- profile 固有の trust rule は observability に含めない
- UI の都合や表示順序は observability に含めない
- carrier 変換の細部は contract で扱い、観測は結果と失敗点に絞る

## 参照

- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](../roadmap/OPERATIONAL_READINESS_BACKLOG.md)
- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
