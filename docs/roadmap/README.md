# ロードマップ

**Status: active** | **Last updated: 2026-07-12**

このディレクトリには、実装・運用準備・releaseのroadmapとbacklog、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [v0.2.0 Release Checklist](./RELEASE_0_2_0_CHECKLIST.md)
3. [v0.2.0 Release Notes](./RELEASE_0_2_0_RELEASE_NOTE.md)
4. [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md)
5. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
6. [運用文書索引](../operations/README.md)

`CURRENT_IMPLEMENTATION_STATUS.md`は、中断後に作業を再開するための引き継ぎ用正本です。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 実装済み機能、安全境界、次の作業
- [v0.2.0 Release Checklist](./RELEASE_0_2_0_CHECKLIST.md): release gateと最終smoke test
- [v0.2.0 Release Notes](./RELEASE_0_2_0_RELEASE_NOTE.md): 公開範囲、upgrade、既知の制約
- [Legacy Admin Token Deprecation](./RBAC_LEGACY_TOKEN_DEPRECATION.md): legacy token移行契約
- [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md): quarantine継続作業
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md): core実装の中長期計画
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md): 全体roadmapのissue分解
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md): 実運用に向けた中長期計画
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md): 運用準備のissue分解
- [v0.1.0公開前チェックリスト](./RELEASE_0_1_0_CHECKLIST.md): 過去releaseの記録
- [v0.1.0 Release Note](./RELEASE_0_1_0_RELEASE_NOTE.md): 過去releaseの記録

## v0.2.0の到達点

### Protocol／carrier／storage

- core protocolの概念、schema、fixture、canonicalization
- HTTP publish carrierとcapability negotiation
- storage node runtime
- archive export／import
- migration／schema versioning
- access／retention policyの責務分離

### Quarantine lifecycle

- persistent quarantine
- single／batch revalidation and promotion
- dry-run
- status CLI／HTTP API
- Prometheus metrics
- scheduler
- append-only annotations
- manual dismissal
- permanent rejection

### Quarantine maintenance

- same-host operation lock
- verified active-ledger index
- archive-aware ordered reader
- byte-preserving verified rotation
- archive-inclusive backup v2
- backup verification and restore
- non-destructive compaction preview and proof

### Admin security

- public／admin listener isolation
- Bearer authentication
- observer／reviewer／operator RBAC
- uniform `401` and bounded `403`
- authentication／authorization audit
- legacy token deprecation diagnostic

## v0.2.0で未実装・非対象

- record-rewriting compaction
- retention deletion
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
```
