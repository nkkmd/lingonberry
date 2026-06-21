# Relay

このディレクトリには、Lingonberry の Rust relay バイナリを置きます。

## 現在の配置

- `src/`
  - publish request の validate / publish
  - append-only log への保存
  - SQLite catalog を使う replay / get / list の最小入口
  - `serve-http` による HTTP carrier の最小実装
  - carrier capabilities の出力
  - multi-node discovery / sync / conflict / capacity の policy manifest を capability に含める
  - archive export / import

`relay` は runtime の入口として backend を受け取り、永続化の具体実装は `core` の runtime helper に寄せています。

`relay` は `storage node` とは別 binary として起動します。
ローカル開発では、`LINGONBERRY_STATE_DIR` を使って個別に起動し、必要なら運用メモに従って保存先を分けます。
`relay` は現時点で `LINGONBERRY_STATE_DIR` を共通の実行ルートとして使い、storage 固有の設定ファイル変数は持ちません。

Phase 2 の最初の切り出しは、ここから始めます。

## 実行手順

### capabilities を確認する

```bash
cargo run -p lingonberry-relay -- capabilities
```

### readiness を確認する

```bash
cargo run -p lingonberry-relay -- ready
```

### HTTP carrier を起動する

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

### publish を HTTP で送る

```bash
curl -sS -X POST http://127.0.0.1:8787/v1/objects \
  -H 'Content-Type: application/json' \
  --data-binary @fixtures/http-publish-request/minimal-request.json
```

### capability を HTTP で確認する

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
```

### publish した object を HTTP で取得する

```bash
curl -sS http://127.0.0.1:8787/v1/objects/lb:obj:example-0001
```

### publish する

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

### archive を書き出す

```bash
cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive
```

### archive を取り込む

```bash
cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive
```

### 別プロセス運用の確認

```bash
cargo run -p lingonberry-storage -- capabilities
cargo run -p lingonberry-storage -- ready
cargo run -p lingonberry-storage -- run
```

詳細は [relay / storage separation](../../docs/operations/RELAY_STORAGE_SEPARATION.md) を参照してください。
