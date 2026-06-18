# API

このディレクトリには、canonical view を組み立てる API 層を置きます。

## 現在の配置

- `canonical-view.mjs`
  - canonical object を API 返却形へ整える

Phase 1 では、CLI の `get` がここを利用する最初の入口になります。
