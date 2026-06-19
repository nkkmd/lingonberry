# Container Execution Templates

**Status: draft** | **Last updated: 2026-06-19**

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

## 3. 使い方

- `storage node` と `relay` は別コンテナとして扱う
- `LINGONBERRY_STATE_DIR` は手動起動と合わせる
- `ready` は起動確認用、`capabilities` は機能確認用として使う
- volume mount は state を失わない場所に向ける

## 参照

- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
