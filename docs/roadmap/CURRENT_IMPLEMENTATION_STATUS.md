# 現在の実装状況

**Status: active** | **Last updated: 2026-07-12**

この文書は、Lingonberry の実装作業を中断・再開するときの引き継ぎ用正本です。

---

## 1. 現在地

2026-07-12 時点で、persistent quarantine lifecycle、same-host concurrency、active-ledger index、archive-aware rotation、archive-inclusive backup、non-destructive compaction proof、role-scoped admin HTTP authorizationまで実装済みです。

| 項目 | 状態 |
|---|---|
| persistent quarantine lifecycle | 実装済み |
| status / metrics / scheduler | 実装済み |
| public/admin listener isolation | 実装済み |
| admin Bearer authentication | 実装済み |
| observer / reviewer / operator RBAC | 実装済み |
| authentication / authorization audit | 実装済み |
| same-host concurrent ledger coordination | 実装済み |
| verified active-ledger index | 実装済み |
| archive-aware ordered readers | 実装済み |
| byte-preserving verified rotation | 実装済み |
| archive-inclusive backup / verify / restore | 実装済み |
| non-destructive compaction preview / proof | 実装済み |
| legacy admin token removal | 未実装 |
| record-rewriting compaction | 未実装・未承認 |
| retention deletion | 未実装・未承認 |
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

## 3. Archive-aware lifecycle

Logical read order：

```text
segment manifestのsequence順
→ active ledger
```

Missing、tampered、duplicate、out-of-order、unlisted segmentはcorruptionです。

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

## 4. Compaction policy v1

```text
lingonberry-quarantine-compaction-policy/v1
mutationAllowed: false
rewritePerformed: false
removableLines: 0
```

Immutable evidence：

```text
quarantine.jsonl
quarantine-annotations.jsonl
admin-auth-audit.jsonl
```

Terminal single-event evidence：

```text
quarantine-resolutions.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
```

Terminal ledgerのduplicate quarantine IDはcompaction candidateではなくcorruptionです。

Preview：

```bash
lingonberry-quarantine-maintenance compaction-preview \
  <verified-backup-v2-dir> \
  <empty-proof-output-dir>

lingonberry-quarantine-maintenance verify-compaction-proof \
  <proof-output-dir>
```

---

## 5. HTTP boundary

### Public listener

```bash
lingonberry-relay serve-http 127.0.0.1:8787
```

Public listenerはreadiness、capabilities、publish、object retrievalだけを公開します。Quarantine管理pathは`404`です。

### Admin listener

```bash
lingonberry-relay serve-admin-http 127.0.0.1:8788
```

Role credential：

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

Legacy compatibility：

```text
LINGONBERRY_ADMIN_TOKEN
```

明示的operator tokenがない場合のみoperator fallbackとして使用し、起動時にwarningを出します。

---

## 6. Admin RBAC

### Observer

Read-only：

```text
metrics
quarantine status
quarantine records
resolutions
annotations
permanent-rejection state
```

### Reviewer

Observer権限に加えて：

```text
append-only annotation creation
```

### Operator

Reviewer権限に加えて：

```text
single promotion
batch promotion
permanent rejection
```

Authorization order：

```text
non-admin path check
→ bearer role resolution
→ invalid / missing: 401
→ method + route permission check
→ insufficient role: 403
→ request body read / parse
→ route execution
```

Unauthorized mutation bodyは認可前に読み取らず、validationやmutation handlerへ渡しません。

Audit：

```text
LB_ADMIN_AUTH_FAILED  role=null
LB_ADMIN_FORBIDDEN    role=observer|reviewer|operator
```

Token、body、note、quarantine payloadはauditへ保存しません。

関連Issue：#40、#41、#43

関連文書：`docs/operations/QUARANTINE_ADMIN_HTTP.md`

---

## 7. Same-host operation lock

Mutation、backup export、restore destination write、index build、rotation、admin audit appendを直列化します。Distributed lockやnetwork filesystem leaseではありません。

---

## 8. 主要ファイル

```text
packages/relay/src/admin_auth.rs
packages/relay/src/main_entry.rs
packages/core/src/quarantine_compaction.rs
packages/core/src/quarantine_complete_backup.rs
packages/core/src/quarantine_segments.rs
packages/core/src/quarantine_ledger_index.rs
docs/operations/QUARANTINE_ADMIN_HTTP.md
docs/operations/QUARANTINE_COMPACTION_PROOF.md
docs/operations/QUARANTINE_BACKUP_RESTORE.md
docs/operations/QUARANTINE_JSONL_MAINTENANCE.md
```

---

## 9. 再開時の確認

```bash
git switch main
git pull --ff-only
git status
cargo test --workspace

export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
export LINGONBERRY_ADMIN_OBSERVER_TOKEN=<observer-secret>
export LINGONBERRY_ADMIN_REVIEWER_TOKEN=<reviewer-secret>
export LINGONBERRY_ADMIN_OPERATOR_TOKEN=<operator-secret>

lingonberry-relay serve-admin-http 127.0.0.1:8788
```

RBAC smoke test：

```bash
curl -i -H "Authorization: Bearer $LINGONBERRY_ADMIN_OBSERVER_TOKEN" \
  http://127.0.0.1:8788/v1/quarantine-status

curl -i -X POST \
  -H "Authorization: Bearer $LINGONBERRY_ADMIN_OBSERVER_TOKEN" \
  http://127.0.0.1:8788/v1/quarantine/lb:q:1/promote
```

後者は`403 Forbidden`でなければなりません。

---

## 10. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only lifecycle eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. terminal lifecycle競合をoperation lock内で再確認する
5. same-host lockをdistributed lockとして扱わない
6. stale indexでrotationしない
7. archive segmentを上書き・変更・削除しない
8. verified backup v2なしでcompaction previewを作らない
9. policy v1 proofでmutationを許可しない
10. public listenerへadmin routeを戻さない
11. missing tokenとinvalid tokenで異なる情報を返さない
12. authenticated roleの権限不足はmutation handler実行前に拒否する
13. bearer tokenをauditまたはresponseへ含めない
14. explicit replacement policyなしでrecord rewriteを実装しない
15. retention deletionをrewriteと同時に暗黙実行しない

---

## 11. 次の推奨作業

### 第一候補

```text
RBAC-1C: Legacy Admin Token Deprecation and Removal Plan
```

`LINGONBERRY_ADMIN_TOKEN`利用状況を検出可能にし、移行期間、warning、削除versionを固定します。即時削除はしません。

### 将来候補

```text
QL-5C3: Verified Rewrite Transaction
```

具体的なreplacement policyとsemantic equivalenceが承認されるまで開始しません。

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
→ #29 active-ledger backup / restore
→ #31 concurrent ledger coordination
→ #33 verified JSONL index and planning
→ #35 archive-aware verified rotation
→ #37 archive-inclusive backup / restore
→ #39 non-destructive compaction preview / proof
→ #42 RBAC credential model and permission matrix
→ #43 role-scoped HTTP authorization enforcement
```
