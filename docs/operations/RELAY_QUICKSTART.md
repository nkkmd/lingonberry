# Relay Quickstart

[English](#english) | [日本語](#日本語)

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

**Status: v1.0.0 documentation normalization** | **Last updated: 2026-07-23**

## English

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

## 日本語

### 目的

このquickstartは、初めてLingonberryを扱うcontributorが、repositoryのcloneから`cargo run`によるrelay起動までを確認するための手順です。開発環境で最短の起動確認を行うことを優先します。production installation、relay／storage分離、systemd deployment、recovery procedureは別文書で扱います。

### 前提条件

- Git
- 現在のRust toolchain
- Cargo
- `curl`または同等のHTTP client

Rustが未導入の場合は、公式の`rustup` installerを使用します。

### 1. Repositoryをcloneする

HTTPS:

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

GitHub SSH keyを設定済みの場合はSSHも使用できます。

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

利用可能なRust toolchainが既にある場合、このsectionは不要です。

### 3. Workspaceを確認する

repository rootで実行します。

```bash
cargo metadata --no-deps
```

正常終了し、workspace packageが表示されることを確認します。

### 4. Relay capabilityを確認する

```bash
cargo run -p lingonberry-relay -- capabilities
```

HTTP listenerを起動する前に、このcommandが正常終了する必要があります。

### 5. HTTP relayを起動する

```bash
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

`serve-http`はforegroundで動作します。このterminalは開いたままにします。

### 6. Capabilityとreadinessを確認する

別のterminalを開きます。

```bash
curl -fsS http://127.0.0.1:8787/v1/capabilities
curl -fsS http://127.0.0.1:8787/v1/ready
```

`curl`が非zeroで終了する、connection refusedになる、またはsuccess以外のHTTP responseになる場合、quickstartは未達です。

### 7. 最小publishを試す

relayが利用可能な状態でrepository fixtureを使用します。

```bash
cargo run -p lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
```

完全なpublish手順は[Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)を参照してください。

### 8. 任意のarchive確認

```bash
cargo run -p lingonberry-relay -- export-archive /tmp/lingonberry-archive
cargo run -p lingonberry-relay -- import-archive /tmp/lingonberry-archive
```

この確認には破棄可能な開発用pathだけを使用してください。

### 9. Relayを停止する

`serve-http`を実行しているterminalへ戻り、`Ctrl+C`を押します。

これは開発用の停止方法であり、production systemd lifecycleではありません。production向け運用ではoperator runbookとsystemd unitを使用します。

### 10. Reverse proxyを使った公開

relayを外部公開する場合は、Caddyまたは承認済みreverse proxyをrelayの前段へ配置します。loopback listenerを公開interfaceとして扱わず、public endpointを確認します。

```bash
curl -fsS https://<public-host>/v1/capabilities
curl -fsS https://<public-host>/v1/ready
```

このquickstartでstorage nodeを直接公開してはいけません。[Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)と[Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)を参照してください。

### トラブルシューティング

- `cargo: command not found`: `source "$HOME/.cargo/env"`を実行して再試行する
- `127.0.0.1:8787`のbind失敗: 他processがportを使用していないか確認する
- 初回buildが長い: 最初の実行ではdependencyのdownloadとcompileが発生する場合がある
- readiness失敗: 再試行前にrelay terminalの出力を確認する
- 公開Caddyの失敗: relayのloopback URLだけでなく、public hostname、TLS certificate、reverse-proxy target、firewallを確認する

### Production境界

このquickstartはsource-based development向けであり、production deploymentを資格確認するものではありません。正式reference platformはUbuntu Server 24.04 LTS、x86_64、systemdで、release build済みbinaryとhardened unitを使用します。

次に読む文書:

- [Operations Index](./README.md)
- [v0.8.0 Operator Runbook](./V0_8_OPERATOR_RUNBOOK.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
