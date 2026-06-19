# Packages

このディレクトリには、実装を役割ごとに分離したパッケージを置きます。

## 現在の配置

- `packages/protocol/`
  - Rust 版の protocol object parser / validator / canonicalizer
- `packages/codecs/`
  - protocol object の validate / normalize / finalize
- `packages/core/`
  - Rust 版の append-only storage と replay / retrieval
- `packages/storage/`
  - Rust 版の storage node バイナリ
- `packages/indexer/`
  - canonical store から派生する search / graph / view index
- `packages/relay/`
  - Rust 版の relay バイナリ
- `packages/api/`
  - canonical view の組み立て
- `packages/cli/`
  - validate / publish / get / list の実行入口

## 進め方

実装が増えても、責務ごとに分けたままこの配下へ追加していきます。
Phase 1 の JavaScript 実装は検証用ブートストラップとして残しつつ、Phase 2 では Rust の protocol / core / relay を追加していきます。
