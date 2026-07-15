# ロードマップ

**Status: active** | **Latest release: v0.3.0** | **Last updated: 2026-07-15**

このディレクトリには、実装・運用準備・releaseのroadmapとbacklog、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [v0.3.0 Release Checklist](./RELEASE_0_3_0_CHECKLIST.md)
3. [v0.3.0 Release Notes](./RELEASE_0_3_0_RELEASE_NOTE.md)
4. [v0.3.0 Roadmap](./RELEASE_0_3_0_ROADMAP.md)
5. [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md)
6. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
7. [運用文書索引](../operations/README.md)

`CURRENT_IMPLEMENTATION_STATUS.md`は、中断後に作業を再開するための引き継ぎ用正本です。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 実装済み機能、安全境界、次の作業
- [v0.3.0 Release Checklist](./RELEASE_0_3_0_CHECKLIST.md): 完了済みrelease gateとpost-release記録
- [v0.3.0 Release Notes](./RELEASE_0_3_0_RELEASE_NOTE.md): 公開範囲、upgrade、operator workflow、既知の制約
- [v0.3.0 Roadmap](./RELEASE_0_3_0_ROADMAP.md): verified replacement transactionの設計・実装・release記録
- [Legacy Admin Token Deprecation](./RBAC_LEGACY_TOKEN_DEPRECATION.md): legacy token移行契約
- [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md): quarantine継続作業とv0.3.0完了記録
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md): core実装の中長期計画
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md): 全体roadmapのissue分解
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md): 実運用に向けた中長期計画
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md): 運用準備のissue分解
- [v0.2.0 Release Checklist](./RELEASE_0_2_0_CHECKLIST.md): 過去releaseの記録
- [v0.2.0 Release Notes](./RELEASE_0_2_0_RELEASE_NOTE.md): 過去releaseの記録
- [v0.1.0公開前チェックリスト](./RELEASE_0_1_0_CHECKLIST.md): 過去releaseの記録
- [v0.1.0 Release Note](./RELEASE_0_1_0_RELEASE_NOTE.md): 過去releaseの記録

## v0.3.0の到達点

### v0.2.0から継続する基盤

- core protocol、schema、fixtures、canonicalization
- HTTP publish carrier、storage node runtime、archive export／import
- persistent quarantine lifecycle
- status、Prometheus metrics、scheduler
- annotations、dismissal、permanent rejection
- same-host operation lock
- verified ledger index、archive-aware ordered reads、byte-preserving rotation
- archive-inclusive backup v2、verify、restore
- public／admin listener isolationとRBAC

### Verified replacement transaction

- replacement policyとsemantic-equivalence contract
- policy-v2 replacement preview／proof
- verified backup v2とproofを必須とするtransaction
- staging-only ledger construction
- sealed generation manifestとgeneration digest
- generation-directory active-ledger resolution
- current-generation pointerのatomic publication
- deterministic status classification
- idempotent resumeとpre-commit rollback
- post-publication index／archive-segment verification
- legacy root-ledger layoutからの互換upgrade

### Operations and hardening

- versioned structured replacement status
- bounded Prometheus metrics
- secret-free append-only audit
- 18箇所のdeterministic failure injection
- machine-readable crash-point inventory
- read-only generation retention inspection
- end-to-end operator smoke test
- v0.2.0-style stateからのupgrade compatibility

## v0.3.0で未実装・非対象

- automatic retention deletion
- automatic generation／transaction-workspace deletion
- deduplication／event collapse
- schema migration／conflict resolution
- archive-segment rewrite／deletion
- distributed locking／multi-node consensus
- remote backup upload
- backup encryption／cryptographic signing
- OAuth／OIDC
- browser sessions／per-record ACL
- deprecated legacy admin token fallbackの完全削除

## 実行の入口

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- quarantine-status
cargo run -p lingonberry-relay -- quarantine-metrics
cargo run -p lingonberry-relay -- quarantine-promote-batch 100 --dry-run
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
cargo run -p lingonberry-relay -- serve-admin-http 127.0.0.1:8788
cargo run -p lingonberry-relay --bin lingonberry-admin-auth-config
cargo run -p lingonberry-relay --bin lingonberry-quarantine-maintenance -- \
  replacement-status <transaction-dir>
cargo run -p lingonberry-relay --bin lingonberry-quarantine-maintenance -- \
  replacement-inspect-generations [transaction-dir ...]
```
