# Relay Quickstart

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、初めて Lingonberry を触る人が、`git clone` から `cargo run` で `relay` を起動するまでをそのまま追えるようにまとめます。  
ここでは `relay` の起動確認を最短で通すことを優先し、`storage node` との分離や運用の詳細は別文書に分けます。

## 1. 事前に必要なもの

- `git`
- Rust toolchain
- `cargo`
- `curl` か同等の HTTP クライアント

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

## 4. 依存関係を確認する

リポジトリのルートで作業していることを確認します。

```bash
pwd
```

必要なら、まずワークスペース全体が見えているかを確かめます。

```bash
cargo metadata --no-deps
```

## 5. relay を起動する

まず能力確認を行います。

```bash
cargo run -p lingonberry-relay -- capabilities
```

次に HTTP carrier を起動します。

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

`serve-http` は前面で動くので、その端末は起動しっぱなしにします。

## 6. 起動確認をする

別端末を開いて、`capabilities` endpoint を確認します。

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
```

余裕があれば readiness も確認します。

```bash
curl -sS http://127.0.0.1:8787/v1/ready
```

## 7. 追加で試す

最小の publish まで確認したい場合は、fixture を使います。

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

archive への export / import も確認できます。

```bash
cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive
cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive
```

## 8. つまずきやすい点

- `cargo` が見つからない場合は、`source "$HOME/.cargo/env"` を実行してから再試行します
- `serve-http` が bind 失敗する場合は、`127.0.0.1:8787` が他プロセスに使われていないか確認します
- ビルドが長い場合は、初回だけ依存取得とコンパイルに時間がかかることがあります

## 参照

- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [relay / storage separation](./RELAY_STORAGE_SEPARATION.md)
- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
