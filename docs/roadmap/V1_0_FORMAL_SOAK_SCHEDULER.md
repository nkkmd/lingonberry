# Lingonberry v1.0.0 Formal Soak Scheduler

**Status: implementation aligned to active candidate; formal execution not started** | **Candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`** | **Tracking: #343 / #114** | **Last updated: 2026-07-27**

## Purpose

This document defines the executable scheduler boundary for the formal v1.0.0 qualification soak. It implements the cadence, evidence, threshold, and fail-closed requirements in `V1_0_SOAK_PLAN.md` without claiming that the 72-hour run has occurred.

The active launcher is:

- `scripts/v1_formal_soak_scheduler.py`

The reviewed scheduling implementation is retained in:

- `scripts/v1_formal_soak_scheduler_legacy.py`

The launcher fixes the active candidate and both candidate-built binary SHA-256 values before delegating to the retained implementation. This preserves the reviewed scheduling logic while preventing superseded identities from being used.

## Frozen identities

```text
candidate:
8c6b48082205a3af555130eec1f3e7d2ac8811fe

lingonberry-storage:
737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507

lingonberry-relay:
23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

## Adapter boundary

### Mock adapter

The mock adapter exists only to rehearse scheduler behavior using virtual time. It always records:

```text
adapter: mock
qualification: false
qualifyingPass: false
```

A green mock run cannot satisfy the formal soak gate.

### systemd adapter

Only the systemd adapter can produce a potentially qualifying run. It fails closed unless all of the following are true:

- Ubuntu Server 24.04 LTS;
- `x86_64` architecture;
- execution as root;
- `systemctl` available;
- `LINGONBERRY_FORMAL_SOAK_ACK` exactly equals the fixed candidate;
- installed storage and relay binaries match the frozen SHA-256 values;
- the configured service is active;
- `--real-time` is supplied;
- scheduled duration is at least 259,200 seconds;
- telemetry cadence is exactly 60 seconds;
- a frozen host configuration and threshold file are supplied.

## Distributed scheduling

Each workload family is distributed across the complete run rather than executed as a startup burst. The scheduler validates exact counts before execution.

Required minima:

| Workload | Minimum |
|---|---:|
| Publish | 10,000 |
| Retrieve | 10,000 |
| Query | 5,000 |
| Graceful restart | 48 |
| Abrupt termination | 12 |
| Verify | 12 |
| Index rebuild | 4 |
| Backup | 6 |
| Isolated restore | 3 |
| Crash matrix | 6 |
| Malformed input | 1,000 |
| Oversized input | 200 |
| Deeply nested input | 200 |
| Disk pressure | 2 |

Disruptive workloads must occur across all three thirds of the run.

## Threshold enforcement

Thresholds are loaded before execution from an immutable JSON manifest and evaluated at every telemetry tick:

- minimum free disk bytes;
- minimum free inodes;
- maximum file descriptors;
- maximum RSS bytes;
- maximum swap used bytes;
- maximum readiness failure seconds;
- maximum unexpected restart count.

Any violation stops the run immediately and retains partial evidence.

## Non-resumable evidence identity

Each run receives a new UUID-backed run ID and a new output directory. A stopped, failed, or completed run cannot resume into the same evidence identity. A retry requires a fresh preflight, clock, schedule, and output path.

## Evidence contract

Each run retains:

- `manifests/run.json`;
- `manifests/schedule.json`;
- `manifests/thresholds.json`;
- `telemetry/metrics.jsonl`;
- `events/timeline.jsonl`;
- `summary.json`;
- `SHA256SUMS`.

A run is a qualifying pass only when the systemd adapter is used, preflight succeeds, real-time duration reaches at least 72 hours, all workload minima are met, no threshold or scenario failure occurs, and evidence finalization succeeds.

## Current GO / NO-GO

```text
scheduler implementation: READY
mock rehearsal: NON-QUALIFYING
reference-host preflight: NEXT
formal 72-hour execution: NO-GO UNTIL #343 PASSES
```

Before formal execution, the reference host must freeze the final command map, thresholds, path layout, service identity, disk-pressure device identity, evidence filesystem, journal filesystem, and operator provenance. No version, tag, or publication action is authorized by scheduler readiness alone.
