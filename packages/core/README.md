# Core

このディレクトリには、単一ノード保存や再取得のような core data access を置きます。

## 現在の配置

- `object-store.mjs`
  - append-only な JSONL ストアと再取得

Phase 1 では、最小の永続化層をここに置き、後続で relay / indexer / api へ分けていきます。
