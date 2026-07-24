# Quarantine Observability Metrics

**Status: implemented** | **v1.0 pre-release normative operations contract**

This document defines the low-cardinality Prometheus text-format snapshot derived from persistent quarantine lifecycle state.

Metrics are reconstructed on demand from the archive-aware quarantine record, resolution, dismissal, and permanent-rejection ledgers. Collection is read-only and does not append, promote, dismiss, reject, annotate, rotate, or compact records.

## Collection interfaces

### CLI

```bash
export LINGONBERRY_STATE_DIR=/var/lib/lingonberry/relay
lingonberry-relay quarantine-metrics
```

The development form remains equivalent:

```bash
cargo run -p lingonberry-relay -- quarantine-metrics
```

### Relay HTTP listener

```text
GET /metrics
```

A successful response uses:

```text
text/plain; version=0.0.4; charset=utf-8
```

The endpoint is part of the relay HTTP surface. The built-in listener does not provide TLS termination, network policy, authentication, rate limiting, or scrape authorization. Operators must control exposure with listener binding, firewalling, and a qualified reverse proxy where required.

Do not assume that the separate authenticated quarantine administrator listener protects the public relay `/metrics` route.

## Snapshot reconstruction

Each request reconstructs status from the logical ordered streams for:

```text
quarantine.jsonl
quarantine-resolutions.jsonl
quarantine-dismissals.jsonl
quarantine-rejections.jsonl
```

Archive-aware readers verify any segment manifest and immutable segments before consuming matching archived segments followed by the active ledger.

Annotations and administrative authentication-failure audit events do not change lifecycle-state counts and are not read into this metric snapshot.

The snapshot is not protected by a multi-file transaction or read lock. Concurrent cooperating mutations are serialized with the mutation lock, but metrics collection itself is read-only. A scrape may therefore observe files at different points around a concurrent operation. Operators requiring a stable maintenance comparison must quiesce writers according to the applicable runbook.

## Lifecycle-state gauge

```text
# HELP lingonberry_quarantine_records Current quarantine records by persistent lifecycle state.
# TYPE lingonberry_quarantine_records gauge
lingonberry_quarantine_records{state="total"} <value>
lingonberry_quarantine_records{state="pending"} <value>
lingonberry_quarantine_records{state="promoted"} <value>
lingonberry_quarantine_records{state="dismissed"} <value>
lingonberry_quarantine_records{state="permanently_rejected"} <value>
```

Definitions:

- `total` is the number of persisted quarantine records.
- `promoted` is the number of distinct known quarantine record IDs with at least one persisted promotion resolution.
- `dismissed` is the number of distinct known record IDs with a dismissal and without a promotion resolution.
- `permanently_rejected` is the number of distinct known record IDs with a permanent-rejection event and without a promotion resolution or dismissal.
- `pending` is `total` minus the four mutually exclusive terminal classifications above.

Unknown resolution, dismissal, or rejection events that do not reference a known quarantine record are not counted as lifecycle-state records. Duplicate lifecycle events that violate the corresponding ledger contract are corruption and cause snapshot construction to fail; they are not silently collapsed merely to produce metrics.

The precedence used by reconstruction is implementation-defined and currently places promotion before dismissal and permanent rejection, then dismissal before permanent rejection. Supported mutation paths prevent those conflicting terminal states for cooperating writers, but metrics do not repair conflicting historical data.

## Oldest-pending age gauge

```text
# HELP lingonberry_quarantine_oldest_pending_age_seconds Age of the oldest pending quarantine record.
# TYPE lingonberry_quarantine_oldest_pending_age_seconds gauge
lingonberry_quarantine_oldest_pending_age_seconds <value>
```

The value is the current Unix time in seconds minus the seconds component of the lexically earliest pending record's `receivedAt` value. Saturating subtraction prevents a negative value when the record timestamp is in the future.

The value is `0` when:

- there is no pending record; or
- the selected timestamp cannot be parsed by the metric helper.

A zero value therefore does not by itself distinguish an empty pending set from an unparsable timestamp. Timestamp validation belongs to the record contract and ingestion path; monitoring must not treat this gauge as a timestamp-integrity verifier.

A system-clock failure while obtaining the current time causes metrics collection to fail.

## Reason-code gauge

```text
# HELP lingonberry_quarantine_reason_code_records Quarantine records grouped by bounded reason code.
# TYPE lingonberry_quarantine_reason_code_records gauge
lingonberry_quarantine_reason_code_records{reason_code="LB_IDENTITY_DEFERRED"} <value>
```

Every persisted quarantine record contributes to exactly one `reason_code` series. Counts include pending and terminal records because the label describes the original quarantine classification rather than current lifecycle state.

