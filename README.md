# Lingonberry

**分散知識コモンズ・プロトコル**

Lingonberry は、分散的に運営されるリレー群のあいだで知識オブジェクトを循環させるためのプロトコルです。ソーシャルネットワークそのものではなく、分野非依存の知識基盤を目的とします。

## v0.3.0

v0.3.0では、v0.2.0までのquarantine lifecycleと管理境界に加えて、quarantine ledgerを安全に置き換えるためのverified／recoverable replacement transactionを導入しました。

- verified complete quarantine backup v2とpolicy-v2 replacement proofを必須gateとして使用
- existing ledgerをin-place overwriteしないstaging-only replacement
- generation directoryへの完全なmaterializationとsealed manifest
- current-generation pointerの1回のatomic renameによるpublication
- legacy root-ledger layoutからgeneration-aware resolutionへの互換upgrade
- durable journalに基づくstatus、idempotent resume、pre-commit rollback
- post-publication ledger index／archive segment verification
- versioned structured statusとbounded Prometheus metrics
- secret-free append-only audit
- 18箇所のdeterministic failure injectionとcrash recovery coverage
- generation retentionのread-only inspection
- v0.2.0-style stateからのupgrade compatibility

Release boundary、upgrade手順、operator workflow、既知の制約は、[v0.3.0 Release Notes](./docs/roadmap/RELEASE_0_3_0_RELEASE_NOTE.md)を参照してください。

v0.2.0で導入したquarantine lifecycle、backup／restore、RBAC、admin listener isolationなどは引き続き維持されます。

## このリポジトリに含めるもの

- プロトコルの概念と用語
- 正規化されたデータモデル
- リレーとstorage nodeの責務
- identity、provenance、revisionの規則
- protocol-nativeなindexとAPI参照面
- carrier、archive、migration、access／retentionの運用契約
- quarantine lifecycleと管理・保全ツール
- verified replacement transaction、recovery、generation inspection
- ドメイン語彙とapplication profileの拡張点

## 中核の考え方

Lingonberryは、知識をappend-onlyで、replay可能で、provenanceを保持するものとして扱います。WebSocket、HTTP、file archive、将来のfederated carrierは、同じprotocol objectを運ぶcarrier実装です。

## まず読む場所

- [アーキテクチャ](./docs/architecture/README.md)
- [ロードマップと現在地](./docs/roadmap/README.md)
- [運用メモ](./docs/operations/README.md)
- [概念](./docs/concepts/README.md)
- [Protocols](./docs/protocols/README.md)
- [Schemas](./schemas/README.md)
- [Changelog](./CHANGELOG.md)

## Quickstart

- [Knowledge Object Publish Quickstart](./docs/operations/KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
- [Relay Quickstart](./docs/operations/RELAY_QUICKSTART.md)
- [Storage Node Quickstart](./docs/operations/STORAGE_NODE_QUICKSTART.md)
- [Quarantine Admin HTTP](./docs/operations/QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Backup and Restore](./docs/operations/QUARANTINE_BACKUP_RESTORE.md)
- [Quarantine Replacement Preview Runbook](./docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [Quarantine Replacement Recovery Runbook](./docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)

## 実行例

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive
cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive
```

Admin listenerは明示的なrole tokenで起動します。

```bash
export LINGONBERRY_ADMIN_OBSERVER_TOKEN=<observer-secret>
export LINGONBERRY_ADMIN_REVIEWER_TOKEN=<reviewer-secret>
export LINGONBERRY_ADMIN_OPERATOR_TOKEN=<operator-secret>

cargo run -p lingonberry-relay -- serve-admin-http 127.0.0.1:8788
cargo run -p lingonberry-relay --bin lingonberry-admin-auth-config
```

Replacement maintenanceの入口：

```bash
cargo run -p lingonberry-relay --bin lingonberry-quarantine-maintenance -- \
  replacement-status <transaction-dir>

cargo run -p lingonberry-relay --bin lingonberry-quarantine-maintenance -- \
  replacement-inspect-generations [transaction-dir ...]
```

## License

Lingonberryは`Apache-2.0`で公開します。詳細は[LICENSE](./LICENSE)を参照してください。
