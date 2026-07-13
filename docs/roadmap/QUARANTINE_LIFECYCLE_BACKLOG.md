# Quarantine Lifecycle Backlog

**Status: active** | **Last updated: 2026-07-13**

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

### 実装済み完了条件

- verified archive-inclusive backup v2を必須化
- archive segment verificationを事前実行
- 全managed ledgerのordered logical streamをscan
- line count、byte count、ordered digest、unique key countを記録
- promoted、dismissed、permanently rejectedのcountを記録
- backup manifestとsegment manifestのdigestをproofへ記録
- runtime fingerprintをpreview前後で比較
- output directory以外を変更しない
- versioned proofとseparate digestをatomic publish
- proof tamperingとunsupported policyを拒否
- policy v1では全line保持、removable lineは常に0

関連文書：`docs/operations/QUARANTINE_COMPACTION_PROOF.md`

---

## QL-5C3: Verified Rewrite Transaction

**優先度: highest** | **Target: v0.3.0**

QL-5C3は、仕様確定前のrewrite実装を防ぐため、次の4段階へ分割します。

### QL-5C3A: Replacement Policy and Semantic-equivalence Contract

**状態: completed (#50 / #51)**

- ledger type別replacement semantics
- immutable evidenceとreplaceable representationの境界
- status／metrics／eligibility／idempotency equivalence
- source evidence mapping
- duplicate／conflict／corruption rules
- policy v2の入力・出力・拒否条件
- fixture／test vector

正本：`docs/operations/QUARANTINE_REPLACEMENT_POLICY.md`

### QL-5C3B: Policy v2 Preview and Proof

**状態: in progress (#52)**

- deterministic replacement plan
- source-to-replacement provenance proof
- semantic-equivalence report
- proof digest and tamper detection
- runtime mutationなしのpreview
- policy-v1 proof verification regression

実装契約：`docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md`

この段階ではproduction ledgerを変更しません。

### QL-5C3C: Rewrite Transaction and Recovery

**状態: blocked by QL-5C3B**

- verified backup v2の事前成功
- same-host operation lock内の再検証
- transaction journal
- staging、verification、atomic publication
- interrupted rewrite recovery
- stale proof／stale index拒否

### QL-5C3D: Operations and Release Hardening

**状態: blocked by QL-5C3C**

- operator CLI
- status／metrics／audit
- failure injection／crash-point tests
- operations runbook
- v0.3.0 release checklist and notes

### 全段階共通の非スコープ

- automatic retention deletion
- distributed locking
- remote archive storage
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
