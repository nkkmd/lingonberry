# ロードマップ

**Status: active** | **Last updated: 2026-07-12**

このディレクトリには、実装・運用準備のロードマップとbacklog、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md)
3. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)
4. [運用文書索引](../operations/README.md)

`CURRENT_IMPLEMENTATION_STATUS.md` は、中断後に作業を再開するための引き継ぎ用正本です。PR #19までのquarantine実装、runtime state、CLI / HTTP surface、安全性ルール、未解決事項、再開コマンドをまとめています。

`QUARANTINE_LIFECYCLE_BACKLOG.md` は、status・metrics・scheduler・operator annotations完了後の継続作業をissue単位で整理します。次の第一候補は、元recordを削除しないappend-only manual dismissal lifecycleです。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 休止・再開の正本
- [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md): quarantine継続作業の短期backlog
- [Quarantine Status API](./QUARANTINE_STATUS_API.md): 永続状態集計、CLI、HTTP APIの契約
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md): core実装の中長期計画
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md): 全体ロードマップのissue分解
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md): 実運用に向けた中長期計画
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md): 運用準備のissue分解
- [v0.1.0 公開前チェックリスト](./RELEASE_0_1_0_CHECKLIST.md): OSS公開直前の確認
- [v0.1.0 Release Note Draft](./RELEASE_0_1_0_RELEASE_NOTE.md): 公開範囲とrelease note草案

## 現在の到達点

実装ロードマップでは、Phase 0とPhase 1の仕様固定・単一object publish経路が完了しています。Phase 4の派生index、Phase 6のHTTP / archive carrier、capability negotiation、access / retention、migration / schema versioningの正本と最小実装も整っています。

quarantine運用では、次を実装済みです。

- persistent quarantine store
- single / batch revalidation and promotion
- dry-run
- status CLI / HTTP API
- Prometheus metrics
- systemd timer / cron fallback
- append-only operator annotations

永続lifecycle stateは現時点で`pending`と`promoted`です。`deferred`と`rejected`は再評価時の一時的判定であり、operator annotationはlifecycle stateではありません。

未実装の主要項目：

- manual dismissal
- permanently rejected lifecycle
- admin authentication / authorization
- JSONL backup / rotation / compaction
- distributed locking

## 実行の入口

read-onlyまたはdry-runの確認を優先します。

```bash
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- quarantine-status
cargo run -p lingonberry-relay -- quarantine-metrics
cargo run -p lingonberry-relay -- quarantine-annotations
cargo run -p lingonberry-relay -- quarantine-promote-batch 100 --dry-run
```

HTTP server：

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

代表endpoint：

```text
GET /v1/quarantine-status
GET /metrics
GET /v1/quarantine/<quarantine-id>/annotations
```

quarantine関連の管理endpointは、authentication / authorizationが未実装のため一般公開しない構成を優先します。
