# v1.0.0 Reference-Host Rehearsal

**Status:** preparation contract defined; privileged rehearsal pending

## Scope

This runbook prepares and validates the dedicated Ubuntu Server 24.04 LTS x86_64 host used for the v1.0.0 disk-pressure rehearsal. It does not start the 72-hour formal soak and does not authorize a release.

## Fixed candidate

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

The installed storage and relay binaries must match the previously recorded candidate binary SHA-256 values before any rehearsal is accepted.

## Safety boundary

The pressure filesystem contains only:

- the scenario workspace;
- the run-owned pressure file;
- the run-owned failed-write probe.

The following remain outside the pressure filesystem:

- Lingonberry state, data, backup, and normal temporary paths;
- the soak journal and evidence bundle;
- the loop backing file;
- the candidate repository and installed binaries.

Never use the root filesystem, an active Lingonberry path, or a shared filesystem as the pressure target.

## Host preparation

Run as root on the dedicated reference host.

1. Install required utilities:

   ```bash
   apt-get update
   apt-get install -y e2fsprogs util-linux curl jq
   ```

2. Create the backing-file directory outside the pressure mount:

   ```bash
   install -d -m 0700 /var/lib/lingonberry-soak-devices
   ```

3. Create a new 1 GiB sparse backing file. Refuse to overwrite an existing file:

   ```bash
   test ! -e /var/lib/lingonberry-soak-devices/disk-pressure.img
   truncate -s 1G /var/lib/lingonberry-soak-devices/disk-pressure.img
   ```

4. Allocate a free loop device and record it:

   ```bash
   LOOP_DEVICE=$(losetup --find --show /var/lib/lingonberry-soak-devices/disk-pressure.img)
   printf '%s\n' "$LOOP_DEVICE"
   ```

5. Create ext4 once. Do not reformat an existing frozen device:

   ```bash
   mkfs.ext4 -F -L lingonberry-v1-pressure "$LOOP_DEVICE"
   ```

6. Read and record the filesystem UUID:

   ```bash
   FILESYSTEM_UUID=$(blkid -s UUID -o value "$LOOP_DEVICE")
   printf '%s\n' "$FILESYSTEM_UUID"
   ```

7. Mount it in the dedicated namespace:

   ```bash
   install -d -m 0700 /mnt/lingonberry-disk-pressure
   mount -t ext4 "$LOOP_DEVICE" /mnt/lingonberry-disk-pressure
   install -d -m 0700 /mnt/lingonberry-disk-pressure/workspace
   ```

8. Copy the example contract to a host-specific evidence input and replace:

   - `device` with the actual loop device;
   - `filesystemUuid` with the recorded UUID;
   - `capacityBytes` with the observed filesystem capacity;
   - ownership fields with the frozen values;
   - `qualificationEnabled` with `true` only for the bounded real-host rehearsal.

The host-specific contract must not be committed and must not contain credentials.

## Preflight

First run static validation:

```bash
python3 scripts/check-v1-disk-pressure-contract.py /path/to/host-contract.json
python3 scripts/check-v1-reference-host.py --contract /path/to/host-contract.json
```

Then run live validation with the exact candidate acknowledgement:

```bash
export LINGONBERRY_REFERENCE_HOST_ACK=f9543019f2c219aea3b085ff90f2da201b268a48
python3 scripts/check-v1-reference-host.py \
  --contract /path/to/host-contract.json \
  --live | tee /var/lib/lingonberry/soak-evidence/reference-host-preflight.json
```

A failure at this stage blocks the rehearsal. Do not repair the host state during the same evidence identity and continue as passing.

## Bounded live rehearsal

After preflight passes:

```bash
export LINGONBERRY_DISK_PRESSURE_ACK=f9543019f2c219aea3b085ff90f2da201b268a48
python3 scripts/run-v1-disk-pressure-driver.py \
  --contract /path/to/host-contract.json \
  --out /var/lib/lingonberry/soak-evidence/disk-pressure-rehearsal-$(date -u +%Y%m%dT%H%M%SZ) \
  --mode live
```

The rehearsal is non-qualifying until its artifact is independently inspected and the reference-host identity is frozen in release evidence.

## Teardown

Only after evidence collection:

```bash
mountpoint -q /mnt/lingonberry-disk-pressure
umount /mnt/lingonberry-disk-pressure
losetup -d "$LOOP_DEVICE"
```

Do not delete the backing file until the rehearsal artifact and frozen UUID/device mapping have been reviewed. Never delete Lingonberry durable state as part of recovery.

## Required evidence

Retain:

- host-specific contract and SHA-256;
- preflight JSON;
- loop device, backing-file metadata, ext4 UUID, capacity, and mount identity;
- installed binary SHA-256 values;
- driver timeline, snapshots, summary, and `SHA256SUMS`;
- system journal covering the rehearsal;
- proof that evidence and journal filesystems differ from the pressure filesystem;
- teardown commands and results.

## Evidence boundary

Completion of this runbook is a bounded reference-host rehearsal only. It does not satisfy the two disk-pressure scenarios required during the 72-hour formal soak, does not close issue #114, and does not authorize versioning, tagging, or publication.
