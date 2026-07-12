# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine、promotion、status / metrics、scheduler、annotations、manual dismissal、admin HTTP isolation、permanent rejection、verified backup / restore、same-host concurrent ledger coordinationまで実装済みです。

| 項目 | 状態 |
|---|---|
| persistent quarantine store | 実装済み |
| single / batch promotion | 実装済み |
| status / metrics | 実装済み |
| scheduler integration | 実装済み |
| append-only annotations | 実装済み |
| append-only manual dismissal | 実装済み |
| admin authentication / isolation | 実装済み |
| append-only permanent rejection | 実装済み |
| verified backup / export / restore | 実装済み |
| same-host concurrent ledger coordination | 実装済み |
| retention / compaction / rotation | 未実装 |
| multi-role authorization | 未実装 |
| distributed locking | 未実装 |

---

## 2. Persistent quarantine states

```text
pending
promoted
dismissed
permanently-rejected
```

Transientな再検証結果`rejected`はpersistent stateへ自動変換しません。

---

## 3. Runtime state

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-annotations.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
admin-auth-audit.jsonl
.quarantine-operation.lock
```

すべてのlifecycle ledgerはappend-onlyです。corruptionとI/O errorを黙って無視しません。

---

## 4. Same-host concurrency coordination

Lock file：

```text
<LINGONBERRY_STATE_DIR>/.quarantine-operation.lock
```

固定条件：

- `create_new`によるstate-directory-wide exclusive lock
- 競合時は`LB_QUARANTINE_BUSY`でfail closed
- indefinite waitやimplicit retryを行わない
- normal scope exitでlockを削除
- 15分を超えたlockはstaleとして1回だけ回収可能
- read-only list / get / status / metrics / backup verify / dry-runはlock不要
- mutation、backup export、restore destination writeは同じlock boundaryを使用

対象操作：

```text
quarantine append
promotion resolution append
annotation append
dismissal
permanent rejection
admin auth failure audit
backup export
restore destination write
```

Promotion resolution、dismissal、permanent rejectionはlock保持中にterminal ledgerを再確認します。cooperating processが同じstate directoryを使用する限り、同一recordを複数のterminal stateへcommitしません。

Lock metadataは次だけです。

```text
operation
pid
acquiredAt
```

bearer token、payload、quarantine ID、operator、noteは保存しません。

関連文書：`docs/operations/QUARANTINE_CONCURRENCY.md`

関連Issue：#30

---

## 5. Backup / Export / Restore

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

Backup exportはsource state directoryのoperation lockを取得します。Restoreはbackup検証後、destination state directoryをlockしてからconflict checkとwriteを実行します。

Versioned manifestは`quarantine-backup-manifest.json`です。現在のintegrity digestは偶発的破損検出用の`fnv1a64:<hex>`であり、暗号学的真正性の保証ではありません。

関連文書：`docs/operations/QUARANTINE_BACKUP_RESTORE.md`

---

## 6. HTTP boundary

Public listenerはreadiness、capabilities、publish、object retrievalのみを公開します。`/metrics`と`/v1/quarantine*`は`404`です。

Admin listener：

```bash
export LINGONBERRY_ADMIN_TOKEN='<long-random-secret>'
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

Backup / restoreはHTTP endpointを持たず、local binaryだけで実行します。

---

## 7. Quarantine CLI

```bash
lingonberry-relay quarantine-list
lingonberry-relay quarantine-get <quarantine-id>
lingonberry-relay quarantine-promote <quarantine-id>
lingonberry-relay quarantine-promote-batch 100 --dry-run
lingonberry-relay quarantine-resolutions
lingonberry-relay quarantine-status
lingonberry-relay quarantine-metrics
lingonberry-relay quarantine-annotate <quarantine-id> <operator> <note>
lingonberry-relay quarantine-annotations [quarantine-id]
lingonberry-relay quarantine-dismiss <quarantine-id> <operator> <note>
lingonberry-relay quarantine-dismissals [quarantine-id]
lingonberry-relay quarantine-permanently-reject <quarantine-id> <operator> <note>
lingonberry-relay quarantine-permanent-rejections [quarantine-id]
```

---

## 8. Statusとmetrics

Persistent lifecycle gauges：

```text
lingonberry_quarantine_records{state="total"}
lingonberry_quarantine_records{state="pending"}
lingonberry_quarantine_records{state="promoted"}
lingonberry_quarantine_records{state="dismissed"}
lingonberry_quarantine_records{state="permanently_rejected"}
```

operator、note、quarantine IDはmetric labelに使用しません。

---

## 9. 主要ファイル

```text
docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md
docs/roadmap/QUARANTINE_LIFECYCLE_BACKLOG.md
docs/operations/QUARANTINE_CONCURRENCY.md
docs/operations/QUARANTINE_BACKUP_RESTORE.md
packages/core/src/quarantine_lock.rs
packages/core/src/quarantine.rs
packages/core/src/quarantine_annotations.rs
packages/core/src/quarantine_dismissals.rs
packages/core/src/quarantine_rejections.rs
packages/core/src/quarantine_backup.rs
packages/relay/src/admin_auth.rs
```

---

## 10. 再開時の確認

```bash
git switch main
git pull --ff-only
git status
cargo test --workspace
```

Lock確認：

```bash
cat "$LINGONBERRY_STATE_DIR/.quarantine-operation.lock"
```

Freshなlockをowner process確認なしに削除しないでください。

---

## 11. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. promotion前に現在のvalidatorとpolicyで再評価する
3. 元quarantine recordとappend-only eventを保持する
4. dry-runでは永続状態を変更しない
5. corruptionとI/O errorを黙って無視しない
6. terminal recordを通常のbatch promotion対象へ戻さない
7. transient `rejected`をpermanent rejectionとして自動保存しない
8. quarantine管理routeを公開listenerへ戻さない
9. bearer tokenや自由文noteをauth auditへ記録しない
10. cooperating mutationはstate-directory-wide lockを取得する
11. terminal stateはlock保持中に再確認する
12. backup exportはmutationと同じsource lockを取得する
13. restoreはdestination lock取得後にwriteする
14. lock metadataへ秘密情報や自由文を保存しない
15. same-host lockをdistributed lockとして扱わない

---

## 12. 次の推奨作業

### 第一候補

```text
QL-5: JSONL Index / Rotation / Compaction
```

append-only監査証跡、backup manifest、lock boundaryを維持しながら長期運用コストを管理します。

### 第二候補

```text
Multi-role authorization
```

単一admin roleをreviewer、operator、observerなどへ分離します。

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
→ #30 concurrent ledger coordination
```
