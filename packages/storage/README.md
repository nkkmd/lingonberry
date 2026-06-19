# storage node

このパッケージには、`storage node` の Rust バイナリを置きます。

## 役割

- raw log を保持する
- canonical catalog を保持する
- replay を支える
- export / import の土台を持つ
- relay とは別プロセスで運用できる

## 実装の入口

- `src/lib.rs`
  - relay と storage binary から共有する backend 構築の入口
- `src/main.rs`
  - `capabilities` と `run` の最小 runtime 入口

## ローカル開発

- `LINGONBERRY_STATE_DIR` を使って relay と同じ state dir を明示できる
- 既定値は `".lingonberry"` で、`storage` と `relay` は同じ保存先を共有できる

## 実行例

```bash
cargo run -p lingonberry-storage -- capabilities
cargo run -p lingonberry-storage -- run
cargo run -p lingonberry-storage -- append fixtures/http-publish-request/minimal-request.json
cargo run -p lingonberry-storage -- retrieve lb:obj:example-0001
cargo run -p lingonberry-storage -- replay
```

## 前提

- `relay` とは独立した binary とする
- backend 生成は `core` の runtime helper と合わせる
- 保存形式や運用ポリシーは、必要に応じて `docs/operations/` 側で正本化する
- storage node の責務境界は `docs/roadmap/OPERATIONAL_READINESS_ROADMAP.md` と整合させる
- 既定の state dir は `".lingonberry"` とする
