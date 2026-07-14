# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-14**

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。v0.3.0の作業順序とrelease gateは [RELEASE_0_3_0_ROADMAP.md](./RELEASE_0_3_0_ROADMAP.md) と [RELEASE_0_3_0_CHECKLIST.md](./RELEASE_0_3_0_CHECKLIST.md) を正本とします。

## 完了済み

| 項目 | PR / Issue | 状態 |
|---|---:|---|
| persistent quarantine through permanent rejection | #8–#27 | 完了 |
| active-ledger verified backup / restore | #28 / #29 | 完了 |
| same-host concurrent ledger coordination | #30 / #31 | 完了 |
| verified read-only JSONL index and planning | #32 / #33 | 完了 |
| archive-aware ordered reads and verified rotation | #34 / #35 | 完了 |
| archive-inclusive backup / verify / restore | #36 / #37 | 完了 |
| non-destructive compaction preview and proof | #38 / #39 | 完了 |
| replacement policy and semantic-equivalence contract | #50 / #51 | 完了 |
| policy-v2 replacement preview and proof | #52 / #53 | 完了 |
| generation-based rewrite transaction and recovery | #54 / #55 | 完了 |

---

## QL-5A: Verified Read-only JSONL Index and Planning

**状態: completed**

---

## QL-5B: Archive-aware Ordered Reads and Verified Rotation

**状態: completed**

---

## QL-5C1: Archive-inclusive Backup / Verify / Restore

**状態: completed**

```text
export: backup/v2
verify / restore: v1 and v2
v2: active ledgers + segment manifest + listed immutable segments
```

---

## QL-5C2: Non-destructive Compaction Preview and Semantic Proof

**状態: completed**

Policy v1はmutationを許可しません。

```text
policy: lingonberry-quarantine-compaction-policy/v1
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

Terminal ledgerのduplicate quarantine IDはremoval candidateではなくcorruptionです。

正本：`docs/operations/QUARANTINE_COMPACTION_PROOF.md`

---

## QL-5C3: Verified Rewrite Transaction

**優先度: highest** | **Target: v0.3.0**

### QL-5C3A: Replacement Policy and Semantic-equivalence Contract

**状態: completed (#50 / PR #51)**

- ledger type別replacement semantics
- immutable evidenceとreplaceable representationの境界
- status／metrics／eligibility／idempotency equivalence
- source evidence mapping
- duplicate／conflict／corruption rules
- policy v2の入力・出力・拒否条件
- fixture／test vector

正本：`docs/operations/QUARANTINE_REPLACEMENT_POLICY.md`

### QL-5C3B: Policy v2 Preview and Proof

**状態: completed (#52 / PR #53)**

実装済み：

- policy v2専用の非破壊replacement preview
- deterministic replacement plan
- plan／proofの個別digest
- archive segmentからactive ledgerまでのone-to-one provenance
- immutable evidence ledgerのbyte retention
- terminal ledgerのcanonical JSON representation replacement判定
- duplicate terminal key拒否
- semantic-equivalence検証
- tamper検出
- runtime fingerprint前後検証
- empty output directoryへのatomic proof publication
- maintenance CLI：`replacement-preview` / `verify-replacement-proof`
- fixture-backed integration tests
- operator runbook
- policy-v1互換性維持

正本：

```text
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md
```

### QL-5C3C: Rewrite Transaction and Recovery

**状態: completed (#54 / PR #55)**

実装済み：

- QL-5C3B verifierのpre-apply gate強制
- verified backup v2、plan、proof、segment manifest、runtime fingerprintのjournal binding
- versioned transaction journalとdigest
- validated state transitions
- transaction-local complete ledger staging
- immutable evidence ledgerのbyte identity
- staged semantic-equivalence／membership／digest verification
- sealed generation manifestとgeneration digest
- generation-directory active-ledger resolver
- pointerなしの場合のlegacy root互換
- invalid pointer／invalid generation時のfail-closed rejection
- publication intentとprevious-pointer binding
- complete generation directoryのmaterializationとfsync
- current-generation pointerの1回のatomic rename
- mixed generationをhealthyとして受理しないresolver
- pointer switch前後のdeterministic recovery classification
- idempotent apply／resume／rollback
- atomic switch後／commit前のresume
- commit前のprevious-generation rollback
- post-publication index rebuild／verification
- archive segment verification
- maintenance CLI：`replacement-apply` / `replacement-status` / `replacement-recover`
- generation contractとoperator recovery runbook
- policy-v1互換性維持

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

Reader-visible layout：

```text
quarantine-current-generation.json
quarantine-generations/<transaction-id>/
```

`committed`と`rolled-back`はterminalです。Committed generationのrollbackは行わず、新しいverified transactionでsupersedeします。

正本：

```text
docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md
docs/operations/QUARANTINE_REPLACEMENT_GENERATION.md
docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md
```

### QL-5C3D: Operations and Release Hardening

**状態: in progress (#56 / Draft PR #60)**

実装済み：

- versioned structured status `lingonberry-quarantine-replacement-status/v1`
- bounded-cardinality Prometheus metrics
- `replacement-metrics <transaction-dir>` CLI
- secret-free append-only audit JSONL
- apply／status／resume／rollback audit integration
- explicit double-opt-in、one-shot failure injection
- failure points：pointer rename、index rebuild、commit transition、rollback pointer restoration、rolled-back transition
- pre-switch／post-switch／commit／rollback crash recovery tests
- read-only retention report `lingonberry-quarantine-replacement-retention-report/v1`
- active committed／previous committed／rolled-back／incomplete／orphan／legacy／corrupt classification
- `replacement-inspect-generations [transaction-dir ...]` CLI
- backup v2 → preview／proof → apply → status／metrics → index／segments → retention operator smoke test
- v0.3.0 release checklistとrelease notes
- workspace package version 0.3.0とCargo.lock更新

残作業：

- journal write／fsync failure injection
- staged ledger write／fsync、staging-directory fsync failure injection
- generation manifest／materialization failure injection
- publication-intent、pointer temporary-write、state-directory fsync failure injection
- index verification／segment verification failure injection
- machine-readableまたはtable-driven crash-point inventory
- PR本文／正本文書／CLI helpの最終整合性確認
- Draft解除、merge、main CI確認、release commit／tag／GitHub Release

Generation cleanupでautomatic deletionを導入する場合は、既存のretention非スコープとは別にpolicy／recovery evidence要件を承認する必要があります。

### 全段階共通の非スコープ

- automatic retention deletion
- automatic generation／workspace deletion
- deduplication／event collapse
- schema migration／conflict resolution
- archive-segment rewrite／deletion
- immutable evidence mutation
- distributed locking
- remote archive／backup storage
- cryptographic signing

---

## 再開時のIssue作成テンプレート

```markdown
## Goal
## Persistent state changes
## CLI / HTTP changes
## Lifecycle semantics
## Idempotency and concurrency
## Error handling
## Tests
## Documentation updates
## Non-goals
```

各quarantine関連PRでは、`CURRENT_IMPLEMENTATION_STATUS.md`を更新するか、更新不要の理由をPR本文へ記載します。
