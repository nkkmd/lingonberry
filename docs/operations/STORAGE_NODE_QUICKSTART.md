# Storage node local quickstart / Storage nodeローカルquickstart

**Status: v1.0.0 pre-release local-evaluation guide / 状態: v1.0.0プレリリース・ローカル評価ガイド** | **Last updated / 最終更新: 2026-07-27**

English is normative for this document. The Japanese section is a synchronized translation.

この文書では英語を正本とします。日本語部分は同期された翻訳です。

## English

### 1. Purpose and boundary

This guide provides a minimal local-development path from a repository checkout to a verified `lingonberry-storage` command environment.

It is not the production installation procedure. For an installed reference node, service-account setup, protected environment files, checked-in systemd units, backup, upgrade, rollback, and release evidence, use [`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md).

The current storage binary is an operator and storage command binary. It is not a long-running daemon:

- `lingonberry-storage run` prints a runtime snapshot and exits;
- `lingonberry-storage ready` evaluates startup readiness and exits;
- `lingonberry-relay` is the long-running HTTP relay process;
- the checked-in storage systemd unit is a oneshot readiness gate.

### 2. Requirements

Use a supported Linux development environment with:

- Git;
- a current stable Rust toolchain;
- Cargo;
- the build dependencies required by the Rust workspace.

Confirm the tools are available:

```bash
 git --version
 rustc --version
 cargo --version
```

The formal reference platform is defined in [`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md).

### 3. Clone the repository

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

For release qualification, do not assume that `main` is the release candidate. Use the exact revision named by the qualification documentation. This quickstart uses the current checkout only for local evaluation.

### 4. Build and test the storage package

```bash
cargo build -p lingonberry-storage
cargo test -p lingonberry-storage
```

For a release-mode local binary:

```bash
cargo build --release -p lingonberry-storage
```

The development examples below use `cargo run`. An installed-node procedure must use reviewed release binaries rather than an implicit rebuild from an arbitrary checkout.

### 5. Create isolated local directories

Use a disposable directory tree so the quickstart cannot modify an existing node:

```bash
export LB_STORAGE_ROOT="$(mktemp -d)"
export LINGONBERRY_STORAGE_STATE_DIR="$LB_STORAGE_ROOT/state"
export LINGONBERRY_STORAGE_DATA_DIR="$LB_STORAGE_ROOT/data"
export LINGONBERRY_STORAGE_BACKUP_DIR="$LB_STORAGE_ROOT/backups"
export LINGONBERRY_STORAGE_TEMP_DIR="$LB_STORAGE_ROOT/tmp"

mkdir -p \
  "$LINGONBERRY_STORAGE_STATE_DIR" \
  "$LINGONBERRY_STORAGE_DATA_DIR" \
  "$LINGONBERRY_STORAGE_BACKUP_DIR" \
  "$LINGONBERRY_STORAGE_TEMP_DIR"
```

The storage-specific variables are preferred for this guide because they make every directory explicit. Configuration precedence is:

```text
defaults
→ config file
→ environment variables
→ CLI options
```

See [`STORAGE_NODE_RUNTIME.md`](./STORAGE_NODE_RUNTIME.md) for the full contract.

### 6. Inspect the command surface

```bash
cargo run -p lingonberry-storage -- capabilities
```

The output is canonical JSON and should identify the storage service and implemented operations.

### 7. Inspect effective configuration

```bash
cargo run -p lingonberry-storage -- config
```

Confirm that the resolved values point under `$LB_STORAGE_ROOT`:

```text
stateDir
dataDir
backupDir
tempDir
rawLogPath
catalogPath
```

`rawLogPath` and `catalogPath` are derived from `dataDir`:

```text
<dataDir>/relay-wire-log.jsonl
<dataDir>/canonical-catalog.sqlite3
```

You can also pass one-off CLI overrides before the command:

```bash
cargo run -p lingonberry-storage -- \
  --data-dir "$LB_STORAGE_ROOT/alternate-data" \
  config
```

CLI overrides have higher precedence than environment variables.

### 8. Understand `status` and `run`

```bash
cargo run -p lingonberry-storage -- status
cargo run -p lingonberry-storage -- run
```

Both commands print snapshots and exit. Neither command starts a resident service, listens on a network port, or proves strict storage integrity.

Do not keep a terminal open expecting `run` to remain active.

### 9. Run diagnostics

#### 9.1 Process-level health

```bash
cargo run -p lingonberry-storage -- health
```

`health` confirms that the command process can run. It does not inspect storage readiness.

#### 9.2 Read-only doctor report

