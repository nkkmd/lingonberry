# Operations / 運用文書案内

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

**Status: v1.0.0 real-host execution deferred** | **Latest published release: v0.9.0** | **Next release target: v1.0.0** | **Last updated: 2026-10-20**

## English

**Execution hold — 2026-10-20:** At the maintainer’s request, privileged reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred. Repository preparation and recorded candidate results remain intact; this hold is neither a test failure nor a passed gate. Resume only after an explicit maintainer request, rechecking the fixed candidate, binary identities, retained artifacts, and host prerequisites. Version 1.0.0, tagging, and publication remain pending.

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

For external publisher and application integration:

- [Developer Documentation](../developers/README.md)
- [Publisher Quickstart](../developers/PUBLISHER_QUICKSTART.md)
- [Repository Publish Walkthrough](../developers/REPOSITORY_PUBLISH_WALKTHROUGH.md)

For relay and storage component development:

- [Relay Quickstart](./RELAY_QUICKSTART.md)
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

<a id="japanese"></a>

## 日本語

**実機試験の保留（2026-10-20）:** 管理者の指示により、管理者権限による基準ホストでの事前検証、ディスク圧迫試験、正式72時間連続稼働試験を一旦保留します。リポジトリ側の準備と、記録済みの公開候補の検証結果は保持します。保留は試験の失敗でも合格でもありません。明示的な再開指示を受けた後に、固定した公開候補、バイナリの識別情報、保存済みの検証記録、ホストの前提条件を再確認します。v1.0.0への更新・タグ・公開は未完了です。

このディレクトリには、Lingonberryの運用仕様、運用手順書、導入ガイド、復旧手順、障害や異常終了の一覧（機械可読形式）をまとめています。

### 現在のリリース状況

最新の公開版は`v0.9.0`です。`v1.0.0`は公開に向けた検証中で、まだ公開されていません。現在の作業は新機能追加ではなく、リリースに必要な検証、文書の整理・統一、運用仕様の最終確定です。

v1.0.0の主要な正本:

- [v1 互換性方針](../architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 適格性検証計画](../roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1.0.0 検証状況](../roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 セキュリティ差分の確認記録](../security/V1_0_SECURITY_DIFF_REVIEW.md)
- [文書方針](../DOCUMENTATION_POLICY.md)
- [文書一覧](../DOCUMENTATION_INVENTORY.md)
- [v1.0.0 連続稼働試験計画](../roadmap/V1_0_SOAK_PLAN.md)
- [v1.0.0 リリースの検証記録](../roadmap/V1_0_RELEASE_EVIDENCE.md)

試行実行、ホスト型CIでの予行演習、仮想時間を使う予行演習は試験基盤の動作を確認するための記録です。最終的な運用受入試験と正式な連続稼働試験の検証記録は、基準環境上の指定公開候補および公開候補からビルドしたバイナリのハッシュ値に結び付いている必要があります。

### 標準運用の流れ

```text
リリース用バイナリを導入
→ 設定
→ doctor / readyで確認
→ systemdの設定を検証して起動
→ データを公開し、保存状態を確認
→ バックアップを作成・検証
→ 分離した環境への復元を計画・実行
→ インデックスを検証・再構築
→ 分離した環境で障害復旧を訓練
→ journalctl / status / doctor / metricsで診断
```

正式な基準環境はUbuntu Server 24.04 LTS、x86_64、systemdです。

### 最初に読む文書

外部公開クライアントおよびアプリケーション連携:

- [開発者向け文書](../developers/README.md)
- [公開クライアントの入門ガイド](../developers/PUBLISHER_QUICKSTART.md)
- [リポジトリ付属のサンプルを使った公開手順](../developers/REPOSITORY_PUBLISH_WALKTHROUGH.md)

リレーおよびストレージの開発:

- [リレーの入門ガイド](./RELAY_QUICKSTART.md)
- [ストレージノードの入門ガイド](./STORAGE_NODE_QUICKSTART.md)

単一ノード運用:

