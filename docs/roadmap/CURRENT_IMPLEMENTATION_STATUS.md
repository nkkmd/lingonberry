# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine lifecycle、same-host concurrency、active-ledger index、archive-aware ordered reads、verified rotation、archive-inclusive backup、non-destructive compaction preview / proofまで実装済みです。

| 項目 | 状態 |
|---|---|
| persistent quarantine lifecycle | 実装済み |
| status / metrics / scheduler | 実装済み |
| admin authentication / isolation | 実装済み |
| same-host concurrent ledger coordination | 実装済み |
| verified active-ledger index | 実装済み |
| archive-aware ordered readers | 実装済み |
| byte-preserving verified rotation | 実装済み |
| archive-inclusive backup / verify / restore | 実装済み |
| non-destructive compaction preview / proof | 実装済み |
| record-rewriting compaction | 未実装・未承認 |
| retention deletion | 未実装・未承認 |
| multi-role authorization | 未実装 |
| distributed locking | 未実装 |

---

## 2. Runtime state

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
quarantine-ledger-index.json
quarantine-segments.json
quarantine-segments/
.quarantine-operation.lock
```

Compaction proofはruntime state外のoperator-selected output directoryへ作成します。

---

## 3. Archive-aware logical ledger

Logical read order：

```text
segment manifestのsequence順
→ active ledger
```

Missing、tampered、duplicate、out-of-order、unlisted segmentはcorruptionです。

---

## 4. Rotation and backup

Rotation：

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance rotate <managed-ledger-name>
lingonberry-quarantine-maintenance verify-segments
```

Archive-inclusive backup：

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

New exportは`lingonberry-quarantine-backup/v2`です。Verify / restoreはv1とv2を受理します。

---

## 5. Compaction policy v1

```text
lingonberry-quarantine-compaction-policy/v1
```

### Immutable evidence

```text
quarantine.jsonl
quarantine-annotations.jsonl
admin-auth-audit.jsonl
```

### Terminal single-event evidence

```text
quarantine-resolutions.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
```

Terminal ledgerのduplicate quarantine IDはcompaction candidateではなくcorruptionです。

Policy v1の結論：

```json
{
  "mutationAllowed": false,
  "rewritePerformed": false,
  "removableLines": 0
}
```

現時点で安全に削除できるlineは定義されていません。

---

## 6. Compaction preview and proof

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay

lingonberry-quarantine-maintenance compaction-preview \
  <verified-backup-v2-dir> \
  <empty-proof-output-dir>

lingonberry-quarantine-maintenance verify-compaction-proof \
  <proof-output-dir>
```

Output：

```text
quarantine-compaction-proof.json
quarantine-compaction-proof.digest
```

Proofは以下を記録します。

- backup manifest digest
- segment manifest digest
- per-ledger line / byte count
- ordered logical stream digest
- unique key count
- retained / removable line count
- promoted / dismissed / permanently rejected count
- proof / policy version
- no mutation / no rewrite statement

Preview前後でruntime fingerprintを比較します。Previewはmutation lockを取得せず、runtime stateを書き換えません。

関連Issue：#38

関連文書：`docs/operations/QUARANTINE_COMPACTION_PROOF.md`

---

## 7. Same-host operation lock

Mutation、backup export、restore destination write、index build、rotationを直列化します。Compaction previewはread-onlyでlock-freeです。Distributed lockやnetwork filesystem leaseではありません。

---

## 8. HTTP boundary

Public listenerへquarantine management routeを公開しません。Backupとmaintenanceはlocal administrative binaryだけで実行します。

---

## 9. 主要ファイル

```text
packages/core/src/quarantine_compaction.rs
packages/core/src/quarantine_complete_backup.rs
packages/core/src/quarantine_segments.rs
packages/core/src/quarantine_ledger_index.rs
packages/relay/src/quarantine_backup_main.rs
packages/relay/src/quarantine_maintenance_main.rs
docs/operations/QUARANTINE_COMPACTION_PROOF.md
docs/operations/QUARANTINE_BACKUP_RESTORE.md
docs/operations/QUARANTINE_JSONL_MAINTENANCE.md
docs/roadmap/QUARANTINE_LIFECYCLE_BACKLOG.md
```

---

## 10. 再開時の確認

```bash
git switch main
git pull --ff-only
git status
cargo test --workspace

export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-maintenance verify-segments
lingonberry-quarantine-backup export /tmp/lingonberry-backup
lingonberry-quarantine-backup verify /tmp/lingonberry-backup
lingonberry-quarantine-maintenance compaction-preview \
  /tmp/lingonberry-backup \
  /tmp/lingonberry-compaction-proof
lingonberry-quarantine-maintenance verify-compaction-proof \
  /tmp/lingonberry-compaction-proof
```

---

## 11. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only lifecycle eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. terminal lifecycleの競合をoperation lock内で再確認する
5. same-host lockをdistributed lockとして扱わない
6. stale indexでrotationしない
7. archive segmentを上書き・変更・削除しない
8. manifest未登録segmentを黙って無視しない
9. rotation前後のlogical stream equivalenceを検証する
10. verified backup v2なしでcompaction previewを作らない
11. duplicate terminal eventをremoval candidateとして扱わない
12. policy v1 proofでmutationを許可しない
13. preview output以外を変更しない
14. explicit replacement policyなしでrecord rewriteを実装しない
15. retention deletionをrewriteと同時に暗黙実行しない

---

## 12. 次の推奨作業

### 第一候補

```text
Multi-role authorization
```

Policy v1ではremovable lineが0のため、具体的なreplacement semanticsが承認されるまでQL-5C3 rewriteへ進むべきではありません。次の独立した改善として、単一admin roleをreviewer、operator、observerへ分離します。

### 将来候補

```text
QL-5C3: Verified Rewrite Transaction
```

開始には、lineを除去または置換してよい具体的なpolicy、semantic equivalence、source evidence preservation、interrupted recoveryの承認が必要です。

---

## 13. 実装順序

```text
#7 acceptance policy
→ #8 persistent quarantine
→ #9 promotion
→ #10 batch promotion
→ #13 status
→ #15 metrics
→ #17 scheduler
→ #19 annotations
→ #23 manual dismissal
→ #25 admin HTTP isolation
→ #27 permanent rejection
→ #29 active-ledger backup / restore
→ #31 concurrent ledger coordination
→ #33 verified JSONL index and planning
→ #35 archive-aware verified rotation
→ #37 archive-inclusive backup / restore
→ #38 non-destructive compaction preview / proof
```
