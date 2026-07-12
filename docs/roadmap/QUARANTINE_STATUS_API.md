# Quarantine Status API

**Status: implementation-ready** | **Issue: #12**

## 1. 目的

quarantine の永続状態を、CLI と HTTP の双方から同一の形式で確認できるようにします。

この API は、次の append-only ledger から再構成可能な状態だけを返します。

```text
<state-dir>/quarantine.jsonl
<state-dir>/quarantine-resolutions.jsonl
```

`deferred` と `rejected` は再評価時の一時的な判定結果であり、現在は恒久状態として保存されません。そのため、本 API の集計対象には含めません。

## 2. API 契約

### CLI

```bash
lingonberry quarantine-status
```

### HTTP

```text
GET /v1/quarantine-status
```

### 応答

```json
{
  "total": 0,
  "pending": 0,
  "promoted": 0,
  "oldestPendingAt": null,
  "latestReceivedAt": null,
  "latestPromotedAt": null,
  "reasonCodeCounts": {}
}
```

## 3. フィールド定義

| フィールド | 定義 |
|---|---|
| `total` | quarantine record の総数 |
| `pending` | resolution が存在しない quarantine record の数 |
| `promoted` | quarantine record と対応する promotion resolution の数 |
| `oldestPendingAt` | pending record の最古 `receivedAt` |
| `latestReceivedAt` | quarantine record 全体の最新 `receivedAt` |
| `latestPromotedAt` | resolution 全体の最新 `resolvedAt` |
| `reasonCodeCounts` | quarantine record 全体の `reasonCode` 別件数 |

## 4. 集計規則

1. `promoted` は quarantine record ID と resolution の `quarantineId` の積集合から算出します。
2. 同じ `quarantineId` の resolution が複数存在しても、1 件として数えます。
3. quarantine record に対応しない unknown resolution は集計に含めません。
4. `pending = total - promoted` とします。
5. `oldestPendingAt` は promoted record を除外して算出します。
6. ledger が存在しない場合は件数を `0`、timestamp を `null` とします。
7. corrupt JSONL または I/O error は黙って無視せず、既存の quarantine error として返します。
8. status の取得は ledger を変更しません。

## 5. 実装配置

### Core

```text
packages/core/src/quarantine.rs
```

追加候補:

```rust
pub struct QuarantineStatus {
    pub total: usize,
    pub pending: usize,
    pub promoted: usize,
    pub oldest_pending_at: Option<String>,
    pub latest_received_at: Option<String>,
    pub latest_promoted_at: Option<String>,
    pub reason_code_counts: BTreeMap<String, usize>,
}
```

`QuarantineStore::status()` を集計の唯一の正本とし、CLI と HTTP はこの関数を共有します。

### Relay

```text
packages/relay/src/main.rs
```

追加対象:

- CLI command dispatch: `quarantine-status`
- CLI handler
- HTTP route: `GET /v1/quarantine-status`
- HTTP handler
- JSON serialization helper

## 6. テスト要件

- empty ledger
- record 追加後の `total` / `pending`
- resolution 追加後の `promoted`
- promoted record が `oldestPendingAt` から除外されること
- 複数 `reasonCode` の集計
- unknown resolution を除外すること
- duplicate resolution を二重計上しないこと
- corrupt JSONL を明示的に失敗させること
- CLI と HTTP が同じ core 集計を利用すること

## 7. Observability への接続

後続実装では、この status を次の gauge の入力として利用できます。

```text
quarantine backlog size = pending
promoted record count = promoted
oldest pending age = now - oldestPendingAt
```

`reasonCode` は既知の低カーディナリティ分類としてのみ使用し、自由文の `reasons` は metric label にしません。

## 8. 非スコープ

- `deferred` / `rejected` の累計永続化
- last-observed decision ledger
- scheduler
- retention / compaction / rotation
- admin authentication / authorization
- Prometheus exposition endpoint
- process 間の concurrent promotion locking

## 9. 完了条件

- [ ] `QuarantineStatus` と `QuarantineStore::status()` を実装
- [ ] `lingonberry quarantine-status` を実装
- [ ] `GET /v1/quarantine-status` を実装
- [ ] core / CLI / HTTP tests を追加
- [ ] `cargo test --workspace` を通過
- [ ] JavaScript conformance tests を通過
- [ ] `CURRENT_IMPLEMENTATION_STATUS.md` を更新
- [ ] `IMPLEMENTATION_BACKLOG.md` を更新
- [ ] `OBSERVABILITY.md` を更新
