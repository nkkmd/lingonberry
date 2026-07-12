# Quarantine Observability Metrics

**Status: implemented** | **Last updated: 2026-07-12**

## 1. 目的

quarantine の永続状態を、運用監視で利用できる低カーディナリティな Prometheus text format として公開します。

値は次の append-only ledger から毎回再構成します。

```text
<state-dir>/quarantine.jsonl
<state-dir>/quarantine-resolutions.jsonl
```

メトリクス取得は ledger を変更しません。

## 2. 取得方法

### CLI

```bash
cargo run -p lingonberry-relay -- quarantine-metrics
```

### HTTP

```text
GET /metrics
```

HTTP response の Content-Type:

```text
text/plain; version=0.0.4; charset=utf-8
```

## 3. Metric contract

### Quarantine records

```text
lingonberry_quarantine_records{state="total"}
lingonberry_quarantine_records{state="pending"}
lingonberry_quarantine_records{state="promoted"}
```

Type: `gauge`

- `total`: quarantine record の総数
- `pending`: promotion resolution がない record 数
- `promoted`: quarantine record と対応する promotion resolution の数

同じ quarantine ID に複数 resolution が存在しても `promoted` は 1 件として数えます。quarantine record に対応しない unknown resolution は含めません。

### Oldest pending age

```text
lingonberry_quarantine_oldest_pending_age_seconds
```

Type: `gauge`

最古の pending record の `receivedAt` から、メトリクス取得時刻までの経過秒数です。pending record が存在しない場合は `0` とします。

### Reason code breakdown

```text
lingonberry_quarantine_reason_code_records{reason_code="LB_IDENTITY_DEFERRED"}
```

Type: `gauge`

quarantine record を `reasonCode` ごとに集計します。`reasonCode` は実装が制御する短い分類コードとして扱います。

## 4. カーディナリティ規則

metric label に使用できるのは、既知の低カーディナリティ分類だけです。

使用する label:

```text
state
reason_code
```

使用しない値:

- quarantine ID
- canonical object ID
- request ID
- 自由文の `reasons`
- publish request 本文
- operator annotation

自由文や一意識別子は、必要に応じてログまたは管理 API で確認します。

## 5. 永続状態との関係

`promoted` は resolution ledger に永続化された lifecycle state です。

`deferred` と `rejected` は再評価時の一時的な decision であり、現在は累積 lifecycle metric として公開しません。これらを counter として公開する場合は、専用の永続 event ledger または process-level instrumentation を別途設計します。

## 6. Error handling

次の場合、CLI / HTTP は正常な metric snapshot を返しません。

- corrupt JSONL
- ledger read error
- system clock error

障害を `0` として偽装せず、既存の quarantine error として明示的に失敗させます。

## 7. 初期 alert の起点

初期運用では、次を監視候補とします。

- `pending` が継続増加する
- oldest pending age が運用上の再評価間隔を超える
- 特定 `reason_code` が急増する
- metric endpoint の取得自体が失敗する

固定閾値は運用データを得てから設定し、短時間の一時的増加だけで alert を発火させない方針とします。

## 8. 非スコープ

- promotion success / rejection の process-local counter
- batch duration histogram
- alert delivery
- scheduler
- authentication / authorization
- retention / compaction
- concurrent promotion locking

## 9. 関連資料

- [Observability](./OBSERVABILITY.md)
- [Quarantine Status API](../roadmap/QUARANTINE_STATUS_API.md)
- [Current Implementation Status](../roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
