# v1.0.0 Disk-Pressure Driver

**Status:** safety contract implemented; privileged reference-host rehearsal pending

## Purpose

This document defines the fail-closed safety boundary for the v1.0.0 formal-soak disk-pressure scenarios. It does not claim that a qualifying disk-pressure run has been executed.

## Safety model

The pressure filesystem must be a dedicated loop-backed or equivalently isolated quota-controlled filesystem. It must contain only the scenario workspace and the driver-owned pressure file.

The following remain outside that filesystem:

- active Lingonberry state and data;
- backups and normal temporary storage;
- soak journal and evidence bundle;
- the loop backing file itself.

The driver must refuse:

- `/`, `/boot`, `/home`, `/var`, `/var/lib`, or an insufficiently specific mountpoint;
- symlink paths;
- overlap with any active Lingonberry path;
- evidence or journal paths on the pressure filesystem;
- a non-block device or unexpected mount source;
- filesystem UUID, capacity, or ownership drift;
- an unfrozen candidate SHA;
- a contract whose `qualificationEnabled` flag is false.

## Frozen contract

The template is:

```text
deploy/soak/v1-disk-pressure-contract.example.json
```

A reference host must replace the placeholder filesystem UUID and freeze the actual device, backing file, capacity, ownership, and mount identity before a rehearsal. The resulting host-specific contract is evidence input and must not contain credentials.

## Current validator

```text
scripts/check-v1-disk-pressure-contract.py
```

Static validation checks path separation and immutable safety rules. Live validation additionally requires root, a mounted block device, matching source and UUID, matching capacity and ownership, separate evidence devices, and `qualificationEnabled:true`.

The checked-in example deliberately uses:

```json
{
  "qualificationEnabled": false
}
```

Therefore it cannot start a qualifying scenario.

## Remaining implementation

Before `disk_pressure` may be enabled in the formal command map:

1. freeze a host-specific contract on Ubuntu Server 24.04 LTS x86_64;
2. implement the pressure allocation and idempotent release driver;
3. record pre-pressure, threshold, expected failure, release, restart, readiness, verify, index verify, and workspace-cleanliness snapshots;
4. prove that cleanup deletes only the marker-owned pressure file;
5. complete a privileged real-host rehearsal and independently inspect its checksums;
6. update the command map only after the rehearsal passes.

## Evidence boundary

GitHub-hosted CI verifies the static safety contract and fail-closed behavior only. It does not provide a qualifying privileged mount rehearsal and does not start the 72-hour soak.
