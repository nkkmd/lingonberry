# ロードマップ

**Status: active** | **Last updated: 2026-07-12**

このディレクトリには、2 本のロードマップとそれぞれの backlog、および作業再開用の現在地文書を置きます。

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md) は、中断後に作業を再開するための引き継ぎ用正本です。
- [Quarantine Status API](./QUARANTINE_STATUS_API.md) は、quarantine の永続状態集計、CLI、HTTP API、監視接続の契約を定義します。
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md) と [実装バックログ](./IMPLEMENTATION_BACKLOG.md) は、Lingonberry の core 実装を進めるための文書です。
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md) と [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md) は、実装後の運用整備を進めるための文書です。
- [v0.1.0 公開前チェックリスト](./RELEASE_0_1_0_CHECKLIST.md) は、OSS 公開・配布の直前に確認するための文書です。
- [v0.1.0 Release Note Draft](./RELEASE_0_1_0_RELEASE_NOTE.md) は、公開範囲、公開直前の確認項目、今後のロードマップ、source archive 手順をまとめる文書です。

実装ロードマップ側では、Phase 0 と Phase 1 は仕様固定と単一オブジェクト publish 経路の実装が完了しています。
実装の本命は ADR にある Rust + SQLite で、現行の Phase 1 JavaScript 実装はその前段の検証用ブートストラップです。
Phase 2 では relay / storage node の分離を起点に、本命実装へ移行します。
Phase 4 は完了済みで、`packages/indexer/` を起点に canonical store から派生 index を組み立てています。
Phase 6 も完了済みで、HTTP carrier、archive carrier、capability negotiation、access / retention、migration / schema versioning の正本と最小実装を整えています。

quarantine 運用では、永続 ledger から `total`、`pending`、`promoted`、timestamp、`reasonCode` 集計を取得する status API を実装しています。`deferred` と `rejected` は永続 lifecycle state ではなく再評価時の判定として区別します。

運用準備ロードマップ側では、実運用に向けた relay / storage の分離、起動・停止、設定、監視、バックアップ、carrier 拡張を扱います。
Phase 12 の文書整理は完了していますが、実装と運用確認はまだ未完了です。
Phase 12 の実装完了条件は [Phase 12 実装完了チェックリスト](./OPERATIONAL_READINESS_PHASE_12_IMPLEMENTATION_CHECKLIST.md) に分けています。

## 実行の入口

まず試すなら、次の順が分かりやすいです。

1. `cargo run -p lingonberry-relay -- capabilities`
2. `cargo run -p lingonberry-relay -- quarantine-status`
3. `cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787`
4. `cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json`
5. `cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive`
6. `cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive`

HTTP server では `GET /v1/quarantine-status` から同じ集計を取得できます。

## 文書
- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
- [Quarantine Status API](./QUARANTINE_STATUS_API.md)
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md)
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md)
- [v0.1.0 Release Note Draft](./RELEASE_0_1_0_RELEASE_NOTE.md)
