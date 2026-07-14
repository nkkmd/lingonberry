# Lingonberry v0.3.0 Roadmap

**Status: implementation complete / release closure pending** | **Target: v0.3.0** | **Last updated: 2026-07-14**

## 1. Purpose

v0.3.0は、quarantine ledgerのverified rewrite transactionを、既存の証拠保全・検証可能性・復旧可能性を崩さず導入し、operatorが安全に監視・復旧・検証できる状態へ仕上げるリリースです。

v0.2.0のnon-destructive compaction preview／proof policy v1はmutationを許可しません。v0.3.0では、明示的なreplacement policy、policy-v2 proof、generation-directory transaction、atomic pointer publication、recovery、observability、deterministic failure injectionを導入しました。

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
- machine-readable crash-point inventory
- read-only generation retention inspection
- operator smoke test、release checklist、release notes

### Out of scope

- automatic retention deletion
- automatic generation／workspace deletion
- distributed locking or multi-node shared state
- remote archive／backup storage
- backup encryption or cryptographic signing
- OAuth／OIDC
- browser session or per-record ACL
- legacy admin token fallback removal
- deduplication／event collapse
- schema migration／conflict resolution

## 4. Work breakdown

### QL-5C3A: Replacement Policy and Semantic-equivalence Contract

**状態: completed (#50 / PR #51)**

- managed ledgerごとの分類
- immutable evidenceとreplaceable representationの境界
- replacement keyとordering semantics
- duplicate／conflict／corruption rules
- status／metrics／eligibility／idempotency equivalence
- source-to-replacement provenance requirements

### QL-5C3B: Policy v2 Preview and Proof

**状態: completed (#52 / PR #53)**

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

- same-host operation lock内での再検証
- verified backup v2の事前検証
- versioned transaction journalとbound digests
- complete stagingとstaged ledger verification
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

**状態: implementation completed (#56 / PR #60)**

- structured status `lingonberry-quarantine-replacement-status/v1`
- bounded Prometheus metrics
- secret-free append-only audit events
- apply／status／resume／rollback audit integration
- read-only retention inspection `lingonberry-quarantine-replacement-retention-report/v1`
- `replacement-metrics`／`replacement-inspect-generations` CLI
- end-to-end operator smoke test
- workspace version 0.3.0
- versioned failure-point registry
- machine-readable crash-point inventory
- registry／inventory consistency CI contract
- explicit double-opt-in、one-shot failure injection
- all 18 registered failure points connected by direct seams or explicit post-boundary aliases
- retry／resume／rollback tests for early durable, pre-switch, post-switch, commit, and rollback boundaries

Failure-point coverage：

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

Exit criteria status：

- [x] end-to-end operational smoke test
- [x] mandatory failure injection tests
- [x] documentation／CLI help consistency
- [x] workspace package version 0.3.0
- [x] `cargo fmt`、全Clippy、workspace tests、JavaScript tests
- [ ] main branch CI after merge
- [ ] release commit／tag／GitHub Release

## 5. Replacement policy boundary

| Ledger | v0.3.0 policy |
|---|---|
| `quarantine.jsonl` | immutable evidence; rewrite禁止 |
| `quarantine-annotations.jsonl` | immutable reviewer evidence; rewrite禁止 |
| `admin-auth-audit.jsonl` | immutable audit evidence; rewrite禁止 |
| terminal lifecycle ledgers | approved canonical JSON representation replacementのみ。single-event semanticsとprovenance proofが必須 |

明示的に承認されていないledger、decision、schemaは常にrewrite禁止です。

## 6. Reader-visible generation contract

```text
<state-dir>/quarantine-current-generation.json
<state-dir>/quarantine-generations/<transaction-id>/
```

Pointerがない場合のみlegacy root ledgerを使用します。Pointerが存在する場合はtarget generationだけを使用し、壊れたpointerやgenerationからlegacy rootへfallbackしません。

複数ledger renameをcollectively atomicとはみなしません。Reader-visible switchはcurrent-generation pointer fileの1回のatomic renameです。

## 7. Operations and observability contract

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

## 8. Release gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables
cargo test --workspace
```

追加gate：

- JavaScript canonicalization／identity／validation tests
- crash-point registry／inventory contract test
- policy-v2 fixture compatibility
- proof tampering rejection
- pointer／generation corruption rejection
- crash-point recovery tests
- backup → preview／proof → apply → observe → verify operator smoke test
- v0.2.0 stateからのupgrade compatibility
- public／admin listener isolation regression
- no temporary diagnostic workflow
- main branch CI after merge

## 9. Release artifacts

```text
docs/roadmap/RELEASE_0_3_0_CHECKLIST.md
docs/roadmap/RELEASE_0_3_0_RELEASE_NOTE.md
tag: v0.3.0
GitHub Release: Lingonberry v0.3.0
```

GitHub Releaseはmandatory checklist完了とmain CI成功後に作成します。
