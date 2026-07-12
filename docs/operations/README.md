# Operations

**Status: active** | **Last updated: 2026-07-12**

このディレクトリには、Lingonberry の技術決定と運用正本を置きます。

作業再開時は、最初に [現在の実装状況](../roadmap/CURRENT_IMPLEMENTATION_STATUS.md) と [Quarantine Lifecycle Backlog](../roadmap/QUARANTINE_LIFECYCLE_BACKLOG.md) を確認してください。前者はPR #19までの実装・runtime state・安全性ルール・再開コマンド、後者は次のissue候補と完了条件をまとめています。

## 文書

### Quickstart

- [Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)

### Contracts and Specs

- [技術決定 ADR](./TECH_DECISION_ADR.md)
- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)

### Multi-node

- [Multi-node Discovery and Topology](./MULTI_NODE_DISCOVERY_AND_TOPOLOGY.md)
- [Multi-node Sync Contract](./MULTI_NODE_SYNC_CONTRACT.md)
- [Multi-node Conflict Policy](./MULTI_NODE_CONFLICT_POLICY.md)
- [Multi-node Capacity and Placement Policy](./MULTI_NODE_CAPACITY_AND_PLACEMENT_POLICY.md)

### Policy and Operations

障害時の一次参照先は [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md) とします。

quarantine backlog の一次確認には、CLI の `quarantine-status` または HTTP の `GET /v1/quarantine-status` を使用します。Prometheus 形式の監視値は CLI の `quarantine-metrics` または HTTP の `GET /metrics` から取得します。定期的な batch revalidation は [Quarantine Scheduler](./QUARANTINE_SCHEDULER.md) を正本とし、運用上の確認事項は [Quarantine Operator Annotations](./QUARANTINE_ANNOTATIONS.md) に従ってappend-onlyで記録します。

- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Access and Retention Audit Checklist](./ACCESS_RETENTION_AUDIT_CHECKLIST.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Quarantine Operator Annotations](./QUARANTINE_ANNOTATIONS.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [Quarantine Scheduler](./QUARANTINE_SCHEDULER.md)
- [Quarantine Status API](../roadmap/QUARANTINE_STATUS_API.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
- [relay / storage separation](./RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)

### Templates and Versions

- [Container Execution Templates](./CONTAINER_EXECUTION_TEMPLATES.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)
- [Migration and Schema Versioning](./MIGRATION_AND_SCHEMA_VERSIONING.md)
