# Indexer

このディレクトリには、canonical store から派生する検索・graph・view 系の index を置きます。

## 現在の配置

- `src/lib.rs`
  - type index、relation graph、lineage graph、provenance graph の最小構成
  - canonical store からの rebuild 手順と read-only query

Phase 4 では、semantic source を canonical store に残したまま、ここに read model を増やしていきます。
