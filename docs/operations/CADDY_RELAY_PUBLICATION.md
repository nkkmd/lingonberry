# Caddy Relay Publication

**Status: draft** | **Last updated: 2026-06-23**

## 目的

この文書は、`relay` を外部公開するときに、`Caddy` を前段の reverse proxy として使うための最小方針をまとめます。  
ここでの `Caddy` は protocol carrier ではなく、公開境界を担う deployment / exposure レイヤです。

`relay` は HTTP carrier の実装として振る舞い、`storage node` は内向きの永続化基盤として残します。  
`Caddy` は `relay` への公開入口を提供し、`storage node` を直接公開しません。

## 役割分担

- `Caddy`
  - TLS 終端
  - host / path routing
  - 公開 URL の安定化
- `relay`
  - HTTP carrier の実装
  - `capabilities` / `ready` / `publish` / `retrieve` の受け口
  - protocol object の validation / routing
- `storage node`
  - 永続化
  - replay
  - export / import

## 推奨配置

- 公開されるのは `Caddy` の URL
- `relay` は同一ホストのローカルポート、または内部ネットワークで待ち受ける
- `storage node` は `Caddy` から直接到達できない場所に置く

## 公開方針

### local / staging

- `relay` は `127.0.0.1:8787` などのローカル待受けにする
- `Caddy` はローカル reverse proxy として動かす
- 証明書の試験が必要な場合は、まず staging の公開ホスト名で確認する

### production

- 公開ホスト名を先に決めてから `Caddyfile` を書く
- TLS 終端は `Caddy` に寄せる
- `relay` は内向きポートのままにする
- `storage node` は公開面から完全に外す
- 変更は `caddy validate` を通してから反映する

## 最小構成例

以下は、`relay` を `127.0.0.1:8787` で起動し、`Caddy` から外向きに公開する例です。

```caddyfile
example.org {
	reverse_proxy 127.0.0.1:8787
}
```

必要なら、`/v1/*` だけを `relay` に流す構成にもできます。

```caddyfile
example.org {
	handle_path /v1/* {
		reverse_proxy 127.0.0.1:8787
	}

	handle {
		respond "Lingonberry relay" 200
	}
}
```

## 運用テンプレート

本番運用では、`Caddy` の設定を `/etc/caddy/Caddyfile` に置き、`relay` をローカルポートで待ち受けさせるのが扱いやすいです。  
`Caddy` の公開ポートは通常 `80` / `443`、`relay` の待受けは `127.0.0.1:8787` などの内向きポートに分けます。

```caddyfile
example.org {
	log {
		output stdout
		format console
	}

	encode zstd gzip

	reverse_proxy 127.0.0.1:8787
}
```

`relay` を複数ホストで切り替える運用では、`Caddy` の upstream を明示しておくと扱いやすいです。

```caddyfile
relay.example.org {
	log {
		output stdout
		format console
	}

	encode zstd gzip

	reverse_proxy 127.0.0.1:8787
}
```

HTTPS を前提にする場合は、`Caddy` の自動証明書管理を使い、公開ホスト名を DNS で解決できるようにします。  
社内向けや閉域向けで自動証明書を使わない場合は、`tls internal` や管理下の証明書を使う構成に切り替えます。

```caddyfile
relay.example.org {
	log {
		output stdout
		format console
	}

	encode zstd gzip

	reverse_proxy 127.0.0.1:8787
	tls internal
}
```

設定を更新したら、まず validate してから reload します。

```bash
caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile
caddy reload --config /etc/caddy/Caddyfile --adapter caddyfile
```

## 確認手順

1. `relay` を起動する
2. `Caddy` を起動する
3. `Caddy` の公開 URL で `GET /v1/capabilities` を確認する
4. `Caddy` の公開 URL で `GET /v1/ready` を確認する
5. 必要なら `POST /v1/objects` を送って publish を確認する
6. `storage node` は内部の `ready`、`config`、`replay`、`list` で確認する

## 運用上の注意

- `Caddy` の設定変更は protocol 仕様の変更とはみなさない
- `Caddy` の証明書やホスト名は deployment の責務として管理する
- `Caddy` のホスト名は relay の公開 URL と一致させる
- HTTP から HTTPS への誘導は Caddy に寄せる
- `relay` と `storage node` の責務境界を文書上で混ぜない
- 公開面の確認は `Caddy` の URL に寄せる
- 内部確認は `relay` と `storage node` の CLI / local endpoint で行う

## 関連

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)
- [Container Execution Templates](./CONTAINER_EXECUTION_TEMPLATES.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [relay / storage separation](./RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
