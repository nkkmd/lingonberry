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
  - `capabilities`、`config`、`run` の最小 runtime 入口
  - `capabilities` には multi-node discovery / sync / conflict / capacity の policy manifest を含める

## ローカル開発

- `LINGONBERRY_STATE_DIR` を使って relay と同じ state dir を明示できる
- `LINGONBERRY_STORAGE_CONFIG` で設定ファイルを明示できる
- 設定ファイルの既定位置は `"$LINGONBERRY_STATE_DIR/storage-config.json"` で、なければ既定値に戻る
- `config` / `run` は解決済みの `stateDir`、`dataDir`、`backupDir`、`tempDir` を表示する
- `stateDir` を設定ファイルで上書きした場合、未指定の `dataDir`、`backupDir`、`tempDir` は新しい `stateDir` を基準に再計算される
- `LINGONBERRY_STATE_DIR` は relay と共有できる共通の実行ルート、`LINGONBERRY_STORAGE_CONFIG` は storage 固有の設定位置として扱う

## 実行例

```bash
cargo run -p lingonberry-storage -- capabilities
cargo run -p lingonberry-storage -- config
cargo run -p lingonberry-storage -- ready
cargo run -p lingonberry-storage -- run
cargo run -p lingonberry-storage -- append fixtures/http-publish-request/minimal-request.json
cargo run -p lingonberry-storage -- retrieve lb:obj:example-0001
cargo run -p lingonberry-storage -- replay
```

## 前提

- `relay` とは独立した binary とする
- backend 生成は `core` の runtime helper と合わせる
- 保存形式や運用ポリシーは、必要に応じて `docs/operations/` 側で正本化する
- 設定と保存レイアウトは [storage node runtime](../../docs/operations/STORAGE_NODE_RUNTIME.md) に合わせる
- relay との別プロセス運用は [relay / storage separation](../../docs/operations/RELAY_STORAGE_SEPARATION.md) に合わせる
- storage node の責務境界は `docs/roadmap/OPERATIONAL_READINESS_ROADMAP.md` と整合させる
- 既定の state dir は `".lingonberry"` とする
