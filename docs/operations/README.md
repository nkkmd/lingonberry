# Operations

**Status: active** | **Latest release: v0.3.0** | **Last updated: 2026-07-15**

このディレクトリには、Lingonberryの技術決定と運用正本を置きます。

作業再開時は、最初に[現在の実装状況](../roadmap/CURRENT_IMPLEMENTATION_STATUS.md)、[v0.3.0 Release Checklist](../roadmap/RELEASE_0_3_0_CHECKLIST.md)、[v0.3.0 Release Notes](../roadmap/RELEASE_0_3_0_RELEASE_NOTE.md)を確認してください。

## Quickstart

- [Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)

## Contracts and Specs

- [技術決定 ADR](./TECH_DECISION_ADR.md)
- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)

## Multi-node

- [Multi-node Discovery and Topology](./MULTI_NODE_DISCOVERY_AND_TOPOLOGY.md)
- [Multi-node Sync Contract](./MULTI_NODE_SYNC_CONTRACT.md)
- [Multi-node Conflict Policy](./MULTI_NODE_CONFLICT_POLICY.md)
- [Multi-node Capacity and Placement Policy](./MULTI_NODE_CAPACITY_AND_PLACEMENT_POLICY.md)

Multi-node文書は将来構成の契約です。v0.3.0のquarantine operation lockもsame-host coordinationであり、distributed lockではありません。

## Policy and Operations

障害時の一次参照先は[Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)です。

### Quarantine administration

- [Quarantine Admin HTTP and RBAC](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Concurrent Operations](./QUARANTINE_CONCURRENCY.md)
- [Quarantine Operator Annotations](./QUARANTINE_ANNOTATIONS.md)
- [Quarantine Manual Dismissals](./QUARANTINE_DISMISSALS.md)
- [Quarantine Permanent Rejections](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [Quarantine Status API](../roadmap/QUARANTINE_STATUS_API.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [Quarantine Scheduler](./QUARANTINE_SCHEDULER.md)

### Quarantine data protection and maintenance

- [Quarantine Backup / Verify / Restore](./QUARANTINE_BACKUP_RESTORE.md)
- [Quarantine JSONL Index, Rotation, and Maintenance](./QUARANTINE_JSONL_MAINTENANCE.md)
- [Quarantine Compaction Preview and Proof](./QUARANTINE_COMPACTION_PROOF.md)

### Quarantine verified replacement

- [Replacement Policy and Semantic-equivalence Contract](./QUARANTINE_REPLACEMENT_POLICY.md)
- [Replacement Preview and Proof Contract](./QUARANTINE_REPLACEMENT_PREVIEW.md)
- [Replacement Preview Runbook](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [Replacement Transaction Contract](./QUARANTINE_REPLACEMENT_TRANSACTION.md)
- [Generation-directory Contract](./QUARANTINE_REPLACEMENT_GENERATION.md)
- [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [Replacement Operations Hardening](./QUARANTINE_REPLACEMENT_OPERATIONS_HARDENING.md)
- [Crash-point Inventory](./quarantine-replacement-crash-points.v1.json)

The canonical operator sequence is backup v2 verification → replacement preview/proof verification → apply → structured status/metrics → index and segment verification → read-only generation inspection. Pointer or journal repair must never be performed manually.

### General operations

- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Access and Retention Audit Checklist](./ACCESS_RETENTION_AUDIT_CHECKLIST.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
- [relay / storage separation](./RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)

## Templates and Versions

- [Container Execution Templates](./CONTAINER_EXECUTION_TEMPLATES.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Migration and Schema Versioning](./MIGRATION_AND_SCHEMA_VERSIONING.md)
- [Legacy Admin Token Deprecation](../roadmap/RBAC_LEGACY_TOKEN_DEPRECATION.md)
- [v0.3.0 Release Notes](../roadmap/RELEASE_0_3_0_RELEASE_NOTE.md)
- [v0.2.0 Release Notes](../roadmap/RELEASE_0_2_0_RELEASE_NOTE.md)
