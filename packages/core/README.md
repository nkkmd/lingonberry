# Core

このディレクトリには、共有される core data access と保存ロジックの共通部を置きます。

## 現在の配置

- `object-store.mjs`
  - append-only な JSONL ストアと再取得
- `src/`
  - Rust 版の file-backed storage backend、SQLite catalog、replay
  - archive export / import
  - capability manifest の組み立て

`relay` と `storage node` は、この共通部の上に runtime を分けて載せます。
