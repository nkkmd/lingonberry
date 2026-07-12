# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-12**

この文書は、quarantine運用の次段階を再開しやすいissue単位で整理します。

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。実装前に必ず両方を確認してください。

---

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
| append-only permanent rejection lifecycle | #26 | 実装・PR化 |

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

**状態: implemented**

### 固定した判断

```text
persistent state: permanently-rejected
対象: pending recordのみ
入口: Core + CLI + authenticated admin HTTP
作成主体: operatorのみ
transient Rejectedからの自動永続化: しない
重複: 1 record 1 active eventとしてidempotent
undo / reopen: 非スコープ
```

### Persistent state

```text
quarantine-rejections.jsonl
```

```json
{
  "id": "lb:qr:...",
  "quarantineId": "lb:q:...",
  "rejectedAt": "...Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_PERMANENTLY_REJECTED",
  "note": "known prohibited content"
}
```

### 実装済み完了条件

- append-only permanent rejection ledger
- unknown、promoted、dismissed recordを拒否
- duplicate requestをidempotentに処理
- duplicate ledger eventをcorruptionとして明示
- default listとbatch promotionから除外
- direct CLI / admin HTTP promotionを拒否
- statusに`permanentlyRejected`と`latestPermanentlyRejectedAt`を追加
- metricsに`permanently_rejected` gaugeを追加
- transient batch `rejected`とは別概念として保持
- 元quarantine recordとannotationを変更しない

### 非スコープ

- transient rejectionからの自動永続化
- physical deletion
- undo / reopen / appeal workflow
- distributed locking

関連文書：`docs/operations/QUARANTINE_PERMANENT_REJECTIONS.md`

---

## QL-4: Backup / Export / Restore

**優先度: highest**

### 対象

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
```

### 完了条件

- 一貫したsnapshot境界が定義される
- restore時の重複と順序を扱える
- 原本hashまたはmanifestを検討する
- restore検証手順がある
- 権限と秘密情報の扱いを文書化する

---

## QL-5: JSONL Index / Rotation / Compaction

**優先度: medium-low**

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

---

## QL-6: Concurrent Ledger Operations

**優先度: high**

### 対象

- 同一recordの同時promotion
- resolution / annotation / dismissal / rejection ledgerへの同時append
- admin auth auditへの同時append
- schedulerと手動操作の競合
- promotion / dismissal / permanent rejectionの競合

### 完了条件

- 同一hostとmulti-hostを区別する
- atomic appendの前提を文書化する
- duplicate eventの意味を固定する
- concurrency testを追加する
- distributed lockを実装しない場合は明記する

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

各quarantine関連PRでは、次を完了条件に含めます。

```text
CURRENT_IMPLEMENTATION_STATUS.mdを更新する、または更新不要の理由をPR本文へ記載する
```
