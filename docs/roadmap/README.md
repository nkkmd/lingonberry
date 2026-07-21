# ロードマップ

**Status: v0.7.0 released** | **Latest published release: v0.7.0** | **Next release target: v0.8.0** | **Last updated: 2026-07-21**

このディレクトリには、実装・運用準備・releaseのroadmap、checklist、release note、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md)
3. [v0.7.0 Release Checklist](./RELEASE_0_7_0_CHECKLIST.md)
4. [v0.7.0 Release Notes](./RELEASE_0_7_0_RELEASE_NOTE.md)
5. [Storage Migration and Upgrade Contract](../operations/STORAGE_MIGRATION_AND_UPGRADE.md)
6. [v0.6.0 Release Checklist](./RELEASE_0_6_0_CHECKLIST.md)
7. [v0.6.0 Release Notes](./RELEASE_0_6_0_RELEASE_NOTE.md)
8. [v0.5.0 Release Roadmap](./RELEASE_0_5_0_ROADMAP.md)
9. [運用文書索引](../operations/README.md)
10. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)

`CURRENT_IMPLEMENTATION_STATUS.md`は中断後に作業を再開するための引き継ぎ用正本です。`ROADMAP_TO_V1_0.md`はrelease-level roadmapです。v0.7.0の実装・検証・公開記録は`RELEASE_0_7_0_CHECKLIST.md`、`RELEASE_0_7_0_RELEASE_NOTE.md`、`STORAGE_MIGRATION_AND_UPGRADE.md`を正本とします。

## v0.7.0の到達点

v0.7.0では、既存single-node installationをデータ保持したまま継続的にupgradeするためのstorage format契約とmigration frameworkを導入しました。

```text
inspect
→ deterministic plan
→ plan-bound verified backup
→ apply
→ verify
→ commit
→ resume or rollback
```

主な到達点:

- `storage-format.manifest` v1とstable layout identifier
- deterministic read-only durable inventoryとsource digest
- `empty`／`legacy_unversioned`／`supported`／`unknown_newer`／`corrupt`分類
- unknown newer format、malformed manifest、unsupported layout、symlink、special fileのfail-closed拒否
- source stateとtarget formatへboundされたdeterministic migration plan
- durable migration journalとvalidated state transition
- plan IDとsource digestへboundされたverified backup snapshot
- apply／verify／commit／resume／rollback／statusの実処理
- dedicated `lingonberry-storage-migrate` operator CLI
- v0.4.0-equivalent persistent fixtureとintegration coverage
- upgrade／downgrade／deprecated-configuration policy
- workspace package version `0.7.0`
- tag `v0.7.0`とGitHub Release公開

v0.7.0のstorage format migrationはcanonical object dataをsemantic rewriteしません。通常起動時のimplicit migrationはなく、automatic downgradeもありません。

## v0.6.0の到達点

v0.6.0では、original Knowledge Objectを変更しないappend-only Transition Objectとdeterministic effective viewを導入しました。

- signed `POST /v1/transitions`
- replace／withdraw Transition Object
- duplicate／immutable conflict／orphan evidence handling
- durable target-scoped reevaluationとrestart reconciliation
- deterministic evidence generations
- last-known-good effective view
- stable bounded diagnosticsとgeneration-fixed pagination

## v0.5.0の到達点

v0.5.0では、通常のknowledge object lifecycleをend-to-endで完成させました。

- ingestion pipelineとtransaction boundary
- public read／write API
- versioned machine-readable error contract
- duplicate／conflict規則
- storageを正本としたindex verify／catch-up／rebuild
- restartとpartial index updateを含むE2E smoke

## v0.4.0の到達点

- inactive committed／rolled-back generationのdeterministic retention evaluation
- retained generation floorとdurable completion age evidence
- canonical cleanup plan／proofとdigest sidecar
- state-bound revalidation
- stale state、path traversal、symlink、unexpected entryのfail-closed rejection
- same-host lock下のcleanup transaction
- durable path-level progress、idempotent resume、pre-processing rollback
- irreversible boundary後のrecovery classification
- explicit double opt-in authorization

v0.4.0ではbackground scheduled deletionを導入していません。cleanupはexact subjectを指定するoperator-triggered double opt-inに限定します。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 現在のrelease state、実装済み機能、安全境界、次の作業
- [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md): release sequence、v1.0の境界、release gate
- [v0.7.0 Release Checklist](./RELEASE_0_7_0_CHECKLIST.md): v0.7.0のrelease gateとpublication記録
- [v0.7.0 Release Notes](./RELEASE_0_7_0_RELEASE_NOTE.md): v0.7.0の公開範囲、互換性、安全性、既知制約
- [Storage Migration and Upgrade Contract](../operations/STORAGE_MIGRATION_AND_UPGRADE.md): storage format、migration、resume、rollback、upgrade／downgrade policyの正本
- [v0.6.0 Release Checklist](./RELEASE_0_6_0_CHECKLIST.md): Transition／effective-view releaseの検証記録
- [v0.6.0 Release Notes](./RELEASE_0_6_0_RELEASE_NOTE.md): v0.6.0の公開範囲
- [実装ロードマップ](./IMPLEMENTATION_ROADMAP.md): core実装の中長期計画
- [実装バックログ](./IMPLEMENTATION_BACKLOG.md): 全体roadmapのissue分解
- [運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md): 実運用に向けた中長期計画
- [運用準備バックログ](./OPERATIONAL_READINESS_BACKLOG.md): 運用準備のissue分解

## Release history

- v0.7.0: storage-format manifest、deterministic migration、verified backup binding、resume／rollback。
- v0.6.0: append-only transitions、durable reevaluation、deterministic effective views、bounded diagnostics。
- v0.5.0: versioned normal object lifecycle、deterministic index lifecycle、checkpoint／catch-up、restart／recovery smoke。
- v0.4.0: deterministic retention cleanup、proof-bound authorization、verified cleanup transaction、path-level recovery。
- v0.3.0: verified replacement transaction、generation publication、recovery、operations hardening。
- v0.2.0: persistent quarantine lifecycle、backup／restore、RBAC。
- v0.1.0: initial protocol／schema／fixtures／carrier contracts。

## 絶対に崩さない境界

- validation未通過objectをcanonical storageへ保存しない
- canonical storageよりindexをsemantic sourceとして優先しない
- duplicateとconflictを同一分類にしない
- conflict時に既存objectを上書きしない
- original Knowledge ObjectをTransition Objectでrewrite／deleteしない
- incomplete evidenceでlast-known-good semantic checkpointを上書きしない
- normal startupでimplicit storage migrationを実行しない
- unknown newer storage formatをmutateしない
- required verified backup evidenceなしにnon-empty migrationを開始しない
- durable verification前にtarget formatをcommittedとして公開しない
- manifestを削除・編集してdowngradeを強制する手順を案内しない
- active generationをcleanup対象にしない
- filesystem timestampだけでcleanup eligibilityを判断しない
- wildcard、implicit-all、partial selectionをcleanup applyに使わない
- symbolic linkやunsupported entry typeをfollow／acceptしない
- stale proofやcontradictory stateを自動修復しない
- archive segmentやimmutable evidence ledgerをrewrite／deleteしない
- irreversible processing開始後にrollbackを案内しない
- same-host lockをdistributed lockとして扱わない
