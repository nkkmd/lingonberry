# v1.0 Upgrade and Rollback / v1.0 更新とロールバックの手順

[English](#english) | [日本語](#japanese)

**Status: pre-release candidate**  
**Reference platform: Ubuntu Server 24.04 LTS / x86_64 / systemd / ext4**

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

**Last updated: 2026-10-20**

## English

**Execution hold (2026-10-20):** Reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred at the maintainer’s request. Resume only after an explicit maintainer request; v1.0.0 remains unpublished.

### 1. Scope and release boundary

This runbook defines the minimum single-node procedure for upgrading a supported Lingonberry installation to a newer v1.x release and selecting a safe rollback path when validation fails.

The latest published release is `v0.9.0`. `v1.0.0` remains under qualification and has not been published. Until the `v1.0.0` tag and GitHub Release exist, this is pre-release guidance rather than evidence that v1.0.0 has shipped.

The designated pre-version qualification candidate remains:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

This runbook does not authorize release publication, formal soak completion, implicit storage migration, destructive in-place restore, or manual repair of durable state.

### 2. Safety rules

- Canonical storage is authoritative; indexes and effective views are rebuildable derived state.
- Binary upgrade and storage migration are separate operator decisions.
- Ordinary startup must not perform an implicit migration.
- Never restore over active state or active data.
- Never manually edit manifests, journals, generation pointers, indexes, proofs, or evidence.
- Stop on `unknown_newer`, `corrupt`, invalid journal state, unresolved migration, unsafe restore target, or contradictory evidence.
- Preserve old binaries, checksums, protected environment files, systemd units, logs, and verified backups before mutation.
- Use `deploy/systemd/` as the source of truth for Lingonberry units.

### 3. Preconditions and evidence capture

Before replacing binaries:

1. confirm the node is readable and healthy;
2. stop or drain write traffic;
3. record installed binary digests;
4. copy active environment files and systemd units;
5. inspect storage and migration state;
6. create and verify a pre-upgrade backup;
7. confirm no migration, restore, replacement, or cleanup transaction is active;
8. record intended release artifact digests.

```bash
sudo systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service

sudo journalctl \
  -u lingonberry-storage-ready.service \
  -u lingonberry-relay.service \
  --since today --no-pager

sha256sum \
  /usr/local/bin/lingonberry-storage \
  /usr/local/bin/lingonberry-storage-migrate \
  /usr/local/bin/lingonberry-relay

sudo install -d -m 0750 \
  /var/backups/lingonberry/pre-upgrade-config
sudo cp -a /etc/lingonberry/. \
  /var/backups/lingonberry/pre-upgrade-config/
sudo cp -a \
  /etc/systemd/system/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-relay.service \
  /var/backups/lingonberry/pre-upgrade-config/
```

Run read-only checks with the protected storage environment:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
  /usr/local/bin/lingonberry-storage-migrate inspect
  if [ -f "${LINGONBERRY_STORAGE_DATA_DIR}/storage-migration.journal" ]; then
    /usr/local/bin/lingonberry-storage-migrate status
  fi
'
```

Create a new, versioned backup destination and verify it:

```bash
BACKUP_ID="pre-upgrade-$(date -u +%Y%m%dT%H%M%SZ)"
sudo -u lingonberry sh -c "
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage backup create \
    /var/backups/lingonberry/${BACKUP_ID}
  /usr/local/bin/lingonberry-storage backup verify \
    /var/backups/lingonberry/${BACKUP_ID}
"
```

Record the exact backup path. Do not continue while any precondition is unresolved.

### 4. Stop services and preserve installed binaries

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service
sudo systemctl is-active --quiet lingonberry-relay.service && exit 1 || true

sudo install -d -m 0755 /usr/local/lib/lingonberry/rollback
sudo install -m 0755 /usr/local/bin/lingonberry-storage \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage.previous
sudo install -m 0755 /usr/local/bin/lingonberry-storage-migrate \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage-migrate.previous
sudo install -m 0755 /usr/local/bin/lingonberry-relay \
  /usr/local/lib/lingonberry/rollback/lingonberry-relay.previous

sha256sum /usr/local/lib/lingonberry/rollback/*.previous
```

### 5. Install new release artifacts

For a published release, use release-built binaries and published checksums. Candidate qualification uses binaries built from the exact designated candidate and does not create a published release artifact.

```bash
sha256sum \
  lingonberry-storage \
  lingonberry-storage-migrate \
  lingonberry-relay

sudo install -m 0755 lingonberry-storage \
  /usr/local/bin/lingonberry-storage.new
sudo install -m 0755 lingonberry-storage-migrate \
  /usr/local/bin/lingonberry-storage-migrate.new
sudo install -m 0755 lingonberry-relay \
  /usr/local/bin/lingonberry-relay.new

sudo mv /usr/local/bin/lingonberry-storage.new \
  /usr/local/bin/lingonberry-storage
sudo mv /usr/local/bin/lingonberry-storage-migrate.new \
  /usr/local/bin/lingonberry-storage-migrate
sudo mv /usr/local/bin/lingonberry-relay.new \
  /usr/local/bin/lingonberry-relay
```

Install reviewed units from the matching source or release bundle. Do not overwrite local environment files without review.

```bash
sudo install -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo install -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service
sudo systemctl daemon-reload
```

### 6. Pre-start gate

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage config
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
  /usr/local/bin/lingonberry-storage-migrate inspect
'

sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-relay.service
```

Any failed check blocks startup. Do not replace a failed check with manual file editing.

### 7. Explicit migration

Run migration only when `inspect` and the release notes both state that migration is required and supported.

The implementation-defined sequence is:

```text
inspect → plan → backup → apply → verify → commit
```

Execute and preserve each phase separately:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage-migrate inspect
  /usr/local/bin/lingonberry-storage-migrate plan
  /usr/local/bin/lingonberry-storage-migrate backup
  /usr/local/bin/lingonberry-storage-migrate apply
  /usr/local/bin/lingonberry-storage-migrate verify
  /usr/local/bin/lingonberry-storage-migrate commit
'
```

For an interruption, inspect the durable journal first:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage-migrate status
'
```

Use `resume` only when the existing journal and source binding remain valid. Use migration `rollback` only before an incompatible committed transition and only when the command accepts the durable journal state.

### 8. Start and validate

```bash
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service

systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage health
  /usr/local/bin/lingonberry-storage ready
  /usr/local/bin/lingonberry-storage verify
  /usr/local/bin/lingonberry-storage index verify
'
```

Perform one controlled publish/read cycle and compare persisted state before and after a relay restart.

### 9. Rollback boundary

#### Binary-only rollback

Use binary-only rollback only when all of the following are proven:

- no incompatible migration was committed;
- no incompatible configuration became authoritative;
- the previous binary can interpret the active storage format;
- the pre-upgrade backup remains verified.

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service

sudo install -m 0755 \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage.previous \
  /usr/local/bin/lingonberry-storage
sudo install -m 0755 \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage-migrate.previous \
  /usr/local/bin/lingonberry-storage-migrate
sudo install -m 0755 \
  /usr/local/lib/lingonberry/rollback/lingonberry-relay.previous \
  /usr/local/bin/lingonberry-relay

sudo cp -a /var/backups/lingonberry/pre-upgrade-config/*.service \
  /etc/systemd/system/
sudo systemctl daemon-reload
```

Run the previous binary's read-only inspection before startup. Do not start it against active storage when compatibility is uncertain.

#### Backup-based rollback

Use backup-based rollback when the newer release committed state that the previous binary cannot safely open, or whenever compatibility cannot be proven.

1. stop write traffic and all services;
2. preserve the post-upgrade data directory for forensic analysis;
3. restore the verified pre-upgrade backup into a new isolated directory;
4. verify canonical records and rebuildable derived state there;
5. switch configured paths only after verification;
6. restore previous binaries, units, and environment files;
7. start services and run read/write acceptance checks.

Never restore over the active directory. Never remove or edit the storage-format manifest to force a downgrade.

### 10. Interrupted upgrade

- **Before replacement:** keep services stopped and repeat artifact verification and installation.
- **After replacement, before startup:** run `config`, `doctor`, `verify`, and migration `inspect`; complete the upgrade or use binary-only rollback.
- **During migration:** use `status`, then deterministic `resume` or migration `rollback` according to the durable journal.
- **After startup writes:** use backup-based rollback when compatibility is uncertain.
- **After migration commit:** do not start an older binary against active storage unless compatibility is explicitly proven.

### 11. Completion evidence

Record old and new release identifiers, all relevant checksums, copied configuration and units, verified backup path, diagnostic output, migration plan and journal stages when applicable, systemd verification, startup journals, health/readiness/index results, publish/read and restart-persistence results, and the rollback decision.

### 12. Related documents

- [v1.0 Single-Node Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Systemd Service Contract](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md) — historical procedure

---

<a id="japanese"></a>

## 日本語

**実機試験の保留（2026-10-20）:** 基準ホストでの事前検証、ディスク圧迫試験、正式72時間連続稼働試験は、管理者の指示により保留しています。明示的な再開指示を受けてから再開します。v1.0.0は未公開です。

### 1. 対象範囲とリリース状況

この手順書は、サポート対象であるLingonberryの単一ノード環境を新しいv1.xリリースへ更新する基本手順と、検証失敗時に安全なロールバック経路を選ぶための境界を定義します。

最新の公開版は`v0.9.0`です。`v1.0.0`は公開に向けた検証中で、まだ公開されていません。`v1.0.0` タグとGitHub Releaseが存在するまでは、これは公開前のガイドであり、v1.0.0公開済みの証拠ではありません。

検証対象として指定した公開候補は次のコミットのままです。

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

この手順書はリリース公開、正式な連続稼働試験完了、暗黙の移行処理、使用中のディレクトリへの破壊的復元、永続化された状態の手作業修復を許可しません。

### 2. 安全規則

- 保存した知識オブジェクトを正本とし、インデックスと遷移を反映した有効ビューは再構築可能な派生状態として扱う
- バイナリの更新と保存形式の移行は、運用担当者が別々に判断する
- 通常起動時に暗黙の移行処理を行わない
- 稼働中の状態ディレクトリやデータディレクトリへ復元しない
- マニフェスト、ジャーナル、世代を指すポインター、インデックス、証明データ、検証記録を手作業で編集しない
- `unknown_newer`、`corrupt`、不正なジャーナルの状態、未解決の移行処理、安全でない復元対象、矛盾した検証記録では停止する
- 変更前に旧バイナリ、チェックサム、保護された環境設定ファイル、systemdユニット、ログ、検証済みのバックアップを保存する
- Lingonberry ユニットの正本として`deploy/systemd/`を使用する

### 3. 前提条件の確認と記録の保存

バイナリを置き換える前に、次を実施します。

1. ノードのデータを読み取れることと、正常に動作していることを確認する
2. 書き込み要求の受付を止めるか、処理中の要求が完了するまで待つ
3. 導入済みバイナリのハッシュ値を記録する
4. 使用中の環境設定ファイルとsystemdユニットを保存する
5. ストレージと移行処理の状態を調べる
6. 更新前のバックアップを作成して検証する
7. 進行中のトランザクションがないことを確認する
8. 導入する配布物のハッシュ値を記録する

```bash
sudo systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service

sudo journalctl \
  -u lingonberry-storage-ready.service \
  -u lingonberry-relay.service \
  --since today --no-pager

sha256sum \
  /usr/local/bin/lingonberry-storage \
  /usr/local/bin/lingonberry-storage-migrate \
  /usr/local/bin/lingonberry-relay

sudo install -d -m 0750 \
  /var/backups/lingonberry/pre-upgrade-config
sudo cp -a /etc/lingonberry/. \
  /var/backups/lingonberry/pre-upgrade-config/
sudo cp -a \
  /etc/systemd/system/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-relay.service \
  /var/backups/lingonberry/pre-upgrade-config/
```

保護されたストレージ環境で読み取り専用の検査を行います。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
  /usr/local/bin/lingonberry-storage-migrate inspect
  if [ -f "${LINGONBERRY_STORAGE_DATA_DIR}/storage-migration.journal" ]; then
    /usr/local/bin/lingonberry-storage-migrate status
  fi
'
```

新しいバージョン付きバックアップ先を作成し、検証します。

```bash
BACKUP_ID="pre-upgrade-$(date -u +%Y%m%dT%H%M%SZ)"
sudo -u lingonberry sh -c "
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage backup create \
    /var/backups/lingonberry/${BACKUP_ID}
  /usr/local/bin/lingonberry-storage backup verify \
    /var/backups/lingonberry/${BACKUP_ID}
"
```

正確なバックアップパスを記録し、未解決の前提条件がある場合は続行しません。

### 4. サービスの停止と導入済みバイナリの保存

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service
sudo systemctl is-active --quiet lingonberry-relay.service && exit 1 || true

sudo install -d -m 0755 /usr/local/lib/lingonberry/rollback
sudo install -m 0755 /usr/local/bin/lingonberry-storage \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage.previous
sudo install -m 0755 /usr/local/bin/lingonberry-storage-migrate \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage-migrate.previous
sudo install -m 0755 /usr/local/bin/lingonberry-relay \
  /usr/local/lib/lingonberry/rollback/lingonberry-relay.previous

sha256sum /usr/local/lib/lingonberry/rollback/*.previous
```

### 5. 新しい公開配布物のインストール

公開リリースではリリース用にビルドされたバイナリと公開されたチェックサムを使用します。公開候補の検証では指定された正確な公開候補からビルドしたバイナリを使用しますが、それは正式な公開配布物ではありません。

```bash
sha256sum \
  lingonberry-storage \
  lingonberry-storage-migrate \
  lingonberry-relay

sudo install -m 0755 lingonberry-storage \
  /usr/local/bin/lingonberry-storage.new
sudo install -m 0755 lingonberry-storage-migrate \
  /usr/local/bin/lingonberry-storage-migrate.new
sudo install -m 0755 lingonberry-relay \
  /usr/local/bin/lingonberry-relay.new

sudo mv /usr/local/bin/lingonberry-storage.new \
  /usr/local/bin/lingonberry-storage
sudo mv /usr/local/bin/lingonberry-storage-migrate.new \
  /usr/local/bin/lingonberry-storage-migrate
sudo mv /usr/local/bin/lingonberry-relay.new \
  /usr/local/bin/lingonberry-relay
```

対応する配布物一式の確認済みユニットをインストールします。ローカル環境設定ファイルを未確認で上書きしてはいけません。

```bash
sudo install -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo install -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service
sudo systemctl daemon-reload
```

### 6. 起動前の検証

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage config
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
  /usr/local/bin/lingonberry-storage-migrate inspect
'

sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo systemd-analyze verify \
  /etc/systemd/system/lingonberry-relay.service
```

検査に失敗した場合は起動を中止します。検査を通すためにファイルを手作業で修正してはいけません。

### 7. 明示的な移行

`inspect`とリリースノートの両方が移行を必要かつ対応済みと示す場合だけ実行します。

```text
inspect → plan → backup → apply → verify → commit
```

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage-migrate inspect
  /usr/local/bin/lingonberry-storage-migrate plan
  /usr/local/bin/lingonberry-storage-migrate backup
  /usr/local/bin/lingonberry-storage-migrate apply
  /usr/local/bin/lingonberry-storage-migrate verify
  /usr/local/bin/lingonberry-storage-migrate commit
'
```

中断時は最初に`status`を確認します。既存ジャーナルと移行元との対応情報が有効な場合だけ`resume`を使用します。移行 `rollback`は互換性のない遷移が確定される前で、コマンドが永続化されたジャーナル状態を受理する場合だけ使用します。

### 8. 起動と検証

```bash
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service
systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage health
  /usr/local/bin/lingonberry-storage ready
  /usr/local/bin/lingonberry-storage verify
  /usr/local/bin/lingonberry-storage index verify
'
```

管理下での公開・読み取りを1回実施し、リレー再起動前後の永続化された状態を比較します。

### 9. ロールバックを選ぶ条件

#### バイナリだけを戻すロールバック

次のすべてを確認できる場合に限り使用します。

- 互換性のない移行がまだ確定されていない
- 互換性のない設定がまだ確定されていない
- 更新前のバイナリで現在の保存形式を解釈できる
- 更新前のバックアップが検証済みである

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service

sudo install -m 0755 \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage.previous \
  /usr/local/bin/lingonberry-storage
sudo install -m 0755 \
  /usr/local/lib/lingonberry/rollback/lingonberry-storage-migrate.previous \
  /usr/local/bin/lingonberry-storage-migrate
sudo install -m 0755 \
  /usr/local/lib/lingonberry/rollback/lingonberry-relay.previous \
  /usr/local/bin/lingonberry-relay

sudo cp -a /var/backups/lingonberry/pre-upgrade-config/*.service \
  /etc/systemd/system/
sudo systemctl daemon-reload
```

起動する前に、更新前のバイナリで読み取り専用の検査を行います。互換性が不明な場合は稼働中のストレージに接続して起動しません。

#### バックアップからのロールバック

新しいリリースで、更新前のバイナリでは安全に開けない状態が確定した場合、または互換性を証明できない場合に使用します。

1. 書き込み要求とサービスを停止する
2. 更新後のデータディレクトリを原因調査用に保存する
3. 検証済みの更新前のバックアップを新しい分離されたディレクトリへ復元する
4. 正規化済みのレコードと派生状態を検証する
5. 検証後にだけ設定済みのパスを切り替える
6. 更新前のバイナリ、ユニット、環境設定ファイルを復元する
7. サービスを起動し、読み書きの受入確認を行う

使用中のディレクトリへ上書き復元せず、ダウングレードを強制するためマニフェストを削除・編集しません。

### 10. 更新を中断した場合

- **置換前:** サービスを停止したまま成果物検証とインストールを再実行する
- **置換後・起動前:** `config`、`doctor`、`verify`、移行 `inspect`を実行し、更新を完了するか、バイナリだけを戻すかを判断する
- **移行中:** `status`後、永続化されたジャーナルに従って`resume`または移行 `rollback`を行う
- **起動後に書き込みが行われた場合:** 互換性が不明ならバックアップからのロールバックを使用する
- **移行コミット後:** 互換性を証明できない古いバイナリを使用中のストレージへ接続しない

### 11. 完了時に残す記録

旧・新リリース識別子、バイナリのチェックサム、保存した設定とユニット、検証済みのバックアップパス、診断結果、移行計画とジャーナルの段階、systemdの検証、起動ジャーナル、正常性・受付準備状態・インデックスの検証結果、公開・読み取りの確認結果、再起動後のデータ保持、ロールバック判断を記録します。

### 12. 関連文書

- [v1.0 単一ノード運用手順書](./V1_0_OPERATOR_RUNBOOK.md)
- [保存形式の移行と更新の仕様](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [systemdユニットのテンプレートと仕様](./SYSTEMD_UNIT_TEMPLATES.md)
- [対応環境](./SUPPORTED_PLATFORMS.md)
- [v0.8.0 更新とロールバックの手順](./V0_8_UPGRADE_AND_ROLLBACK.md) — 過去バージョンの手順
