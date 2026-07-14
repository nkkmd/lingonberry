# 現在の実装状況

**Status: v0.2.0 released / v0.3.0 QL-5C3D in progress** | **Last updated: 2026-07-14**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.2.0は2026-07-12にリリース済みです。v0.3.0ではreplacement policy、policy-v2 preview／proof、generation-directory方式のverified rewrite transaction／recoveryを完了し、現在はQL-5C3D operations、observability、failure injection、release hardeningを実装中です。

```text
released version: 0.2.0
release tag: v0.2.0
release candidate workspace version: 0.3.0
v0.3.0 issue: #56
v0.3.0 draft PR: #60
release checklist: docs/roadmap/RELEASE_0_3_0_CHECKLIST.md
release notes: docs/roadmap/RELEASE_0_3_0_RELEASE_NOTE.md
remaining target: early durable write/fsync failure injection and final release closure
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
| verified rewrite transaction／recovery | 完了（#54 / PR #55） |
| generation-directory active-ledger resolution | 実装済み |
| atomic current-generation pointer publication | 実装済み |
| resumable／rollback-capable transaction | 実装済み |
| versioned replacement status v1 | PR #60で実装済み |
| bounded replacement Prometheus metrics | PR #60で実装済み |
| secret-free append-only replacement audit | PR #60で実装済み |
| deterministic failure injection | PR #60で実装中 |
| read-only generation retention inspection | PR #60で実装済み |
| end-to-end operator smoke test | PR #60で実装済み |
| workspace package version 0.3.0 | PR #60で更新済み |
| public／admin listener isolation | 実装済み |
| observer／reviewer／operator RBAC | 実装済み |
| authn／authz audit | 実装済み |
| legacy token deprecation diagnostic | 実装済み |

## 3. Runtime state

Pointerがないlegacy state：

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

Generation publication後：

```text
quarantine-current-generation.json
quarantine-generations/
└── <transaction-id>/
    ├── quarantine-replacement-generation.json
    ├── quarantine-replacement-generation.digest
    └── managed ledger files
```

Pointerが存在しない場合はlegacy root ledgersを使用します。Pointerが存在する場合、すべてのmanaged active-ledger read／writeは参照先generation directoryへ解決されます。Pointer、generation manifest、digest、ledger membershipのいずれかが不正な場合はrootへfallbackせずfail closedです。

Derived indexとoperation lockはbackup対象ではありません。Archive-inclusive backup v2はactive ledgers、segment manifest、manifestで列挙されたimmutable segmentsを含みます。

## 4. Quarantine lifecycle

Persistent terminal states：

```text
promoted
dismissed
permanently-rejected
```

`Rejected`と`Deferred`はrevalidation時の判定です。元quarantine recordとappend-only lifecycle eventは削除しません。Terminal ledgerのduplicate quarantine IDはcorruptionです。

## 5. Ledger maintenance CLI

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

lingonberry-quarantine-maintenance replacement-apply \
  <verified-backup-v2-dir> <verified-proof-dir> <transaction-dir>
lingonberry-quarantine-maintenance replacement-status <transaction-dir>
lingonberry-quarantine-maintenance replacement-metrics <transaction-dir>
lingonberry-quarantine-maintenance replacement-inspect-generations \
  [transaction-dir ...]
lingonberry-quarantine-maintenance replacement-recover \
  <transaction-dir> --resume|--rollback
```

`replacement-apply`ではtransaction directoryのbasenameをtransaction ID／generation IDとして使用します。IDはcore journal validatorがbounded ASCII identifierとして検証します。

`replacement-inspect-generations`はread-onlyです。generation、pointer、journalを分類しますが、delete、rename、truncate、repair、pointer mutationを行いません。

## 6. Replacement transaction model

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

実装済み境界：

