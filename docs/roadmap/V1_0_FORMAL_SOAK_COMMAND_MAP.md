# Lingonberry v1.0.0 Formal Soak Command Map

**Status: pre-real-host validation** | **Candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`** | **Tracking: #134 / #114**

## Purpose

This document records the frozen operation-routing boundary for the formal v1.0.0 soak. It distinguishes operations that can be executed through installed operator surfaces from operations that still require a dedicated real-host driver.

The machine-readable source is:

- `deploy/soak/v1-formal-command-map.json`

The validator is:

- `scripts/check-v1-formal-command-map.py`

## Safety properties

- commands are represented as argv arrays, not shell command strings;
- shell metacharacters are rejected;
- placeholders must reference declared variables or one of the two scheduler-generated archive placeholders;
- every required workload family must exist in the map;
- disabled operations require a non-empty reason;
- a disabled required operation makes `qualificationReady` false;
- the candidate commit is a full immutable SHA;
- the map does not contain credentials or bearer tokens.

## Current routing

| Workload family | Adapter | Enabled | Evidence boundary |
|---|---|---:|---|
| publish | installed relay CLI | yes | canonical fixture path is frozen before execution |
| retrieve | installed storage CLI | yes | `list` is the current bounded retrieval visibility surface |
| query | installed storage CLI | yes | `status` is the current bounded query/observability surface |
| graceful restart | systemd | yes | documented unit restart |
| abrupt termination | systemd | yes | SIGKILL to the unit main process; recovery verification remains scheduler responsibility |
| verify | installed storage CLI | yes | strict verification exit code |
| index rebuild | installed storage CLI | yes | canonical storage remains authoritative |
| backup | installed storage CLI | yes | scheduler generates a unique archive directory |
| isolated restore | installed storage CLI | yes | uses a previously verified archive |
| malformed input | installed relay CLI + stdin fixture | yes | rejection is required |
| oversized input | installed relay CLI + stdin fixture | yes | rejection is required |
| deeply nested input | installed relay CLI + stdin fixture | yes | rejection is required |
| crash matrix | test-suite only | no | no installed operator command exposes the complete proof-bound matrix |
| disk pressure | host scenario | no | requires isolated quota-controlled volume and host-specific recovery map |

Additional non-minimum operations are retained for later activation:

- migration via `lingonberry-storage-migrate`;
- quarantine/replacement/cleanup via authenticated admin HTTP/RBAC.

## Qualification blockers

The command map is intentionally not yet qualification-ready.

1. `crash_matrix`
   - define whether the formal soak may execute the candidate-bound test binary on the reference host;
   - otherwise expose a supported installed operator surface that exercises the complete matrix;
   - bind every injected point and result to retained evidence.

2. `disk_pressure`
   - provision an isolated filesystem, loop device, or project-quota target;
   - freeze its mount identity and recovery ownership;
   - prove the harness can only remove files it created;
   - preserve host capacity for journal and evidence;
   - run post-pressure restart, verification, index, archive, proof, and workspace checks.

## Exit condition

This gate may move to `qualificationReady:true` only when:

- all 14 required workload families are enabled;
- the validator passes;
- a real Ubuntu 24.04 x86_64 systemd rehearsal executes every enabled operation;
- expected exit codes and machine-readable status values are recorded;
- generated archive and restore paths are proven isolated and non-symlinked;
- credentials required by admin HTTP flows are injected outside the map and redacted from all evidence.

This document does not start or pass the 72-hour soak.
