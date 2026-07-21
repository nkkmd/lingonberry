# v0.8.0 Upgrade and Rollback

**Reference platform: Ubuntu Server 24.04 LTS x86_64 with systemd**

## Scope

This procedure upgrades a single-node v0.7.0 installation to v0.8.0 and defines the supported rollback boundary. It does not permit implicit migration, in-place destructive restore, manual pointer repair, or automatic downgrade of committed storage formats.

## Preconditions

Before changing binaries:

1. Confirm the current node is healthy enough to read canonical storage.
2. Record the currently installed binary versions and checksums.
3. Save copies of the active systemd units and environment files.
4. Run the existing migration inspection command.
5. Create and verify a v0.7.0-compatible archive or verified migration backup.
6. Stop write traffic and ensure no migration, replacement, or cleanup transaction is active.

Required commands include:

```bash
sudo systemctl status lingonberry-relay.service
sudo journalctl -u lingonberry-relay.service --since today
lingonberry-storage-migrate inspect
lingonberry-storage backup create /var/lib/lingonberry/backups/pre-v0.8.0
lingonberry-storage backup verify /var/lib/lingonberry/backups/pre-v0.8.0
```

Do not continue when inspection reports an unknown newer format, corrupt manifest, unresolved migration journal, invalid generation pointer, partial replacement/cleanup state, or an unverified backup.

## Upgrade procedure

### 1. Stop the service

```bash
sudo systemctl stop lingonberry-relay.service
sudo systemctl is-active --quiet lingonberry-relay.service && exit 1 || true
```

### 2. Preserve the v0.7.0 binaries

```bash
sudo install -m 0755 /usr/local/bin/lingonberry-storage \
  /usr/local/lib/lingonberry/lingonberry-storage-v0.7.0
sudo install -m 0755 /usr/local/bin/lingonberry-relay \
  /usr/local/lib/lingonberry/lingonberry-relay-v0.7.0
```

### 3. Install v0.8.0 binaries and units

Install the newly built binaries atomically through temporary paths, then rename them into place. Install the v0.8.0 unit files and environment examples only after reviewing local path and secret settings.

```bash
sudo install -m 0755 target/release/lingonberry-storage \
  /usr/local/bin/lingonberry-storage.new
sudo install -m 0755 target/release/lingonberry-relay \
  /usr/local/bin/lingonberry-relay.new
sudo mv /usr/local/bin/lingonberry-storage.new /usr/local/bin/lingonberry-storage
sudo mv /usr/local/bin/lingonberry-relay.new /usr/local/bin/lingonberry-relay
sudo install -m 0644 deploy/systemd/lingonberry-storage-ready.service /etc/systemd/system/
sudo install -m 0644 deploy/systemd/lingonberry-relay.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### 4. Run the pre-start gate

```bash
sudo -u lingonberry /usr/local/bin/lingonberry-storage \
  --config /etc/lingonberry/storage.json doctor
sudo -u lingonberry /usr/local/bin/lingonberry-storage \
  --config /etc/lingonberry/storage.json verify
sudo systemd-analyze verify /etc/systemd/system/lingonberry-storage-ready.service
sudo systemd-analyze verify /etc/systemd/system/lingonberry-relay.service
```

Warnings must be understood and recorded. Failed checks block startup.

### 5. Start and verify

```bash
sudo systemctl start lingonberry-relay.service
sudo systemctl status lingonberry-relay.service
sudo journalctl -u lingonberry-relay.service -n 100 --no-pager
lingonberry-storage --config /etc/lingonberry/storage.json health
lingonberry-storage --config /etc/lingonberry/storage.json ready
lingonberry-storage --config /etc/lingonberry/storage.json index verify
```

Then perform one controlled publish/read cycle and an isolated restore drill from the pre-upgrade archive.

## Rollback decision boundary

Binary rollback is permitted only when the v0.8.0 process has not committed an incompatible storage migration or other state transition that v0.7.0 cannot interpret.

### Binary-only rollback

Use this path when:

- no storage migration was committed;
- no incompatible configuration was made authoritative;
- doctor and migration inspection show the original format remains compatible.

```bash
sudo systemctl stop lingonberry-relay.service
sudo install -m 0755 /usr/local/lib/lingonberry/lingonberry-storage-v0.7.0 \
  /usr/local/bin/lingonberry-storage
sudo install -m 0755 /usr/local/lib/lingonberry/lingonberry-relay-v0.7.0 \
  /usr/local/bin/lingonberry-relay
sudo systemctl daemon-reload
lingonberry-storage-migrate inspect
sudo systemctl start lingonberry-relay.service
```

### Backup-based rollback

Use this path when v0.8.0 has committed state that the v0.7.0 binary cannot safely open.

1. Stop all write traffic.
2. Preserve the current v0.8.0 data directory for forensic analysis.
3. Restore the verified pre-upgrade archive into a new isolated directory.
4. Verify every restored record and the rebuilt index.
5. Switch configured paths only after verification.
6. Reinstall the v0.7.0 binaries and units.
7. Start the service and perform read/write acceptance checks.

Never restore over the active data directory. Never manually edit storage manifests, generation pointers, journals, or proof artifacts.

## Failed or interrupted upgrade

If interruption occurs before binary replacement, keep the v0.7.0 service stopped and repeat the installation step.

If interruption occurs after binary replacement but before startup:

- run `doctor`, `verify`, and migration `inspect`;
- compare installed checksums with the intended release artifacts;
- either complete the upgrade or use the binary-only rollback path.

If startup writes occurred and compatibility is uncertain, use backup-based rollback rather than guessing.

## Completion evidence

Record:

- old and new binary versions/checksums;
- backup path and verification result;
- doctor/verify outputs;
- migration inspection result;
- systemd unit verification result;
- startup timestamp and journal excerpt;
- health/readiness/index results;
- publish/read and DR drill results;
- rollback decision, when applicable.
