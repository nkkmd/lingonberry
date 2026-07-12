# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-12**

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。

## 完了済み

| 項目 | PR / Issue | 状態 |
|---|---:|---|
| persistent quarantine through permanent rejection | #8–#27 | 完了 |
| active-ledger verified backup / restore | #28 / #29 | 完了 |
| same-host concurrent ledger coordination | #30 / #31 | 完了 |
| verified read-only JSONL index and planning | #32 / #33 | 完了 |
| archive-aware ordered reads and verified rotation | #34 / #35 | 完了 |
| archive-inclusive backup / verify / restore | #36 | 実装・PR化 |

---

## QL-5A: Verified Read-only JSONL Index and Planning

**状態: completed**

---

## QL-5B: Archive-aware Ordered Reads and Verified Rotation

**状態: completed**

```text
manifest: quarantine-segments.json
archive dir: quarantine-segments/
read order: segment sequence順 → active ledger
rotation: fresh index + shared lock + semantic equivalence
archive evidence: immutable
```

---

## QL-5C1: Archive-inclusive Backup / Verify / Restore

**状態: implemented**

### 固定した判断

```text
new export version: lingonberry-quarantine-backup/v2
v1 verify / restore: backward compatible
v2 contents: six active ledgers + segment manifest + listed segments
excluded: derived index + operation lock
export: source lock + segment verification + source re-read
restore: destination lock + conflict rejection + final segment verification
```

### 実装済み完了条件

- post-rotation stateを一つのbackupで完全保存
- active-only stateにも対応
- v1 backupのverify / restore互換性を維持
- path traversal、missing、tampered、unlisted archive fileを拒否
- segment manifestとbackup manifestの不一致を拒否
- restore後にruntime segment verifierを実行
- final verification失敗時にrestoreが書いたfileをrollback
- bearer token、derived index、lock fileをbackupへ含めない

関連文書：`docs/operations/QUARANTINE_BACKUP_RESTORE.md`

---

## QL-5C2: Verified Compaction Policy and Proof

**優先度: highest**

### 前提

1. ledger type別のcompaction policy
2. status / metrics / eligibility / idempotencyのsemantic equivalence
3. source evidenceまたは検証可能なreplacement proof
4. interrupted compaction recovery
5. archive-inclusive backupの事前成功

### 完了条件

- unknown / corrupt lineを黙って除外しない
- compaction前後でstatus、metrics、lifecycle判定が一致する
- duplicate detectionの意味を維持する
- source segmentを即時削除しない
- retention deletionは別の明示的承認段階に分離する

### 非スコープ

- distributed locking
- remote archive storage
- cryptographic signing
- automatic retention deletion

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