- [v1.0 単一ノード運用手順書](./V1_0_OPERATOR_RUNBOOK.md)
- [v1.0 更新とロールバックの手順](./V1_0_UPGRADE_AND_ROLLBACK.md)
- [対応環境](./SUPPORTED_PLATFORMS.md)
- [運用CLIの仕様](./OPERATOR_CLI_CONTRACT.md)
- [systemdユニットのテンプレートと仕様](./SYSTEMD_UNIT_TEMPLATES.md)

v1.0の運用手順書と更新手順書は、公開前の単一ノード運用ガイドです。正式な連続稼働試験、バージョン更新、`v1.0.0` タグ、GitHub Releaseが完了するまでは適格性検証文書として扱います。v0.8の手順書は、公開済みの基準構成を示す過去の記録として残します。

### 主要な運用仕様

- [保存形式の移行と更新の仕様](./STORAGE_MIGRATION_AND_UPGRADE.md)
- [ストレージノードの実行時仕様](./STORAGE_NODE_RUNTIME.md)
- [リレーとストレージの分離](./RELAY_STORAGE_SEPARATION.md)
- [ノードの起動から停止までの運用手順](./NODE_LIFECYCLE_RUNBOOK.md)
- [秘密情報の管理](./SECRET_MANAGEMENT.md)
- [稼働状況の監視](./OBSERVABILITY.md)
- [Caddyを使ったリレーの公開](./CADDY_RELAY_PUBLICATION.md)

### 隔離領域の管理と復旧

- [隔離領域の管理用HTTP APIと役割に基づくアクセス制御](./QUARANTINE_ADMIN_HTTP.md)
- [隔離領域での並行操作](./QUARANTINE_CONCURRENCY.md)
- [隔離データへの運用担当者による注記](./QUARANTINE_ANNOTATIONS.md)
- [隔離データの手動除外](./QUARANTINE_DISMISSALS.md)
- [隔離データの恒久的な拒否](./QUARANTINE_PERMANENT_REJECTIONS.md)
- [隔離領域の監視指標](./QUARANTINE_OBSERVABILITY_METRICS.md)
- [隔離領域の処理スケジューラー](./QUARANTINE_SCHEDULER.md)
- [隔離領域のバックアップ・検証・復元](./QUARANTINE_BACKUP_RESTORE.md)
- [隔離領域のJSONL保守](./QUARANTINE_JSONL_MAINTENANCE.md)
- [隔離データの置換方針](./QUARANTINE_REPLACEMENT_POLICY.md)
- [置換前の確認手順](./QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md)
- [置換処理の復旧手順](./QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md)
- [置換処理後の整理手順](./QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)

### 設計とcarrierに関する判断

- [技術判断の記録](./TECH_DECISION_ADR.md)
- [運用上の前提条件](./OPERATIONAL_PREMISES_MEMO.md)
- [carrierに関する判断記録](./CARRIER_DECISION_MEMO.md)
- [HTTP carrierの仕様](./HTTP_CARRIER_CONTRACT.md)
- [ファイル・アーカイブcarrierの仕様](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [carrierの対応機能の確認](./CARRIER_CAPABILITY_NEGOTIATION.md)

### 過去リリースの記録

- [v0.8.0 運用手順書](./V0_8_OPERATOR_RUNBOOK.md)
- [v0.8.0 更新とロールバックの手順](./V0_8_UPGRADE_AND_ROLLBACK.md)
- [v0.9.0 リリース確認表](../roadmap/RELEASE_0_9_0_CHECKLIST.md)
- [v0.9.0 リリースノート](../roadmap/RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.9.0 リリースの検証記録](../roadmap/V0_9_RELEASE_EVIDENCE.md)
- [v0.8.0 リリースノート](../roadmap/RELEASE_0_8_0_RELEASE_NOTE.md)
- [v0.7.0 リリースノート](../roadmap/RELEASE_0_7_0_RELEASE_NOTE.md)
- [v0.6.0 リリースノート](../roadmap/RELEASE_0_6_0_RELEASE_NOTE.md)
- [v0.5.0 リリースノート](../roadmap/RELEASE_0_5_0_RELEASE_NOTE.md)

過去の文書と保守担当者専用の文書は、文書方針で別途指定されない限り英語のみとします。