```bash
cargo run -p lingonberry-storage -- doctor
```

On a newly created empty workspace, warnings are expected for items such as an empty storage format, missing log or catalog files, or an empty backup inventory. Warnings do not make `doctor` fail.

Failed checks must not be ignored. They indicate conditions such as corrupt or unsupported storage, invalid file types, symlink rejection, inconsistent index state, or failed capacity checks.

#### 9.3 Startup readiness

```bash
cargo run -p lingonberry-storage -- ready
```

`ready` fails only when the doctor report contains a failed check. A warning-only empty local workspace may therefore be ready.

This is the command used by the checked-in oneshot unit:

```text
deploy/systemd/lingonberry-storage-ready.service
```

#### 9.4 Strict verification

```bash
cargo run -p lingonberry-storage -- verify
```

`verify` treats warnings as non-zero. A pristine empty workspace may not pass strict verification until the storage state and required operational artifacts have been initialized.

Use this distinction deliberately:

```text
doctor = warnings allowed
ready  = failed checks rejected
verify = warnings and failed checks rejected
```

### 10. Exercise read-only storage commands

```bash
cargo run -p lingonberry-storage -- list
cargo run -p lingonberry-storage -- replay
```

On an empty workspace, these commands should return canonical JSON representing an empty result rather than starting a service.

`retrieve` requires a canonical ID:

```bash
cargo run -p lingonberry-storage -- retrieve 'lb:obj:example'
```

A missing object is an expected not-found failure, not proof of runtime corruption.

### 11. Optional append smoke test

`append` is mutating and requires a valid publish-request JSON file. Use a reviewed fixture or create an isolated test request that conforms to the current protocol contract.

Example with a checked-in fixture when appropriate for the current checkout:

```bash
cargo run -p lingonberry-storage -- \
  append fixtures/http-publish-request/minimal-request.json
```

Then inspect the result:

```bash
cargo run -p lingonberry-storage -- list
cargo run -p lingonberry-storage -- replay
cargo run -p lingonberry-storage -- doctor
```

Do not run `append`, restore, migration, index rebuild, or drill commands against an existing production directory from this quickstart.

### 12. Metrics snapshot

```bash
cargo run -p lingonberry-storage -- metrics
```

This emits a bounded-cardinality command snapshot. It is not a continuously served metrics endpoint.

### 13. Cleanup

After local evaluation, remove only the disposable directory created by this guide:

```bash
printf 'quickstart root: %s\n' "$LB_STORAGE_ROOT"
rm -rf -- "$LB_STORAGE_ROOT"
unset LB_STORAGE_ROOT
unset LINGONBERRY_STORAGE_STATE_DIR
unset LINGONBERRY_STORAGE_DATA_DIR
unset LINGONBERRY_STORAGE_BACKUP_DIR
unset LINGONBERRY_STORAGE_TEMP_DIR
```

Before running `rm -rf`, inspect the printed path and confirm that it is the temporary quickstart root. Never substitute an installed-node path such as `/var/lib/lingonberry` or `/var/backups/lingonberry`.

### 14. Moving to an installed node

For a reference installed node, stop using `cargo run` and follow the operator runbook. The installed model uses:

- release-mode binaries under `/usr/local/bin`;
- service account `lingonberry:lingonberry`;
- protected environment file `/etc/lingonberry/storage.env`;
- durable paths under `/var/lib/lingonberry` and `/var/backups/lingonberry`;
- checked-in systemd unit templates;
- explicit backup, migration, verification, and rollback procedures.

Do not expose `lingonberry-storage` directly to the network. Public HTTP service belongs to the relay and its reviewed reverse-proxy boundary.

### 15. Release boundary

This guide describes the current v1.0.0 pre-release implementation. It does not indicate that v1.0.0 has been published, that the formal 72-hour soak has completed, or that privileged reference-host qualification has completed.

The designated pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Documentation and evidence commits after that candidate do not redefine it.

### References

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Storage Node Runtime Contract](./STORAGE_NODE_RUNTIME.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)

---

## 日本語

### 1. 目的と適用範囲

このガイドは、リポジトリのcheckoutから検証済みの`lingonberry-storage` command環境までを構築する、最小限のローカル開発手順を示します。

これは本番環境へのinstall手順ではありません。install済みreference node、service account設定、保護されたenvironment file、リポジトリに含まれるsystemd unit、backup、upgrade、rollback、およびrelease evidenceについては、[`V1_0_OPERATOR_RUNBOOK.md`](./V1_0_OPERATOR_RUNBOOK.md)を使用してください。

