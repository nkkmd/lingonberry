# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine lifecycle、運用API、same-host concurrency、verified backup、verified active-ledger index、archive-aware ordered reads、byte-preserving verified rotationまで実装済みです。

| 項目 | 状態 |
|---|---|
| persistent quarantine lifecycle | 実装済み |
| status / metrics / scheduler | 実装済み |
| admin authentication / isolation | 実装済み |
| verified active-ledger backup / restore | 実装済み |
| same-host concurrent ledger coordination | 実装済み |
| verified read-only active-ledger index | 実装済み |
| archive-aware ordered readers | 実装済み |
| byte-preserving verified rotation | 実装済み |
| archive-inclusive backup / restore | 未実装 |
| record-rewriting compaction / retention | 未実装 |
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

`quarantine-ledger-index.json`はactive ledger用derived indexです。`quarantine-segments.json`と`quarantine-segments/`はimmutable archive evidenceです。

---

## 3. Archive-aware logical ledger

各managed ledgerの論理read orderは次です。

```text
manifestに記載されたsegment sequence順
→ active ledger
```

Archive-aware readerへ移行済み：

- quarantine records
- promotion resolutions
- annotations
- dismissals
- permanent rejections

Duplicate lifecycle event、malformed JSONL、missing / tampered / unlisted segmentはcorruptionとして拒否します。

---

## 4. Verified rotation

Manifest：

```text
<LINGONBERRY_STATE_DIR>/quarantine-segments.json
```

Archive directory：

```text
<LINGONBERRY_STATE_DIR>/quarantine-segments/
```

CLI：

```bash
lingonberry-quarantine-maintenance verify-segments
lingonberry-quarantine-maintenance rotate <managed-ledger-name>
```

Rotation contract：

1. shared operation lockを取得
2. fresh active-ledger indexを検証
3. missing / empty active ledgerを拒否
4. rotation前のordered logical streamを取得
5. active bytesを新しいimmutable segmentへ保存
6. active ledgerを空fileへ置換
7. manifestをtemporary file + atomic renameで更新
8. archived + active streamを再読込
9. logical line countとordered-stream digestを比較
10. equivalence失敗時はactive、manifest、新segmentをrollback

Rotationはbyte-preservingであり、record rewrite、deduplication、deletionを行いません。

関連Issue：#34

関連文書：`docs/operations/QUARANTINE_JSONL_MAINTENANCE.md`

---

## 5. Index and maintenance planning

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance plan <byte-threshold> <line-threshold>
```

Indexはactive ledgerのみを対象とします。Active ledger mutation後、rotation前には必ず再構築してください。

---

## 6. Same-host operation lock

```text
<LINGONBERRY_STATE_DIR>/.quarantine-operation.lock
```

Mutation、backup export、restore destination write、index build、rotationを直列化します。これはdistributed lockやnetwork filesystem leaseではありません。

---

## 7. Backup limitation

QL-4 backup manifestは現在、六つのactive ledger pathのみを対象とします。

Post-rotation stateの完全backupには以下を一体で保存する必要があります。

```text
six active ledgers
quarantine-segments.json
quarantine-segments/
```

Archive-inclusive backup / verify / restoreはQL-5Cの最優先作業です。現行の`lingonberry-quarantine-backup`だけをpost-rotation完全backupとして扱ってはいけません。

---

## 8. HTTP boundary

Public listenerはreadiness、capabilities、publish、object retrievalのみです。Quarantine maintenanceとbackupはlocal administrative binaryだけで実行します。

---

## 9. 主要ファイル

```text
packages/core/src/quarantine_segments.rs
packages/core/src/quarantine_ledger_index.rs
packages/core/src/quarantine.rs
packages/core/src/quarantine_annotations.rs
packages/core/src/quarantine_dismissals.rs
packages/core/src/quarantine_rejections.rs
packages/relay/src/quarantine_maintenance_main.rs
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
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
```

Rotation例：

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance rotate quarantine.jsonl
lingonberry-quarantine-maintenance verify-segments
```

---

## 11. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only lifecycle eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. terminal recordを通常のpromotion対象へ戻さない
5. transient `rejected`をpermanent rejectionとして自動保存しない
6. quarantine管理routeを公開listenerへ戻さない
7. terminal stateはoperation lock保持中に再確認する
8. same-host lockをdistributed lockとして扱わない
9. stale active-ledger indexでrotationしない
10. archive segmentを上書き・変更・削除しない
11. manifest未登録segmentを黙って読む、または無視しない
12. rotation前後のlogical stream equivalenceを検証する
13. equivalence失敗時にpartial transitionを残さない
14. archive-inclusive backup完成前にretentionやcompactionを実行しない
15. record rewriteのsemantic equivalence検証前にsource evidenceを削除しない

---

## 12. 次の推奨作業

### 第一候補

```text
QL-5C: Archive-inclusive Backup, Verified Compaction, and Retention
```

最初にarchive segmentを含むbackup / verify / restoreへ拡張します。その後、ledger type別compaction policyとsemantic equivalence検証を設計します。

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
→ #29 verified backup / restore
→ #31 concurrent ledger coordination
→ #33 verified JSONL index and planning
→ #35 archive-aware verified rotation
```
