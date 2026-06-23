# Container Execution Templates

**Status: draft** | **Last updated: 2026-06-23**

## 目的

この文書は、`relay` と `storage node` を container で配布・起動する場合の最小実行例をまとめます。  
`Node Lifecycle Runbook` は手順の正本として保ち、この文書は container 実行の再利用用テンプレートとして扱います。

## 1. Storage node

```bash
docker run --rm \
  -e LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage \
  -v /var/lib/lingonberry/storage:/var/lib/lingonberry/storage \
  <storage-image> lingonberry-storage ready
```

## 2. Relay

```bash
docker run --rm \
  -e LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay \
  -p 8787:8787 \
  <relay-image> lingonberry-relay serve-http 0.0.0.0:8787
```

## 3. Caddy

```bash
docker run --rm \
  -p 80:80 \
  -p 443:443 \
  -v /etc/caddy/Caddyfile:/etc/caddy/Caddyfile:ro \
  -v caddy_data:/data \
  -v caddy_config:/config \
  <caddy-image> caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
```

## 4. 使い方

- `storage node` と `relay` は別コンテナとして扱う
- `LINGONBERRY_STATE_DIR` は手動起動と合わせる共通の実行ルートとして使う
- `LINGONBERRY_STORAGE_CONFIG` は storage node の設定位置を明示するときだけ使う
- `ready` は起動確認用、`capabilities` は機能確認用として使う
- volume mount は state を失わない場所に向ける
- `Caddy` は `relay` の前段に置き、公開ポートと内向きポートを分ける
- `Caddy` の設定ファイルは read-only mount で渡す

## 参照

- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
