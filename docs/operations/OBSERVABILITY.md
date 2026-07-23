# Observability Contract

**Status: v1.0.0 pre-release**  
**Normative language: English**

This document defines the observability surfaces that operators may rely on for the v1.0.0 pre-release line. It describes implemented command output, HTTP readiness, systemd state, journal output, and release evidence. It does not promise a general telemetry platform or a stable structured-log schema that the binaries do not currently implement.

Lingonberry v1.0.0 has not been published. The designated pre-version candidate remains:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Evidence and documentation commits after that candidate do not redefine it.

## 1. Observability principles

Operators must:

- use command exit status together with command output;
- distinguish a point-in-time diagnostic snapshot from a cumulative metric;
- treat `stdout` as the primary machine-readable result channel for CLI commands that emit canonical JSON;
- treat `stderr` and the systemd journal as diagnostic text unless a narrower document defines a structured record;
- preserve evidence before restarting, restoring, migrating, or deleting state;
- avoid high-cardinality labels or alert dimensions such as canonical object IDs;
- never infer application correctness from process health alone.

The v1 contract does not require an external Prometheus, OpenTelemetry, log aggregation, or alert-management stack.

## 2. Runtime model

The reference-node runtime has two different lifecycle surfaces:

- `lingonberry-relay serve-http` is the resident network-facing process;
- `lingonberry-storage ready` is a systemd `Type=oneshot` readiness gate with `RemainAfterExit=yes`.

There is no resident `lingonberry-storage` daemon in the checked-in reference-node units. Consequently, storage does not emit a continuous daemon startup/shutdown event stream. Storage observability is obtained by rerunning its CLI diagnostics against the configured directories.

See:

- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)

## 3. Storage CLI observability

All commands below resolve storage configuration using the normal precedence rules. Operators must ensure that the intended config file and directory overrides are active before interpreting output.

### 3.1 Process health

```bash
lingonberry-storage health
```

The command emits canonical JSON describing process-level health. A successful result does not prove that configured storage directories, indexes, backups, or migration state are ready.

Expected fields include:

```json
{
  "service": "storage",
  "status": "ok",
  "scope": "process"
}
```

Use this only to confirm that the binary can start and execute its health path.

### 3.2 Readiness

```bash
lingonberry-storage ready
```

The command runs the storage doctor and emits a bounded canonical JSON summary containing:

- `service`;
- `status`;
- `ready`;
- `diagnosticStatus`.

Readiness is true when no doctor check has severity `failed`. Warning-only reports remain ready. The command exits nonzero when failed checks are present.

The checked-in `lingonberry-storage-ready.service` executes this command as its `ExecStart` gate.

### 3.3 Doctor and strict verification

```bash
lingonberry-storage doctor
lingonberry-storage verify
```

Both commands emit a read-only canonical JSON report containing the overall status, check count, and individual checks.

- `doctor` fails on failed checks but permits warning-only reports;
- `verify` fails on failed checks and also fails on warning-only reports.

Use `doctor` for initial diagnosis and `verify` when an operation or evidence step requires a clean result.

Individual checks may cover directory safety, storage format, migration journal state, raw log and catalog state, generation pointers, index consistency, backup inventory, workspace safety, and disk capacity. The exact check list is implementation-controlled; automation should consume documented severity and name fields rather than array position.

### 3.4 Metrics snapshot

```bash
lingonberry-storage metrics
```

This command emits a point-in-time canonical JSON snapshot. It is not a cumulative Prometheus registry and it does not expose counters for every storage operation.

The current stable fields are:

```json
{
  "service": "storage",
  "metricsVersion": "1",
  "boundedCardinality": true,
  "ready": 1,
  "doctorChecksOk": 0,
  "doctorChecksWarning": 0,
  "doctorChecksFailed": 0
}
```

Interpretation:

- `ready` is `1` when the underlying doctor has no failed checks, otherwise `0`;
- the three `doctorChecks*` values are counts from the current diagnostic run;
- values do not represent historical totals;
- polling frequency and retention are operator policy, not part of the binary contract.

### 3.5 Configuration and layout snapshots

The following commands are useful supporting evidence:

```bash
lingonberry-storage config
lingonberry-storage status
lingonberry-storage run
```

`run` prints a runtime snapshot and exits. It does not start a resident process. Capture these outputs when directory resolution, config precedence, or storage layout is under investigation.

## 4. Relay observability

### 4.1 CLI readiness

```bash
lingonberry-relay ready
```

The command emits canonical JSON with relay service status. It confirms the command path and configured storage backend can be constructed; it does not prove that a particular TCP listener is already bound.

### 4.2 HTTP readiness

For a running public relay:

```bash
curl --fail --silent --show-error http://127.0.0.1:8787/v1/ready
```

A successful response is HTTP `200` with canonical JSON equivalent to:

```json
{
  "status": "ok",
  "service": "relay"
}
```

This endpoint confirms that the resident listener accepted and routed the request. It is not a deep storage verification endpoint. Use storage `ready`, `doctor`, or `verify` for storage-state diagnosis.

### 4.3 Process and systemd state

Reference-node operators should inspect:

```bash
systemctl status lingonberry-relay.service
systemctl status lingonberry-storage-ready.service
systemctl show lingonberry-relay.service \
  --property=ActiveState,SubState,Result,ExecMainStatus,NRestarts
systemctl show lingonberry-storage-ready.service \
  --property=ActiveState,SubState,Result,ExecMainStatus
```

