# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-12**

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。

## 完了済み

| 項目 | PR / Issue | 状態 |
|---|---:|---|
| persistent quarantine store | #8 | 完了 |
| revalidation / promotion | #9 | 完了 |
| batch promotion / dry-run | #10 | 完了 |
| status API | #13 | 完了 |
| Prometheus metrics | #15 | 完了 |
| scheduler integration | #17 | 完了 |
| operator annotations | #19 | 完了 |
| append-only manual dismissal lifecycle | #22 / #23 | 完了 |
| admin authentication and network isolation | #24 / #25 | 完了 |
| append-only permanent rejection lifecycle | #26 / #27 | 完了 |
| verified backup / export / restore | #28 / #29 | 完了 |
| same-host concurrent ledger coordination | #30 | 実装・PR化 |

---

## QL-1: Append-only Manual Dismissal Lifecycle

**状態: completed**

関連文書：`docs/operations/QUARANTINE_DISMISSALS.md`

---

## QL-2: Admin Authentication and Network Isolation

**状態: completed**

関連文書：`docs/operations/QUARANTINE_ADMIN_HTTP.md`

---

## QL-3: Persistent Rejection Decisions

**状態: completed**

関連文書：`docs/operations/QUARANTINE_PERMANENT_REJECTIONS.md`

---

## QL-4: Backup / Export / Restore

**状態: completed**

関連文書：`docs/operations/QUARANTINE_BACKUP_RESTORE.md`

---

## QL-6: Concurrent Ledger Operations

**状態: implemented**

### 固定した判断

```text
lock scope: state directory全体
lock file: .quarantine-operation.lock
対象: mutation + backup export + restore destination write
read-only operation: lock不要
競合時: LB_QUARANTINE_BUSYでfail closed
stale recovery: 15分を超えたlockを1回だけ回収
multi-host / NFS distributed lock: 非スコープ
```

### 対象操作

- quarantine record append
- promotion resolution append
- annotation append
- dismissal
- permanent rejection
- admin authentication failure audit
- backup export
- restore destination write

### 実装済み完了条件

- `create_new`による同一host filesystem exclusion
- normal scope exitでlock fileを削除
- stale lock recovery
- promotion / dismissal / permanent rejectionを同じ排他境界で再確認
- terminal lifecycle stateの二重commitを拒否
- backup export中のcooperating mutationを拒否
- restore中のdestination mutationを拒否
- lock metadataをoperation、PID、timestampのみに制限
- bearer token、payload、operator、note、quarantine IDをlockへ保存しない
- duplicate terminal eventは引き続きcorruption

### 非スコープ

- distributed locking
- multi-node consensus
- network filesystem lease
- older binaryや手動file editとの協調
- indefinite wait queue

関連文書：`docs/operations/QUARANTINE_CONCURRENCY.md`

---

## QL-5: JSONL Index / Rotation / Compaction

**優先度: highest**

### 検討順

1. read-only index
2. archive export
3. rotation
4. verified compaction
5. retention policy

### 固定条件

- compaction前の原本を検証可能にする
- lifecycle eventの意味を失わない
- unknown / corrupt lineを黙って除外しない
- statusとmetricsの値がcompaction前後で一致する
- lockとbackup manifestの意味を維持する

---

## 再開時のIssue作成テンプレート

```markdown
## Goal
## Persistent state changes
## CLI / HTTP changes
## Lifecycle semantics
## Idempotency and concurrency
## Error handling
## Tests
## Documentation updates
## Non-goals
```

各quarantine関連PRでは、`CURRENT_IMPLEMENTATION_STATUS.md`を更新するか、更新不要の理由をPR本文へ記載します。
