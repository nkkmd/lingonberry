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
| same-host concurrent ledger coordination | #30 / #31 | 完了 |
| verified read-only JSONL index and planning | #32 | 実装・PR化 |

---

## QL-1 — QL-4, QL-6

**状態: completed**

関連文書：

```text
docs/operations/QUARANTINE_DISMISSALS.md
docs/operations/QUARANTINE_ADMIN_HTTP.md
docs/operations/QUARANTINE_PERMANENT_REJECTIONS.md
docs/operations/QUARANTINE_BACKUP_RESTORE.md
docs/operations/QUARANTINE_CONCURRENCY.md
```

---

## QL-5A: Verified Read-only JSONL Index and Planning

**状態: implemented**

### 固定した判断

```text
index file: quarantine-ledger-index.json
index build: shared operation lockを取得
index verify: read-only、lock不要
対象: exact managed ledger set
validation: 全non-empty lineをJSON parse
partial trailing line: corruptionとして拒否
planning: threshold超過を報告するだけ
rotation / compaction: QL-5Bまで禁止
```

### Index fields

- file presence
- byte length
- non-empty JSONL line count
- first record byte offset
- last record byte offset
- integrity digest

### 実装済み完了条件

- sparse / complete state directoryに対応
- malformed JSONとpartial trailing lineを拒否
- source mutationを再読込で検出
- versionとexact managed-file setを検証
- stale / tampered indexを拒否
- indexをtemporary file + atomic renameで最後に発行
- byte / line thresholdによるnon-destructive plan
- plannerは`destructiveActionsBlocked: true`を明示
- ledger contentsを変更しない

関連文書：`docs/operations/QUARANTINE_JSONL_MAINTENANCE.md`

---

## QL-5B: Archive-aware Rotation and Verified Compaction

**優先度: highest**

### 前提

1. active + archived segmentを順序付きで読む共通reader
2. segment manifestとprovenance
3. interrupted transitionのrecovery contract
4. maintenance前後のsemantic-equivalence verification

### 完了条件

- archive segmentを含めてもstatus / metricsが一致する
- lifecycle eligibilityとduplicate/corruption detectionが一致する
- original segmentを検証可能なまま保持する
- rotationをatomic state transitionとして扱う
- compactionでunknown / corrupt lineを黙って除外しない
- retention enforcementはverified compaction後にのみ許可する

### 非スコープのまま維持するもの

- archive-aware reader実装前のactive ledger truncation
- evidenceの即時削除
- distributed locking
- remote archive storage

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
