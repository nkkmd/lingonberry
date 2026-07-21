# Operations

**Status: active** | **Latest published release: v0.8.0** | **Next release target: v0.9.0** | **Last updated: 2026-07-22**

このディレクトリには、Lingonberryの技術決定、運用契約、operator runbook、機械可読なfailure／crash inventoryを置きます。

## v0.8.0 operational readiness

- [v0.8.0 Release Checklist](../roadmap/RELEASE_0_8_0_CHECKLIST.md)
- [v0.8.0 Release Notes](../roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [v0.8.0 Operator Runbook](./V0_8_OPERATOR_RUNBOOK.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)

v0.8.0の正式なLinux基準環境は、Ubuntu Server 24.04 LTS、x86_64、systemdです。この環境でread-only diagnosis、systemd起動契約、verified backup、isolated restore、index再構築、DR drill、restart persistenceを検証しました。他のsystemdベースLinuxはbest-effort supportとし、実装とデータ契約はUbuntu固有にしません。

Canonical operator path:

```text
install release-built binaries
→ configure
→ doctor / ready
→ start relay with systemd
→ publish / inspect
→ backup create / verify
→ isolated restore plan / apply
→ index verify / rebuild
→ isolated DR drill
→ journalctl / status / doctor / metrics diagnosis
```

## v0.7.0 storage migration and upgrade

- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [v0.7.0 Release Checklist](../roadmap/RELEASE_0_7_0_CHECKLIST.md)
- [v0.7.0 Release Notes](../roadmap/RELEASE_0_7_0_RELEASE_NOTE.md)

v0.7.0では、既存のsingle-node data directoryを明示的なoperator workflowで現在のstorage formatへ移行できます。通常起動時のimplicit migrationはありません。

```text
inspect
→ plan
→ verified backup
→ apply
→ verify
→ commit
→ resume or rollback when interrupted
```

## Quickstart

- [Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)

## Contracts and specifications

- [技術決定 ADR](./TECH_DECISION_ADR.md)
- [運用前提メモ](./OPERATIONAL_PREMISES_MEMO.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)

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

## Quarantine verified replacement and cleanup

- [Replacement Policy and Semantic-equivalence Contract](./QUARANTINE_REPLACEMENT_POLICY.md)
- [Replacement Preview and Proof Contract](./QUARANTINE_REPLACEMENT_PREVIEW.md)
- [Replacement Preview Runbook](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [Replacement Transaction Contract](./QUARANTINE_REPLACEMENT_TRANSACTION.md)
- [Generation-directory Contract](./QUARANTINE_REPLACEMENT_GENERATION.md)
- [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [Replacement Operations Hardening](./QUARANTINE_REPLACEMENT_OPERATIONS_HARDENING.md)
- [Cleanup Retention Policy](./QUARANTINE_REPLACEMENT_RETENTION_POLICY.md)
- [Cleanup Operations Runbook](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)

Canonical sequence:

```text
backup verification
→ replacement preview / proof verification
→ replacement apply / recovery
→ terminal completion evidence
→ retention evaluation
→ cleanup preview / proof
→ cleanup preparation
→ irreversible acknowledgement
→ terminal status
```

Pointer、journal、manifest、proof、inventory、completion evidence、cleanup evidenceのmanual repairは禁止です。

## General operations

- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)
- [Access and Retention Audit Checklist](./ACCESS_RETENTION_AUDIT_CHECKLIST.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)

## Multi-node

- [Multi-node Discovery and Topology](./MULTI_NODE_DISCOVERY_AND_TOPOLOGY.md)
- [Multi-node Sync Contract](./MULTI_NODE_SYNC_CONTRACT.md)
- [Multi-node Conflict Policy](./MULTI_NODE_CONFLICT_POLICY.md)
- [Multi-node Capacity and Placement Policy](./MULTI_NODE_CAPACITY_AND_PLACEMENT_POLICY.md)

Multi-node文書は将来構成の契約です。quarantine replacement／cleanup operation lockとstorage migration lockはsame-host coordinationであり、distributed lockではありません。

## Release notes

- [v0.8.0 Release Notes](../roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [v0.7.0 Release Notes](../roadmap/RELEASE_0_7_0_RELEASE_NOTE.md)
- [v0.6.0 Release Notes](../roadmap/RELEASE_0_6_0_RELEASE_NOTE.md)
- [v0.5.0 Release Notes](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)
- [v0.4.0 Release Notes](../roadmap/RELEASE_0_4_0_RELEASE_NOTE.md)
- [v0.3.0 Release Notes](../roadmap/RELEASE_0_3_0_RELEASE_NOTE.md)
- [v0.2.0 Release Notes](../roadmap/RELEASE_0_2_0_RELEASE_NOTE.md)
