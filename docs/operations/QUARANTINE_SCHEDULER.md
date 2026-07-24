# Quarantine Scheduler

**Status: normative v1.0 pre-release operations contract** | **Last reviewed: 2026-07-24**

This document defines the implemented periodic batch-promotion deployment for unresolved quarantine records. The scheduler re-evaluates records against the current validator and acceptance policy and promotes only records that are currently acceptable.

The authoritative scheduled operation is the local CLI command:

```bash
lingonberry-relay quarantine-promote-batch 100
```

Scheduling does not authorize dismissal, permanent rejection, annotation, ledger rotation, replacement, retention cleanup, or any other maintenance mutation.

## 1. Execution model

The checked-in deployment model is:

```text
systemd timer
    ↓
oneshot service
    ↓
host-local flock
    ↓
quarantine-promote-batch 100
    ↓
current validation and acceptance policy
    ├─ promoted
    ├─ already-promoted
    ├─ deferred
    └─ rejected
```

The command emits one canonical JSON report containing:

```text
dryRun
limit
scanned
promoted
alreadyPromoted
deferred
rejected
outcomes
```

Each outcome is operation-specific and may contain quarantine or canonical identifiers. Treat command output as operator evidence, not as low-cardinality metric labels.

## 2. Batch limits and dry run

The CLI accepts:

```text
quarantine-promote-batch [limit] [--dry-run]
```

Rules:

- the default limit is `100`;
- the accepted limit range is `1..=1000`;
- `--dry-run` may be used without an explicit limit and then uses `100`;
- invalid, zero, or greater-than-1000 limits are rejected;
- the command scans no more than the selected limit;
- processing stops with a non-zero command result when a store or promotion operation returns an error.

Dry run evaluates the batch without writing canonical storage or resolution events. It still reads the current quarantine state, validator configuration, acceptance policy, and storage backend and can fail on malformed state or I/O errors.

Before enabling a production timer, execute:

```bash
sudo -u lingonberry \
  env LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay \
  /usr/local/bin/lingonberry-relay quarantine-promote-batch 100 --dry-run
```

Verify:

- `dryRun` is true;
- canonical storage is unchanged;
- `quarantine-resolutions.jsonl` is unchanged;
- promoted, already-promoted, deferred, and rejected counts are plausible;
- no JSONL, permission, capacity, policy, or backend error occurred.

## 3. Checked-in systemd units

Templates:

```text
deploy/systemd/lingonberry-quarantine-promote.service
deploy/systemd/lingonberry-quarantine-promote.timer
```

The service is a `Type=oneshot` unit running as user and group `lingonberry` with:

```text
LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
```

Its checked-in command is:

```text
/usr/bin/flock --nonblock /run/lock/lingonberry-quarantine-promote.lock \
  /usr/local/bin/lingonberry-relay quarantine-promote-batch 100
```

The service also configures:

```text
TimeoutStartSec=10m
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/lingonberry/relay /run/lock
```

Operators must review paths, binary location, user, group, and hardening directives against the actual host before installation.

## 4. Timer semantics

The checked-in timer uses:

```text
OnBootSec=5m
OnUnitActiveSec=15m
RandomizedDelaySec=60
Persistent=true
```

`Persistent=true` allows one catch-up activation after a missed timer event. It does not replay every missed interval and does not provide a retry queue.

`OnUnitActiveSec=15m` schedules relative to the previous activation. Long-running or failed service behavior should be verified on the target systemd version rather than treated as a fixed wall-clock cron schedule.

## 5. Installation

```bash
sudo install -m 0644 deploy/systemd/lingonberry-quarantine-promote.service /etc/systemd/system/
sudo install -m 0644 deploy/systemd/lingonberry-quarantine-promote.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now lingonberry-quarantine-promote.timer
```

Inspect the installed units before enabling:

```bash
systemd-analyze verify /etc/systemd/system/lingonberry-quarantine-promote.service
systemd-analyze verify /etc/systemd/system/lingonberry-quarantine-promote.timer
systemctl cat lingonberry-quarantine-promote.service
systemctl cat lingonberry-quarantine-promote.timer
```

## 6. Operation

Status:

```bash
systemctl status lingonberry-quarantine-promote.timer
systemctl list-timers lingonberry-quarantine-promote.timer
systemctl status lingonberry-quarantine-promote.service
journalctl -u lingonberry-quarantine-promote.service
```

Manual activation:

```bash
sudo systemctl start lingonberry-quarantine-promote.service
```

Disable scheduling:

```bash
sudo systemctl disable --now lingonberry-quarantine-promote.timer
```

