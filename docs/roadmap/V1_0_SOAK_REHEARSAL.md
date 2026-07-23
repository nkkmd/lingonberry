# Lingonberry v1.0.0 Soak Rehearsal

**Status: rehearsal passed; formal runner blocked** | **Candidate: `f9543019f2c219aea3b085ff90f2da201b268a48`** | **Tracking: #134 / #114** | **Last updated: 2026-07-23**

## Purpose

This document records the bounded, non-qualifying rehearsal of the v1.0.0 soak evidence harness.

The rehearsal proves that scenario drivers, telemetry capture, failed-run retention, machine-readable summaries, and bundle checksums work against the designated candidate. It does **not** satisfy the 72-hour qualification soak in `V1_0_SOAK_PLAN.md`.

## Evidence identity

| Field | Value |
|---|---|
| Candidate commit | `f9543019f2c219aea3b085ff90f2da201b268a48` |
| Storage binary SHA-256 | `22228c6ee424c697114f1fcbb1f8aa2ad6c3a3feb4b0c1a71298c2cd7acbbeb0` |
| Relay binary SHA-256 | `9552773a6138cbbbcd32d88a313e01865972facf5b9cbfb3104d091573d7625d` |
| Workflow | `v1 soak rehearsal` run 3 |
| Workflow run ID | `29978326834` |
| Artifact ID | `8552257427` |
| Artifact digest | `sha256:d15e290c78eb3a3818f6a4a0e9695e48889dc9271167bec6685cd6500e98a4fc` |
| Artifact retention | Through 2026-10-21 |

## Passing rehearsal

The passing bundle recorded:

- profile: `rehearsal`
- qualification: `false`
- candidate SHA: exact match
- continuous duration: 26 seconds
- status: `passed`
- scenario count: 10
- failed scenarios: 0
- checksum entries: 28, independently verified

Executed scenario groups:

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

## Forced-failure rehearsal

A second run used `SOAK_FORCE_FAILURE=1` and retained a complete partial evidence bundle.

The failed bundle recorded:

- profile: `rehearsal`
- qualification: `false`
- candidate SHA: exact match
- status: `failed`
- stop reason: `forced rehearsal failure`
- scenario records retained: 10
- checksum entries: 28, independently verified

The workflow itself passed only after verifying that the forced-failure run could not be represented as successful.

## Rehearsal defects found and corrected

1. The first workflow revision checked out only the candidate and could not retrieve the PR-head harness.
2. The second revision overlaid the harness inside the candidate checkout, correctly triggering the clean-check guard.
3. The final workflow uses separate harness and exact-candidate checkouts and executes the harness from outside the candidate tree.

These were evidence-infrastructure defects, not candidate runtime failures.

## Formal-run blockers discovered by review

The initial `formal` profile was not acceptable because it would have:

- executed the workload minima near the beginning and then waited for the duration threshold, violating the required distribution across the run;
- managed the relay as a direct child process rather than requiring the documented systemd deployment;
- recorded resource thresholds without enforcing them continuously;
- used a temporary workspace rather than a fixed dedicated-host path layout;
- lacked a durable checkpoint/resume policy that still prevents a stopped run from passing.

Therefore `SOAK_PROFILE=formal` is intentionally fail-closed. It exits before execution until the dedicated-host scheduler is implemented and reviewed.

## Requirements before formal execution can be enabled

- distribute normal and disruptive workloads across at least 72 continuous hours;
- operate the published systemd units on the dedicated Ubuntu Server 24.04 x86_64 host;
- enforce predeclared resource and readiness thresholds on every telemetry sample;
- retain hourly/daily checkpoints without permitting a stopped run to resume as the same passing run;
- schedule backup, restore, verify, rebuild, crash matrix, abrupt termination, and disk-pressure scenarios at the plan cadence;
- retain system journal, unit state, process restart counters, and fixed path growth telemetry;
- prove formal summary generation requires both duration and all workload minima;
- complete another bounded scheduler rehearsal before starting Issue #114.

## Disposition

**Rehearsal harness: PASS**

**Formal qualification soak: NOT STARTED / BLOCKED ON SCHEDULER IMPLEMENTATION**

No release, versioning, tag, or publication action is authorized by this rehearsal.
