# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine lifecycle、same-host concurrency、active-ledger index、archive-aware ordered reads、verified rotation、archive-inclusive backup / verify / restoreまで実装済みです。

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
| record-rewriting compaction | 未実装 |
| retention deletion | 未実装 |
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

`quarantine-ledger-index.json`はderived indexです。`quarantine-segments.json`と`quarantine-segments/`はimmutable archive evidenceです。

---

## 3. Archive-aware logical ledger

Logical read order：

```text
segment manifestのsequence順
→ active ledger
```

Quarantine records、resolutions、annotations、dismissals、permanent rejectionsはarchive-aware readerを使用します。Missing、tampered、duplicate、out-of-order、unlisted segmentはcorruptionです。

---

## 4. Verified rotation

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance rotate <managed-ledger-name>
lingonberry-quarantine-maintenance verify-segments
```

Rotationはfresh active-ledger indexとshared operation lockを要求し、元byte列をimmutable segmentへ移します。前後のlogical line countとordered-stream digestが一致しなければrollbackします。

---

## 5. Archive-inclusive backup v2

New export format：

```text
lingonberry-quarantine-backup/v2
```

CLIは変更ありません。

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

V2は以下を保存します。

```text
six active ledgers
quarantine-segments.json when present
all listed quarantine-segments/* files
```

Derived indexとoperation lockは保存しません。

Compatibility：

- `export`はv2を作成
- `verify`はv1とv2を受理
- `restore`はv1とv2を受理

Exportはsource lockとsegment verificationを使用します。Restoreはdestination lock、conflict check、temporary file + atomic rename、final segment verificationを使用し、final verification失敗時はrestoreが書いたfileをrollbackします。

関連Issue：#36

関連文書：`docs/operations/QUARANTINE_BACKUP_RESTORE.md`

---

## 6. Backup対象外

```text
quarantine-ledger-index.json
.quarantine-operation.lock
```

Indexはrestore後に再構築します。Lockはruntime coordination stateであり、移送しません。Environment variableとbearer tokenもbackupへ含めません。

---

## 7. Same-host operation lock

Mutation、backup export、restore destination write、index build、rotationを直列化します。Distributed lockやnetwork filesystem leaseではありません。

---

## 8. HTTP boundary

Public listenerへquarantine management routeを公開しません。Backupとmaintenanceはlocal administrative binaryだけで実行します。

---

## 9. 主要ファイル

```text
packages/core/src/quarantine_complete_backup.rs
packages/core/src/quarantine_backup.rs
packages/core/src/quarantine_segments.rs
packages/core/src/quarantine_ledger_index.rs
packages/relay/src/quarantine_backup_main.rs
packages/relay/src/quarantine_maintenance_main.rs
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
```

Restore rehearsal：

```bash
lingonberry-quarantine-backup restore \
  /tmp/lingonberry-backup \
  /tmp/lingonberry-restored

LINGONBERRY_STATE_DIR=/tmp/lingonberry-restored \
  lingonberry-quarantine-maintenance verify-segments
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
10. v2 backup manifestを全file copy完了前に発行しない
11. archive-inclusive backupを検証せずrestoreしない
12. restore destinationの既存stateを上書きしない
13. derived indexやlock fileをbackupからrestoreしない
14. compactionのsemantic equivalence検証前にsource evidenceを削除しない
15. retention deletionをcompactionと同時に暗黙実行しない

---

## 12. 次の推奨作業

### 第一候補

```text
QL-5C2: Verified Compaction Policy and Proof
```

まずledger typeごとの「削除可能な情報」と「永久保持する監査証跡」を定義し、実データを書き換えないcompaction previewとsemantic proofから始めます。

### 第二候補

```text
Multi-role authorization
```

単一admin roleをreviewer、operator、observerへ分離します。

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
→ #36 archive-inclusive backup / restore
```
