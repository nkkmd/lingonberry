# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-12**

この文書は、PR #19 まで完了した quarantine 運用の次段階を、再開しやすい issue 単位で整理します。

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。実装前に必ず両方を確認してください。

---

## 完了済み

| 項目 | PR | 状態 |
|---|---:|---|
| persistent quarantine store | #8 | 完了 |
| revalidation / promotion | #9 | 完了 |
| batch promotion / dry-run | #10 | 完了 |
| status API | #13 | 完了 |
| Prometheus metrics | #15 | 完了 |
| scheduler integration | #17 | 完了 |
| operator annotations | #19 | 完了 |

---

## QL-1: Append-only Manual Dismissal Lifecycle

**優先度: highest**

### 目的

pending recordを物理削除せず、operator判断によって通常のpromotion対象から外せるようにします。

### 依存

- operator annotations #19
- status API #13
- metrics #15
- batch promotion #10

### 実装前に固定する判断

- pending recordだけをdismiss可能とするか
- promoted recordへのdismissalを拒否するか
- 1 record 1 active dismissalとしてidempotentにするか
- undo / reopenを初期版へ含めるか
- bounded `reasonCode` と自由文`note`の両方を持たせるか
- HTTP管理APIを同時実装するか

### 推奨初期判断

```text
対象: pending recordのみ
重複: idempotent
undo: 非スコープ
理由: bounded reasonCode + operator note
入口: Core + CLIを先行
HTTP: admin authと分離可能
```

### 推奨event schema

```json
{
  "id": "lb:qd:...",
  "quarantineId": "lb:q:...",
  "dismissedAt": "...Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_DISMISSED",
  "note": "duplicate external submission"
}
```

### 完了条件

- append-only dismissal ledgerがある
- unknown quarantine IDを拒否する
- promoted recordの扱いがtestで固定される
- batch promotionがdismissed recordを除外する
- statusに`dismissed`が追加される
- metricsにdismissed gaugeが追加される
-元quarantine recordとannotationを変更しない
- corruption / I/O errorを明示する
- operations文書を追加する
- `CURRENT_IMPLEMENTATION_STATUS.md`を更新する

### 非スコープ

- physical deletion
- undo / reopen
- distributed locking
- retention / compaction

---

## QL-2: Admin Authentication and Network Isolation

**優先度: high**

### 目的

quarantineの参照・promotion・annotation・将来のdismissal APIを、公開relayの一般surfaceから分離します。

### 検討項目

- authentication方式
- role / permission model
- loopback-onlyまたは別listen address
- reverse proxyでのpath isolation
- audit log
- rate limit
- secret管理
- failure responseの情報量

### 完了条件

-管理endpointの公開境界が文書化される
-無認証の一般公開構成を推奨しない
-少なくともlocal-only運用templateがある
-annotationやnoteを不必要に公開しない
-認証失敗をaudit可能にする

---

## QL-3: Persistent Rejection Decisions

**優先度: medium**

### 目的

現在のtransientな`rejected`判定と、operatorまたはpolicyによる恒久的なlifecycle stateを分離します。

### 検討項目

- `permanently-rejected`を自動判定できるか
- policy変更後の再評価を許すか
- operator承認を必須にするか
- dismissalとの違い
- status / metrics上の分類
- event取消の表現

### 注意

現在の`rejected`をそのまま永続状態として数えてはいけません。

---

## QL-4: Backup / Export / Restore

**優先度: medium**

### 対象

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
将来のdismissal / rejection ledger
```

### 完了条件

-一貫したsnapshot境界が定義される
-restore時の重複と順序を扱える
-原本hashまたはmanifestを検討する
-restore検証手順がある
-権限と秘密情報の扱いを文書化する

---

## QL-5: JSONL Index / Rotation / Compaction

**優先度: medium-low**

### 目的

append-only監査証跡を維持しつつ、長期運用時の読み取りコストと容量増加を管理します。

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

**優先度: medium**

### 対象

-同一recordの同時promotion
-resolution ledgerへの同時append
-annotation ledgerへの同時append
-schedulerと手動操作の競合
-将来のdismissalとの競合

### 完了条件

-同一hostとmulti-hostを区別する
-atomic appendの前提を文書化する
-duplicate eventの意味を固定する
-concurrency testを追加する
-distributed lockを実装しない場合は明記する

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
