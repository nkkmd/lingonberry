# v1.0 Upgrade and Rollback / v1.0 Upgrade・Rollback

[English](#english) | [日本語](#日本語)

**Status: pre-release candidate**  
**Reference platform: Ubuntu Server 24.04 LTS / x86_64 / systemd / ext4**

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

## English

### 1. Scope and release boundary

This runbook defines the minimum single-node procedure for upgrading a supported Lingonberry installation to a newer v1.x release and selecting a safe rollback path when validation fails.

The latest published release is `v0.9.0`. `v1.0.0` remains under qualification and has not been published. Until the `v1.0.0` tag and GitHub Release exist, this is pre-release guidance rather than evidence that v1.0.0 has shipped.

The designated pre-version qualification candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
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

## 日本語

### 1. 対象範囲とリリース境界

このrunbookは、対応済みLingonberry single-node環境を新しいv1.x releaseへupgradeする最小手順と、検証失敗時に安全なrollback経路を選ぶための境界を定義します。

最新の公開済みreleaseは`v0.9.0`です。`v1.0.0`は資格確認中で、まだ公開されていません。`v1.0.0` tagとGitHub Releaseが存在するまでは、これはpre-release guidanceであり、v1.0.0公開済みの証拠ではありません。

指定qualification candidateは次のcommitのままです。

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

このrunbookはrelease公開、formal soak完了、implicit migration、active directoryへの破壊的restore、durable stateの手作業修復を許可しません。

### 2. 安全規則

- canonical storageを正本とし、indexとeffective viewは再構築可能な派生状態として扱う
- binary upgradeとstorage migrationを別のoperator判断として扱う
- 通常起動時にimplicit migrationを行わない
- active stateまたはactive dataへrestoreしない
- manifest、journal、generation pointer、index、proof、evidenceを手作業で編集しない
- `unknown_newer`、`corrupt`、不正なjournal state、未解決migration、unsafe restore target、矛盾したevidenceでは停止する
- mutation前に旧binary、checksum、保護されたenvironment file、systemd unit、log、verified backupを保存する
- Lingonberry unitの正本として`deploy/systemd/`を使用する

### 3. 前提条件とevidence取得

binary置換前に、nodeのhealth確認、write traffic停止、installed binary digest記録、active設定とunitの保存、storageとmigration stateのinspection、pre-upgrade backupの作成・verify、進行中transactionがないことの確認、導入artifact digestの記録を行います。

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

protected storage environmentでread-only checkを行います。

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

新しいversion付きbackup先を作成し、verifyします。

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

正確なbackup pathを記録し、未解決のpreconditionがある場合は続行しません。

### 4. Service停止とinstalled binary保存

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

### 5. 新release artifactのinstall

公開releaseではrelease build済みbinaryと公開checksumを使用します。candidate qualificationではexact candidateからbuildしたbinaryを使用しますが、それは公開release artifactではありません。

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

対応bundleのreview済みunitをinstallします。local environment fileを未確認で上書きしてはいけません。

```bash
sudo install -m 0644 \
  deploy/systemd/lingonberry-storage-ready.service \
  /etc/systemd/system/lingonberry-storage-ready.service
sudo install -m 0644 \
  deploy/systemd/lingonberry-relay.service \
  /etc/systemd/system/lingonberry-relay.service
sudo systemctl daemon-reload
```

### 6. 起動前gate

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

失敗したcheckはstartupをblockします。手作業によるfile編集で失敗を置き換えてはいけません。

### 7. 明示的migration

`inspect`とrelease noteの両方がmigrationを必要かつ対応済みと示す場合だけ実行します。

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

中断時は最初に`status`を確認します。既存journalとsource bindingが有効な場合だけ`resume`を使用します。migration `rollback`は互換性のないtransitionがcommitされる前で、commandがdurable journal stateを受理する場合だけ使用します。

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

controlled publish/readを1回実施し、relay restart前後のpersisted stateを比較します。

### 9. Rollback境界

#### Binary-only rollback

互換性のないmigrationが未commit、互換性のない設定が未確定、previous binaryがactive formatを解釈可能、pre-upgrade backupがverified、のすべてを証明できる場合だけ使用します。

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

起動前にprevious binaryのread-only inspectionを実行します。互換性が不明な場合はactive storageへ接続して起動しません。

#### Backup-based rollback

new releaseがprevious binaryでは安全に開けないstateをcommitした場合、または互換性を証明できない場合に使用します。

1. write trafficとserviceを停止する
2. post-upgrade data directoryをforensic analysis用に保存する
3. verified pre-upgrade backupを新しいisolated directoryへrestoreする
4. canonical recordとderived stateをverifyする
5. verification後にだけconfigured pathを切り替える
6. previous binary、unit、environment fileを復元する
7. serviceを起動しread/write acceptanceを行う

active directoryへ上書きrestoreせず、downgrade強制のためmanifestを削除・編集しません。

### 10. Upgrade中断時

- **置換前:** serviceを停止したままartifact verificationとinstallを再実行する
- **置換後・startup前:** `config`、`doctor`、`verify`、migration `inspect`を実行し、完了またはbinary-only rollbackを選ぶ
- **migration中:** `status`後、durable journalに従って`resume`またはmigration `rollback`を行う
- **startup write後:** 互換性が不明ならbackup-based rollbackを使用する
- **migration commit後:** 互換性を証明できないolder binaryをactive storageへ接続しない

### 11. 完了evidence

旧・新release identifier、binary checksum、保存した設定とunit、verified backup path、diagnostic output、migration planとjournal stage、systemd verification、startup journal、health/readiness/index、publish/read、restart persistence、rollback判断を記録します。

### 12. 関連文書

- [v1.0 Single-Node Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Systemd Service Contract](./SYSTEMD_UNIT_TEMPLATES.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md) — 過去versionの手順
