# Core

このディレクトリには、単一ノード保存や再取得のような core data access を置きます。

## 現在の配置

- `object-store.mjs`
  - append-only な JSONL ストアと再取得
- `src/`
  - Rust 版の file-backed storage backend、SQLite catalog、replay
  - archive export / import
  - capability manifest の組み立て

Phase 1 の JS 実装を残しつつ、Phase 2 では Rust 版の relay / storage node の最小実装をここから育てます。
