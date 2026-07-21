# v0.8.0 Operator Runbook

**Status: implementation candidate** | **Last updated: 2026-07-22**

## Purpose

This runbook is the canonical single-node Linux procedure for installing, starting, diagnosing, backing up, restoring, rebuilding the index, and running an isolated disaster-recovery drill.

## Reference platform

The formally validated environment is Ubuntu Server 24.04 LTS on x86_64 with systemd. See [Supported Platforms](./SUPPORTED_PLATFORMS.md).

Other systemd-based Linux distributions may work, but commands, package names, permissions, service behavior, backup, restore, and DR procedures are release-tested against the reference platform.

## 1. Prepare Ubuntu Server 24.04 LTS

```bash
sudo apt update
sudo apt install -y build-essential curl git pkg-config libssl-dev sqlite3
```

Install the Rust stable toolchain when it is not already available.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
uname -m
```

`uname -m` must report `x86_64` for the formal v0.8.0 reference-platform procedure.

## 2. Build and install

```bash
cargo build --release -p lingonberry-storage -p lingonberry-relay
sudo install -m 0755 target/release/lingonberry-storage /usr/local/bin/lingonberry-storage
sudo install -m 0755 target/release/lingonberry-relay /usr/local/bin/lingonberry-relay
sudo useradd --system --home /var/lib/lingonberry --shell /usr/sbin/nologin lingonberry || true
sudo install -d -o lingonberry -g lingonberry /var/lib/lingonberry/storage/data /var/lib/lingonberry/storage/tmp /var/backups/lingonberry /etc/lingonberry
sudo install -m 0644 deploy/systemd/lingonberry-storage-ready.service /etc/systemd/system/
sudo install -m 0644 deploy/systemd/lingonberry-relay.service /etc/systemd/system/
sudo install -m 0640 deploy/systemd/storage.env.example /etc/lingonberry/storage.env
sudo install -m 0640 deploy/systemd/relay.env.example /etc/lingonberry/relay.env
sudo chown root:lingonberry /etc/lingonberry/*.env
```

Review both environment files before starting. Configuration precedence is `defaults < config file < environment < CLI`.

## 3. Validate effective configuration

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage config
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage doctor
```

`doctor` is read-only. Do not manually edit manifests, journals, pointers, indexes, or evidence files.

## 4. Start

```bash
sudo systemd-analyze verify /etc/systemd/system/lingonberry-storage-ready.service
sudo systemd-analyze verify /etc/systemd/system/lingonberry-relay.service
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service
systemctl status lingonberry-storage-ready.service lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

The storage unit is a oneshot readiness gate. The relay unit is the long-running process.

## 5. Publish and inspect

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage/data \
  /usr/local/bin/lingonberry-relay publish fixtures/http-publish-request/minimal-request.json
LINGONBERRY_STORAGE_DATA_DIR=/var/lib/lingonberry/storage/data \
  /usr/local/bin/lingonberry-storage list
```

## 6. Backup

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage backup create /var/backups/lingonberry/manual-backup
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage backup verify /var/backups/lingonberry/manual-backup
```

A created backup is not reported successful until an isolated import and index-consistency check pass.

## 7. Restore

Never restore over the active state or data directory.

```bash
sudo install -d -o lingonberry -g lingonberry /var/lib/lingonberry/restore-candidate
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage restore plan \
  /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage restore apply \
  /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
```

The target must be explicit, empty, isolated, and not a symbolic link.

## 8. Index operations

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage index verify
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage index rebuild
```

Canonical storage is authoritative; the index is derived state.

## 9. DR drill

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage drill restore /var/backups/lingonberry/manual-backup
```

A passing drill reports `readVerified`, `writeVerified`, and `cleanupVerified` as `true`. It restores into a temporary isolated directory, reads every restored record, verifies a duplicate-safe re-import, checks index consistency, and removes the temporary directory.

## 10. Restart persistence check

Capture the persisted record listing, restart the long-running service, and compare the listing after readiness returns.

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage list > /tmp/lingonberry-list-before.json
sudo systemctl restart lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage list > /tmp/lingonberry-list-after.json
cmp /tmp/lingonberry-list-before.json /tmp/lingonberry-list-after.json
```

A mismatch is a release-blocking persistence failure. Preserve both files and the journal output before further action.

## 11. Quarantine inspection and maintenance routing

The integrated storage CLI does not duplicate the existing proof-bound quarantine administration surfaces.

- Inspection, authentication, and authorization: [Quarantine Admin HTTP and RBAC](./QUARANTINE_ADMIN_HTTP.md)
- Backup, verification, and restore: [Quarantine Backup / Verify / Restore](./QUARANTINE_BACKUP_RESTORE.md)
- Replacement preview and proof: [Replacement Preview Runbook](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- Replacement recovery: [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- Verified cleanup: [Cleanup Operations Runbook](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)

Pointer, journal, proof, inventory, completion-evidence, and cleanup-evidence files must never be manually repaired. Use the corresponding verifier, resume, rollback, or explicitly acknowledged cleanup procedure.

## 12. Ubuntu failure diagnosis

```bash
systemctl --failed
journalctl -u lingonberry-storage-ready.service -u lingonberry-relay.service --since today
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage status
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage doctor
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage metrics
```

When UFW is enabled and the relay is intentionally exposed beyond localhost, add only the required port and source scope. The reference unit listens on `127.0.0.1:8787` by default, so no UFW rule is required for local reverse-proxy operation.

For corrupt, unknown-newer, symlink, active migration journal, or restore-target errors, stop and preserve evidence. Do not attempt implicit repair.
