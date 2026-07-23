# Operations

[English](#english) | [日本語](#日本語)

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

**Status: v1.0.0 qualification active** | **Latest published release: v0.9.0** | **Next release target: v1.0.0** | **Last updated: 2026-07-23**

## English

This directory contains Lingonberry operational contracts, operator runbooks, deployment guidance, recovery procedures, and machine-readable failure or crash inventories.

### Current release boundary

`v0.9.0` is the latest published release. `v1.0.0` is under qualification and has not been published. The current work is release qualification, documentation normalization, and contract finalization rather than feature expansion.

Primary v1.0.0 sources:

- [v1 Compatibility Policy](../architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 Qualification Plan](../roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1.0.0 Qualification Status](../roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 Security Diff Review](../security/V1_0_SECURITY_DIFF_REVIEW.md)
- [Documentation Policy](../DOCUMENTATION_POLICY.md)
- [Documentation Inventory](../DOCUMENTATION_INVENTORY.md)
- [v1.0.0 Soak Plan](../roadmap/V1_0_SOAK_PLAN.md)
- [v1.0.0 Release Evidence](../roadmap/V1_0_RELEASE_EVIDENCE.md)

A dry run, hosted-CI rehearsal, or virtual-time rehearsal is infrastructure evidence only. Final operator acceptance and formal soak evidence must remain bound to the designated candidate and candidate-built binary digests on the reference platform.

### Canonical operator path

```text
install release-built binaries
→ configure
→ doctor / ready
→ validate and start through systemd
→ publish / inspect persisted state
→ backup create / verify
→ isolated restore plan / apply
→ index verify / rebuild
→ isolated disaster-recovery drill
→ journalctl / status / doctor / metrics diagnosis
```

The formal reference platform is Ubuntu Server 24.04 LTS, x86_64, systemd.

### Start here

For development:

- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)

For single-node operation:

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)

The v1.0 operator and upgrade runbooks are the current pre-release single-node operating guides. They remain qualification documentation until the formal soak, version update, `v1.0.0` tag, and GitHub Release are complete. The v0.8 runbooks remain historical published baselines.

### Core operational contracts

- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)

### Quarantine administration and recovery

