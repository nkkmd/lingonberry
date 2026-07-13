# 現在の実装状況

**Status: v0.2.0 released / v0.3.0 QL-5C3C design in progress** | **Last updated: 2026-07-14**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.2.0は2026-07-12にリリース済みです。v0.3.0ではreplacement policyと非破壊policy-v2 preview／proofが完了し、verified rewrite transactionとrecoveryの設計・実装へ進みます。

```text
released version: 0.2.0
release tag: v0.2.0
release checklist: docs/roadmap/RELEASE_0_2_0_CHECKLIST.md
release notes: docs/roadmap/RELEASE_0_2_0_RELEASE_NOTE.md
next roadmap: docs/roadmap/RELEASE_0_3_0_ROADMAP.md
completed issue: #52
active issue: #54
active branch: agent/ql-5c3c-rewrite-transaction
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
| replacement policy／semantic-equivalence contract | 完了（#50 / PR #51） |
| policy v2 replacement preview／proof | 完了（#52 / PR #53） |
| verified rewrite transaction／recovery | 設計中（#54） |
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
lingonberry-quarantine-maintenance replacement-preview \
  <verified-backup-v2-dir> <empty-proof-dir>
lingonberry-quarantine-maintenance verify-replacement-proof <proof-dir>
```

Policy v1：

```json
{
  "mutationAllowed": false,
  "rewritePerformed": false,
  "removableLines": 0
}
```

Policy v2 preview／proof：

```text
policy: lingonberry-quarantine-compaction-policy/v2
plan: lingonberry-quarantine-replacement-plan/v1
proof: lingonberry-quarantine-replacement-proof/v1
allowed transformation: canonical-json-representation
mutation in QL-5C3B: forbidden
```

QL-5C3B implements deterministic plans, separate plan/proof digests, archive-to-active provenance, immutable-ledger byte retention, duplicate-key rejection, semantic-equivalence verification, tamper detection, runtime-fingerprint verification, and atomic proof publication into an empty output directory.

正本：

```text
docs/operations/QUARANTINE_REPLACEMENT_POLICY.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md
```

## 6. Backup and restore

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

New exportは`lingonberry-quarantine-backup/v2`です。Verify／restoreはv1とv2を受理します。Policy-v2 previewとQL-5C3C applyはverified backup v2だけを受理します。

## 7. v0.3.0 current target

QL-5C3Cは、QL-5C3Bで検証済みのplanだけをtransactionally applyし、interruption後にresumeまたはrollbackできる機構を扱います。

必須境界：

1. existing ledgerを直接上書きしない
2. staging領域へ完全なledger generationを生成する
3. verified backup v2を適用前ゲートとjournal-bound rollback sourceにする
4. QL-5C3B verifierを適用前ゲートとして再利用する
5. same-host operation lock内でruntime fingerprintを再検証する
6. fsyncとatomic renameをdurable boundaryで使用する
7. transaction journalですべての状態遷移を判定可能にする
8. crash後のresume／rollbackをidempotentにする
9. immutable evidence ledgerとarchive segmentsを変更しない
10. retention deletion、deduplication、event collapse、schema migrationを行わない
11. policy v1互換性を維持する
12. 失敗時はfail closedとする

Transaction states：

```text
prepared
writing
staged
verified
publishing
committed
rolled-back
recovery-required
```

設計正本：`docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md`

## 8. v0.3.0非対象

- automatic retention deletion
- deduplication／event collapse
- schema migration／conflict resolution
- archive segment rewrite／deletion
- distributed locking／multi-node shared state
- remote backup／archive storage
- backup encryption／cryptographic signing
- OAuth／OIDC
- browser session／per-record ACL
- legacy admin token fallbackの完全削除
- policyで明示されていないrecord rewrite

## 9. 開発時のrelease gate

各PRで必須：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

JavaScript canonicalization／identity／validation testsも必須です。

QL-5C3Cでは追加で、journal transition、fsync／rename failure injection、crash-point resume／rollback、mixed-generation rejection、post-commit semantic-equivalenceを検証します。

## 10. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only lifecycle eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. terminal競合をsame-host operation lock内で再確認する
5. same-host lockをdistributed lockとして扱わない
6. stale indexでrotationまたはrewriteしない
7. archive segmentを上書き・変更・削除しない
8. verified backup v2なしでpreviewまたはrewriteを開始しない
9. policy v1 proofでmutationを許可しない
10. explicit replacement policyなしでrecord rewriteを実装しない
11. retention deletionをrewrite transactionへ混在させない
12. rewrite後のsemantic equivalenceを機械検証できないpolicyを承認しない
13. interrupted transactionを正常完了として扱わない
14. QL-5C3B previewからruntime stateを変更しない
15. generated timestampをdeterministic plan digestへ含めない
16. duplicate terminal keyをdeduplication opportunityとして扱わない
17. existing ledgerをin-place overwriteしない
18. staged outputを完全検証する前にpublishしない
19. 複数renameをcollectively atomicと仮定しない
20. runtime fingerprint変更時はfail closedで中止する
21. immutable evidence ledgerをtransactionで変更しない
22. ambiguous recovery stateを自動修復または成功扱いしない

## 11. 次の作業

Issue #54のQL-5C3Cを進めます。最初の実装単位は、versioned transaction journal、state-transition validator、pre-apply gate、staging-only writerです。active ledger publicationは、generation boundaryとmixed-generation rejectionをテストで固定するまで実装しません。