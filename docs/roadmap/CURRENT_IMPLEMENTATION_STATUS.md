# 現在の実装状況

**Status: v0.2.0 released / v0.3.0 QL-5C3B in progress** | **Last updated: 2026-07-13**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.2.0は2026-07-12にリリース済みです。v0.3.0はreplacement policy確定を完了し、非破壊policy-v2 preview／proofの実装フェーズです。

```text
released version: 0.2.0
release tag: v0.2.0
release checklist: docs/roadmap/RELEASE_0_2_0_CHECKLIST.md
release notes: docs/roadmap/RELEASE_0_2_0_RELEASE_NOTE.md
next roadmap: docs/roadmap/RELEASE_0_3_0_ROADMAP.md
active issue: #52
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
| non-destructive compaction preview／proof policy v1 | 実装済み |
| replacement policy／semantic-equivalence contract | 確定済み |
| policy v2 replacement preview／proof | 実装中 |
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

Policy v2 implementation contract：

```text
policy: lingonberry-quarantine-compaction-policy/v2
proof: lingonberry-quarantine-replacement-proof/v1
allowed transformation: canonical-json-representation
mutation in QL-5C3B: forbidden
```

正本：

```text
docs/operations/QUARANTINE_REPLACEMENT_POLICY.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md
```

## 6. Backup and restore

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

New exportは`lingonberry-quarantine-backup/v2`です。Verify／restoreはv1とv2を受理します。Policy-v2 previewはverified backup v2だけを受理します。

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

## 8. v0.3.0の目標

v0.3.0は、verified rewrite transactionを安全に実装可能な状態へ進めるリリースです。QL-5C3Aでreplacement policyとsemantic-equivalence contractは確定しました。現在はQL-5C3Bとして、runtime stateを変更しないdeterministic replacement planとproofを実装します。

優先順：

1. policy-v2 plan／proof data modelを実装する
2. archive-aware logical scanとone-to-one provenanceを実装する
3. semantic-equivalence verificationを実装する
4. deterministic digestとtamper rejectionを実装する
5. policy-v1 verification regressionを維持する
6. QL-5C3B完了後にQL-5C3C transaction designへ進む

正本：`docs/roadmap/RELEASE_0_3_0_ROADMAP.md`

## 9. v0.3.0非対象

- automatic retention deletion
- distributed locking／multi-node shared state
- remote backup upload
- backup encryption／signing
- OAuth／OIDC
- browser session／per-record ACL
- legacy admin token fallbackの完全削除
- replacement policyで明示されていないrecord rewrite

## 10. 開発時のrelease gate

各PRで必須：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

JavaScript canonicalization／identity／validation testsも必須です。

v0.3.0 release checklistとrelease notesは、実装スコープ確定後に作成します。

## 11. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only lifecycle eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. terminal競合をsame-host operation lock内で再確認する
5. same-host lockをdistributed lockとして扱わない
6. stale indexでrotationしない
7. archive segmentを上書き・変更・削除しない
8. verified backup v2なしでcompaction previewまたはrewriteを開始しない
9. policy v1 proofでmutationを許可しない
10. public listenerへadmin routeを公開しない
11. missing tokenとinvalid tokenで異なる情報を返さない
12. 権限不足をbody読込・mutation前に拒否する
13. token、body、note、payloadをaudit／diagnosticへ含めない
14. explicit replacement policyなしでrecord rewriteを実装しない
15. retention deletionをrewrite transactionへ混在させない
16. rewrite後のsemantic equivalenceを機械検証できないpolicyを承認しない
17. interrupted transactionを正常完了として扱わない
18. QL-5C3B previewからruntime stateを変更しない
19. generated timestampをdeterministic plan digestへ含めない
20. duplicate terminal keyをdeduplication opportunityとして扱わない

## 12. 次の作業

Issue #52のQL-5C3Bを実装します。最初のコード単位はpolicy-v2 plan／proof data model、deterministic canonical plan serialization、fixture-driven verifierです。その後にarchive-aware runtime scanとCLIを接続します。QL-5C3Cのrewrite applicationはQL-5C3Bのproofが完成するまで開始しません。
