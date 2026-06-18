# Relay

このディレクトリには、Lingonberry の Rust relay バイナリを置きます。

## 現在の配置

- `src/`
  - publish request の validate / publish
  - append-only log への保存
  - SQLite catalog を使う replay / get / list の最小入口

Phase 2 の最初の切り出しは、ここから始めます。
