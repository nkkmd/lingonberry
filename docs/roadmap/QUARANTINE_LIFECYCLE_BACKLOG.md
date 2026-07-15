# Quarantine Lifecycle Backlog

**Status: v0.3.0 released** | **Last updated: 2026-07-15**

現在地の正本は [CURRENT_IMPLEMENTATION_STATUS.md](./CURRENT_IMPLEMENTATION_STATUS.md) です。v0.3.0の完了記録は [RELEASE_0_3_0_CHECKLIST.md](./RELEASE_0_3_0_CHECKLIST.md) と [RELEASE_0_3_0_RELEASE_NOTE.md](./RELEASE_0_3_0_RELEASE_NOTE.md) を参照してください。

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
| operations, observability, failure injection, release hardening | #56 / #60 | 完了・v0.3.0でrelease |

## QL-5C3: Verified Rewrite Transaction

**Priority: completed** | **Release: v0.3.0** | **Implementation: complete**

### QL-5C3A: Replacement Policy and Semantic-equivalence Contract

**状態: completed (#50 / PR #51)**

- ledger type別replacement semantics
- immutable evidenceとreplaceable representationの境界
- status／metrics／eligibility／idempotency equivalence
- source evidence mapping
- duplicate／conflict／corruption rules
- policy v2の入力・出力・拒否条件

正本：`docs/operations/QUARANTINE_REPLACEMENT_POLICY.md`

### QL-5C3B: Policy v2 Preview and Proof

**状態: completed (#52 / PR #53)**

- deterministic replacement plan
- plan／proofの個別digest
- archive segmentからactive ledgerまでのone-to-one provenance
- immutable evidence ledgerのbyte retention
- terminal ledgerのcanonical JSON representation replacement判定
- duplicate terminal key拒否
- semantic-equivalence検証
- tamper検出
- runtime fingerprint前後検証
- non-destructive maintenance CLIとoperator runbook

正本：

```text
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md
docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md
```

### QL-5C3C: Rewrite Transaction and Recovery

**状態: completed (#54 / PR #55)**

- QL-5C3B verifierとverified backup v2のpre-apply gate
- versioned transaction journalとbound digests
- complete stagingとsemantic／membership／digest verification
- immutable evidence ledgerのbyte identity
- sealed generation manifest
- generation-directory resolverとlegacy root互換
- publication intentとprevious-pointer binding
- complete generation materialization
- current-generation pointerの1回のatomic rename
- deterministic recovery classification
- idempotent apply／resume／rollback
- post-publication index／segment verification
- committed／rolled-back terminal states

正本：

```text
docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md
docs/operations/QUARANTINE_REPLACEMENT_GENERATION.md
docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md
```

### QL-5C3D: Operations and Release Hardening

**状態: completed and released (#56 / PR #60)**

実装・release済み：

- versioned structured status `lingonberry-quarantine-replacement-status/v1`
- bounded-cardinality Prometheus metrics
- secret-free append-only audit JSONL
- apply／status／resume／rollback audit integration
- recovery-required failure audit classification
- read-only retention report `lingonberry-quarantine-replacement-retention-report/v1`
- active／previous／rolled-back／incomplete／orphan／legacy／corrupt classification
- `replacement-metrics`／`replacement-inspect-generations` CLI
- end-to-end operator smoke test
- workspace package version 0.3.0とCargo.lock更新
- versioned failure-point registry
- machine-readable crash-point inventory
- registry／inventory consistency CI contract
- explicit double-opt-in、one-shot failure injection
- 全18 durable／publication／rollback failure points
- early write／fsync、pre-switch、post-switch、commit、rollback recovery tests

Failure points：

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

Release closure：

- [x] PR #60 Draft解除／squash merge
- [x] main branch CI成功
- [x] release commit `efb77415f76b4ba4340536b5b29f5754a1173d59`
- [x] tag `v0.3.0`
- [x] GitHub Release `Lingonberry v0.3.0`
- [x] post-release checklist update

Generation cleanupでautomatic deletionを導入する場合は、別Issueでpolicy／recovery evidence要件を承認する必要があります。

## 全段階共通の非スコープ

- automatic retention deletion
- automatic generation／workspace deletion
- deduplication／event collapse
- schema migration／conflict resolution
- archive-segment rewrite／deletion
- immutable evidence mutation
- distributed locking
- remote archive／backup storage
- cryptographic signing

## 次段階

v0.3.0リリース後の新規作業は、既存のrewrite transactionへ暗黙に混在させず、個別Issueとして安全境界・persistent state・recovery semanticsを定義します。
