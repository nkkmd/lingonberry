# API

このディレクトリには、canonical view を組み立てる API 層を置きます。

## 現在の配置

- `canonical-view.mjs`
  - canonical object と read-only list / get / graph view を整える

Phase 1 では、CLI の `get` がここを利用する最初の入口になります。
Phase 4 では、indexer の派生結果をこの層に載せ、semantic source を canonical store に残します。
