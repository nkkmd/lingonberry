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
| verified backup / export / restore | #28 | 実装・PR化 |

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

**状態: implemented**

### 固定した判断

```text
対象: 全quarantine関連append-only state
入口: local administrative binary
snapshot境界: source copy後にlength + digestを再検証
manifest: versioned JSON、最後にatomic rename
restore先: managed fileが存在しないdirectoryのみ
secret環境変数: backup対象外
```

### 対象ファイル

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
```

### 実装済み完了条件

- sparse snapshotで不在ファイルを明示
- source変更検出時にvalid manifestを発行しない
- exact managed-file setをmanifestで検証
- byte lengthとintegrity digestを検証
- path traversal、unsupported version、tamperingを拒否
- restore前にbackup全体を検証
- destinationの既存managed fileを上書きしない
- temporary file + atomic renameでrestore
- bearer tokenと環境変数を保存しない
- export / verify / restore CLIを追加

### 非スコープ

- distributed snapshot coordination
- encryption at rest
- remote object storage
- retention scheduling
- compaction / rotation

関連文書：`docs/operations/QUARANTINE_BACKUP_RESTORE.md`

---

## QL-5: JSONL Index / Rotation / Compaction

**優先度: medium**

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

**優先度: highest**

### 対象

- 同一recordの同時promotion
- resolution / annotation / dismissal / rejection ledgerへの同時append
- admin auth auditへの同時append
- schedulerと手動操作の競合
- promotion / dismissal / permanent rejectionの競合
- backup export中のmutation

### 完了条件

- 同一hostとmulti-hostを区別する
- atomic appendの前提を文書化する
- duplicate eventの意味を固定する
- concurrency testを追加する
- backup snapshot lockとの関係を固定する
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

各quarantine関連PRでは、`CURRENT_IMPLEMENTATION_STATUS.md`を更新するか、更新不要の理由をPR本文へ記載します。
