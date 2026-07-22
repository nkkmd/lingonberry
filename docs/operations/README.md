# Operations

**Status: active** | **Latest published release: v0.9.0** | **Next release target: v1.0.0** | **Last updated: 2026-07-22**

このディレクトリには、Lingonberryの技術決定、運用契約、operator runbook、機械可読なfailure／crash inventoryを置きます。

## v0.9.0 hardening release

- [v0.9.0 Release Checklist](../roadmap/RELEASE_0_9_0_CHECKLIST.md)
- [v0.9.0 Release Notes](../roadmap/RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.9.0 Release Evidence](../roadmap/V0_9_RELEASE_EVIDENCE.md)
- [v0.9.0 Hardening Plan](../roadmap/V0_9_HARDENING_PLAN.md)
- [v0.9.0 Security Review](../security/V0_9_SECURITY_REVIEW.md)
- [v0.9.0 Security Findings](../security/V0_9_SECURITY_FINDINGS.md)
- [Signature Workspace Remediation](../security/V0_9_SIGNATURE_WORKSPACE_REMEDIATION.md)
- [v0.9.0 Public API Freeze Candidate](../architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md)
- [v0.9.0 Rust API Inventory](../architecture/V0_9_RUST_API_INVENTORY.md)

v0.9.0はv0.8.0のsingle-node operator contractを維持したまま、protocol JSON parserへ1 MiB input limitとdepth 128 limitを導入し、signature verification temporary workspaceをexclusive creation、owner-only permission、create-new artifact、normal-path cleanupでhardeningしました。

Operationally relevant boundaries:

- oversized／deeply nested JSONはcanonical processingへ進む前にfail closedで拒否する
- signature verification artifactは既存pathへ上書きしない
- request payload、signature、temporary pathをpublic errorへ含めない
- normal success／failure pathでtemporary verification workspaceを残さない
- process crash後のstale temporary entryはsecure eraseを保証せず、host temporary-directory policyの対象とする
- v0.9.0は新しいstorage formatやimplicit migrationを導入しない

Publication record:

- PR #108 merged
- merge commit `971155340603afdc0c9c5bd37e596f49c260d15e`
- tag `v0.9.0`
- GitHub Release `v0.9.0` published

## v0.8.0 operational baseline

- [v0.8.0 Release Checklist](../roadmap/RELEASE_0_8_0_CHECKLIST.md)
- [v0.8.0 Release Notes](../roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [v0.8.0 Operator Runbook](./V0_8_OPERATOR_RUNBOOK.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)

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

Multi-node文書は将来構成の契約です。same-host coordinationをdistributed lockとして扱いません。

## Release notes

- [v0.9.0 Release Notes](../roadmap/RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.8.0 Release Notes](../roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [v0.7.0 Release Notes](../roadmap/RELEASE_0_7_0_RELEASE_NOTE.md)
- [v0.6.0 Release Notes](../roadmap/RELEASE_0_6_0_RELEASE_NOTE.md)
- [v0.5.0 Release Notes](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)
- [v0.4.0 Release Notes](../roadmap/RELEASE_0_4_0_RELEASE_NOTE.md)
- [v0.3.0 Release Notes](../roadmap/RELEASE_0_3_0_RELEASE_NOTE.md)
- [v0.2.0 Release Notes](../roadmap/RELEASE_0_2_0_RELEASE_NOTE.md)
