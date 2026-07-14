# Lingonberry v0.3.0 Roadmap

**Status: release hardening in progress** | **Target: v0.3.0** | **Last updated: 2026-07-14**

## 1. Purpose

v0.3.0は、quarantine ledgerのverified rewrite transactionを、既存の証拠保全・検証可能性・復旧可能性を崩さず導入し、operatorが安全に監視・復旧・検証できる状態へ仕上げるリリースです。

v0.2.0のnon-destructive compaction preview／proof policy v1はmutationを許可しません。v0.3.0では、明示的なreplacement policy、policy-v2 proof、generation-directory transaction、atomic pointer publication、recovery、observabilityを段階的に導入します。

## 2. Release principles

1. **Policy before mutation** — 承認済みreplacement policyなしでrewriteを実装しない
2. **Evidence preservation** — 元recordと変換根拠を追跡可能にする
3. **Semantic equivalence** — rewrite前後の観測可能な意味を機械検証する
4. **Transactional publication** — reader-visible publicationを1つのatomic pointer switchへ集約する
5. **Recoverability** — interruption後にrollbackまたは安全なresumeが可能である
6. **Retention separation** — retention deletionをrewrite transactionへ混在させない
7. **Fail closed** — pointer、journal、generation、proofが曖昧または不正な状態をhealthyとして扱わない
8. **Bounded observability** — metrics／auditへsecret、path、transaction ID、free-form errorを流さない

## 3. Scope

### In scope

- ledger type別replacement semantics
- semantic-equivalence contract
- versioned replacement policy v2
- replacement proof format
- transaction journal and state machine
- generation-directory active-ledger resolution
- atomic current-generation pointer publication
- interrupted rewrite recovery and pre-commit rollback
- verified backup v2 precondition
- policy-aware preview and proof verification
- versioned status／bounded metrics／secret-free audit
- deterministic failure injection and crash recovery tests
- read-only generation retention inspection
- operator smoke test、release checklist、release notes

### Out of scope

- automatic retention deletion
- automatic generation／workspace deletion
- distributed locking or multi-node shared state
- remote archive storage
- remote backup upload
- backup encryption or cryptographic signing
- OAuth／OIDC
- browser session or per-record ACL
- legacy admin token fallback removal
- deduplication／event collapse
- schema migration／conflict resolution

## 4. Work breakdown

### QL-5C3A: Replacement Policy and Semantic-equivalence Contract

