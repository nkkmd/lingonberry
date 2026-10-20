# Storage Node Quickstart / ストレージノードの入門ガイド

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a synchronized translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は同期翻訳です。内容に差異がある場合は英語版を優先します。

**Status: v1.0.0 pre-release local-evaluation guide** | **Last updated: 2026-10-20**

## English

**Execution hold (2026-10-20):** Reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred at the maintainer’s request. Resume only after an explicit maintainer request; v1.0.0 remains unpublished.

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
8c6b48082205a3af555130eec1f3e7d2ac8811fe
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

<a id="japanese"></a>

## 日本語

**実機試験の保留（2026-10-20）:** 基準ホストでの事前検証、ディスク圧迫試験、正式72時間連続稼働試験は、管理者の指示により保留しています。明示的な再開指示を受けてから再開します。v1.0.0は未公開です。

### 1. 目的と適用範囲

このガイドは、リポジトリのチェックアウトから検証済みの`lingonberry-storage` コマンドを検証して利用できるようにする、最小限のローカル開発手順を示します。

これは本番環境へのインストール手順ではありません。導入済みの基準構成のノード、サービス用アカウントの設定、保護された環境設定ファイル、リポジトリに含まれるsystemdユニット、バックアップ、更新、ロールバック、およびリリースの検証記録については、[v1.0 単一ノード運用手順書](./V1_0_OPERATOR_RUNBOOK.md)を使用してください。

現在のストレージバイナリは、運用とストレージ操作に使うコマンド用の実行ファイルです。長時間稼働する常駐プロセスではありません。

- `lingonberry-storage run`は実行時点の状態を出力して終了します。
- `lingonberry-storage ready`は起動準備状態を評価して終了します。
- `lingonberry-relay`が長時間稼働するHTTPリレープロセスです。
- リポジトリに含まれるストレージ用systemdユニットは一度だけ起動準備を確認するユニットです。

### 2. 必要条件

次を備えた、サポート対象のLinux開発環境を使用してください。

- Git
- 現在の安定版Rustツールチェーン
- Cargo
- Rustのワークスペースに必要なビルド依存関係

ツールを利用できることを確認します。

```bash
 git --version
 rustc --version
 cargo --version
```

正式な基準環境は[対応環境](./SUPPORTED_PLATFORMS.md)で定義されています。

### 3. リポジトリを複製する

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
```

リリースの適格性検証では、`main`が公開候補であると仮定しないでください。適格性検証文書で指定された正確なリビジョンを使用します。この入門ガイドでは、現在のチェックアウトをローカル評価にのみ使用します。

### 4. ストレージパッケージをビルドおよび試験する

```bash
cargo build -p lingonberry-storage
cargo test -p lingonberry-storage
```

リリースモードのローカルバイナリを作成する場合は、次を実行します。

```bash
cargo build --release -p lingonberry-storage
```

以下の開発例では`cargo run`を使用します。導入済みのノードの手順では、任意のチェックアウトから暗黙にビルドし直すのではなく、確認済み公開用バイナリを使用しなければなりません。

### 5. 分離されたローカルディレクトリを作成する

この入門ガイドが既存ノードを変更できないように、評価後に削除できる専用ディレクトリ一式を使用します。

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

このガイドでは、すべてのディレクトリを明示できるため、ストレージ固有の変数を推奨します。設定の優先順位は次のとおりです。

```text
既定値
→ 設定ファイル
→ 環境変数
→ CLIオプション
```

完全な仕様については[ストレージノードの実行時仕様](./STORAGE_NODE_RUNTIME.md)を参照してください。

### 6. 利用できるコマンドを確認する

```bash
cargo run -p lingonberry-storage -- capabilities
```

出力は正規化済みのJSONであり、ストレージサービスと実装済み操作を識別できる必要があります。

### 7. 実際に適用される設定を確認する

```bash
cargo run -p lingonberry-storage -- config
```

実際に適用された値が`$LB_STORAGE_ROOT`配下を指していることを確認します。

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

コマンドの前に、一時的な設定を一時的に上書きするCLIオプションを指定することもできます。

```bash
cargo run -p lingonberry-storage -- \
  --data-dir "$LB_STORAGE_ROOT/alternate-data" \
  config