Metric-label escaping covers backslash, line feed, and double quote. The implementation does not apply a second allowlist or cardinality cap during rendering. Low cardinality therefore depends on the quarantine record contract and trusted implementation-controlled reason codes.

Operators must investigate unexpected or unbounded reason-code series as a data-contract or ingestion problem rather than normalize them away in the monitoring layer.

## Cardinality and sensitive-data rules

The metric contract uses only these label names:

```text
state
reason_code
```

The following must not appear as metric labels or values derived from free text:

- quarantine ID;
- canonical object ID;
- request ID;
- publisher key or signature;
- request payload;
- free-form reasons;
- operator name or note;
- annotation body;
- authentication token; or
- administrative authentication-failure metadata.

Metrics are aggregate operational data, not a record-inspection API or audit ledger.

## Persistent state versus transient decisions

`promoted`, `dismissed`, and `permanently_rejected` reflect persisted lifecycle events.

Transient reevaluation outcomes such as deferred, rejected, conflict, or failed are not cumulative counters in this metric contract unless they produce one of the explicit persistent lifecycle records above. Adding process-local counters, durable decision-event counters, histograms, or rates requires a separate instrumentation and persistence contract.

## Failure behavior

The CLI or HTTP route must not return a successful zero-valued snapshot when reconstruction fails.

Failures include, as applicable:

- malformed active or archived JSONL;
- duplicate lifecycle events treated as corruption;
- invalid or inconsistent segment manifest data;
- missing or modified immutable segments;
- ledger or filesystem read errors; and
- current-system-time errors.

The route reports the checked-in quarantine error rather than disguising unavailable data as healthy metrics. Monitoring must alert on scrape failure or non-success status independently of value-based alerts.

## Recommended initial alerts

Initial alerting should be based on observed workload and service-level objectives rather than fixed repository defaults. Useful signals include:

- `pending` increases continuously across multiple scrape intervals;
- oldest-pending age exceeds the expected reevaluation or operator-review interval;
- one bounded reason code rises unexpectedly;
- dismissed or permanently rejected counts change outside an approved operator window;
- the `/metrics` scrape fails;
- scrape duration or timeout behavior degrades; or
- a post-maintenance snapshot differs unexpectedly from recorded pre-maintenance evidence.

Use sustained conditions and appropriate `for` durations to avoid alerting on short-lived ingestion bursts. This repository does not define production thresholds, alert routing, or escalation policy.

## Evidence and maintenance comparisons

For a qualification or maintenance evidence bundle, record:

- application commit and binary identity;
- resolved state directory;
- collection interface and listener binding;
- complete metric snapshot before and after the operation;
- corresponding `quarantine-status` output;
- segment verification and derived-index verification when rotation or restore is involved;
- scrape timestamp and host clock context; and
- any concurrent writer quiescence used to establish a stable comparison.

Metric equality alone does not prove byte-for-byte ledger equivalence, backup completeness, absence of unknown events, or successful reference-host qualification.

## Security and privacy boundary

Although labels are intentionally low-cardinality and exclude direct identifiers, aggregate counts can reveal operational volume and moderation activity. Treat endpoint exposure as deployment-sensitive.

The metric route provides no built-in confidentiality guarantee. A reverse proxy or monitoring network boundary must not be documented as authenticated unless it is actually configured and qualified.

## Non-goals

The v1.0 metric contract does not provide:

- process-local promotion success or failure counters;
- transient reevaluation outcome counters;
- batch-duration or request-duration histograms;
- scheduler health metrics;
- lock-owner or stale-lock metrics;
- backup-age or restore-success metrics;
- automatic alert rules or alert delivery;
- authentication or authorization for the relay `/metrics` route;
- retention or compaction telemetry;
- distributed or per-node aggregation;
- exemplars, traces, or record identifiers; or
- proof of formal soak or privileged reference-host qualification.

## Related documents

- [`OBSERVABILITY.md`](./OBSERVABILITY.md)
- [`QUARANTINE_ADMIN_HTTP.md`](./QUARANTINE_ADMIN_HTTP.md)
- [`QUARANTINE_CONCURRENCY.md`](./QUARANTINE_CONCURRENCY.md)
- [`QUARANTINE_JSONL_MAINTENANCE.md`](./QUARANTINE_JSONL_MAINTENANCE.md)
- [`QUARANTINE_BACKUP_RESTORE.md`](./QUARANTINE_BACKUP_RESTORE.md)

## Release boundary

This documentation normalization does not redefine the fixed v1.0.0 candidate:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Formal 72-hour soak evidence remains unperformed, privileged reference-host qualification/rehearsal remains incomplete, and version update, release PR, tag, and GitHub Release remain outstanding.