- [Quarantine Admin HTTP and RBAC](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Concurrent Operations](./QUARANTINE_CONCURRENCY.md)
- [Quarantine Operator Annotations](./QUARANTINE_ANNOTATIONS.md)
- [Quarantine Manual Dismissals](./QUARANTINE_DISMISSALS.md)
- [Quarantine Permanent Rejections](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [Quarantine Scheduler](./QUARANTINE_SCHEDULER.md)
- [Quarantine Backup / Verify / Restore](./QUARANTINE_BACKUP_RESTORE.md)
- [Quarantine JSONL Maintenance](./QUARANTINE_JSONL_MAINTENANCE.md)
- [Replacement Policy](./QUARANTINE_REPLACEMENT_POLICY.md)
- [Replacement Preview Runbook](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [Cleanup Operations Runbook](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)

### Architecture and carrier decisions

- [Technical Decision ADR](./TECH_DECISION_ADR.md)
- [Operational Premises Memo](./OPERATIONAL_PREMISES_MEMO.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)

### Historical release records

- [v0.8.0 Operator Runbook](./V0_8_OPERATOR_RUNBOOK.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md)
- [v0.9.0 Release Checklist](../roadmap/RELEASE_0_9_0_CHECKLIST.md)
- [v0.9.0 Release Notes](../roadmap/RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.9.0 Release Evidence](../roadmap/V0_9_RELEASE_EVIDENCE.md)
- [v0.8.0 Release Notes](../roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [v0.7.0 Release Notes](../roadmap/RELEASE_0_7_0_RELEASE_NOTE.md)
- [v0.6.0 Release Notes](../roadmap/RELEASE_0_6_0_RELEASE_NOTE.md)
- [v0.5.0 Release Notes](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)

Historical and maintainer-only documents remain English-only unless the documentation policy explicitly classifies them otherwise.

---

## 日本語

このdirectoryには、Lingonberryの運用契約、operator runbook、deployment guidance、recovery procedure、機械可読なfailure／crash inventoryを配置します。

### 現在のリリース境界

最新の公開済みreleaseは`v0.9.0`です。`v1.0.0`は資格確認中で、まだ公開されていません。現在の作業は新機能追加ではなく、release qualification、文書正規化、契約の最終確定です。

v1.0.0の主要な正本:

- [v1 Compatibility Policy](../architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 Qualification Plan](../roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1.0.0 Qualification Status](../roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 Security Diff Review](../security/V1_0_SECURITY_DIFF_REVIEW.md)
- [Documentation Policy](../DOCUMENTATION_POLICY.md)
- [Documentation Inventory](../DOCUMENTATION_INVENTORY.md)
- [v1.0.0 Soak Plan](../roadmap/V1_0_SOAK_PLAN.md)
- [v1.0.0 Release Evidence](../roadmap/V1_0_RELEASE_EVIDENCE.md)

dry run、hosted CI rehearsal、virtual-time rehearsalはinfrastructure evidenceに限られます。最終operator acceptanceとformal soak evidenceは、reference platform上の指定candidateおよびcandidate build済みbinary digestに結び付いている必要があります。

### 標準operator path

```text
release build済みbinaryをinstall
→ configure
→ doctor / ready
→ systemdでvalidateして起動
→ publish / persisted state確認
→ backup create / verify
→ isolated restore plan / apply
→ index verify / rebuild
→ isolated disaster-recovery drill
→ journalctl / status / doctor / metricsで診断
```

正式reference platformはUbuntu Server 24.04 LTS、x86_64、systemdです。

### 最初に読む文書

開発用途:

- [Relay Quickstart](./RELAY_QUICKSTART.md)
- [Knowledge Object Publish Quickstart](./KNOWLEDGE_OBJECT_PUBLISH_QUICKSTART.md)
- [Storage Node Quickstart](./STORAGE_NODE_QUICKSTART.md)

single-node運用:

- [v1.0 Operator Runbook](./V1_0_OPERATOR_RUNBOOK.md)
- [v1.0 Upgrade and Rollback](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [Supported Platforms](./SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](./OPERATOR_CLI_CONTRACT.md)
- [Systemd Unit Templates](./SYSTEMD_UNIT_TEMPLATES.md)

v1.0 operator runbookとupgrade runbookは、現在のpre-release single-node運用guideです。formal soak、version更新、`v1.0.0` tag、GitHub Releaseが完了するまではqualification文書として扱います。v0.8 runbookは公開済みbaselineの履歴として残します。

### 主要運用契約

- [Storage Migration and Upgrade Contract](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [Storage Node Runtime](./STORAGE_NODE_RUNTIME.md)
- [Relay / Storage Separation](./RELAY_STORAGE_SEPARATION.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
- [Observability](./OBSERVABILITY.md)
- [Caddy Relay Publication](./CADDY_RELAY_PUBLICATION.md)

### Quarantine管理とrecovery

- [Quarantine Admin HTTP and RBAC](./QUARANTINE_ADMIN_HTTP.md)
- [Quarantine Concurrent Operations](./QUARANTINE_CONCURRENCY.md)
- [Quarantine Operator Annotations](./QUARANTINE_ANNOTATIONS.md)
- [Quarantine Manual Dismissals](./QUARANTINE_DISMISSALS.md)
- [Quarantine Permanent Rejections](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [Quarantine Observability Metrics](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [Quarantine Scheduler](./QUARANTINE_SCHEDULER.md)
- [Quarantine Backup / Verify / Restore](./QUARANTINE_BACKUP_RESTORE.md)
- [Quarantine JSONL Maintenance](./QUARANTINE_JSONL_MAINTENANCE.md)
- [Replacement Policy](./QUARANTINE_REPLACEMENT_POLICY.md)
- [Replacement Preview Runbook](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [Replacement Recovery Runbook](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [Cleanup Operations Runbook](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)

### Architectureとcarrier判断

- [Technical Decision ADR](./TECH_DECISION_ADR.md)
- [Operational Premises Memo](./OPERATIONAL_PREMISES_MEMO.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)

### 過去releaseの記録

- [v0.8.0 Operator Runbook](./V0_8_OPERATOR_RUNBOOK.md)
- [v0.8.0 Upgrade and Rollback](./V0_8_UPGRADE_AND_ROLLBACK.md)
- [v0.9.0 Release Checklist](../roadmap/RELEASE_0_9_0_CHECKLIST.md)
- [v0.9.0 Release Notes](../roadmap/RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.9.0 Release Evidence](../roadmap/V0_9_RELEASE_EVIDENCE.md)
- [v0.8.0 Release Notes](../roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [v0.7.0 Release Notes](../roadmap/RELEASE_0_7_0_RELEASE_NOTE.md)
- [v0.6.0 Release Notes](../roadmap/RELEASE_0_6_0_RELEASE_NOTE.md)
- [v0.5.0 Release Notes](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)

historical文書とmaintainer-only文書は、documentation policyで別途指定されない限り英語のみとします。
