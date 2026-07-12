# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine、promotion、status / metrics、scheduler、annotations、manual dismissal、admin HTTP isolation、permanent rejection、verified backup / restore、same-host concurrency、verified read-only JSONL indexまで実装済みです。

| 項目 | 状態 |
|---|---|
| persistent quarantine lifecycle | 実装済み |
| status / metrics / scheduler | 実装済み |
| admin authentication / isolation | 実装済み |
| verified backup / export / restore | 実装済み |
| same-host concurrent ledger coordination | 実装済み |
| verified read-only JSONL index | 実装済み |
| non-destructive maintenance planning | 実装済み |
| archive-aware rotation | 未実装 |
| verified compaction / retention | 未実装 |
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
.quarantine-operation.lock
```

`quarantine-ledger-index.json`はderived read-only indexであり、監査原本ではありません。Lifecycle ledgerはappend-onlyです。

---

## 3. Persistent lifecycle

```text
pending
promoted
dismissed
permanently-rejected
```

Transientな再検証結果`rejected`はpersistent stateへ自動変換しません。Terminal lifecycle eventの競合はsame-host operation lock内で再確認します。

---

## 4. Same-host operation lock

```text
<LINGONBERRY_STATE_DIR>/.quarantine-operation.lock
```

- mutation、backup export、restore destination write、index buildを直列化
- 競合時は`LB_QUARANTINE_BUSY`
- 15分を超えたlockをstaleとして回収可能
- read-only status / metrics / backup verify / index verify / planはlock不要
- distributed lockやnetwork filesystem leaseではない

関連文書：`docs/operations/QUARANTINE_CONCURRENCY.md`

---

## 5. Verified JSONL index

Index file：

```text
<LINGONBERRY_STATE_DIR>/quarantine-ledger-index.json
```

各managed ledgerについて次を記録します。

```text
presence
byte length
non-empty JSONL line count
first record byte offset
last record byte offset
integrity digest
```

Index buildは全non-empty lineをprotocol JSON parserで検証し、partial trailing line、malformed JSON、source mutationを拒否します。Indexはtemporary fileからatomic renameで最後に発行します。

CLI：

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance plan <byte-threshold> <line-threshold>
```

Plannerはthreshold超過だけを報告し、ledgerを変更しません。結果には常に次を含めます。

```json
{"destructiveActionsBlocked":true}
```

関連文書：`docs/operations/QUARANTINE_JSONL_MAINTENANCE.md`

関連Issue：#32

---

## 6. Rotation / compaction boundary

Active ledgerのtruncate、archive segmentへの移動、compaction、retention enforcementは未実装であり禁止です。

QL-5Bの前提：

1. active + archived segmentを順序付きで読む共通reader
2. segment manifestとprovenance
3. interrupted transitionのrecovery contract
4. maintenance前後のsemantic equivalence verification
5. status / metrics / lifecycle eligibility / corruption detectionの一致

---

## 7. Backup / restore

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

Backup manifestは`quarantine-backup-manifest.json`です。Backup対象は監査原本ledgerであり、derived indexとlock fileは含みません。

---

## 8. HTTP boundary

Public listenerはreadiness、capabilities、publish、object retrievalのみです。`/metrics`と`/v1/quarantine*`は`404`です。

Admin listener：

```bash
export LINGONBERRY_ADMIN_TOKEN='<long-random-secret>'
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

Maintenanceとbackupはlocal administrative binaryだけで実行します。

---

## 9. 主要ファイル

```text
docs/operations/QUARANTINE_JSONL_MAINTENANCE.md
docs/operations/QUARANTINE_CONCURRENCY.md
docs/operations/QUARANTINE_BACKUP_RESTORE.md
docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md
docs/roadmap/QUARANTINE_LIFECYCLE_BACKLOG.md
packages/core/src/quarantine_ledger_index.rs
packages/core/src/quarantine_lock.rs
packages/core/src/quarantine_backup.rs
packages/relay/src/quarantine_maintenance_main.rs
```

---

## 10. 再開時の確認

```bash
git switch main
git pull --ff-only
git status
cargo test --workspace

export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance plan 67108864 100000
```

---

## 11. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. terminal recordを通常のpromotion対象へ戻さない
5. transient `rejected`をpermanent rejectionとして自動保存しない
6. quarantine管理routeを公開listenerへ戻さない
7. terminal stateはoperation lock保持中に再確認する
8. same-host lockをdistributed lockとして扱わない
9. backup manifestをmanaged fileより先に発行しない
10. index build中のsource mutationを拒否する
11. malformed JSONLやpartial trailing lineをindexから黙って除外しない
12. stale indexをmaintenance判断へ使用しない
13. archive-aware reader完成前にactive ledgerをtruncateしない
14. semantic equivalence検証前にcompactionやretentionを実行しない

---

## 12. 次の推奨作業

### 第一候補

```text
QL-5B: Archive-aware Rotation and Verified Compaction
```

まずarchive-aware ordered readerとsegment manifestを導入し、その後にverified rotationを実装します。

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
→ #32 verified JSONL index and planning
```
