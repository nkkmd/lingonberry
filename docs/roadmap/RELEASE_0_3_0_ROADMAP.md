# Lingonberry v0.3.0 Roadmap

**Status: planning** | **Target: v0.3.0** | **Last updated: 2026-07-13**

## 1. Purpose

v0.3.0は、quarantine ledgerのverified rewrite transactionを、既存の証拠保全・検証可能性・復旧可能性を崩さず導入するためのリリースです。

v0.2.0で実装したnon-destructive compaction previewは、policy v1として全lineを保持し、mutationを一切許可しません。v0.3.0では、rewrite実装そのものより先に、何を同値なreplacementとして認めるかを明示し、機械検証可能なcontractへ落とし込みます。

## 2. Release principles

1. **Policy before mutation** — 承認済みreplacement policyなしでrewriteを実装しない
2. **Evidence preservation** — 元recordと変換根拠を追跡可能にする
3. **Semantic equivalence** — rewrite前後の観測可能な意味を機械検証する
4. **Transactional publication** — 部分的に公開されたledger stateを作らない
5. **Recoverability** — interruption後にrollbackまたは安全なresumeが可能である
6. **Retention separation** — retention deletionをrewrite transactionへ混在させない

## 3. Scope

### In scope

- ledger type別replacement semantics
- semantic-equivalence contract
- versioned compaction policy v2
- replacement proof format
- transaction journal and state machine
- interrupted rewrite recovery
- verified backup v2 precondition
- policy-aware preview and proof verification
- approved policyに限定したverified rewrite transaction
- status／metrics／CLI／operations documentation

### Out of scope

- automatic retention deletion
- distributed locking or multi-node shared state
- remote archive storage
- remote backup upload
- backup encryption or cryptographic signing
- OAuth／OIDC
- browser session or per-record ACL
- legacy admin token fallback removal

## 4. Work breakdown

### QL-5C3A: Replacement Policy and Semantic-equivalence Contract

**Goal:** rewrite可能性を実装から分離し、review可能な仕様として固定する。

Deliverables：

- managed ledgerごとの分類
- immutable evidenceとreplaceable representationの境界
- replacement keyとordering semantics
- duplicate／conflict／corruption rules
- status equivalence
- metrics equivalence
- eligibility equivalence
- idempotency equivalence
- source-to-replacement provenance requirements
- unsupported policy／unknown ledger拒否条件

Exit criteria：

- policy documentがreview済み
- contractがfixtureまたはtest vectorで表現されている
- policy v1との非互換点が明記されている
- retention deletionが含まれていない

### QL-5C3B: Policy v2 Preview and Proof

**Goal:** mutationなしで、policy v2によるreplacement planとequivalence proofを生成・検証する。

Deliverables：

- `lingonberry-quarantine-compaction-policy/v2`
- deterministic replacement plan
- input ledger fingerprints
- retained／replaced／rejected line classification
- source evidence mapping
- semantic-equivalence report
- proof digest and tamper detection
- unsupported／unsafe plan rejection

Exit criteria：

- previewはruntime stateを変更しない
-同一入力から同一planとdigestが生成される
- corruptionとpolicy violationが明示的に失敗する
- v1 proof verificationを継続サポートする

### QL-5C3C: Rewrite Transaction and Recovery

**Goal:** 承認済みplanのみをatomicに適用し、interruptionから安全に回復する。

Transaction states：

```text
prepared
writing
verified
committed
rolled-back
recovery-required
```

Deliverables：

- same-host operation lock内での再検証
- verified backup v2の事前検証
- transaction journal
- staging directoryへのrewrite
- staged ledger verification
- atomic publication
- post-commit index rebuild／verification
- rollbackまたはresume command
- crash-point tests

Exit criteria：

- partial publicationが発生しない
- stale proof／stale index／changed runtime fingerprintを拒否する
- interruption後の状態が判定可能である
- commit後にstatus／metrics／eligibility／idempotencyがcontractと一致する

### QL-5C3D: Operations, Observability, and Release Hardening

**Goal:** operatorがrewrite前後の状態を確認し、安全に運用できるようにする。

Deliverables：

- dry-run／apply／recover CLI
- structured status
- Prometheus metrics
- secret-free audit events
- backup／restore／rewrite runbook
- release checklist
- release notes

Exit criteria：

- end-to-end operational smoke testが記録されている
- failure injection testが通る
- documentationとCLI helpが一致する
- `cargo fmt`、`cargo clippy -D warnings`、workspace tests、JavaScript testsが通る

## 5. Proposed policy boundary

v0.3.0の初期設計では、次を原則として扱います。

| Ledger | Default policy direction |
|---|---|
| `quarantine.jsonl` | immutable evidence; rewrite禁止 |
| `quarantine-annotations.jsonl` | immutable reviewer evidence; rewrite禁止 |
| `admin-auth-audit.jsonl` | immutable audit evidence; rewrite禁止 |
| terminal lifecycle ledgers | replacement候補になり得るが、single-event semanticsとprovenance proofが必須 |

この表はQL-5C3Aで確定するまで実装上の許可を意味しません。明示的に承認されていないledgerは常にrewrite禁止です。

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

## 7. Release gates

各実装PR：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

JavaScript canonicalization／identity／validation testsも必須です。

v0.3.0 release前には追加で以下を満たします。

- policy v2 fixture compatibility
- proof tampering rejection
- crash-point recovery tests
- backup → preview → apply → verify → restore smoke test
- v0.2.0 stateからのupgrade test
- public／admin listener isolation regression test

## 8. First implementation task

最初のIssueは **QL-5C3A: Define replacement policy and semantic-equivalence contract** とします。

このIssueではproduction ledgerを変更するコードを書きません。仕様、fixture、test vector、拒否条件を確定し、その後にQL-5C3Bのpreview実装へ進みます。
