# Storage Node Quickstart

**Status: draft** | **Last updated: 2026-06-23**

## 目的

この文書は、初めて Lingonberry を触る人が、`git clone` から `cargo run` で `storage node` を起動するまでをそのまま追えるようにまとめます。  
ここでは `storage node` の設定確認と起動確認を優先し、relay との分離や複数ノード運用は別文書に分けます。

## 1. 事前に必要なもの

- `git`
- Rust toolchain
- `cargo`

Rust が入っていない場合は、公式の `rustup` を使う前提で進めます。

## 2. リポジトリを取得する

```bash
git clone git@github.com:nkkmd/lingonberry.git lingonberry
cd lingonberry
```

## 3. Rust を導入する

Rust が未導入なら、`rustup` で入れます。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

導入できたか確認します。

```bash
rustc --version
cargo --version
```

既に Rust が入っているなら、この手順は飛ばしてかまいません。

## 4. 起動前の考え方

`storage node` は `LINGONBERRY_STATE_DIR` で実行ルートを決めます。  
迷ったら、まず専用のディレクトリを 1 つ切ってください。

```bash
export LINGONBERRY_STATE_DIR="$HOME/.lingonberry-storage"
```

設定ファイルを使う場合は、`LINGONBERRY_STORAGE_CONFIG` で明示できます。  
初見のうちは、まず `LINGONBERRY_STATE_DIR` だけで動かすと分かりやすいです。

## 5. storage node を起動する

まず能力確認を行います。

```bash
cargo run -p lingonberry-storage -- capabilities
```

次に解決済み設定を確認します。

```bash
cargo run -p lingonberry-storage -- config
```

最後に実行状態を確認します。

```bash
cargo run -p lingonberry-storage -- run
```

## 6. 起動確認をする

`run` の出力で、次が確認できると安心です。

- `configPath`
- `stateDir`
- `dataDir`
- `backupDir`
- `tempDir`
- `rawLogPath`
- `catalogPath`

必要なら、保存系のコマンドも試せます。

```bash
cargo run -p lingonberry-storage -- replay
cargo run -p lingonberry-storage -- list
```

## 7. つまずきやすい点

- `cargo` が見つからない場合は、`source "$HOME/.cargo/env"` を実行してから再試行します
- `config` の結果が想定と違う場合は、`LINGONBERRY_STATE_DIR` と `LINGONBERRY_STORAGE_CONFIG` の両方を確認します
- 初回ビルドは時間がかかることがあります

## 8. relay と合わせて使うとき

`storage node` は `relay` と別プロセスで立てる前提です。  
両方を同じホストで試す場合でも、`stateDir` は分けるのが基本です。
`Caddy` は `relay` の公開フロントとして使い、`storage node` を直接公開する用途には使いません。
外部からの確認は `relay` 側の quickstart に寄せ、`storage node` は内部保存と replay の確認に集中します。
公開面の構成や `Caddy` の設定例は [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md) を参照してください。

## 参照

- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
- [relay / storage separation](./RELAY_STORAGE_SEPARATION.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
