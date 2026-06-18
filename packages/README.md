# Packages

このディレクトリには、実装を役割ごとに分離したパッケージを置きます。

## 現在の配置

- `packages/codecs/`
  - protocol object の validate / normalize / finalize
- `packages/core/`
  - 単一ノード保存と再取得の最小データアクセス
- `packages/api/`
  - canonical view の組み立て
- `packages/cli/`
  - validate / publish / get / list の実行入口

## 進め方

実装が増えても、責務ごとに分けたままこの配下へ追加していきます。
Phase 1 では、まず CLI と codec と core と api の最小構成で進めます。
