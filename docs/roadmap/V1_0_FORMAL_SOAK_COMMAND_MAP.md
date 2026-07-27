# Lingonberry v1.0.0 Formal Soak Command Map

**Status: active-candidate pre-real-host validation** | **Candidate: `8c6b48082205a3af555130eec1f3e7d2ac8811fe`** | **Tracking: #343 / #114** | **Last updated: 2026-07-27**

## Purpose

This document records the operation-routing boundary for the formal v1.0.0 soak. It distinguishes operations executable through installed operator surfaces from operations requiring a dedicated reference-host driver.

Machine-readable source:

- `deploy/soak/v1-formal-command-map.json`

Validator:

- `scripts/check-v1-formal-command-map.py`

## Frozen identities

The command map is bound to:

```text
candidate:
8c6b48082205a3af555130eec1f3e7d2ac8811fe

lingonberry-storage:
737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507

lingonberry-relay:
23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

Any host-specific copy must preserve those identities. A different candidate or digest requires a new qualification cycle and evidence record.

## Safety properties

- commands are argv arrays, not shell command strings;
- shell metacharacters are rejected;
- placeholders must reference declared variables or scheduler-generated archive placeholders;
- every required workload family must exist;
- disabled operations require a reason;
- disabled required operations keep `qualificationReady` false;
- candidate and binary identities are immutable full digests;
- credentials and bearer tokens are excluded from the map.

## Current routing

| Workload family | Adapter | Enabled | Evidence boundary |
|---|---|---:|---|
| publish | installed relay CLI | yes | signed canonical fixture frozen before execution |
| retrieve | installed storage CLI | yes | bounded retrieval visibility surface |
| query | installed storage CLI | yes | bounded status/query surface |
| graceful restart | systemd | yes | documented unit restart |
| abrupt termination | systemd | yes | SIGKILL to unit main process followed by recovery verification |
| verify | installed storage CLI | yes | strict verification exit code |
| index rebuild | installed storage CLI | yes | canonical storage remains authoritative |
| backup | installed storage CLI | yes | unique generated archive directory |
| isolated restore | installed storage CLI | yes | previously verified archive and isolated target |
| malformed input | relay CLI + stdin fixture | yes | rejection required |
| oversized input | relay CLI + stdin fixture | yes | rejection required |
| deeply nested input | relay CLI + stdin fixture | yes | rejection required |
| crash matrix | candidate-bound test driver | yes for rehearsal | formal cadence/evidence must be frozen on host |
| disk pressure | isolated host scenario | pending live enablement | requires frozen device, filesystem UUID, ownership, and recovery map |

Additional surfaces retained for controlled activation:

- migration through `lingonberry-storage-migrate`;
- quarantine/replacement/cleanup through authenticated admin HTTP/RBAC.

## Remaining reference-host inputs

Before `qualificationReady:true` may be recorded:

1. freeze the real Ubuntu 24.04 x86_64 systemd host;
2. verify both installed binary SHA-256 values;
3. freeze state, data, backup, temp, journal, proof, archive, evidence, and workspace paths;
4. freeze the disk-pressure device, backing file, ext4 UUID, capacity, ownership, and cleanup marker;
5. execute every enabled operation and record expected exit/status values;
6. prove generated archive and restore targets are isolated and non-symlinked;
7. verify valid, malformed, invalid-signature, verifier-failure, duplicate, and conflict publish paths;
8. retain command-map, contract, threshold, and evidence digests;
9. independently inspect the reference-host artifact.

## Exit condition

This gate passes only when the reference-host preflight in Issue #343 completes with no unresolved release blocker. Passing this gate authorizes preparation of the formal soak, not the release itself.

This document does not start or pass the 72-hour soak.