現在のstorage binaryはoperatorおよびstorage command用のbinaryです。長時間稼働するdaemonではありません。

- `lingonberry-storage run`はruntime snapshotを出力して終了します。
- `lingonberry-storage ready`はstartup readinessを評価して終了します。
- `lingonberry-relay`が長時間稼働するHTTP relay processです。
- リポジトリに含まれるstorage systemd unitはoneshot readiness gateです。

### 2. 必要条件

次を備えた、サポート対象のLinux開発環境を使用してください。

- Git
- 現行のstable Rust toolchain
- Cargo
- Rust workspaceに必要なbuild dependency

toolを利用できることを確認します。

```bash
 git --version
 rustc --version
 cargo --version
```

正式なreference platformは[`SUPPORTED_PLATFORMS.md`](./SUPPORTED_PLATFORMS.md)で定義されています。

### 3. リポジトリをcloneする

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

release qualificationでは、`main`がrelease candidateであると仮定しないでください。qualification文書で指定された正確なrevisionを使用します。このquickstartでは、現在のcheckoutをローカル評価にのみ使用します。

### 4. Storage packageをbuildおよびtestする

```bash
cargo build -p lingonberry-storage
cargo test -p lingonberry-storage
```

release modeのローカルbinaryを作成する場合は、次を実行します。

```bash
cargo build --release -p lingonberry-storage
```

以下の開発例では`cargo run`を使用します。install済みnodeの手順では、任意のcheckoutから暗黙にrebuildするのではなく、review済みrelease binaryを使用しなければなりません。

### 5. 分離されたローカルdirectoryを作成する

このquickstartが既存nodeを変更できないように、破棄可能なdirectory treeを使用します。

```bash
export LB_STORAGE_ROOT="$(mktemp -d)"
export LINGONBERRY_STORAGE_STATE_DIR="$LB_STORAGE_ROOT/state"
export LINGONBERRY_STORAGE_DATA_DIR="$LB_STORAGE_ROOT/data"
export LINGONBERRY_STORAGE_BACKUP_DIR="$LB_STORAGE_ROOT/backups"
export LINGONBERRY_STORAGE_TEMP_DIR="$LB_STORAGE_ROOT/tmp"

mkdir -p \
  "$LINGONBERRY_STORAGE_STATE_DIR" \
  "$LINGONBERRY_STORAGE_DATA_DIR" \
  "$LINGONBERRY_STORAGE_BACKUP_DIR" \
  "$LINGONBERRY_STORAGE_TEMP_DIR"
```

このガイドでは、すべてのdirectoryを明示できるため、storage固有のvariableを推奨します。configurationの優先順位は次のとおりです。

```text
defaults
→ config file
→ environment variables
→ CLI options
```

完全なcontractについては[`STORAGE_NODE_RUNTIME.md`](./STORAGE_NODE_RUNTIME.md)を参照してください。

### 6. Command surfaceを確認する

```bash
cargo run -p lingonberry-storage -- capabilities
```

出力はcanonical JSONであり、storage serviceと実装済みoperationを識別できる必要があります。

### 7. Effective configurationを確認する

```bash
cargo run -p lingonberry-storage -- config
```

解決された値が`$LB_STORAGE_ROOT`配下を指していることを確認します。

```text
stateDir
dataDir
backupDir
tempDir
rawLogPath
catalogPath
```

`rawLogPath`と`catalogPath`は`dataDir`から導出されます。

```text
<dataDir>/relay-wire-log.jsonl
<dataDir>/canonical-catalog.sqlite3
```

commandの前に、一時的なCLI overrideを渡すこともできます。

```bash
cargo run -p lingonberry-storage -- \
  --data-dir "$LB_STORAGE_ROOT/alternate-data" \
  config
```

CLI overrideはenvironment variableより優先されます。

### 8. `status`と`run`を理解する

```bash
cargo run -p lingonberry-storage -- status
cargo run -p lingonberry-storage -- run
```

どちらのcommandもsnapshotを出力して終了します。常駐serviceを開始せず、network portをlistenせず、厳密なstorage integrityを証明するものでもありません。

`run`が稼働し続けることを期待してterminalを開いたままにしないでください。

### 9. Diagnosticを実行する

#### 9.1 Process-level health

```bash
cargo run -p lingonberry-storage -- health
```

`health`はcommand processを実行できることを確認します。storage readinessは検査しません。

#### 9.2 Read-only doctor report

```bash
cargo run -p lingonberry-storage -- doctor
```

