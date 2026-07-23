# v1.0.0 Disk-Pressure Driver

**Status:** safety contract and non-qualifying execution driver implemented; privileged reference-host rehearsal pending

## Purpose

This document defines the fail-closed safety boundary and execution evidence model for the v1.0.0 formal-soak disk-pressure scenarios. It does not claim that a qualifying disk-pressure run has been executed.

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

## Contract validator

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

## Execution driver

```text
scripts/run-v1-disk-pressure-driver.py
```

The driver has separate `dry-run` and `live` modes. Both modes use a non-resumable output directory and produce a timeline, phase snapshots, summary, and `SHA256SUMS`.

The ordered phases are:

1. preflight;
2. pre-pressure snapshot;
3. pressure allocation;
4. threshold crossing;
5. expected write failure;
6. pressure release;
7. service restart;
8. readiness verification;
9. storage verification;
10. index verification;
11. workspace-cleanliness verification;
12. completion.

A failure at any phase retains partial evidence and attempts idempotent removal of only the contract-named pressure file. The driver never deletes Lingonberry durable state as a recovery mechanism.

`dry-run` validates phase ordering, evidence schema, output-reuse refusal, forced-failure retention, and cleanup semantics. It always records:

```json
{
  "qualification": false,
  "qualifyingPass": false,
  "referenceHostRehearsalComplete": false
}
```

`live` additionally requires:

- root execution;
- candidate acknowledgement through `LINGONBERRY_DISK_PRESSURE_ACK`;
- a passing live contract validation;
- `qualificationEnabled:true` in the frozen host contract;
- the exact mounted device, filesystem UUID, capacity, ownership, and path separation required by the contract.

The checked-in example therefore rejects live execution.

## Remaining implementation

Before `disk_pressure` may be enabled in the formal command map:

1. freeze a host-specific contract on Ubuntu Server 24.04 LTS x86_64;
2. complete one privileged live rehearsal using the execution driver;
3. inspect pre-pressure, threshold, failure, release, restart, readiness, verify, index-verify, and cleanliness evidence;
4. prove that cleanup removed only the marker-owned pressure file;
5. independently verify the evidence checksums;
6. update the command map only after the rehearsal passes.

## Evidence boundary

GitHub-hosted CI verifies the static safety contract and non-qualifying driver semantics only. It does not provide a qualifying privileged mount rehearsal and does not start the 72-hour soak.
