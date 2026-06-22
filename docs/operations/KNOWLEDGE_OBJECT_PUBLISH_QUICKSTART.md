# Knowledge Object Publish Quickstart

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、初めて Lingonberry を触る人が、`git clone` から `cargo run` で `knowledge object` を publish するところまでを、1 本で追えるようにまとめます。  
まずは最小の publish 経路を通し、そのあとで必要に応じて HTTP carrier や `storage node` 側の文書へ進める構成にします。

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

## 4. リポジトリの位置を確認する

```bash
pwd
```

必要なら、ワークスペースの情報を確認します。

```bash
cargo metadata --no-deps
```

## 5. publish の入口を確認する

`knowledge object` の publish は、まず `relay` の publish 経路で試します。  
最小の確認として、relay の能力を見ます。

```bash
cargo run -p lingonberry-relay -- capabilities
```

## 6. まず 1 件 publish する

fixture を使って、最小の publish を通します。

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

このコマンドは、`http-publish-request` envelope を入力にして、`knowledge object` を publish する基本経路を確認するためのものです。

## 7. HTTP carrier で試す

HTTP carrier を起動して確認したい場合は、別端末で次を実行します。

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

起動後は、capabilities を確認できます。

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
```

readiness も確認できます。

```bash
curl -sS http://127.0.0.1:8787/v1/ready
```

publish 自体を HTTP 経由で試す場合は、`POST /v1/objects` に `http-publish-request` envelope を送ります。  
request の中身と `publisher` の扱いは [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md) と [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md) を参照してください。

## 8. publish 後に見るもの

publish 後は、少なくとも次を確認します。

- `status`
- `id`
- `identityKey`
- `canonical`
- `rawRef`

`canonical` は canonical view、`rawRef` は raw / wire 側の参照です。  
両方が保持されていることが、このプロトコルでは重要です。

## 9. storage node が必要なとき

publish の後に保存・再構成・再取得を deeper に確認したい場合は、別途 `storage node` を立てます。  
その場合は [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md) を参照してください。

## 10. つまずきやすい点

- `cargo` が見つからない場合は、`source "$HOME/.cargo/env"` を実行してから再試行します
- `serve-http` が bind 失敗する場合は、`127.0.0.1:8787` が他プロセスに使われていないか確認します
- `publish` が validation error になる場合は、fixture が壊れていないか、schema version が合っているかを確認します
- 初回ビルドは時間がかかることがあります

## 参照

- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
