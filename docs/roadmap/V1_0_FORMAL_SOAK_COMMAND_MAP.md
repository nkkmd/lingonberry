# Lingonberry v1.0.0 Formal Soak Command Map

**Status: implementation complete; real-host rehearsal pending** | **Candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`** | **Tracking: #134 / #114**

## Purpose

This document records the frozen operation-routing boundary for the formal v1.0.0 soak. The machine-readable source is `deploy/soak/v1-formal-command-map.json`; `scripts/check-v1-formal-command-map.py` validates it.

The real-host executor is `scripts/v1_systemd_soak_runner.py`. It consumes the frozen command map, a host-specific configuration derived from `deploy/soak/v1-formal-host-config.example.json`, and host-derived thresholds based on `deploy/soak/v1-formal-host-thresholds.example.json`.

## Safety properties

- Commands are argv arrays; shell command strings and metacharacters are rejected.
- Placeholders must reference declared variables or scheduler-generated, run-owned paths.
- All 14 required workload families are enabled.
- Candidate and binary digests are checked before execution.
- Generated archive and disk-pressure evidence paths must be new, non-symlinked children of the frozen generated root.
- Expected non-zero exit codes for malformed and boundary inputs are explicit.
- Credentials are not stored in the command map.

## Frozen routing

| Workload family | Adapter | Enabled | Evidence boundary |
|---|---|---:|---|
| publish | installed relay CLI | yes | frozen canonical fixture |
| retrieve | installed storage CLI | yes | bounded retrieval visibility |
| query | installed storage CLI | yes | bounded query and status surface |
| graceful restart | systemd | yes | deliberate restart is accounted separately |
| abrupt termination | systemd | yes | SIGKILL followed by bounded readiness recovery |
| verify | installed storage CLI | yes | strict verification exit code |
| index rebuild | installed storage CLI | yes | canonical durable storage remains authoritative |
| backup | generated path | yes | unique run-owned archive directory |
| isolated restore | installed storage drill | yes | latest verified archive only |
| crash matrix | candidate-bound driver | yes | complete matrix evidence retained by the driver |
| malformed input | stdin fixture | yes | deterministic rejection required |
| oversized input | stdin fixture | yes | deterministic rejection required |
| deeply nested input | stdin fixture | yes | deterministic rejection required |
| disk pressure | isolated host driver | yes | frozen contract and unique evidence directory |

## Executor guarantees

The real-host executor adds the remaining scheduler controls:

- one-minute scheduled telemetry for formal execution;
- immediate telemetry before and after every disruptive operation;
- fixed-path byte and file counts for state, data, backup, temporary, evidence, and generated paths;
- journal byte measurement;
- host-derived path-growth and journal thresholds;
- deliberate-versus-unexpected restart accounting;
- readiness failure duration accounting;
- atomic durable checkpoints after each workload event;
- non-resumable output identity;
- checksummed partial evidence on failure;
- a short `--rehearsal` mode that can never set `qualifyingPass`.

## Remaining gate

The implementation is qualification-ready, but publication remains blocked until a dedicated Ubuntu Server 24.04 x86_64 systemd host completes:

1. host-specific command-map variable review;
2. host-derived threshold freeze;
3. privileged preflight;
4. a short real-time systemd rehearsal executing every workload family;
5. independent inspection of its evidence bundle;
6. the formal 72-hour execution.

This document does not claim that the real-host rehearsal or formal soak has passed.