```

CLIオプションによる上書きは環境変数より優先されます。

### 8. `status`と`run`を理解する

```bash
cargo run -p lingonberry-storage -- status
cargo run -p lingonberry-storage -- run
```

どちらのコマンドもその時点の状態を出力して終了します。常駐サービスを開始せず、ネットワークポートで待ち受けず、厳密なストレージ整合性を証明するものでもありません。

`run`が稼働し続けることを期待して端末を開いたままにしないでください。

### 9. 診断を実行する

#### 9.1 プロセスの正常性

```bash
cargo run -p lingonberry-storage -- health
```

`health`はコマンドプロセスを実行できることを確認します。ストレージの受付準備状態は検査しません。

#### 9.2 読み取り専用の診断レポート

```bash
cargo run -p lingonberry-storage -- doctor
```

新しく作成した空のワークスペースでは、空の保存形式、ログまたはカタログファイルの欠如、空のバックアップ一覧などに対する警告が想定されます。警告だけでは`doctor`は失敗しません。

失敗した検査項目を無視してはいけません。これは、破損または未対応のストレージ、無効なファイル種別、シンボリックリンクの拒否、不整合なインデックス状態、または容量検査の失敗などを示します。

#### 9.3 起動準備状態

```bash
cargo run -p lingonberry-storage -- ready
```

`ready`は`doctor`の診断結果に失敗した検査項目が含まれる場合にのみ失敗します。そのため、警告だけが存在する空のローカルワークスペースでも受付準備が整ったと判定される場合があります。

これは、リポジトリに含まれる一度だけ実行するユニットが使用するコマンドです。

```text
deploy/systemd/lingonberry-storage-ready.service
```

#### 9.4 厳密な検証

```bash
cargo run -p lingonberry-storage -- verify
```

`verify`は警告も0以外の終了コードとして扱います。まっさらな空のワークスペースは、ストレージ状態と必要な運用成果物が初期化されるまで厳密な検証を通過しない場合があります。

次の違いを意図的に使い分けてください。

```text
doctor = 警告だけなら成功
ready  = 失敗した検査項目があれば失敗
verify = 警告または失敗した検査項目があれば失敗
```

### 10. 読み取り専用のストレージコマンドを実行する

```bash
cargo run -p lingonberry-storage -- list
cargo run -p lingonberry-storage -- replay
```

空のワークスペースでは、これらのコマンドはサービスを開始するのではなく、空の結果を表す正規化済みのJSONを返す必要があります。

`retrieve`には正規IDが必要です。

```bash
cargo run -p lingonberry-storage -- retrieve 'lb:obj:example'
```

オブジェクトが存在しないことによる「見つからない」という結果は想定される結果であり、実行環境が破損している証拠ではありません。

### 11. 追記の簡易動作確認（任意）

`append`は状態を変更するコマンドであり、有効な公開要求を記述したJSONファイルが必要です。確認済みサンプルデータを使用するか、現在のプロトコル仕様に適合する分離された試験要求を作成してください。

現在のチェックアウトに適した、リポジトリに含まれるサンプルデータを使用する例は次のとおりです。

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

この入門ガイドから既存の本番ディレクトリに対して、`append`、復元、移行、インデックス再構築、または復旧訓練コマンドを実行しないでください。

### 12. 運用指標の現時点の値

```bash
cargo run -p lingonberry-storage -- metrics
```

このコマンドは、実行時点の運用指標を、項目の種類数に上限を設けて出力します。継続的に提供される運用指標の取得先ではありません。

### 13. 後片付け

ローカル評価後は、このガイドで作成した破棄可能なディレクトリだけを削除します。

```bash
printf 'quickstart root: %s\n' "$LB_STORAGE_ROOT"
rm -rf -- "$LB_STORAGE_ROOT"
unset LB_STORAGE_ROOT
unset LINGONBERRY_STORAGE_STATE_DIR
unset LINGONBERRY_STORAGE_DATA_DIR
unset LINGONBERRY_STORAGE_BACKUP_DIR
unset LINGONBERRY_STORAGE_TEMP_DIR
```

`rm -rf`を実行する前に、表示されたパスを確認し、この入門ガイドで作成した一時ディレクトリであることを確かめてください。`/var/lib/lingonberry`や`/var/backups/lingonberry`などの導入済みのノードのパスへ置き換えてはいけません。

### 14. 導入済みのノードへ移行する

基準構成で導入済みのノードでは`cargo run`の使用を終了し、運用手順書に従ってください。導入構成では次を使用します。

- `/usr/local/bin`配下のリリースモードバイナリ
- サービス用アカウント `lingonberry:lingonberry`
- 保護された環境設定ファイル `/etc/lingonberry/storage.env`
- `/var/lib/lingonberry`および`/var/backups/lingonberry`配下の永続保存先
- リポジトリに含まれるsystemdユニットのテンプレート
- 明示的なバックアップ、移行、検証、およびロールバック手順

`lingonberry-storage`をネットワークへ直接公開しないでください。公開するHTTPサービスはリレーと、確認済みのリバースプロキシ構成が担当します。

### 15. リリースの状況と適用範囲

このガイドは、現在のv1.0.0プレリリース実装を説明します。v1.0.0が公開済みであること、正式な72時間連続稼働試験が完了したこと、または管理者権限で行う基準ホスト適格性検証が完了したことを示すものではありません。

指定されたバージョン更新前の公開候補は引き続き次のコミットです。

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

この公開候補以後の文書および検証記録コミットは、公開候補を再定義しません。

### 参照文書

- [v1.0 単一ノード運用手順書](./V1_0_OPERATOR_RUNBOOK.md)
- [ストレージノードの実行時仕様](./STORAGE_NODE_RUNTIME.md)
- [運用CLIの仕様](./OPERATOR_CLI_CONTRACT.md)
- [systemdユニットのテンプレートと仕様](./SYSTEMD_UNIT_TEMPLATES.md)
- [対応環境](./SUPPORTED_PLATFORMS.md)
- [保存形式の移行と更新の仕様](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 更新とロールバックの手順](./V1_0_UPGRADE_AND_ROLLBACK.md)
