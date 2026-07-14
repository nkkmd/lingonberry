# 現在の実装状況

**Status: v0.2.0 released / v0.3.0 QL-5C3D complete** | **Last updated: 2026-07-14**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.2.0は2026-07-12にリリース済みです。v0.3.0ではreplacement policy、policy-v2 preview／proof、generation-directory方式のverified rewrite transaction／recovery、operations／observability／failure injection／release hardeningを実装しました。

```text
released version: 0.2.0
release tag: v0.2.0
release candidate workspace version: 0.3.0
v0.3.0 issue: #56
v0.3.0 PR: #60
release checklist: docs/roadmap/RELEASE_0_3_0_CHECKLIST.md
release notes: docs/roadmap/RELEASE_0_3_0_RELEASE_NOTE.md
remaining target: merge, main CI, release commit/tag/GitHub Release
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
| operations／observability／release hardening | 完了（#56 / PR #60） |
| generation-directory active-ledger resolution | 実装済み |
| atomic current-generation pointer publication | 実装済み |
| resumable／rollback-capable transaction | 実装済み |
| versioned replacement status v1 | 実装済み |
| bounded replacement Prometheus metrics | 実装済み |
| secret-free append-only replacement audit | 実装済み |
| deterministic failure injection（18 points） | 実装済み |
| machine-readable crash-point inventory | 実装済み |
| read-only generation retention inspection | 実装済み |
| end-to-end operator smoke test | 実装済み |
| workspace package version 0.3.0 | 更新済み |
| public／admin listener isolation | 実装済み |
| observer／reviewer／operator RBAC | 実装済み |
| authn／authz audit | 実装済み |

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

## 4. Replacement transaction model

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

1. QL-5C3B verifierとverified backup v2をpre-apply gateとして強制
2. plan／proof／segment manifest／runtime fingerprintをjournalへbind
3. existing ledgerを直接上書きしないstaging-only writer
4. immutable evidence ledgerのbyte identity維持
5. staged ledger set／semantic equivalence／membership／digestの再検証
6. sealed generation manifestとgeneration digest
7. transaction-local publication intent
8. complete generation directoryのmaterializationとfsync
9. current-generation pointerの1回のatomic rename
10. pointer switch前後を区別するdeterministic status classification
11. crash後のidempotent resume
12. commit前のexact previous-pointer rollback
13. post-switch index rebuild／verificationとsegment verification
14. committed／rolled-back terminal state
15. legacy root layout互換とmixed generationのfail-closed rejection
16. versioned status、bounded metrics、secret-free append-only audit
17. read-only generation retention classification
18. double opt-in／one-shot deterministic failure injection

`committed`後のtransactionはterminalです。以前のgenerationへ戻す場合もpointerを手動編集せず、新しいverified transactionとして実行します。

## 5. QL-5C3D completion

実装済み：

- `lingonberry-quarantine-replacement-status/v1`
- bounded-cardinality Prometheus metrics
- append-only audit JSONLとfsync
- apply／status／resume／rollback audit integration
- `lingonberry-quarantine-replacement-retention-report/v1`
- active／previous／rolled-back／incomplete／orphan／legacy／corrupt分類
- backup → preview/proof → apply → observe → verify operator smoke test
- versioned crash-point registryとmachine-readable inventory
- registry／inventory consistency CI contract
- 全18 failure pointsのdirect seamまたはexplicit post-boundary alias
- pre-switch／post-switch／commit／rollback／early durable boundary recovery tests
- v0.3.0 workspace version、release checklist、release notes

全18 failure points：

```text
journal.write
journal.fsync
staging.ledger-write
staging.ledger-fsync
staging.directory-fsync
generation.manifest-write
generation.manifest-fsync
publication.intent-write
publication.generation-materialize-rename
publication.pointer-temporary-write
publication.pointer-rename
publication.state-directory-fsync
publication.index-rebuild
publication.index-verification
publication.segment-verification
publication.commit-transition
rollback.pointer-restore
rollback.rolled-back-transition
```

残作業はPR #60のmerge、main CI確認、release commit確定、`v0.3.0` tag／GitHub Releaseです。

## 6. Operator CLI

```bash
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

`replacement-inspect-generations`はread-onlyです。delete、rename、truncate、repair、pointer mutationを行いません。

## 7. 正本文書

```text
docs/operations/QUARANTINE_REPLACEMENT_POLICY.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md
docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md
docs/operations/QUARANTINE_REPLACEMENT_GENERATION.md
docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md
docs/operations/QUARANTINE_REPLACEMENT_OPERATIONS_HARDENING.md
docs/operations/quarantine-replacement-crash-points.v1.json
docs/roadmap/RELEASE_0_3_0_CHECKLIST.md
docs/roadmap/RELEASE_0_3_0_RELEASE_NOTE.md
```

## 8. v0.3.0非対象

- automatic retention deletion
- automatic generation／workspace deletion
- deduplication／event collapse
- schema migration／conflict resolution
- archive segment rewrite／deletion
- distributed locking／multi-node shared state
- remote backup／archive storage
- backup encryption／cryptographic signing
- OAuth／OIDC
- browser session／per-record ACL
- policyで明示されていないrecord rewrite

## 9. Release gate

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables
cargo test --workspace
```

JavaScript canonicalization／identity／validation／crash-point contract testsも必須です。

## 10. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. 元quarantine recordとappend-only lifecycle eventを保持する
3. corruptionとI/O errorを黙って無視しない
4. same-host lockをdistributed lockとして扱わない
5. stale indexでrotationまたはrewriteしない
6. archive segmentを上書き・変更・削除しない
7. verified backup v2なしでpreviewまたはrewriteを開始しない
8. policy v1 proofでmutationを許可しない
9. retention deletionをrewrite transactionへ混在させない
10. existing ledgerをin-place overwriteしない
11. staged outputを完全検証する前にpublishしない
12. 複数renameをcollectively atomicと仮定しない
13. runtime fingerprint変更時はfail closedで中止する
14. immutable evidence ledgerをtransactionで変更しない
15. ambiguous recovery stateを自動修復または成功扱いしない
16. pointerが存在する状態でlegacy rootへfallbackしない
17. committed generationのpointerを手動で巻き戻さない
18. generation／workspaceを自動削除しない
19. audit／metricsへsecret、path、transaction ID、free-form errorを出さない
