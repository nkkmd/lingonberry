# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-14**

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。v0.3.0の作業順序は [RELEASE_0_3_0_ROADMAP.md](./RELEASE_0_3_0_ROADMAP.md) を正本とします。

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

### Policy v1

```text
policy: lingonberry-quarantine-compaction-policy/v1
mutationAllowed: false
rewritePerformed: false
removableLines: 0
```

### Ledger classification

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

QL-5C3は、仕様確定前のrewrite実装を防ぐため、次の4段階へ分割します。

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

CI run #130 passed：Rust formatting、library／binary／test Clippy、workspace Rust tests、JavaScript tests。

正本：

```text
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md
```

この段階ではproduction ledgerを変更しません。

### QL-5C3C: Rewrite Transaction and Recovery

**状態: in progress (#54)**

作業ブランチ：`agent/ql-5c3c-rewrite-transaction`

設計正本：`docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md`

必須範囲：

- QL-5C3B verifierをpre-apply gateとして強制
- verified backup v2の事前検証とjournal binding
- same-host operation lock内でのruntime fingerprint再検証
- versioned transaction journalとstate-transition validation
- existing ledgerを直接上書きしないstaging-only generation
- immutable evidence ledgerのbyte identity維持
- staged semantic-equivalence verification
- fsyncとatomic renameを用いたdurable publication
- mixed-generationを正常状態として受理しないpublication model
- interrupted transactionのdeterministic classification
- idempotent resume／rollback
- post-commit index rebuild／verification
- policy-v1 regression維持

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

初期実装順：

1. journal schema／serializer／digest
2. transition validator
3. pre-apply gates
4. staging writer
5. staged verifier
6. publication generation model
7. resume／rollback
8. failure injection／crash-point tests

Active ledger publicationは、generation boundaryとmixed-generation rejectionをテストで固定するまで開始しません。

### QL-5C3D: Operations and Release Hardening

**状態: blocked by QL-5C3C**

- operator CLI hardening
- status／metrics／audit
- failure injection／crash-point test expansion
- operations runbook
- v0.3.0 release checklist and notes

### 全段階共通の非スコープ

- automatic retention deletion
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