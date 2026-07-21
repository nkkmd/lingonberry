# v0.8.0 Operator Runbook

**Status: implementation candidate** | **Last updated: 2026-07-22**

## Purpose

This runbook is the canonical single-node Linux procedure for installing, starting, diagnosing, backing up, restoring, rebuilding the index, and running an isolated disaster-recovery drill.

## 1. Build and install

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

## 2. Validate effective configuration

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage config
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage doctor
```

`doctor` is read-only. Do not manually edit manifests, journals, pointers, indexes, or evidence files.

## 3. Start

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-storage-ready.service
sudo systemctl enable --now lingonberry-relay.service
systemctl status lingonberry-storage-ready.service lingonberry-relay.service
curl -fsS http://127.0.0.1:8787/v1/ready
```

The storage unit is a oneshot readiness gate. The relay unit is the long-running process.

## 4. Publish and inspect

```bash
LINGONBERRY_STATE_DIR=/var/lib/lingonberry/storage/data \
  /usr/local/bin/lingonberry-relay publish fixtures/http-publish-request/minimal-request.json
LINGONBERRY_STORAGE_DATA_DIR=/var/lib/lingonberry/storage/data \
  /usr/local/bin/lingonberry-storage list
```

## 5. Backup

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage backup create /var/backups/lingonberry/manual-backup
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage backup verify /var/backups/lingonberry/manual-backup
```

A created backup is not reported successful until an isolated import and index-consistency check pass.

## 6. Restore

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

## 7. Index operations

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage index verify
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage index rebuild
```

Canonical storage is authoritative; the index is derived state.

## 8. DR drill

```bash
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) \
  /usr/local/bin/lingonberry-storage drill restore /var/backups/lingonberry/manual-backup
```

The drill restores into a temporary isolated directory, verifies consistency, and removes the temporary directory.

## 9. Failure diagnosis

```bash
journalctl -u lingonberry-storage-ready.service -u lingonberry-relay.service --since today
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage status
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage doctor
sudo -u lingonberry env $(cat /etc/lingonberry/storage.env | xargs) /usr/local/bin/lingonberry-storage metrics
```

For corrupt, unknown-newer, symlink, active migration journal, or restore-target errors, stop and preserve evidence. Do not attempt implicit repair.