1. QL-5C3B verifierをpre-apply gateとして強制
2. verified backup v2とplan／proof／runtime fingerprintをjournalへbind
3. existing ledgerを直接上書きしないstaging-only writer
4. immutable evidence ledgerのbyte identity維持
5. staged ledger set、semantic equivalence、duplicate key、digestの再検証
6. sealed generation manifestとgeneration digest
7. transaction-local publication intent
8. complete generation directoryのmaterializationとfsync
9. current-generation pointerの1回のatomic rename
10. pointer switch前後を区別するdeterministic status classification
11. crash後のidempotent resume
12. commit前のexact previous-pointer rollback
13. post-switch index rebuild／verificationとsegment verification
14. committed／rolled-back terminal state
15. legacy root layout互換
16. invalid pointer／mixed generationのfail-closed rejection
17. versioned structured statusとbounded metrics
18. secret-free append-only operation audit
19. read-only generation retention classification

`committed`後のtransactionはterminalです。以前のgenerationへ戻す場合もpointerを手動編集せず、新しいverified transactionとして実行します。

## 7. QL-5C3D observability and hardening

実装済み：

- `lingonberry-quarantine-replacement-status/v1`
- bounded-cardinality Prometheus metrics
- append-only audit JSONLとfsync
- apply／status／resume／rollback audit integration
- double opt-in、one-shot failure injection
- pointer rename failure
- index rebuild failure
- commit transition failure
- rollback pointer restoration failure
- rolled-back transition failure
- pre-switch／post-switch／commit／rollback crash recovery tests
- `lingonberry-quarantine-replacement-retention-report/v1`
- active、previous、rolled-back、incomplete、orphan、legacy、corrupt分類
- backup → preview/proof → apply → observe → verify operator smoke test
- v0.3.0 release checklist／release notes

未完了：

- journal write／fsync failure injection
- staging write／fsync failure injection
- generation manifest／materialization failure injection
- publication-intent／pointer temporary-write／state-directory fsync failure injection
- index／segment verification failure injection
- machine-readable crash-point inventory
- PR merge後のmain CI、release commit確定、tag／GitHub Release

## 8. 正本文書

```text
docs/operations/QUARANTINE_REPLACEMENT_POLICY.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md
docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md
docs/operations/QUARANTINE_REPLACEMENT_GENERATION.md
docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md
docs/operations/QUARANTINE_REPLACEMENT_OPERATIONS_HARDENING.md
docs/roadmap/RELEASE_0_3_0_CHECKLIST.md
docs/roadmap/RELEASE_0_3_0_RELEASE_NOTE.md
```

## 9. Backup and restore

```bash
lingonberry-quarantine-backup export <empty-backup-dir>
lingonberry-quarantine-backup verify <backup-dir>
lingonberry-quarantine-backup restore <backup-dir> <empty-state-dir>
```

New exportは`lingonberry-quarantine-backup/v2`です。Verify／restoreはv1とv2を受理します。Policy-v2 previewとreplacement applyはverified backup v2だけを受理します。

## 10. v0.3.0非対象

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

## 11. Release gate

各PRで必須：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables
cargo test --workspace
```

JavaScript canonicalization／identity／validation testsも必須です。

PR #60のoperator smoke testは、backup v2 export／verify、replacement preview／proof verification、committed apply、status、bounded metrics、index／segment verification、retention classification、repeat apply／resume idempotencyを含みます。

## 12. 絶対に崩さない安全性ルール

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
12. duplicate terminal keyをdeduplication opportunityとして扱わない
13. existing ledgerをin-place overwriteしない
14. staged outputを完全検証する前にpublishしない
15. 複数renameをcollectively atomicと仮定しない
16. runtime fingerprint変更時はfail closedで中止する
17. immutable evidence ledgerをtransactionで変更しない
18. ambiguous recovery stateを自動修復または成功扱いしない
19. pointerが存在する状態でlegacy rootへfallbackしない
20. committed generationのpointerを手動で巻き戻さない
21. generation／workspaceを自動削除しない
22. audit／metricsへsecret、path、transaction ID、free-form errorを出さない
