# v1.0 Single-Node Operator Runbook / v1.0 Single-Node運用Runbook

[English](#english) | [日本語](#日本語)

**Status: pre-release candidate**  
**Reference platform: Ubuntu Server 24.04 LTS / x86_64 / systemd**

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

## English

### 1. Scope and release boundary

This is the minimum single-node operator procedure intended for the Lingonberry v1.x operational contract. It covers installation, configuration validation, service lifecycle, health checks, backup, isolated restore, index verification, restart persistence, and first-line diagnosis.

The latest published release is `v0.9.0`. `v1.0.0` is still under qualification and has not been published. Until the v1.0.0 tag and GitHub Release exist, this document is a pre-release runbook and must not be treated as proof that v1.0.0 has shipped.

The designated pre-version qualification candidate is:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
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
git checkout --detach f9543019f2c219aea3b085ff90f2da201b268a48
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

## 日本語

### 1. 対象範囲とリリース境界

この文書は、Lingonberry v1.x運用契約で想定する最小single-node手順です。install、設定検証、service lifecycle、health check、backup、isolated restore、index検証、restart persistence、一次切り分けを扱います。

最新の公開済みreleaseは`v0.9.0`です。`v1.0.0`は資格確認中で、まだ公開されていません。v1.0.0 tagとGitHub Releaseが存在するまでは、この文書はpre-release runbookであり、v1.0.0公開済みの証拠として扱ってはいけません。

version更新前の指定qualification candidateは次のcommitです。

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

qualification固有のsoak、crash matrix、disk pressure、evidence手順はmaintainer workflowであり、このoperator runbookの対象外です。

### 2. 安全規則

- canonical storageを正本とし、indexとeffective viewは再構築可能な派生物として扱う
- active stateまたはactive data directoryへrestoreしない
- manifest、journal、generation pointer、index、proof、evidenceを手作業で編集しない
- `doctor`、`verify`、migration inspection、readinessがcorrupt、unsupported、unknown-newer、ambiguous、incompleteを報告した場合は停止する
- 通常起動時にimplicit migrationを行わない
- recoveryを試みる前にlogとdiagnostic outputを保存する
- systemd templateの正本として`deploy/systemd/`配下のchecked-in fileを使用する

### 3. Reference platformと前提条件

release test済みのreference platformは次のとおりです。

```text
Ubuntu Server 24.04 LTS
x86_64 (amd64)
systemd
ext4などのlocal Linux filesystem
```

OS側の前提packageを導入します。

```bash
sudo apt update
sudo apt install -y ca-certificates curl sqlite3
uname -m
systemctl --version
```

正式reference-platform手順では、`uname -m`が`x86_64`を返す必要があります。

Rust toolchainが必要なのは、qualification candidateをsourceからbuildする場合だけです。公開後のproduction運用では、release build済みbinaryと公開checksumを使用します。

### 4. Service accountとdirectoryを作成する

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

active state、active data、backup、temporary file、configurationは明示的に分離し、個別に確認できる状態を維持します。

### 5. Binaryをinstallする

公開releaseでは、そのrelease用に提供された検証済みrelease binaryを使用します。置換前後のchecksumを記録します。

```bash
sha256sum lingonberry-storage lingonberry-relay
sudo install -m 0755 lingonberry-storage /usr/local/bin/lingonberry-storage
sudo install -m 0755 lingonberry-relay /usr/local/bin/lingonberry-relay
sha256sum /usr/local/bin/lingonberry-storage /usr/local/bin/lingonberry-relay
```

candidate qualificationの場合だけ、locked dependencyでexact candidateをbuildし、binary digestを記録します。

```bash
git checkout --detach f9543019f2c219aea3b085ff90f2da201b268a48
cargo build --locked --release -p lingonberry-storage -p lingonberry-relay
sha256sum target/release/lingonberry-storage target/release/lingonberry-relay
```

source buildしたcandidateはqualification inputであり、公開済みv1.0.0 release artifactではありません。

### 6. Configurationとsystemd unitをinstallする

対応するsourceまたはrelease bundleからinstallします。

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

起動前に両方のenvironment fileを確認します。設定の優先順位は次のとおりです。

```text
defaults < configuration file < environment < CLI
```

`env $(cat /etc/lingonberry/storage.env | xargs)`は使用しません。保護されたenvironment fileはservice-user shellの内部で読み込みます。

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

失敗したcheckは起動をblockします。`doctor`はread-onlyであり、手作業によるfile修復で置き換えてはいけません。

### 8. 起動、停止、再起動

起動してenableします。

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service
```

statusとreadinessを確認します。

```bash
systemctl status \
  lingonberry-storage-ready.service \
  lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

long-running relayを再起動します。

```bash
sudo systemctl restart lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

serviceを停止します。

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl stop lingonberry-storage-ready.service
```

storage unitはoneshot readiness gateです。relay unitがlong-running processです。

### 9. 通常のhealth check

service userとしてstorage checkを実行します。

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

より詳しいread-only inspectionを行います。

```bash
sudo -u lingonberry sh -c '
  set -a
  . /etc/lingonberry/storage.env
  /usr/local/bin/lingonberry-storage doctor
  /usr/local/bin/lingonberry-storage verify
'
```

readinessまたはverificationが失敗した場合、原因と対応を理解して記録するまで通常運用を続けません。

### 10. Publishとpersisted stateの確認

canonical data directoryへ直接アクセスするpublish commandはservice userとして実行します。

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

production clientは通常、repository fixtureへ依存せず、設定済みrelay interfaceを通してpublishします。

### 11. Backupの作成と検証

新しいdestination、または明示的に再利用可能なdestinationを使用します。作成だけを成功として扱いません。

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

backup path、timestamp、verification result、関連binary digestを記録します。

### 12. Isolated targetだけにrestoreする

active stateまたはactive dataへrestoreしてはいけません。

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

targetは明示的、空、isolated、かつsymbolic linkではない必要があります。active pathへの切り替えは、verification後に別のoperator判断として行います。

### 13. Derived indexの検証と再構築

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

rebuildはderived stateだけに使用します。corrupt canonical storageの修復手段ではありません。

### 14. Restart persistence check

persisted listingを取得し、relayを再起動してreadinessを待ち、listingを比較します。

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

不一致はreleaseまたはoperationをblockするpersistence failureです。追加作業の前に両listingとservice journalを保存します。

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

corrupt、unknown-newer、symlink、active migration journal、incomplete replacement、incomplete cleanup、unsafe restore-target errorの場合は、次のように対応します。

1. write trafficを停止する
2. logとdiagnostic outputを保存する
3. state fileを手作業で編集しない
4. 対応するmigration、quarantine、replacement、recovery手順に従う

### 16. 関連手順

- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Quarantine Admin HTTP and RBAC](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Backup / Verify / Restore](./QUARANTINE_BACKUP_RESTORE.md)
- [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [Cleanup Operations Runbook](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
