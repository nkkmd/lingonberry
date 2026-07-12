# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine、promotion、status / metrics、scheduler、annotations、manual dismissal、admin HTTP isolation、permanent rejection、verified backup / restoreまで実装済みです。

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
| concurrent ledger coordination | 未実装 |
| retention / compaction / rotation | 未実装 |
| multi-role authorization | 未実装 |

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
```

すべてのlifecycle ledgerはappend-onlyです。corruptionとI/O errorを黙って無視しません。

---

## 4. Backup / Export / Restore

専用local administrative binary：

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

固定条件：

- versioned manifestは`quarantine-backup-manifest.json`
- manifestはmanaged fileのcopy完了後に最後に発行
- source fileをcopy後に再読込し、byte lengthとdigestが変化していないことを確認
- source変更時はexport失敗
- exact managed-file setを検証
- sparse stateでは不在ファイルをmanifestに明示
- restore前にbackup全体を検証
- destinationの既存managed fileは上書きしない
- restoreはtemporary file + atomic rename
- bearer tokenや環境変数はbackupしない

現在のintegrity digestは`fnv1a64:<hex>`です。これは偶発的な破損検出用であり、暗号学的真正性の保証ではありません。

関連文書：`docs/operations/QUARANTINE_BACKUP_RESTORE.md`

関連Issue：#28

---

## 5. HTTP boundary

Public listenerはreadiness、capabilities、publish、object retrievalのみを公開します。`/metrics`と`/v1/quarantine*`は`404`です。

Admin listener：

```bash
export LINGONBERRY_ADMIN_TOKEN='<long-random-secret>'
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

Backup / restoreはHTTP endpointを持たず、local binaryだけで実行します。

---

## 6. Quarantine CLI

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

## 7. Statusとmetrics

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

## 8. 主要ファイル

```text
docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md
docs/roadmap/QUARANTINE_LIFECYCLE_BACKLOG.md
docs/operations/QUARANTINE_BACKUP_RESTORE.md
packages/core/src/quarantine_backup.rs
packages/core/src/quarantine.rs
packages/core/src/quarantine_status.rs
packages/relay/src/quarantine_backup_main.rs
packages/relay/src/main_entry.rs
```

---

## 9. 再開時の確認

```bash
git switch main
git pull --ff-only
git status
cargo test --workspace
```

Backup smoke test：

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-quarantine-backup export /tmp/lingonberry-quarantine-backup
lingonberry-quarantine-backup verify /tmp/lingonberry-quarantine-backup
```

---

## 10. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. promotion前に現在のvalidatorとpolicyで再評価する
3. 元quarantine recordとappend-only eventを保持する
4. dry-runでは永続状態を変更しない
5. corruptionとI/O errorを黙って無視しない
6. terminal recordを通常のbatch promotion対象へ戻さない
7. transient `rejected`をpermanent rejectionとして自動保存しない
8. quarantine管理routeを公開listenerへ戻さない
9. bearer tokenや自由文noteをauth auditへ記録しない
10. backup manifestをmanaged fileより先に発行しない
11. verification失敗backupをrestoreしない
12. restore時に既存managed fileを上書きしない
13. backup export中のmutationを完全に防げるとは仮定しない

---

## 11. 次の推奨作業

### 第一候補

```text
QL-6: Concurrent Ledger Operations
```

promotion、dismissal、permanent rejection、annotation、auth audit、backup exportの競合条件を固定します。

### 第二候補

```text
QL-5: JSONL Index / Rotation / Compaction
```

append-only監査証跡を維持しながら長期運用コストを管理します。

---

## 12. 実装順序

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
→ #28 verified backup / restore
```