Important distinctions:

- an active storage readiness unit records the last successful oneshot execution;
- it does not continuously re-evaluate storage after later filesystem or configuration changes;
- restore, migration, binary replacement, or material config changes require the readiness gate to be rerun explicitly;
- relay restart success does not replace storage verification.

### 4.4 Journal output

The relay currently writes listener and connection diagnostics as text to `stderr`, which systemd captures in the journal.

Use bounded queries such as:

```bash
journalctl -u lingonberry-relay.service --since '-30 minutes' --no-pager
journalctl -u lingonberry-storage-ready.service -n 200 --no-pager
```

The current general journal output is not a stable JSON log schema. Operators and automation must not assume fields such as `requestId`, `durationMs`, `event`, or `errorCode` exist for every relay request.

Useful text includes listener startup, bind failure, connection handling errors, accept errors, and explicit warnings. Preserve the surrounding timestamp, unit name, boot ID, and relevant config snapshot when collecting evidence.

## 5. Admin and quarantine observability

Lingonberry contains narrower quarantine-specific observability surfaces. These are governed by their dedicated documents and must not be generalized into a repository-wide metric schema.

The authenticated admin HTTP surface can return quarantine metrics text on its defined route. Access is subject to admin authentication and role authorization. Authentication and authorization failures are written to the dedicated admin audit path.

See:

- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [Quarantine Admin HTTP](./QUARANTINE_ADMIN_HTTP.md)
- [Access Retention Policy](./ACCESS_RETENTION_POLICY.md)

Do not expose the admin listener publicly merely to obtain metrics.

## 6. Evidence surfaces

Release qualification and formal-soak tooling produce evidence that is separate from live runtime telemetry. Evidence may include:

- exact candidate identity;
- command transcripts and exit statuses;
- doctor, verify, readiness, replay, index, backup, and restore results;
- crash-matrix and disk-pressure results;
- scheduler state and bounded runner output;
- timestamps, host facts, checksums, and manifests;
- documentation walkthrough bundles.

Evidence bundles are immutable records of a particular execution. They are not proof that the current host remains healthy after subsequent changes.

The formal 72-hour soak has not been executed or started merely because the rehearsal tooling and command map exist.

## 7. Minimum operator checks

### 7.1 Before starting or restarting relay

```bash
lingonberry-storage ready
systemctl start lingonberry-relay.service
curl --fail --silent --show-error http://127.0.0.1:8787/v1/ready
```

For strict maintenance completion, replace or supplement `ready` with:

```bash
lingonberry-storage verify
```

### 7.2 Routine inspection

```bash
systemctl is-active lingonberry-relay.service
lingonberry-storage metrics
lingonberry-storage doctor
journalctl -u lingonberry-relay.service --since '-30 minutes' --no-pager
```

### 7.3 After migration or restore

Follow the relevant runbook. At minimum, preserve evidence and rerun:

```bash
lingonberry-storage verify
lingonberry-storage index verify
lingonberry-storage ready
systemctl restart lingonberry-relay.service
curl --fail --silent --show-error http://127.0.0.1:8787/v1/ready
```

A pre-commit migration verification has a narrower meaning than full operator verification. See [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md).

## 8. Alerting recommendations

The repository does not ship a complete alerting stack. When integrating with one, alert on implemented signals rather than names from an aspirational metric catalog.

Recommended conditions:

- `lingonberry-relay.service` is inactive or repeatedly restarting;
- public `GET /v1/ready` fails for a sustained interval;
- `lingonberry-storage ready` exits nonzero;
- `doctorChecksFailed` is greater than zero;
- warning counts remain elevated and the warning is relevant to the intended operation;
- disk-capacity doctor checks approach or cross the operational threshold;
- the journal shows recurring bind, accept, storage, validation, or admin-auth failures;
- scheduled evidence generation fails or produces an incomplete bundle.

Alert dimensions should remain low-cardinality. Suitable dimensions include host, unit, command, diagnostic check name, and coarse result. Do not use object IDs, request bodies, signatures, tokens, filesystem paths containing secrets, or unbounded error text as metric labels.

Thresholds must be calibrated from the reference host and workload. This document intentionally does not define unsupported universal percentages for publish failure, latency histograms, or rate-limit volume.

## 9. Incident triage order

1. Record the time, host, deployed binary checksums, and current candidate or release identity.
2. Inspect systemd unit state and recent journal output.
3. Query relay HTTP readiness if the listener is expected to be running.
4. Run storage `health`, then `ready` or `doctor`.
5. Capture `config`, `status`, and `metrics` snapshots.
6. Run strict `verify` when safe and required.
7. Inspect migration journal, backup inventory, index state, and disk capacity as indicated by doctor checks.
8. Follow the operator, upgrade/rollback, or specialized quarantine runbook.
9. Preserve evidence before any mutating recovery action.

## 10. Non-contractual and future surfaces

The following are not guaranteed by the current v1 pre-release contract:

- a universal JSON log event schema for relay and storage;
- per-request request IDs in all journal messages;
- cumulative publish, append, replay, retrieve, latency, or in-flight metrics;
- a public general-purpose `/metrics` endpoint;
- distributed tracing;
- automatic external alert configuration;
- continuously refreshed storage readiness;
- a resident storage process lifecycle.

Future implementations may add these surfaces only with explicit compatibility, privacy, cardinality, and security review.

## 11. Related procedures

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Storage Migration and Upgrade](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
