# v1.0 Upgrade and Rollback / v1.0 Upgrade・Rollback

[English](#english) | [日本語](#日本語)

**Status: pre-release candidate**  
**Reference platform: Ubuntu Server 24.04 LTS / x86_64 / systemd / ext4**

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

## English

### 1. Scope and release boundary

This runbook defines the minimum single-node procedure for upgrading a supported Lingonberry installation to a newer v1.x release and for selecting a safe rollback path when validation fails.

The latest published release is `v0.9.0`. `v1.0.0` is still under qualification and has not been published. Until the `v1.0.0` tag and GitHub Release exist, this document is pre-release guidance and must not be treated as evidence that v1.0.0 has shipped.

The designated pre-version qualification candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

This document does not authorize release publication, formal soak completion, implicit storage migration, in-place destructive restore, or manual repair of durable state.

### 2. Safety model

- Canonical storage is authoritative. Indexes and effective views are derived and rebuildable.
- Upgrade and migration are separate decisions. Installing a new binary must not silently migrate storage.
- Never restore over active state or active data.
- Never edit manifests, migration journals, generation pointers, indexes, proof files, or evidence files manually.
- Stop when inspection reports `unknown_newer`, `corrupt`, an invalid journal stage, an unresolved migration, an unsafe restore target, or contradictory state.
- Preserve old binaries, checksums, environment files, systemd units, logs, and verified backups before mutation.
- Use the checked-in files under `deploy/systemd/` as the systemd source of truth.

### 3. Preconditions

Before replacing binaries:

1. confirm the node is healthy and readable;
2. stop or drain write traffic;
3. record installed binary digests;
4. copy the active environment files and systemd units;
5. inspect storage format and migration-journal state;
6. create and verify a pre-upgrade backup;
7. confirm that no migration, restore, replacement, or cleanup transaction is active;
8. record the intended release artifact digests.

Capture the current state:

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

sudo cp -a /etc/lingonberry /var/backups/lingonberry/pre-upgrade-config
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
  if [ -f "${LINGONBERRY_STATE_DIR}/storage-migration.journal" ]; then
    /usr/local/bin/lingonberry-storage-migrate status
  fi
'
```

Create and verify a backup before any migration or binary replacement:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage backup create \
    /var/backups/lingonberry/pre-upgrade
  /usr/local/bin/lingonberry-storage backup verify \
    /var/backups/lingonberry/pre-upgrade
'
```

Do not continue when any precondition is unresolved.

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
```

Record checksums of the preserved copies.

### 5. Install the new release artifacts

For a published release, use release-built binaries and published checksums. For candidate qualification only, use binaries built from the exact designated candidate.

Verify artifacts before installation:

```bash
sha256sum \
  lingonberry-storage \
  lingonberry-storage-migrate \
  lingonberry-relay
```

Install through temporary paths and rename into place:

```bash
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

Install reviewed units and examples from the matching release bundle. Do not overwrite local environment files without review.

```bash
sudo install -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo install -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service
sudo systemctl daemon-reload
```

### 6. Run the pre-start gate

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

A failed pre-start check blocks startup. Do not replace failure with manual file edits.

### 7. Explicit migration procedure

Run migration only when `inspect` and the release notes show that migration is required and supported. Ordinary startup must not perform migration implicitly.

The required order is:

```text
inspect
→ plan
→ backup
→ apply
→ verify
→ commit
```

Run each phase separately and preserve its output:

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

For an interrupted migration, inspect the durable journal first:

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage-migrate status
'
```

Use `resume` only when the existing journal and source binding are valid. Use migration `rollback` only before an incompatible committed format transition and only when the command accepts the durable journal state.

### 8. Start and validate

```bash
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service

systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service

curl -fsS http://127.0.0.1:8787/v1/ready
```

Run storage validation:

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

Then perform one controlled publish/read cycle and compare persisted state before and after a relay restart.

### 9. Rollback decision boundary

There are two distinct rollback paths.

#### 9.1 Binary-only rollback

Binary-only rollback is permitted only when:

- no incompatible storage migration was committed;
- no incompatible configuration became authoritative;
- migration inspection shows that the previous binary can still interpret the active storage format;
- the pre-upgrade backup remains verified.

Procedure:

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

Before startup, run the previous binary's read-only inspection and validation. If compatibility is uncertain, do not start it against active storage.

#### 9.2 Backup-based rollback

Use backup-based rollback when the newer release committed state that the previous binary cannot safely open, or whenever compatibility cannot be proven.

1. stop all write traffic and services;
2. preserve the current post-upgrade data directory for forensic analysis;
3. restore the verified pre-upgrade backup into a new isolated directory;
4. verify canonical records and rebuildable derived state in that directory;
5. switch configured paths only after verification;
6. restore the previous binaries, units, and environment files;
7. start services and perform read/write acceptance checks.

Never restore over the active directory. Never remove or edit the storage-format manifest to force downgrade.

### 10. Failed or interrupted upgrade

- **Before binary replacement:** keep services stopped and repeat artifact verification and installation.
- **After binary replacement but before startup:** run `config`, `doctor`, `verify`, and migration `inspect`; either complete the upgrade or use binary-only rollback.
- **During migration:** inspect `status`; use deterministic `resume` or migration `rollback` according to the durable journal. Do not create a new plan over changed source state.
- **After startup writes:** if compatibility is uncertain, use backup-based rollback.
- **After committed migration:** do not start an older binary against the active directory unless compatibility is explicitly proven.

### 11. Completion evidence

Record:

- old and new release identifiers;
- old, new, and installed binary checksums;
- copied environment files and systemd units;
- backup path and verification result;
- `doctor`, `verify`, and migration inspection output;
- migration plan ID, journal stages, and commit result when applicable;
- systemd verification result;
- startup timestamp and journal excerpt;
- health, readiness, index, publish/read, and restart-persistence results;
- rollback decision and evidence when applicable.

### 12. Related documents

- [v1.0 Single-Node Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Systemd Service Contract](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md) — historical version-specific procedure

---

## 日本語

### 1. 対象範囲とリリース境界

このrunbookは、対応済みのLingonberry single-node環境を新しいv1.x releaseへupgradeするための最小手順と、検証失敗時に安全なrollback経路を選択するための境界を定義します。

最新の公開済みreleaseは`v0.9.0`です。`v1.0.0`は資格確認中で、まだ公開されていません。`v1.0.0` tagとGitHub Releaseが存在するまでは、この文書はpre-release guidanceであり、v1.0.0公開済みの証拠として扱ってはいけません。

version更新前の指定qualification candidateは次のcommitです。

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

この文書はrelease公開、formal soak完了、implicit storage migration、active directoryへの破壊的restore、durable stateの手作業修復を許可するものではありません。

### 2. 安全モデル

- canonical storageを正本とし、indexとeffective viewは再構築可能な派生物として扱う
- binary upgradeとstorage migrationを別の判断として扱う。新binaryのinstallによって暗黙にmigrationしてはいけない
- active stateまたはactive dataへrestoreしない
- manifest、migration journal、generation pointer、index、proof、evidenceを手作業で編集しない
- inspectionが`unknown_newer`、`corrupt`、不正なjournal stage、未解決migration、unsafe restore target、矛盾状態を報告した場合は停止する
- mutation前に旧binary、checksum、environment file、systemd unit、log、verified backupを保存する
- systemdの正本として`deploy/systemd/`配下のchecked-in fileを使用する

### 3. 前提条件

binaryを置き換える前に次を行います。

1. nodeが正常でcanonical storageを読み取れることを確認する
2. write trafficを停止またはdrainする
3. installed binary digestを記録する
4. active environment fileとsystemd unitをコピーする
5. storage formatとmigration journalをinspectする
6. pre-upgrade backupを作成してverifyする
7. migration、restore、replacement、cleanup transactionが進行していないことを確認する
8. 導入予定release artifactのdigestを記録する

現状を取得します。

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

sudo cp -a /etc/lingonberry /var/backups/lingonberry/pre-upgrade-config
sudo cp -a \
  /etc/systemd/system/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-relay.service \
  /var/backups/lingonberry/pre-upgrade-config/
```

保護されたstorage environmentを読み込んでread-only checkを行います。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
  /usr/local/bin/lingonberry-storage-migrate inspect
  if [ -f "${LINGONBERRY_STATE_DIR}/storage-migration.journal" ]; then
    /usr/local/bin/lingonberry-storage-migrate status
  fi
'
```

migrationまたはbinary replacement前にbackupを作成してverifyします。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage backup create \
    /var/backups/lingonberry/pre-upgrade
  /usr/local/bin/lingonberry-storage backup verify \
    /var/backups/lingonberry/pre-upgrade
'
```

未解決のpreconditionがある場合は続行しません。

### 4. Serviceを停止しinstalled binaryを保存する

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
```

保存したcopyのchecksumを記録します。

### 5. 新しいrelease artifactをinstallする

公開releaseではrelease build済みbinaryと公開checksumを使用します。candidate qualificationの場合だけ、exact designated candidateからbuildしたbinaryを使用します。

install前にartifactをverifyします。

```bash
sha256sum \
  lingonberry-storage \
  lingonberry-storage-migrate \
  lingonberry-relay
```

temporary pathへinstallしてからrenameします。

```bash
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

対応するrelease bundleからreview済みunitとexampleをinstallします。local environment fileは確認せず上書きしてはいけません。

```bash
sudo install -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo install -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service
sudo systemctl daemon-reload
```

### 6. 起動前gateを実行する

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

pre-start checkが失敗した場合は起動しません。失敗を手作業によるfile編集で置き換えてはいけません。

### 7. 明示的migration手順

`inspect`とrelease noteの両方がmigrationを必要かつ対応済みと示す場合だけmigrationを実行します。通常起動によってimplicit migrationしてはいけません。

必須順序は次のとおりです。

```text
inspect
→ plan
→ backup
→ apply
→ verify
→ commit
```

各phaseを個別に実行し、outputを保存します。

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

migrationが中断された場合は、最初にdurable journalをinspectします。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage-migrate status
'
```

既存journalとsource bindingが有効な場合だけ`resume`を使用します。migration `rollback`は、互換性のないformat transitionがcommitされる前で、commandがdurable journal stateを受理する場合だけ使用します。

### 8. 起動して検証する

```bash
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service

systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service

curl -fsS http://127.0.0.1:8787/v1/ready
```

storage validationを行います。

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

その後、controlled publish/read cycleを1回実施し、relay restart前後のpersisted stateを比較します。

### 9. Rollback判断境界

rollbackには異なる2つの経路があります。

#### 9.1 Binary-only rollback

次をすべて満たす場合だけbinary-only rollbackを使用できます。

- 互換性のないstorage migrationがcommitされていない
- 互換性のないconfigurationがauthoritativeになっていない
- migration inspectionによりprevious binaryがactive storage formatを解釈できることを確認できる
- pre-upgrade backupがverifiedのまま存在する

手順:

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

起動前にprevious binaryのread-only inspectionとvalidationを実行します。互換性が不明な場合はactive storageへ接続して起動しません。

#### 9.2 Backup-based rollback

newer releaseがprevious binaryでは安全に開けないstateをcommitした場合、または互換性を証明できない場合はbackup-based rollbackを使用します。

1. write trafficとserviceをすべて停止する
2. 現在のpost-upgrade data directoryをforensic analysis用に保存する
3. verified pre-upgrade backupを新しいisolated directoryへrestoreする
4. そのdirectory内のcanonical recordと再構築可能なderived stateをverifyする
5. verification後にだけconfigured pathを切り替える
6. previous binary、unit、environment fileを復元する
7. serviceを起動してread/write acceptance checkを行う

active directoryへ上書きrestoreしてはいけません。downgradeを強制するためにstorage-format manifestを削除または編集してはいけません。

### 10. Upgrade失敗または中断時

- **binary replacement前:** serviceを停止したままartifact verificationとinstallを再実行する
- **binary replacement後、startup前:** `config`、`doctor`、`verify`、migration `inspect`を実行し、upgradeを完了するかbinary-only rollbackを使用する
- **migration中:** `status`を確認し、durable journalに従ってdeterministic `resume`またはmigration `rollback`を使用する。変更済みsource state上で新しいplanを作らない
- **startup write後:** 互換性が不明な場合はbackup-based rollbackを使用する
- **migration commit後:** 互換性が明示的に証明されない限り、older binaryをactive directoryへ接続して起動しない

### 11. 完了evidence

次を記録します。

- 旧releaseと新releaseのidentifier
- 旧、新、installed binaryのchecksum
- コピーしたenvironment fileとsystemd unit
- backup pathとverification result
- `doctor`、`verify`、migration inspection output
- migrationを行った場合のplan ID、journal stage、commit result
- systemd verification result
- startup timestampとjournal excerpt
- health、readiness、index、publish/read、restart persistenceの結果
- rollbackを行った場合の判断とevidence

### 12. 関連文書

- [v1.0 Single-Node Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Systemd Service Contract](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md) — 過去version固有の手順
