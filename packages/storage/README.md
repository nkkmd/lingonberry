# storage node

このパッケージには、`storage node` の Rust バイナリを置きます。

## 役割

- raw log を保持する
- canonical catalog を保持する
- replay を支える
- export / import の土台を持つ
- relay とは別プロセスで運用できる

## 前提

- `relay` とは独立した binary とする
- 保存形式や運用ポリシーは、必要に応じて `docs/operations/` 側で正本化する
- storage node の責務境界は `docs/roadmap/OPERATIONAL_READINESS_ROADMAP.md` と整合させる