新しく作成した空のworkspaceでは、空のstorage format、logまたはcatalog fileの欠如、空のbackup inventoryなどに対するwarningが想定されます。warningだけでは`doctor`は失敗しません。

failed checkを無視してはいけません。これは、破損または未対応のstorage、無効なfile type、symlink rejection、不整合なindex state、またはcapacity check失敗などを示します。

#### 9.3 Startup readiness

```bash
cargo run -p lingonberry-storage -- ready
```

`ready`はdoctor reportにfailed checkが含まれる場合にのみ失敗します。そのため、warningだけが存在する空のローカルworkspaceはreadyになる場合があります。

これは、リポジトリに含まれるoneshot unitが使用するcommandです。

```text
deploy/systemd/lingonberry-storage-ready.service
```

#### 9.4 Strict verification

```bash
cargo run -p lingonberry-storage -- verify
```

`verify`はwarningもnon-zeroとして扱います。まっさらな空のworkspaceは、storage stateと必要な運用artifactが初期化されるまでstrict verificationを通過しない場合があります。

次の違いを意図的に使い分けてください。

```text
doctor = warnings allowed
ready  = failed checks rejected
verify = warnings and failed checks rejected
```

### 10. Read-only storage commandを実行する

```bash
cargo run -p lingonberry-storage -- list
cargo run -p lingonberry-storage -- replay
```

空のworkspaceでは、これらのcommandはserviceを開始するのではなく、空の結果を表すcanonical JSONを返す必要があります。

`retrieve`にはcanonical IDが必要です。

```bash
cargo run -p lingonberry-storage -- retrieve 'lb:obj:example'
```

objectが存在しないことによるnot-found failureは想定される結果であり、runtime破損の証明ではありません。

### 11. 任意のappend smoke test

`append`は状態を変更するcommandであり、有効なpublish-request JSON fileが必要です。review済みfixtureを使用するか、現在のprotocol contractに適合する分離されたtest requestを作成してください。

現在のcheckoutに適した、リポジトリに含まれるfixtureを使用する例は次のとおりです。

```bash
cargo run -p lingonberry-storage -- \
  append fixtures/http-publish-request/minimal-request.json
```

その後、結果を確認します。

```bash
cargo run -p lingonberry-storage -- list
cargo run -p lingonberry-storage -- replay
cargo run -p lingonberry-storage -- doctor
```

このquickstartから既存の本番directoryに対して、`append`、restore、migration、index rebuild、またはdrill commandを実行しないでください。

### 12. Metrics snapshot

```bash
cargo run -p lingonberry-storage -- metrics
```

これはbounded-cardinalityのcommand snapshotを出力します。継続的に提供されるmetrics endpointではありません。

### 13. Cleanup

ローカル評価後は、このガイドで作成した破棄可能なdirectoryだけを削除します。

```bash
printf 'quickstart root: %s\n' "$LB_STORAGE_ROOT"
rm -rf -- "$LB_STORAGE_ROOT"
unset LB_STORAGE_ROOT
unset LINGONBERRY_STORAGE_STATE_DIR
unset LINGONBERRY_STORAGE_DATA_DIR
unset LINGONBERRY_STORAGE_BACKUP_DIR
unset LINGONBERRY_STORAGE_TEMP_DIR
```

`rm -rf`を実行する前に、表示されたpathを確認し、一時quickstart rootであることを確かめてください。`/var/lib/lingonberry`や`/var/backups/lingonberry`などのinstall済みnodeのpathへ置き換えてはいけません。

### 14. Install済みnodeへ移行する

reference installed nodeでは`cargo run`の使用を終了し、operator runbookに従ってください。install modelでは次を使用します。

- `/usr/local/bin`配下のrelease-mode binary
- service account `lingonberry:lingonberry`
- 保護されたenvironment file `/etc/lingonberry/storage.env`
- `/var/lib/lingonberry`および`/var/backups/lingonberry`配下のdurable path
- リポジトリに含まれるsystemd unit template
- 明示的なbackup、migration、verification、およびrollback手順

`lingonberry-storage`をnetworkへ直接公開しないでください。public HTTP serviceはrelayと、そのreview済みreverse-proxy境界が担当します。

### 15. Release境界

このガイドは、現在のv1.0.0プレリリース実装を説明します。v1.0.0が公開済みであること、formal 72-hour soakが完了したこと、またはprivileged reference-host qualificationが完了したことを示すものではありません。

指定されたpre-version candidateは引き続き次のcommitです。

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

このcandidate以後の文書およびevidence commitは、candidateを再定義しません。

### 参照文書

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Storage Node Runtime Contract](./STORAGE_NODE_RUNTIME.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
