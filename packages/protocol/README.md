# Protocol

このディレクトリには、Lingonberry の protocol object を扱う Rust 実装を置きます。

## 現在の配置

- `src/`
  - JSON parser
  - knowledge object validation
  - canonicalization

Phase 2 では、この crate を relay と storage backend から共有します。
