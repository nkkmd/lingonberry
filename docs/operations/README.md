# Operations

**Status: active** | **Latest published release: v0.5.0** | **Next release target: v0.6.0** | **Last updated: 2026-07-19**

このディレクトリには、Lingonberryの技術決定、運用契約、operator runbook、機械可読なfailure／crash inventoryを置きます。

作業再開時は、[現在の実装状況](../roadmap/CURRENT_IMPLEMENTATION_STATUS.md)、[v0.5.0 Release Checklist](../roadmap/RELEASE_0_5_0_CHECKLIST.md)、[v0.5.0 Release Notes](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)を最初に確認してください。

## v0.5.0 normal object lifecycle

- [v0.5.0 Release Roadmap](../roadmap/RELEASE_0_5_0_ROADMAP.md)
- [v0.5.0 Release Checklist](../roadmap/RELEASE_0_5_0_CHECKLIST.md)
- [v0.5.0 Release Notes](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)
- [Index Lifecycle Contract](../../packages/indexer/INDEX_LIFECYCLE.md)
- [Index Catch-up Contract](../../packages/indexer/INDEX_CATCH_UP.md)

v0.5.0は2026-07-19に公開されました。canonical storageを正本とし、indexをdeterministicに検証・再構築可能な派生状態として扱います。corrupt、unsupported、partial、stale、ambiguous stateはfail closedで扱い、inconsistent resultからcheckpointを更新しません。

- Tag: `v0.5.0`
- Release target commit: `bf8176da0d992152fb116ca0c45177904d1aa61c`

## v0.4.0 verified cleanup

- [Quarantine Replacement Retention Policy](./QUARANTINE_REPLACEMENT_RETENTION_POLICY.md)
- [Cleanup Operations Runbook](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
- [Cleanup Failure-point Inventory](./quarantine-replacement-cleanup-failure-points.v1.json)
- [Cleanup Crash Matrix](./quarantine-replacement-cleanup-crash-matrix.v1.json)
- [v0.4.0 Smoke Test Procedure](./QUARANTINE_REPLACEMENT_V0_4_0_SMOKE_TEST.md)

v0.4.0 cleanupはexact-subject、proof-bound、operator-triggered、double opt-inです。scheduled／unattended cleanupはなく、terminal cleanup transaction workspaceは運用証拠として保持します。

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

Multi-node文書は将来構成の契約です。quarantine replacement／cleanup operation lockはsame-host coordinationであり、distributed lockではありません。

## Quarantine administration

- [Quarantine Admin HTTP and RBAC](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Concurrent Operations](./QUARANTINE_CONCURRENCY.md)
- [Quarantine Operator Annotations](./QUARANTINE_ANNOTATIONS.md)
- [Quarantine Manual Dismissals](./QUARANTINE_DISMISSALS.md)
- [Quarantine Permanent Rejections](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [Quarantine Status API](../roadmap/QUARANTINE_STATUS_API.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [Quarantine Scheduler](./QUARANTINE_SCHEDULER.md)

## Quarantine data protection and maintenance

- [Quarantine Backup / Verify / Restore](./QUARANTINE_BACKUP_RESTORE.md)
- [Quarantine JSONL Index, Rotation, and Maintenance](./QUARANTINE_JSONL_MAINTENANCE.md)
- [Quarantine Compaction Preview and Proof](./QUARANTINE_COMPACTION_PROOF.md)

## Quarantine verified replacement

- [Replacement Policy and Semantic-equivalence Contract](./QUARANTINE_REPLACEMENT_POLICY.md)
- [Replacement Preview and Proof Contract](./QUARANTINE_REPLACEMENT_PREVIEW.md)
- [Replacement Preview Runbook](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [Replacement Transaction Contract](./QUARANTINE_REPLACEMENT_TRANSACTION.md)
- [Generation-directory Contract](./QUARANTINE_REPLACEMENT_GENERATION.md)
- [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [Replacement Operations Hardening](./QUARANTINE_REPLACEMENT_OPERATIONS_HARDENING.md)
- [Replacement Crash-point Inventory](./quarantine-replacement-crash-points.v1.json)

Canonical sequence: backup v2 verification → replacement preview/proof verification → replacement apply/recovery → terminal completion evidence → retention evaluation → cleanup preview/proof → cleanup preparation → separate irreversible acknowledgement → terminal status. Pointer、journal、inventoryのmanual repairは禁止です。

## General operations

- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Access and Retention Audit Checklist](./ACCESS_RETENTION_AUDIT_CHECKLIST.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)

## Templates and Versions

- [Container Execution Templates](./CONTAINER_EXECUTION_TEMPLATES.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Migration and Schema Versioning](./MIGRATION_AND_SCHEMA_VERSIONING.md)
- [Legacy Admin Token Deprecation](../roadmap/RBAC_LEGACY_TOKEN_DEPRECATION.md)
- [v0.5.0 Release Notes](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)
- [v0.4.0 Release Notes](../roadmap/RELEASE_0_4_0_RELEASE_NOTE.md)
- [v0.3.0 Release Notes](../roadmap/RELEASE_0_3_0_RELEASE_NOTE.md)
- [v0.2.0 Release Notes](../roadmap/RELEASE_0_2_0_RELEASE_NOTE.md)