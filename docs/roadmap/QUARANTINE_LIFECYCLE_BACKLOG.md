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
| verified read-only JSONL index and planning | #32 / #33 | 完了 |
| archive-aware ordered reads and verified rotation | #34 | 実装・PR化 |

---

## QL-1 — QL-4, QL-6

**状態: completed**

---

## QL-5A: Verified Read-only JSONL Index and Planning

**状態: completed**

関連文書：`docs/operations/QUARANTINE_JSONL_MAINTENANCE.md`

---

## QL-5B: Archive-aware Ordered Reads and Verified Rotation

**状態: implemented**

### 固定した判断

```text
manifest: quarantine-segments.json
archive dir: quarantine-segments/
read order: ledger別segment sequence順 → active ledger
rotation prerequisite: fresh quarantine-ledger-index.json
rotation scope: 1 managed ledger
coordination: state-directory-wide operation lock
archive evidence: immutable、削除・上書き禁止
semantic verification: logical line count + ordered-stream digest
compaction / retention: QL-5Cまで禁止
```

### Archive-aware reader対象

- quarantine records
- promotion resolutions
- annotations
- dismissals
- permanent rejections
- admin auth audit向け共通utility

### 実装済み完了条件

- active-onlyとarchived + activeを同じ論理streamとして読む
- ledgerごとのsequenceをstrictly increasingにする
- segmentのbyte length、line count、digest、JSONL妥当性を検証
- missing、tampered、duplicate、out-of-order segmentを拒否
- manifest未登録segmentをcorruptionとして拒否
- stale indexではrotationしない
- missing / empty active ledgerをrotationしない
- original bytesをimmutable segmentへ保存
- manifestをtemporary file + atomic renameで発行
- rotation前後の論理line streamを検証
- equivalence失敗時にactive、manifest、新segmentをrollback
- repeated rotationに対応

### 明示的制限

現行のQL-4 backup manifestはactive ledgerのみを対象としており、archive segmentを含みません。Post-rotation stateはfilesystem-level snapshot等でactive ledger、segment manifest、archive directoryを一体保存する必要があります。

関連文書：`docs/operations/QUARANTINE_JSONL_MAINTENANCE.md`

---

## QL-5C: Archive-inclusive Backup, Verified Compaction, and Retention

**優先度: highest**

### 前提

1. archive segmentを含むbackup / verify / restore
2. ledger type別のcompaction policy
3. status / metrics / eligibility / idempotencyのsemantic equivalence
4. source evidenceまたは検証可能なreplacement proof
5. interrupted compaction recovery

### 完了条件

- archive-inclusive backupから完全restoreできる
- compactionでunknown / corrupt lineを黙って除外しない
- compaction前後でstatus、metrics、lifecycle判定が一致する
- duplicate detectionの意味を維持する
- retention deletionはverified compaction後にのみ許可する
- source evidenceの削除条件を明文化する

### 非スコープのまま維持するもの

- distributed locking
- remote archive storage
- cryptographic signing

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
