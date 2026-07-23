# Lingonberry v1.0.0 Formal Soak Scheduler

**Status: scheduler implementation under rehearsal; formal execution not started** | **Candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`** | **Tracking: #134 and #114**

## 1. Purpose

This document defines the executable scheduler boundary for the formal v1.0.0 qualification soak. It implements the cadence, evidence, threshold, and fail-closed requirements in `V1_0_SOAK_PLAN.md` without claiming that the 72-hour run has occurred.

The implementation is:

- `scripts/v1_formal_soak_scheduler.py`
- `deploy/soak/v1-formal-thresholds.example.json`
- `.github/workflows/v1-formal-soak-scheduler-rehearsal.yml`

## 2. Adapter boundary

### 2.1 Mock adapter

The mock adapter exists only to rehearse scheduler behavior using virtual time.

A mock run always records:

- `adapter: "mock"`
- `qualification: false`
- `qualifyingPass: false`

A green mock run cannot satisfy Issue #114.

### 2.2 systemd adapter

Only the systemd adapter can produce a potentially qualifying run. It fails closed unless all of the following are true:

- the host is Ubuntu 24.04;
- the architecture is `x86_64`;
- the process runs as root;
- `systemctl` is available;
- `LINGONBERRY_FORMAL_SOAK_ACK` exactly equals the designated candidate SHA;
- both installed binary SHA-256 values match the recorded candidate digests;
- the configured service is active;
- `--real-time` is supplied;
- the scheduled duration is at least 259,200 seconds;
- telemetry cadence is exactly 60 seconds;
- a frozen host configuration file is supplied.

A systemd run that does not meet every condition cannot set `qualifyingPass: true`.

## 3. Distributed scheduling

Each workload family is evenly distributed across the full run rather than executed as a startup burst. The scheduler validates exact counts before execution.

For disruptive workload families, events must occupy all three thirds of the run:

- graceful restart;
- abrupt termination;
- index rebuild;
- isolated restore;
- replacement/cleanup crash matrix;
- disk-pressure scenario.

The scheduler uses the exact minima from `V1_0_SOAK_PLAN.md`:

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

## 4. Threshold enforcement

Thresholds are loaded before execution from an immutable JSON manifest. They are evaluated at every telemetry tick.

Current threshold fields are:

- minimum free disk bytes;
- minimum free inodes;
- maximum file descriptors;
- maximum RSS bytes;
- maximum swap used bytes;
- maximum readiness failure seconds;
- maximum unexpected restart count.

Any violation stops the run immediately and retains partial evidence. Thresholds cannot be modified in an existing output directory.

The example values are placeholders for rehearsal. The dedicated reference host must freeze host-derived values before the formal run. Values may not be chosen after observing the run.

## 5. Non-resumable run identity

Each run receives a UUID-backed run ID. The output directory must not already exist.

A stopped, failed, or completed run cannot be resumed into the same evidence identity. A retry requires:

- a new output directory;
- a new run ID;
- a fresh preflight;
- a new continuous-duration clock;
- a complete new workload schedule.

## 6. Evidence contract

Each run retains:

- `manifests/run.json`;
- `manifests/schedule.json`;
- `manifests/thresholds.json`;
- `telemetry/metrics.jsonl`;
- `events/timeline.jsonl`;
- `summary.json`;
- `SHA256SUMS`.

The summary distinguishes:

- scheduler execution status;
- adapter qualification eligibility;
- final `qualifyingPass` decision;
- workload counts and minima;
- stop reason;
- candidate identity;
- environment preflight.

A run is a qualifying pass only when all conditions are simultaneously true:

1. adapter is the systemd adapter;
2. preflight succeeds;
3. real time is enabled;
4. duration is at least 72 continuous hours;
5. all workload counts equal or exceed the minima;
6. no threshold or scenario failure occurs;
7. evidence finalization succeeds.

## 7. Scheduler rehearsal

The CI rehearsal performs four independent checks:

1. virtual 72-hour schedule reaches every workload minimum and spreads disruptive events across all thirds;
2. a threshold violation stops the run and preserves partial evidence;
3. an existing output directory cannot be reused;
4. systemd qualification refuses to start outside the reference-host contract.

The rehearsal artifact is non-qualifying. Its purpose is to validate scheduler mechanics before a dedicated-host scheduler rehearsal.

## 8. Remaining work before formal execution

The scheduler core does not itself define the frozen host command map. Before Issue #134 can close, the repository still requires:

- a reviewed systemd host configuration with argv-only commands for every workload family;
- exact storage, backup, restore, quarantine, journal, proof, archive, evidence, and workspace paths;
- host-specific thresholds derived and frozen before execution;
- immediate pre/post disruptive-operation telemetry snapshots;
- journal and fixed-path byte/file-count collection in the systemd adapter;
- a short real-host scheduler rehearsal using the same systemd adapter;
- verification that deliberate SIGKILL events are distinguished from unexplained restarts;
- independent inspection of the real-host rehearsal bundle.

Until those items pass, Issue #114 remains not started and the release decision remains Pending.
