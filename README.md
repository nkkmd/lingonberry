# Lingonberry

**分散知識コモンズ・プロトコル**

Lingonberry は、分散的に運営されるリレー群のあいだで知識オブジェクトを循環させるためのプロトコルです。ソーシャルネットワークそのものではなく、分野非依存の知識基盤を目的とします。

## v0.2.0

v0.2.0では、最小publish／storage経路に加えて、運用可能なquarantine lifecycleと管理境界を整備しました。

- persistent quarantine、single／batch promotion、annotation、dismissal、permanent rejection
- status、Prometheus metrics、scheduled revalidation
- same-host operation lock
- verified ledger index、archive-aware ordered reads、byte-preserving rotation
- archive-inclusive backup v2、verify、restore
- non-destructive compaction preview and semantic proof
- public／admin listener isolation
- observer／reviewer／operator RBAC
- authentication／authorization audit
- legacy admin token deprecation diagnostic

Release boundaryと既知の制約は、[v0.2.0 Release Notes](./docs/roadmap/RELEASE_0_2_0_RELEASE_NOTE.md)を参照してください。

## このリポジトリに含めるもの

- プロトコルの概念と用語
- 正規化されたデータモデル
- リレーとstorage nodeの責務
- identity、provenance、revisionの規則
- protocol-nativeなindexとAPI参照面
- carrier、archive、migration、access／retentionの運用契約
- quarantine lifecycleと管理・保全ツール
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

## License

Lingonberryは`Apache-2.0`で公開します。詳細は[LICENSE](./LICENSE)を参照してください。
