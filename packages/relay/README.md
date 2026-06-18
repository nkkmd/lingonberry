# Relay

このディレクトリには、Lingonberry の Rust relay バイナリを置きます。

## 現在の配置

- `src/`
  - publish request の validate / publish
  - append-only log への保存
  - SQLite catalog を使う replay / get / list の最小入口
  - `serve-http` による HTTP carrier の最小実装
  - carrier capabilities の出力
  - archive export / import

Phase 2 の最初の切り出しは、ここから始めます。

## 実行手順

### capabilities を確認する

```bash
cargo run -p lingonberry-relay -- capabilities
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
