# ロードマップ

**Status: v0.4.0 release candidate** | **Latest published release: v0.3.0** | **Last updated: 2026-07-17**

このディレクトリには、実装・運用準備・releaseのroadmap、checklist、release note、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md)
3. [v0.4.0 Release Checklist](./RELEASE_0_4_0_CHECKLIST.md)
4. [v0.4.0 Release Notes](./RELEASE_0_4_0_RELEASE_NOTE.md)
5. [v0.4.0 Roadmap](./RELEASE_0_4_0_ROADMAP.md)
6. [Quarantine Replacement Retention Policy](../operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md)
7. [Cleanup Operations Runbook](../operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md)
8. [運用文書索引](../operations/README.md)
9. [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md)
10. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)

`CURRENT_IMPLEMENTATION_STATUS.md`は中断後に作業を再開するための引き継ぎ用正本です。`ROADMAP_TO_V1_0.md`はv0.4.0以降のrelease-level roadmapです。v0.4.0のrelease可否はchecklist、release notes、main branch CIを正本として判断します。

## v0.4.0の到達点

- inactive committed／rolled-back generationのdeterministic retention evaluation
- retained generation floorとdurable completion age evidence
- canonical cleanup plan／proofとdigest sidecar
- active pointer、journal、generation、completion evidence、managed-path inventoryのstate-bound revalidation
- stale state、path traversal、symlink、unexpected entryのfail-closed rejection
- same-host lock下のcleanup transaction
- same-filesystem tomb preparationとsealed inventory
- deterministic `generation_id/managed_path` processing order
- durable path-level progress、idempotent resume、pre-processing rollback
- irreversible boundary後の`recovery-required`／`partially-deleted`分類
- explicit double opt-in authorization
- operator runbook、failure-point inventory、crash matrix、smoke procedure、release checklist／notes

v0.4.0ではbackground scheduled deletionを導入しません。cleanupはexact subjectを指定するoperator-triggered double opt-inに限定します。

## v0.4.0で意図的に保持するもの

terminal cleanup transaction workspaceは、journal、digest、sealed inventory、path-level progress、terminal stateを含む運用証拠として自動削除しません。将来のworkspace retention policyはIssue #72で別versionとして設計します。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 現在のrelease state、実装済み機能、安全境界、次の作業
- [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md): v0.4.0以降のrelease sequence、v1.0の境界、release gate
- [v0.4.0 Release Checklist](./RELEASE_0_4_0_CHECKLIST.md): release gateとpublication確認
- [v0.4.0 Release Notes](./RELEASE_0_4_0_RELEASE_NOTE.md): 公開範囲、互換性、安全性、既知制約
- [v0.4.0 Roadmap](./RELEASE_0_4_0_ROADMAP.md): verified retention cleanupの設計・実装計画
- [Retention Policy](../operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md): cleanup適格性、proof、revalidation、不可逆境界の正本
- [Cleanup Runbook](../operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md): operatorの停止条件、回復分類、証拠保全
- [v0.3.0 Release Checklist](./RELEASE_0_3_0_CHECKLIST.md): 前releaseの完了記録
- [v0.3.0 Release Notes](./RELEASE_0_3_0_RELEASE_NOTE.md): 前releaseの公開記録
- [Quarantine Lifecycle Backlog](./QUARANTINE_LIFECYCLE_BACKLOG.md): quarantine継続作業
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md): core実装の中長期計画
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md): 全体roadmapのissue分解
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md): 実運用に向けた中長期計画
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md): 運用準備のissue分解

## Release history

- v0.4.0: release candidate。code、package version、release notesはmainへmerge済み。tag／GitHub Releaseはmain CI確認後に公開する。
- v0.3.0: verified replacement transaction、generation publication、recovery、operations hardening。
- v0.2.0: persistent quarantine lifecycle、backup／restore、RBAC。
- v0.1.0: initial protocol／schema／fixtures／carrier contracts。

## 絶対に崩さない境界

- active generationをcleanup対象にしない
- filesystem timestampだけでeligibilityを判断しない
- wildcard、implicit-all、partial selectionをapplyに使わない
- symbolic linkやunsupported entry typeをfollow／acceptしない
- stale proofやcontradictory stateを自動修復しない
- archive segmentやimmutable evidence ledgerをrewrite／deleteしない
- irreversible processing開始後にrollbackを案内しない
- same-host lockをdistributed lockとして扱わない
- terminal cleanup workspaceをv0.4.0で自動削除しない
