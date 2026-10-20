# Relay Quickstart / リレーの入門ガイド

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

**Status: v1.0.0 documentation normalization** | **Last updated: 2026-10-20**

## English

**Execution hold (2026-10-20):** Reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred at the maintainer’s request. Resume only after an explicit maintainer request; v1.0.0 remains unpublished.

### Purpose

This quickstart takes a first-time contributor from cloning the repository to starting the Lingonberry relay with `cargo run`. It prioritizes the shortest development path. Production installation, relay/storage separation, systemd deployment, and recovery procedures are documented separately.

### Prerequisites

- Git
- a current Rust toolchain
- Cargo
- `curl` or another HTTP client

When Rust is not installed, use the official `rustup` installer.

### 1. Clone the repository

HTTPS:

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

SSH may be used when your GitHub SSH key is configured:

```bash
git clone git@github.com:nkkmd/lingonberry.git lingonberry
cd lingonberry
```

### 2. Install Rust when needed

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Verify the installation:

```bash
rustc --version
cargo --version
```

Skip this section when a suitable Rust toolchain is already installed.

### 3. Confirm the workspace

From the repository root:

```bash
cargo metadata --no-deps
```

This should complete successfully and list the workspace packages.

### 4. Inspect relay capabilities

```bash
cargo run -p lingonberry-relay -- capabilities
```

The command must exit successfully before starting the HTTP listener.

### 5. Start the HTTP relay

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

`serve-http` remains in the foreground. Leave that terminal open.

### 6. Check capabilities and readiness

Open another terminal:

```bash
curl -fsS http://127.0.0.1:8787/v1/capabilities
curl -fsS http://127.0.0.1:8787/v1/ready
```

A nonzero `curl` exit status, connection refusal, or non-success HTTP response means the quickstart has not passed.

### 7. Try a minimal publish

With the relay available, use the repository fixture:

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

For a complete publish walkthrough, see [Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md).

### 8. Optional archive exercise

```bash
cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive
cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive
```

Use only disposable development paths for this exercise.

### 9. Stop the relay

Return to the terminal running `serve-http` and press `Ctrl+C`.

This development shutdown path is not the production systemd lifecycle. For production-oriented operation, use the operator runbook and systemd units.

### 10. Reverse-proxy publication

When exposing the relay externally, place Caddy or another approved reverse proxy in front of the relay. Validate the public endpoint rather than treating the loopback listener as the public interface.

```bash
curl -fsS https://<public-host>/v1/capabilities
curl -fsS https://<public-host>/v1/ready
```

Do not expose the storage node directly through this quickstart. See [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md) and [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md).

### Troubleshooting

- `cargo: command not found`: run `source "$HOME/.cargo/env"`, then retry.
- bind failure on `127.0.0.1:8787`: check whether another process is already using the port.
- long first build: the initial run may download and compile dependencies.
- readiness failure: inspect the relay terminal output before retrying.
- public Caddy failure: check the public hostname, TLS certificate, reverse-proxy target, and firewall rather than only the relay loopback URL.

### Production boundary

This quickstart is for source-based development. It does not qualify a production deployment. The formal reference platform is Ubuntu Server 24.04 LTS, x86_64, systemd, using release-built binaries and hardened units.

Read next:

- [Operations Index](./README.md)
- [v0.8.0 Operator Runbook](./V0_8_OPERATOR_RUNBOOK.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)

---

<a id="japanese"></a>

## 日本語

**実機試験の保留（2026-10-20）:** 基準ホストでの事前検証、ディスク圧迫試験、正式72時間連続稼働試験は、管理者の指示により保留しています。明示的な再開指示を受けてから再開します。v1.0.0は未公開です。

### 目的

この入門ガイドは、初めてLingonberryを扱う開発参加者が、リポジトリの複製から`cargo run`によるリレー起動までを確認するための手順です。開発環境で最短の起動確認を行うことを優先します。本番環境への導入、リレー／ストレージ分離、systemdによる配置、復旧手順は別文書で扱います。

### 前提条件

- Git
- 最新のRustツールチェーン
- Cargo
- `curl`または同等のHTTPクライアント

