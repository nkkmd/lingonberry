# v1.0 Single-Node Operator Runbook / v1.0 単一ノード運用手順書

[English](#english) | [日本語](#japanese)

**Status: pre-release candidate**  
**Reference platform: Ubuntu Server 24.04 LTS / x86_64 / systemd**

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

**Last updated: 2026-10-20**

## English

**Execution hold (2026-10-20):** Reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred at the maintainer’s request. Resume only after an explicit maintainer request; v1.0.0 remains unpublished.

### 1. Scope and release boundary

This is the minimum single-node operator procedure intended for the Lingonberry v1.x operational contract. It covers installation, configuration validation, service lifecycle, health checks, backup, isolated restore, index verification, restart persistence, and first-line diagnosis.

The latest published release is `v0.9.0`. `v1.0.0` is still under qualification and has not been published. Until the v1.0.0 tag and GitHub Release exist, this document is a pre-release runbook and must not be treated as proof that v1.0.0 has shipped.

The designated pre-version qualification candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

Qualification-specific soak, crash-matrix, disk-pressure, and evidence procedures are maintainer workflows and are intentionally outside this operator runbook.

### 2. Safety rules

- Canonical storage is authoritative; indexes and effective views are derived and rebuildable.
- Never restore over an active state or data directory.
- Never manually edit manifests, journals, generation pointers, indexes, proof files, or evidence files.
- Stop when `doctor`, `verify`, migration inspection, or readiness reports corrupt, unsupported, unknown-newer, ambiguous, or incomplete state.
- Do not perform implicit migration during ordinary startup.
- Preserve logs and diagnostic output before attempting recovery.
- Use the checked-in files under `deploy/systemd/` as the systemd template source of truth.

### 3. Reference platform and prerequisites

The release-tested reference platform is:

```text
Ubuntu Server 24.04 LTS
x86_64 (amd64)
systemd
local Linux filesystem such as ext4
```

Install the operating-system prerequisites:

```bash
sudo apt update
sudo apt install -y ca-certificates curl sqlite3
uname -m
systemctl --version
```

`uname -m` must report `x86_64` for the formal reference-platform procedure.

A Rust toolchain is required only when building a qualification candidate from source. Published production operation should use release-built binaries and their published checksums.

### 4. Install the service account and directories

```bash
sudo useradd --system \
  --home /var/lib/lingonberry \
  --shell /usr/sbin/nologin \
  lingonberry 2>/dev/null || true

sudo install -d -o lingonberry -g lingonberry \
  /var/lib/lingonberry/storage/data \
  /var/lib/lingonberry/storage/tmp \
  /var/backups/lingonberry

sudo install -d -o root -g lingonberry -m 0750 /etc/lingonberry
```

The active state, active data, backups, temporary files, and configuration must remain explicit and separately reviewable.

### 5. Install binaries

For a published release, install the verified release-built binaries supplied for that release. Record their checksums before replacement:

```bash
sha256sum lingonberry-storage lingonberry-relay
sudo install -m 0755 lingonberry-storage /usr/local/bin/lingonberry-storage
sudo install -m 0755 lingonberry-relay /usr/local/bin/lingonberry-relay
sha256sum /usr/local/bin/lingonberry-storage /usr/local/bin/lingonberry-relay
```

For candidate qualification only, build the exact designated candidate with locked dependencies, then record the resulting binary digests:

```bash
git checkout --detach 8c6b48082205a3af555130eec1f3e7d2ac8811fe
cargo build --locked --release -p lingonberry-storage -p lingonberry-relay
sha256sum target/release/lingonberry-storage target/release/lingonberry-relay
```

A source-built candidate is qualification input, not a published v1.0.0 release artifact.

### 6. Install configuration and systemd units

From the matching source or release bundle:

```bash
sudo install -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service

sudo install -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service

sudo install -m 0640 deploy/systemd/storage.env.example \
  /etc/lingonberry/storage.env
sudo install -m 0640 deploy/systemd/relay.env.example \
  /etc/lingonberry/relay.env
sudo chown root:lingonberry /etc/lingonberry/*.env
```

Review both environment files before starting. Configuration precedence is:

```text
defaults < configuration file < environment < CLI
```

Do not use `env $(cat /etc/lingonberry/storage.env | xargs)`. Load the protected environment file inside the service-user shell.

### 7. Validate configuration before startup

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage config
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage doctor
'

sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-relay.service
```

A failed check blocks startup. `doctor` is read-only and must not be replaced by manual file repair.

### 8. Start, stop, and restart

Start and enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service
```

Check status and readiness:

```bash
systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

Restart the long-running relay:

```bash
sudo systemctl restart lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

Stop services:

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service
```

The storage unit is a oneshot readiness gate. The relay unit is the long-running process.

### 9. Routine health checks

Run storage checks as the service user:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage health
  /usr/local/bin/lingonberry-storage ready
  /usr/local/bin/lingonberry-storage status
  /usr/local/bin/lingonberry-storage metrics
'
```

For a deeper read-only inspection:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
'
```

Do not continue normal operation when readiness or verification fails without an understood and recorded disposition.

### 10. Publish and inspect persisted state

Direct publish commands that access the canonical data directory must run as the service user:

```bash
sudo -u lingonberry env \
  LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage/data \
  /usr/local/bin/lingonberry-relay publish \
  fixtures/http-publish-request/minimal-request.json

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage list
'
```

Production clients should normally publish through the configured relay interface rather than relying on repository fixtures.

### 11. Create and verify a backup

Use a new destination or an explicitly supported reusable destination. Never treat creation alone as success.

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage backup create \
    /var/backups/lingonberry/manual-backup
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage backup verify \
    /var/backups/lingonberry/manual-backup
'
```

Record the backup path, timestamps, verification result, and relevant binary digests.

### 12. Restore only into an isolated target

Never restore over active state or active data.

```bash
sudo install -d -o lingonberry -g lingonberry \
  /var/lib/lingonberry/restore-candidate

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage restore plan \
    /var/backups/lingonberry/manual-backup \
    /var/lib/lingonberry/restore-candidate
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage restore apply \
    /var/backups/lingonberry/manual-backup \
    /var/lib/lingonberry/restore-candidate
'
```

The target must be explicit, empty, isolated, and not a symbolic link. Switching active paths is a separate operator decision after verification.

### 13. Verify and rebuild the derived index

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage index verify
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage index rebuild
'
```

Use rebuild only for derived state. It is not a repair mechanism for corrupt canonical storage.

### 14. Restart persistence check

Capture the persisted listing, restart the relay, wait for readiness, and compare the listing:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage list
' > /tmp/lingonberry-list-before.json

sudo systemctl restart lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage list
' > /tmp/lingonberry-list-after.json

cmp /tmp/lingonberry-list-before.json \
  /tmp/lingonberry-list-after.json
```

A mismatch is a release- or operation-blocking persistence failure. Preserve both listings and the service journal before further action.

### 15. First-line diagnosis

```bash
systemctl --failed
systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service
journalctl \
  -u lingonberry-storage-ready.service \
  -u lingonberry-relay.service \
  --since today --no-pager

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage status
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage metrics
'
```

For corrupt, unknown-newer, symlink, active migration journal, incomplete replacement, incomplete cleanup, or unsafe restore-target errors:

1. stop write traffic;
2. preserve logs and diagnostic output;
3. do not edit state files manually;
4. follow the corresponding migration, quarantine, replacement, or recovery procedure.

### 16. Related procedures

- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Quarantine Admin HTTP and RBAC](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Backup / Verify / Restore](./QUARANTINE_BACKUP_RESTORE.md)
- [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [Cleanup Operations Runbook](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)

---

<a id="japanese"></a>

## 日本語

**実機試験の保留（2026-10-20）:** 基準ホストでの事前検証、ディスク圧迫試験、正式72時間連続稼働試験は、管理者の指示により保留しています。明示的な再開指示を受けてから再開します。v1.0.0は未公開です。

### 1. 対象範囲とリリース状況

この文書は、Lingonberry v1.xの運用仕様で想定する、単一ノードの基本的な運用手順書です。インストール、設定検証、サービスの起動・停止・再起動、正常性の確認、バックアップ、分離した環境への復元、インデックス検証、再起動後のデータ保持、一次切り分けを扱います。

最新の公開版は`v0.9.0`です。`v1.0.0`は公開に向けた検証中で、まだ公開されていません。v1.0.0 タグとGitHub Releaseが存在するまでは、この文書は公開前の手順書であり、v1.0.0公開済みの証拠として扱ってはいけません。

バージョン更新前の検証対象として指定した公開候補は次のコミットです。

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

適格性検証固有の連続稼働試験、障害注入試験、ディスク圧迫試験、検証記録手順は保守担当者の作業であり、この運用手順書の対象外です。

### 2. 安全規則

- 保存した知識オブジェクトを正本とし、インデックスと遷移を反映した有効ビューは再構築可能な派生物として扱う
- 稼働中の状態ディレクトリやデータディレクトリへ復元しない
- マニフェスト、ジャーナル、世代を指すポインター、インデックス、証明データ、検証記録を手作業で編集しない
- `doctor`、`verify`、移行状態の調査、受付準備状態が破損、未対応、未知の新しい形式、曖昧、不完全を報告した場合は停止する
- 通常起動時に暗黙の移行処理を行わない
- 復旧を試みる前にログと診断結果を保存する
- systemdのテンプレートの正本として`deploy/systemd/`配下のリポジトリで管理するファイルを使用する

### 3. 基準環境と前提条件

リリース試験済みの基準環境は次のとおりです。

```text
Ubuntu Server 24.04 LTS
x86_64 (amd64)
systemd
ext4などのlocal Linux filesystem
```

OS側で必要なパッケージを導入します。

```bash
sudo apt update
sudo apt install -y ca-certificates curl sqlite3
uname -m
systemctl --version
```

正式な基準環境手順では、`uname -m`が`x86_64`を返す必要があります。

Rustツールチェーンが必要なのは、検証対象の公開候補をソースコードからビルドする場合だけです。公開後の本番運用では、リリース用にビルドされたバイナリと公開されたチェックサムを使用します。

### 4. サービス用アカウントとディレクトリを作成する

```bash
sudo useradd --system \
  --home /var/lib/lingonberry \
  --shell /usr/sbin/nologin \
  lingonberry 2>/dev/null || true

sudo install -d -o lingonberry -g lingonberry \
  /var/lib/lingonberry/storage/data \
  /var/lib/lingonberry/storage/tmp \
  /var/backups/lingonberry

sudo install -d -o root -g lingonberry -m 0750 /etc/lingonberry
```

稼働中の状態とデータ、バックアップ、一時ファイル、設定は明示的に分離し、個別に確認できる状態を維持します。

### 5. バイナリをインストールする

公開リリースでは、そのリリース用に提供された検証済み公開用バイナリを使用します。置換前後のチェックサムを記録します。

```bash
sha256sum lingonberry-storage lingonberry-relay
sudo install -m 0755 lingonberry-storage /usr/local/bin/lingonberry-storage
sudo install -m 0755 lingonberry-relay /usr/local/bin/lingonberry-relay
sha256sum /usr/local/bin/lingonberry-storage /usr/local/bin/lingonberry-relay
```

公開候補の検証の場合だけ、固定された依存関係で指定された正確な公開候補をビルドし、バイナリのハッシュ値を記録します。

```bash
git checkout --detach 8c6b48082205a3af555130eec1f3e7d2ac8811fe
cargo build --locked --release -p lingonberry-storage -p lingonberry-relay
sha256sum target/release/lingonberry-storage target/release/lingonberry-relay
```

ソースコードからビルドした公開候補は検証への入力であり、公開済みv1.0.0の配布物ではありません。

### 6. 設定とsystemdユニットをインストールする

対応するソースコードまたはリリース配布物一式からインストールします。

```bash
sudo install -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service

sudo install -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service

sudo install -m 0640 deploy/systemd/storage.env.example \
  /etc/lingonberry/storage.env
sudo install -m 0640 deploy/systemd/relay.env.example \
  /etc/lingonberry/relay.env
sudo chown root:lingonberry /etc/lingonberry/*.env
```

起動前に両方の環境設定ファイルを確認します。設定の優先順位は次のとおりです。

```text
既定値 < 設定ファイル < 環境変数 < CLIオプション
```

`env $(cat /etc/lingonberry/storage.env | xargs)`は使用しません。保護された環境設定ファイルはサービス用ユーザーのシェルの内部で読み込みます。

### 7. 起動前に設定を検証する

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage config
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage doctor
'

sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-relay.service
```

検査に失敗した場合は起動を中止します。`doctor`は読み取り専用であり、手作業によるファイル修復で置き換えてはいけません。

### 8. 起動、停止、再起動

起動して自動起動を有効化します。

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service
```

状況と受付準備状態を確認します。

```bash
systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

常駐するリレーを再起動します。

```bash
sudo systemctl restart lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

サービスを停止します。

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service
```

ストレージユニットは一度だけ起動準備を確認するユニットです。リレーユニットが常駐プロセスです。

### 9. 日常の正常性確認

サービス用ユーザーとしてストレージの検査を実行します。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage health
  /usr/local/bin/lingonberry-storage ready
  /usr/local/bin/lingonberry-storage status
  /usr/local/bin/lingonberry-storage metrics
'
```

より詳しい読み取り専用の調査を行います。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
'
```

受付準備状態または検証が失敗した場合、原因と対応を理解して記録するまで通常運用を再開しません。

### 10. 公開と永続化された状態の確認

正本となるデータディレクトリへ直接アクセスする公開コマンドはサービス用ユーザーとして実行します。

```bash
sudo -u lingonberry env \
  LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage/data \
  /usr/local/bin/lingonberry-relay publish \
  fixtures/http-publish-request/minimal-request.json

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage list
'
```

本番環境クライアントは通常、リポジトリ付属のサンプルデータへ依存せず、設定済みリレーインターフェースを通して公開します。

### 11. バックアップの作成と検証

新しい保存先、または再利用が明示的に認められた保存先を使用します。作成だけを成功として扱いません。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage backup create \
    /var/backups/lingonberry/manual-backup
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage backup verify \
    /var/backups/lingonberry/manual-backup
'
```

バックアップパス、時刻、検証結果、関連バイナリのハッシュ値を記録します。

### 12. 分離された復元先だけに復元する

稼働中の状態ディレクトリやデータディレクトリへ復元してはいけません。

```bash
sudo install -d -o lingonberry -g lingonberry \
  /var/lib/lingonberry/restore-candidate

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage restore plan \
    /var/backups/lingonberry/manual-backup \
    /var/lib/lingonberry/restore-candidate
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage restore apply \
    /var/backups/lingonberry/manual-backup \
    /var/lib/lingonberry/restore-candidate
'
```

復元先には、明示的に指定した、稼働環境から分離された空のディレクトリを使用します。シンボリックリンクは使用できません。使用中のパスへの切り替えは、検証後に運用担当者による別の判断として行います。

### 13. 派生インデックスの検証と再構築

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage index verify
'

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage index rebuild
'
```

再構築は派生状態だけに使用します。正本となるストレージの破損を修復する手段ではありません。

### 14. 再起動後のデータ保持の確認

保存済みデータの一覧を取得し、リレーを再起動して受付準備が整うのを待ち、一覧を比較します。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage list
' > /tmp/lingonberry-list-before.json

sudo systemctl restart lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  exec /usr/local/bin/lingonberry-storage list
' > /tmp/lingonberry-list-after.json

cmp /tmp/lingonberry-list-before.json \
  /tmp/lingonberry-list-after.json
```

一覧が一致しなければ、データ保持の確認に失敗しています。リリースまたは運用を中止してください。追加作業の前に両方の一覧とサービスのジャーナルを保存します。

### 15. 一次切り分け

```bash
systemctl --failed
systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service
journalctl \
  -u lingonberry-storage-ready.service \
  -u lingonberry-relay.service \
  --since today --no-pager

sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage status
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage metrics
'
```

破損、未知の新しい形式、シンボリックリンク、処理中の移行ジャーナル、未完了の置換処理や整理処理、安全でない復元先に関するエラーが報告された場合は、次のように対応します。

1. 書き込み要求を停止する
2. ログと診断結果を保存する
3. 状態ファイルを手作業で編集しない
4. 対応する移行、隔離領域、置換処理、復旧手順に従う

### 16. 関連手順

- [対応環境](./SUPPORTED_PLATFORMS.md)
- [運用CLIの仕様](./OPERATOR_CLI_CONTRACT.md)
- [保存形式の移行と更新の仕様](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 更新とロールバックの手順](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [隔離領域の管理用HTTP APIと役割に基づくアクセス制御](./QUARANTINE_ADMIN_HTTP.md)
- [隔離領域のバックアップ・検証・復元](./QUARANTINE_BACKUP_RESTORE.md)
- [置換処理の復旧手順](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [置換処理後の整理手順](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
