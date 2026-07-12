# 現在の実装状況

**Status: v0.2.0 release candidate** | **Last updated: 2026-07-12**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.2.0の機能範囲は固定済みです。残っているrelease gateは、release preparation PRのCI成功、merge後の`main`再検証、annotated tag、GitHub Release公開です。

```text
version: 0.2.0
release tag: v0.2.0
release checklist: docs/roadmap/RELEASE_0_2_0_CHECKLIST.md
release notes: docs/roadmap/RELEASE_0_2_0_RELEASE_NOTE.md
```

## 2. 実装済み

| 項目 | 状態 |
|---|---|
| core protocol／schema／fixtures | 実装済み |
| HTTP publish carrier | 実装済み |
| storage node／archive export-import | 実装済み |
| persistent quarantine lifecycle | 実装済み |
| single／batch promotion and dry-run | 実装済み |
| annotations／dismissal／permanent rejection | 実装済み |
| status／metrics／scheduler | 実装済み |
| same-host operation lock | 実装済み |
| verified ledger index | 実装済み |
| archive-aware ordered readers | 実装済み |
| byte-preserving verified rotation | 実装済み |
| archive-inclusive backup v2 | 実装済み |
| backup verify／restore | 実装済み |
| non-destructive compaction preview／proof | 実装済み |
| public／admin listener isolation | 実装済み |
| observer／reviewer／operator RBAC | 実装済み |
| authn／authz audit | 実装済み |
| legacy token deprecation diagnostic | 実装済み |

## 3. Runtime state

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

Derived indexとoperation lockはbackup対象ではありません。Archive-inclusive backup v2はactive ledgers、segment manifest、manifestで列挙されたimmutable segmentsを含みます。

## 4. Quarantine lifecycle

Persistent terminal states：

```text
promoted
dismissed
permanently-rejected
```

`Rejected`と`Deferred`はrevalidation時の判定です。元quarantine recordとappend-only lifecycle eventは削除しません。Terminal ledgerのduplicate quarantine IDはcorruptionです。

## 5. Ledger maintenance

Logical read order：

```text
verified archive segments in manifest sequence
→ active ledger
```

Maintenance CLI：

```bash
lingonberry-quarantine-maintenance build-index
lingonberry-quarantine-maintenance verify-index
lingonberry-quarantine-maintenance rotate <managed-ledger-name>
lingonberry-quarantine-maintenance verify-segments
lingonberry-quarantine-maintenance compaction-preview \
  <verified-backup-v2-dir> <empty-proof-dir>
lingonberry-quarantine-maintenance verify-compaction-proof <proof-dir>
```

Compaction policy v1：

```json
{
  "mutationAllowed": false,
  "rewritePerformed": false,
  "removableLines": 0
}
```

## 6. Backup and restore

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

New exportは`lingonberry-quarantine-backup/v2`です。Verify／restoreはv1とv2を受理します。

## 7. Admin RBAC

Role credentials：

```text
LINGONBERRY_ADMIN_OBSERVER_TOKEN
LINGONBERRY_ADMIN_REVIEWER_TOKEN
LINGONBERRY_ADMIN_OPERATOR_TOKEN
```

Permission hierarchy：

```text
observer: read-only
reviewer: observer + annotation creation
operator: reviewer + promotion and permanent rejection
```

Authorization order：

```text
non-admin path → 404
missing／invalid credential → 401
authenticated but insufficient role → 403
authorized → read body → parse → execute
```

`LINGONBERRY_ADMIN_TOKEN`はdeprecated operator fallbackです。次でsecret-free診断を行います。

```bash
lingonberry-admin-auth-config
```

完全削除はfuture major releaseまで行いません。

## 8. v0.2.0非対象

- record-rewriting compaction
- retention deletion
- distributed locking／multi-node shared state
- remote backup upload
- backup encryption／signing
- OAuth／OIDC
- browser session／per-record ACL
- legacy admin token fallbackの完全削除

## 9. Release gate

Release PRで必須：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

JavaScript canonicalization／identity／validation testsも必須です。

Merge後：

```bash
git switch main
git pull --ff-only
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git tag -a v0.2.0 -m "Lingonberry v0.2.0"
git push origin v0.2.0
```

## 10. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only lifecycle eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. terminal競合をsame-host operation lock内で再確認する
5. same-host lockをdistributed lockとして扱わない
6. stale indexでrotationしない
7. archive segmentを上書き・変更・削除しない
8. verified backup v2なしでcompaction previewを作らない
9. policy v1 proofでmutationを許可しない
10. public listenerへadmin routeを公開しない
11. missing tokenとinvalid tokenで異なる情報を返さない
12. 権限不足をbody読込・mutation前に拒否する
13. token、body、note、payloadをaudit／diagnosticへ含めない
14. explicit replacement policyなしでrecord rewriteを実装しない
15. retention deletionを暗黙実行しない

## 11. 次の作業

v0.2.0 release後の第一候補は、release運用結果と利用実績を確認したうえでroadmapを再優先順位付けすることです。QL-5C3は具体的なreplacement policyとsemantic-equivalence contractが承認されるまで開始しません。
