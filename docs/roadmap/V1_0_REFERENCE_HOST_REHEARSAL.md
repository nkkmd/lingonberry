# v1.0.0 Reference-Host Rehearsal

**Status: active-candidate preparation contract ready; privileged rehearsal pending** | **Last updated: 2026-07-27**

## Scope

This runbook prepares and validates the dedicated Ubuntu Server 24.04 LTS x86_64 host used for the v1.0.0 reference-host and disk-pressure rehearsal. It does not start the 72-hour formal soak and does not authorize a release.

## Fixed identities

```text
candidate:
8c6b48082205a3af555130eec1f3e7d2ac8811fe

lingonberry-storage SHA-256:
737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507

lingonberry-relay SHA-256:
23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

The installed storage and relay binaries must match these values before any live rehearsal is accepted.

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

3. Create a new 1 GiB sparse backing file and refuse overwrite:

   ```bash
   test ! -e /var/lib/lingonberry-soak-devices/disk-pressure.img
   truncate -s 1G /var/lib/lingonberry-soak-devices/disk-pressure.img
   ```

4. Allocate a free loop device:

   ```bash
   LOOP_DEVICE=$(losetup --find --show /var/lib/lingonberry-soak-devices/disk-pressure.img)
   printf '%s\n' "$LOOP_DEVICE"
   ```

5. Create ext4 once. Do not reformat a frozen device:

   ```bash
   mkfs.ext4 -F -L lingonberry-v1-pressure "$LOOP_DEVICE"
   ```

6. Record the filesystem UUID:

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

8. Copy `deploy/soak/v1-disk-pressure-contract.example.json` to a host-specific evidence input and replace:

   - `device` with the actual loop device;
   - `filesystemUuid` with the recorded UUID;
   - `capacityBytes` with observed capacity;
   - ownership fields with frozen values;
   - binary paths with the installed active-candidate binaries;
   - `qualificationEnabled` with `true` only for the bounded live rehearsal.

Do not change `candidateCommit`, `storageSha256`, or `relaySha256`. The host-specific contract must not be committed and must not contain credentials.

## Preflight

Run static validation first:

```bash
python3 scripts/check-v1-disk-pressure-contract.py /path/to/host-contract.json
python3 scripts/check-v1-reference-host.py --contract /path/to/host-contract.json
```

Then run live validation with the exact candidate acknowledgement:

```bash
export LINGONBERRY_REFERENCE_HOST_ACK=8c6b48082205a3af555130eec1f3e7d2ac8811fe
python3 scripts/check-v1-reference-host.py \
  --contract /path/to/host-contract.json \
  --live | tee /var/lib/lingonberry/soak-evidence/reference-host-preflight.json
```

A failure blocks the rehearsal. Do not repair host state during the same evidence identity and continue as passing.

## Bounded live rehearsal

After preflight passes:

```bash
export LINGONBERRY_DISK_PRESSURE_ACK=8c6b48082205a3af555130eec1f3e7d2ac8811fe
python3 scripts/run-v1-disk-pressure-driver.py \
  --contract /path/to/host-contract.json \
  --out /var/lib/lingonberry/soak-evidence/disk-pressure-rehearsal-$(date -u +%Y%m%dT%H%M%SZ) \
  --mode live
```

The live rehearsal remains non-qualifying until its artifact is independently inspected and the host identity, command map, and thresholds are frozen in release evidence.

## Required functional checks

In the same reference-host evidence cycle, execute and retain results for:

- startup and readiness;
- graceful restart;
- abrupt termination and recovery;
- valid signed publish and restart persistence;
- malformed signature/public key rejection;
- invalid signature rejection;
- verifier execution failure rejection;
- duplicate and conflict non-bypass;
- diagnostics;
- backup, verification, isolated restore;
- index verify and rebuild;
- disk-pressure stop and recovery conditions.

## Teardown

Only after evidence collection:

```bash
mountpoint -q /mnt/lingonberry-disk-pressure
umount /mnt/lingonberry-disk-pressure
losetup -d "$LOOP_DEVICE"
```

Do not delete the backing file until the artifact and frozen UUID/device mapping have been reviewed. Never delete Lingonberry durable state as part of recovery.

## Required evidence

Retain:

- host-specific contract and SHA-256;
- preflight JSON;
- loop device, backing-file metadata, ext4 UUID, capacity, mount identity, and ownership;
- installed binary SHA-256 values;
- systemd unit and environment-file identities;
- command-map and threshold digests;
- driver timeline, snapshots, summary, and `SHA256SUMS`;
- system journal covering the rehearsal;
- proof that evidence and journal filesystems differ from the pressure filesystem;
- operator identity and UTC start/end timestamps;
- deviations and teardown results.

## Evidence boundary

Completion of this runbook is a bounded reference-host rehearsal. It authorizes the formal 72-hour soak only after independent inspection records a PASS with no unresolved release blocker. It does not itself satisfy the formal soak or authorize versioning, tagging, or publication.