Disabling the timer does not terminate an already running oneshot service. Stop or inspect the service separately when maintenance requires writer quiescence.

## 7. Concurrency boundary

The checked-in service uses `/usr/bin/flock --nonblock` to reject overlapping service executions on the same host and lock path.

This lock:

- prevents the checked-in service command from waiting behind another holder;
- does not coordinate a manually invoked command that omits the same `flock` path;
- does not prove safety across containers with separate `/run/lock` namespaces;
- does not provide distributed locking across hosts;
- does not make shared-network-filesystem execution safe;
- is separate from operation-specific local locks used by individual quarantine mutations.

Do not enable systemd and cron scheduling simultaneously. Do not schedule the same state directory from multiple hosts. During backup, replacement, restore, rotation, cleanup, or other quiescent maintenance, disable the timer and ensure the service is inactive.

## 8. Cron fallback

When systemd timers are unavailable, an operator may use a cron entry such as:

```cron
*/15 * * * * /usr/bin/flock --nonblock /run/lock/lingonberry-quarantine-promote.lock env LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay /usr/local/bin/lingonberry-relay quarantine-promote-batch 100 >> /var/log/lingonberry/quarantine-promote.log 2>&1
```

The fallback should use the same lock path, state directory, user permissions, and explicit binary path as the systemd deployment.

Cron does not provide the checked-in systemd sandbox, timeout, missed-run behavior, unit state, or journal integration. Log rotation, permissions, and failure alerting must be configured separately.

## 9. Failure handling

A non-zero CLI result causes the oneshot unit to fail. Inspect in this order:

```bash
systemctl status lingonberry-quarantine-promote.service
journalctl -u lingonberry-quarantine-promote.service -n 100
lingonberry-relay quarantine-status
lingonberry-relay quarantine-metrics
```

Then verify:

- state-directory ownership and permissions;
- canonical backend availability;
- filesystem capacity and inode availability;
- active and archived managed-ledger integrity;
- malformed JSONL or unsupported fields;
- validator and acceptance-policy configuration;
- whether another process holds the external lock;
- whether maintenance changed the active generation or state-directory path.

Do not translate a ledger, backend, or policy failure into a successful zero-record run. Preserve stderr and the canonical report when available.

The checked-in unit does not automatically retry a failed invocation before the next timer activation. It has no exponential backoff or dead-letter queue.

## 10. Observability

After scheduled runs, use:

```bash
lingonberry-relay quarantine-status
lingonberry-relay quarantine-metrics
```

Review:

- pending count and oldest-pending age;
- promoted, dismissed, and permanently rejected lifecycle totals;
- reason-code distribution;
- repeated service failures;
- timer activation history;
- batch reports showing persistent deferred or rejected outcomes.

`quarantine-metrics` is archive-aware and fails closed on corrupt state. Free-form identifiers and errors must not be converted into metric labels.

Systemd journal output and CLI batch outcomes may contain identifiers or validation details. Apply operational log access and retention controls accordingly.

## 11. Security

Run scheduling as a dedicated unprivileged service account.

Requirements:

- grant only the state, backend, binary, and lock-path permissions required by the unit;
- keep `NoNewPrivileges`, filesystem protection, and private temporary-directory settings unless target-host constraints require a reviewed change;
- do not interpolate untrusted shell input into the command;
- do not place bearer tokens or private keys directly in world-readable unit files;
- protect any environment file with restrictive ownership and mode;
- do not expose the public relay listener as a scheduler control surface;
- use the authenticated administrator API only for its documented operations, not as a substitute for this local scheduler contract.

The scheduler-supplied process identity is not automatically recorded as a cryptographically bound operator identity in quarantine lifecycle events.

## 12. Explicitly unscheduled operations

The checked-in timer performs batch promotion only. It must not automatically run:

- dismissal;
- permanent rejection;
- annotation;
- backup or restore;
- ledger rotation;
- compaction or replacement preview;
- replacement apply, resume, or rollback;
- retention evaluation;
- generation cleanup or workspace deletion;
- proof generation;
- schema migration.

In particular, there is no unattended retention or cleanup entry point.

## 13. Non-goals

This scheduler contract does not provide:

- distributed locking;
- exactly-once scheduling;
- replay of every missed interval;
- automatic retry queue or exponential backoff;
- remote scheduler authentication;
- cluster leader election;
- multi-host shared-state coordination;
- automatic remediation of corrupt records;
- formal soak or reference-host qualification.

The fixed v1.0.0 candidate remains `f9543019f2c219aea3b085ff90f2da201b268a48`. Scheduler documentation and ordinary unit tests do not redefine the candidate or satisfy the outstanding release gates.
