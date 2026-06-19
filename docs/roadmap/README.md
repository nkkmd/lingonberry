# ロードマップ

**Status: active** | **Last updated: 2026-06-19**

このディレクトリには、2 本のロードマップとそれぞれの backlog を置きます。

- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md) と [実装バックログ](./IMPLEMENTATION_BACKLOG.md) は、Lingonberry の core 実装を進めるための文書です。
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md) と [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md) は、実装後の運用整備を進めるための文書です。

実装ロードマップ側では、Phase 0 と Phase 1 は仕様固定と単一オブジェクト publish 経路の実装が完了しています。
実装の本命は ADR にある Rust + SQLite で、現行の Phase 1 JavaScript 実装はその前段の検証用ブートストラップです。
Phase 2 では relay / storage node の分離を起点に、本命実装へ移行します。
Phase 4 は完了済みで、`packages/indexer/` を起点に canonical store から派生 index を組み立てています。
Phase 6 も完了済みで、HTTP carrier、archive carrier、capability negotiation、access / retention、migration / schema versioning の正本と最小実装を整えています。

運用準備ロードマップ側では、実運用に向けて relay / storage の分離、起動・停止、設定、監視、バックアップ、carrier 拡張の順に整備していきます。

## 実行の入口

まず試すなら、次の順が分かりやすいです。

1. `cargo run -p lingonberry-relay -- capabilities`
2. `cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787`
3. `cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json`
4. `cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive`
5. `cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive`

## 文書
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md)
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md)
