# Systemd Unit Templates

**Status: draft** | **Last updated: 2026-06-23**

## 目的

この文書は、`relay` と `storage node` を systemd で併設運用する場合の最小 unit 例をまとめます。  
`Node Lifecycle Runbook` は手順の正本として保ち、この文書は unit の再利用用テンプレートとして扱います。

## 1. Storage node

```ini
[Unit]
Description=Lingonberry Storage Node

[Service]
Environment=LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage
ExecStart=/usr/local/bin/lingonberry-storage ready
KillSignal=SIGTERM
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## 2. Relay

```ini
[Unit]
Description=Lingonberry Relay

[Service]
Environment=LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
ExecStart=/usr/local/bin/lingonberry-relay serve-http 0.0.0.0:8787
KillSignal=SIGTERM
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## 3. Caddy

```ini
[Unit]
Description=Caddy for Lingonberry Relay
After=network-online.target
Wants=network-online.target

[Service]
ExecStart=/usr/bin/caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
ExecReload=/usr/bin/caddy reload --config /etc/caddy/Caddyfile --adapter caddyfile
KillSignal=SIGTERM
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## 4. 使い方

- `ExecStart` は runbook の手動起動コマンドと合わせる
- `KillSignal` は `SIGTERM` を基本とする
- `Restart` は `on-failure` を基本とする
- unit は `storage` と `relay` で分ける
- `Caddy` は `relay` とは別 unit として管理する
- `Caddy` の unit は `relay` の内向き待受け先を reverse proxy する
- `LINGONBERRY_STATE_DIR` は共通の実行ルートとして使う
- `LINGONBERRY_STORAGE_CONFIG` は storage node の設定位置を明示するときだけ使う

## 参照

- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
