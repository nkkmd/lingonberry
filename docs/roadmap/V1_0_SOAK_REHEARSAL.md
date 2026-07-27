# Lingonberry v1.0.0 Soak Rehearsal

**Status: active-candidate rehearsal harness aligned; formal soak not authorized** | **Candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`** | **Tracking: #343 / #114** | **Last updated: 2026-07-27**

## Purpose

This document records the bounded, non-qualifying rehearsal boundary for the v1.0.0 soak tooling. A rehearsal validates scenario drivers, telemetry capture, failed-run retention, machine-readable summaries, and bundle checksums. It does **not** satisfy the 72-hour qualification soak in `V1_0_SOAK_PLAN.md`.

## Active evidence identity

```text
candidate:
8c6b48082205a3af555130eec1f3e7d2ac8811fe

lingonberry-storage SHA-256:
737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507

lingonberry-relay SHA-256:
23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

The current workflow checks out the exact candidate and passes all three identities explicitly to `scripts/run-v1-soak.sh`. The harness rejects any different candidate or binary digest before build or scenario execution.

## Historical rehearsal

The previous passing rehearsal was bound to superseded candidate `f9543019f2c219aea3b085ff90f2da201b268a48`:

- workflow run `29978326834`
- artifact `8552257427`
- artifact digest `sha256:d15e290c78eb3a3818f6a4a0e9695e48889dc9271167bec6685cd6500e98a4fc`

That result remains valid only as historical tooling evidence. It does not authorize the active candidate, reference-host execution, or formal soak.

## Rehearsal scenario groups

The bounded harness executes:

1. release build and binary-digest verification;
2. health/readiness baseline;
3. publish, retrieval, and query driver;
4. graceful restart driver;
5. abrupt termination and recovery driver;
6. storage and index verification/rebuild;
7. backup, verify, and isolated restore;
8. replacement/cleanup crash matrix;
9. malformed, oversized, and deeply nested inputs;
10. controlled test-file disk-pressure driver.

It also executes a forced-failure run and requires the failed partial evidence bundle to remain internally verifiable.

## Fail-closed boundaries

- `SOAK_PROFILE=formal` exits before execution in this rehearsal harness.
- candidate checkout must be exact and clean.
- both candidate-built binary SHA-256 values must match the frozen identities.
- altered identity environment variables are rejected.
- an existing evidence identity is not silently reused.
- a forced-failure result cannot be represented as passed.

## Formal scheduler boundary

Formal execution uses `scripts/v1_formal_soak_scheduler.py`, not `run-v1-soak.sh`. The formal scheduler:

- distributes workloads across at least 72 continuous hours;
- requires the systemd adapter, Ubuntu 24.04, x86_64, root, and exact acknowledgement;
- verifies both installed binary digests;
- enforces frozen thresholds at each telemetry sample;
- retains partial evidence on any threshold or scenario failure;
- prevents a stopped run from resuming as the same passing evidence identity.

The scheduler implementation is present, but formal execution remains **NO-GO** until Issue #343 reference-host preflight passes and host-specific command/threshold inputs are frozen.

## Disposition

```text
bounded rehearsal harness: READY / NON-QUALIFYING
privileged reference-host preflight: NEXT
formal 72-hour soak: NOT STARTED / NO-GO UNTIL PREFLIGHT PASS
```

No release, versioning, tag, or publication action is authorized by rehearsal success.