Rustが未導入の場合は、公式の`rustup` インストーラーを使用します。

### 1. リポジトリを複製する

HTTPS:

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

GitHub SSH鍵を設定済みの場合はSSHも使用できます。

```bash
git clone git@github.com:nkkmd/lingonberry.git lingonberry
cd lingonberry
```

### 2. 必要な場合はRustを導入する

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

導入を確認します。

```bash
rustc --version
cargo --version
```

利用可能なRustツールチェーンが既にある場合、この節は省略できます。

### 3. ワークスペースを確認する

リポジトリのルートで実行します。

```bash
cargo metadata --no-deps
```

正常終了し、ワークスペース内のパッケージが表示されることを確認します。

### 4. リレーの提供機能を確認する

```bash
cargo run -p lingonberry-relay -- capabilities
```

HTTP待受処理を起動する前に、このコマンドが正常終了する必要があります。

### 5. HTTPリレーを起動する

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

`serve-http`は端末を占有して動作します（フォアグラウンド実行）。実行中はこの端末を開いたままにしてください。

### 6. 提供機能と受付準備状態を確認する

別の端末を開きます。

```bash
curl -fsS http://127.0.0.1:8787/v1/capabilities
curl -fsS http://127.0.0.1:8787/v1/ready
```

`curl`が0以外の終了コードで終了する、接続拒否になる、または成功以外のHTTP応答になる場合、この手順の動作確認は完了していません。

### 7. 最小構成でデータを公開する

リレーが利用可能な状態でリポジトリ付属のサンプルデータを使用します。

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

完全な公開手順は[知識オブジェクトの公開入門](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)を参照してください。

### 8. アーカイブの確認（任意）

```bash
cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive
cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive
```

この確認には不要になれば削除できる開発用の保存先だけを使用してください。

### 9. リレーを停止する

`serve-http`を実行している端末へ戻り、`Ctrl+C`を押します。

これは開発環境での停止方法です。本番環境ではsystemdで起動・停止・再起動を管理します。本番環境向け運用では運用手順書とsystemdユニットを使用します。

### 10. リバースプロキシを使った公開

リレーを外部公開する場合は、Caddyまたは承認済みリバースプロキシをリレーの前段へ配置します。ループバック待受を公開インターフェースとして扱わず、公開エンドポイントを確認します。

```bash
curl -fsS https://<public-host>/v1/capabilities
curl -fsS https://<public-host>/v1/ready
```

この入門ガイドでストレージノードを直接公開してはいけません。[Caddyを使ったリレーの公開](./CADDY_RELAY_PUBLICATION.md)と[リレーとストレージの分離](./RELAY_STORAGE_SEPARATION.md)を参照してください。

### トラブルシューティング

- `cargo: command not found`: `source "$HOME/.cargo/env"`を実行して再試行する
- `127.0.0.1:8787`のポートを確保できない: 他プロセスがポートを使用していないか確認する
- 初回ビルドが長い: 最初の実行では依存パッケージのダウンロードとコンパイルが発生する場合がある
- 受付準備状態の確認に失敗する: 再試行前にリレーを起動した端末の出力を確認する
- Caddy経由の公開に失敗する: リレーのループバックURLだけでなく、公開ホスト名、TLS証明書、リバースプロキシの転送先、ファイアウォールを確認する

### 本番運用への適用範囲

この入門ガイドはソースコードを使った開発向けであり、本番環境への導入が適切であると確認するためのものではありません。正式な基準環境はUbuntu Server 24.04 LTS、x86_64、systemdで、リリース用にビルドされたバイナリと安全対策を施したユニットを使用します。

次に読む文書:

- [文書案内](./README.md)
- [v0.8.0 運用手順書](./V0_8_OPERATOR_RUNBOOK.md)
- [対応環境](./SUPPORTED_PLATFORMS.md)
- [ノードの起動から停止までの運用手順](./NODE_LIFECYCLE_RUNBOOK.md)
- [リレーとストレージの分離](./RELAY_STORAGE_SEPARATION.md)
