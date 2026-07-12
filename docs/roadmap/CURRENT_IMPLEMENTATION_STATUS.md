# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine、promotion、batch revalidation、status、metrics、scheduler、operator annotations、manual dismissal、admin HTTP isolation、permanent rejection lifecycleまで実装済みです。

| 項目 | 状態 |
|---|---|
| persistent quarantine store | 実装済み |
| single / batch promotion | 実装済み |
| status / metrics | 実装済み |
| scheduler integration | 実装済み |
| append-only annotations | 実装済み |
| append-only manual dismissal | 実装済み |
| public/admin listener separation | 実装済み |
| admin Bearer authentication / failure audit | 実装済み |
| append-only permanent rejection | 実装済み |
| backup / restore boundary | 未実装 |
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

- `pending`: terminal lifecycle eventがないrecord
- `promoted`: promotion resolutionがあるrecord
- `dismissed`: operator dismissal eventがあるrecord
- `permanently-rejected`: operator permanent rejection eventがあるrecord

Transient revalidation outcomes:

```text
accept
still-deferred
rejected
```

Transientな`rejected`は自動的にpersistent stateへ変換しません。

---

## 3. Permanent rejectionの固定仕様

```text
対象: pending recordのみ
作成主体: operator
理由: LB_OPERATOR_PERMANENTLY_REJECTED + required note
重複: 1 record 1 active eventとしてidempotent
undo / reopen: 非スコープ
入口: Core + CLI + authenticated admin HTTP
```

永続event：

```json
{
  "id": "lb:qr:...",
  "quarantineId": "lb:q:...",
  "rejectedAt": "...Z",
  "operator": "operator-name",
  "reasonCode": "LB_OPERATOR_PERMANENTLY_REJECTED",
  "note": "known prohibited content"
}
```

固定条件：

- unknown、promoted、dismissed recordを拒否
- duplicate requestは既存eventを返す
- duplicate ledger eventはcorruption
- 元quarantine recordとannotationを保持
- default quarantine listとbatch promotionから除外
- direct CLI / admin HTTP promotionを拒否
- transient batch `rejected` counterとは別概念

関連文書：`docs/operations/QUARANTINE_PERMANENT_REJECTIONS.md`

関連Issue：#26

---

## 4. Runtime state

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

## 5. HTTP boundary

Public listener:

```bash
lingonberry-relay serve-http 127.0.0.1:8787
```

公開routeはreadiness、capabilities、publish、object retrievalのみです。`/metrics`と`/v1/quarantine*`は`404`になります。

Admin listener:

```bash
export LINGONBERRY_ADMIN_TOKEN='<long-random-secret>'
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

Bearer認証後にquarantine管理routeを利用できます。

Permanent rejection routes:

```text
POST /v1/quarantine/<quarantine-id>/permanent-rejection
GET  /v1/quarantine/<quarantine-id>/permanent-rejection
```

---

## 6. CLI

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

Status fields：

```text
total
pending
promoted
dismissed
permanentlyRejected
oldestPendingAt
latestReceivedAt
latestPromotedAt
latestDismissedAt
latestPermanentlyRejectedAt
reasonCodeCounts
```

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
docs/operations/QUARANTINE_ADMIN_HTTP.md
docs/operations/QUARANTINE_DISMISSALS.md
docs/operations/QUARANTINE_PERMANENT_REJECTIONS.md
packages/core/src/quarantine.rs
packages/core/src/quarantine_dismissals.rs
packages/core/src/quarantine_rejections.rs
packages/core/src/quarantine_status.rs
packages/relay/src/admin_auth.rs
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

Read-only確認：

```bash
cargo run -p lingonberry-relay -- quarantine-status
cargo run -p lingonberry-relay -- quarantine-metrics
cargo run -p lingonberry-relay -- quarantine-permanent-rejections
cargo run -p lingonberry-relay -- quarantine-promote-batch 100 --dry-run
```

---

## 10. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. `defer`されたobjectはquarantineにのみ保存する
3. promotion前に現在のvalidatorとpolicyで再評価する
4. promotionの再実行はidempotentに扱う
5. 元quarantine recordを監査証跡として保持する
6. dry-runでは永続状態を変更しない
7. corruptionとI/O errorを黙って無視しない
8. annotationをlifecycle stateとして解釈しない
9. dismissalとpermanent rejectionは専用append-only eventで表現する
10. terminal recordを通常のbatch promotion対象へ戻さない
11. transient `rejected`を自動的にpermanent rejectionとして保存しない
12. quarantine管理routeを公開listenerへ戻さない
13. bearer tokenや自由文noteをauth auditへ記録しない

---

## 11. 次の推奨作業

### 第一候補

```text
QL-4: Backup / Export / Restore
```

全quarantine ledgerの一貫したsnapshot、manifest、restore検証を定義します。

### 第二候補

```text
QL-6: Concurrent Ledger Operations
```

promotion、dismissal、permanent rejection、annotation、audit appendの競合を扱います。

### 第三候補

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
→ #26 permanent rejection
```