**状態: completed (#50 / PR #51)**

Deliverables：

- managed ledgerごとの分類
- immutable evidenceとreplaceable representationの境界
- replacement keyとordering semantics
- duplicate／conflict／corruption rules
- status／metrics／eligibility／idempotency equivalence
- source-to-replacement provenance requirements
- unsupported policy／unknown ledger拒否条件

### QL-5C3B: Policy v2 Preview and Proof

**状態: completed (#52 / PR #53)**

Deliverables：

- deterministic replacement plan
- input ledger fingerprints
- retained／replaced／rejected line classification
- source evidence mapping
- semantic-equivalence report
- plan／proof digest and tamper detection
- runtime fingerprint verification
- non-destructive CLI preview and proof verification

### QL-5C3C: Rewrite Transaction and Recovery

**状態: completed (#54 / PR #55)**

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

Deliverables：

- same-host operation lock内での再検証
- verified backup v2の事前検証
- versioned transaction journalとbound digests
- staging directoryへのrewrite
- staged ledger verification
- sealed generation manifest
- complete generation materialization
- generation-directory resolver
- atomic current-generation pointer publication
- post-publication index／segment verification
- deterministic status classification
- idempotent apply／resume／rollback
- legacy root layout compatibility
- invalid pointer／mixed generation fail-closed rejection

### QL-5C3D: Operations, Observability, and Release Hardening

**状態: in progress (#56 / Draft PR #60)**

実装済みdeliverables：

- structured status `lingonberry-quarantine-replacement-status/v1`
- bounded Prometheus metrics
- secret-free append-only audit events
- apply／status／resume／rollback audit integration
- explicit double-opt-in、one-shot failure injection
- stable failure points for pointer rename、index rebuild、commit transition、rollback pointer restoration、rolled-back transition
- crash recovery tests for pre-switch、post-switch、commit、rollback states
- read-only retention inspection `lingonberry-quarantine-replacement-retention-report/v1`
- `replacement-metrics`／`replacement-inspect-generations` CLI
- end-to-end operator smoke test
- v0.3.0 checklist／release notes
- workspace version 0.3.0

残るdeliverables：

- early durable write／fsync failure injection
- generation／publication-intent／pointer temporary-write failure injection
- index／segment verification failure injection
- machine-readableまたはtable-driven crash-point inventory
- canonical status／backlog／roadmap／PR本文の最終同期
- merge後のmain CI、release commit、tag、GitHub Release

Exit criteria：

- end-to-end operational smoke testが記録されている
- mandatory failure injection testが通る
- documentationとCLI helpが一致する
- workspace package versionが0.3.0である
- `cargo fmt`、全Clippy、workspace tests、JavaScript testsが通る
- main branch CIが通る
- release checklistのmandatory itemが完了する

## 5. Replacement policy boundary

| Ledger | v0.3.0 policy |
|---|---|
| `quarantine.jsonl` | immutable evidence; rewrite禁止 |
| `quarantine-annotations.jsonl` | immutable reviewer evidence; rewrite禁止 |
| `admin-auth-audit.jsonl` | immutable audit evidence; rewrite禁止 |
| terminal lifecycle ledgers | approved canonical JSON representation replacementのみ。single-event semanticsとprovenance proofが必須 |

明示的に承認されていないledger、decision、schemaは常にrewrite禁止です。

## 6. Semantic-equivalence minimum contract

rewrite前後で少なくとも次を比較します。

```text
logical record identity
terminal state per quarantine ID
ordered-read result where order is semantically relevant
promotion eligibility
batch idempotency outcome
status counts
Prometheus metric values
corruption detection behavior
operator-visible provenance
```

単純なline countやbyte countの一致だけではsemantic equivalenceとみなしません。

## 7. Reader-visible generation contract

```text
<state-dir>/quarantine-current-generation.json
<state-dir>/quarantine-generations/<transaction-id>/
```

Pointerがない場合のみlegacy root ledgerを使用します。Pointerが存在する場合はtarget generationだけを使用し、壊れたpointerやgenerationからlegacy rootへfallbackしません。

複数ledger renameをcollectively atomicとはみなしません。Reader-visible switchはcurrent-generation pointer fileの1回のatomic renameです。

## 8. Operations and observability contract

Operator CLI：

```bash
lingonberry-quarantine-maintenance replacement-apply \
  <verified-backup-v2-dir> <verified-proof-dir> <transaction-dir>
lingonberry-quarantine-maintenance replacement-status <transaction-dir>
lingonberry-quarantine-maintenance replacement-metrics <transaction-dir>
lingonberry-quarantine-maintenance replacement-inspect-generations \
  [transaction-dir ...]
lingonberry-quarantine-maintenance replacement-recover \
  <transaction-dir> --resume|--rollback
```

Metricsはbounded labelのみを使用します。Auditはappend-only、secret-freeで、mutating operationの開始auditに失敗した場合はoperationを開始しません。

Retention inspectorはclassificationだけを返し、delete、rename、truncate、repair、pointer mutationを行いません。

## 9. Release gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables
cargo test --workspace
```

JavaScript canonicalization／identity／validation testsも必須です。

追加gate：

- policy-v2 fixture compatibility
- proof tampering rejection
- pointer／generation corruption rejection
- crash-point recovery tests
- backup → preview／proof → apply → observe → verify operator smoke test
- v0.2.0 stateからのupgrade compatibility
- public／admin listener isolation regression
- no temporary diagnostic workflow
- main branch CI after merge

## 10. Release artifacts

```text
docs/roadmap/RELEASE_0_3_0_CHECKLIST.md
docs/roadmap/RELEASE_0_3_0_RELEASE_NOTE.md
tag: v0.3.0
GitHub Release: Lingonberry v0.3.0
```

GitHub Releaseはmandatory checklist完了とmain CI成功後に作成します。
