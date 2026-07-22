# ロードマップ

**Status: v1.0.0 qualification active** | **Latest published release: v0.9.0** | **Next release target: v1.0.0** | **Last updated: 2026-07-23**

このディレクトリには、Lingonberryの実装・運用準備・releaseに関するroadmap、checklist、release note、release evidence、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

現在のv1.0.0作業では、次の順に確認します。

1. [v1.0.0 Qualification Status](./V1_0_QUALIFICATION_STATUS.md)
2. [v1.0.0 Qualification Plan](./V1_0_QUALIFICATION_PLAN.md)
3. [v1 Compatibility Policy](../architecture/V1_COMPATIBILITY_POLICY.md)
4. [v1 Rust Public API Audit](../architecture/V1_0_RUST_API_AUDIT.md)
5. [v1.0.0 Security Diff Review](../security/V1_0_SECURITY_DIFF_REVIEW.md)
6. [v1.0.0 Documentation Freeze Plan](./V1_0_DOCUMENTATION_FREEZE_PLAN.md)
7. [v1.0.0 Documentation Walkthrough](./V1_0_DOCUMENTATION_WALKTHROUGH.md)
8. [v1.0.0 Soak Plan](./V1_0_SOAK_PLAN.md)
9. [v1.0.0 Release Evidence](./V1_0_RELEASE_EVIDENCE.md)
10. [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md)
11. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
12. [運用文書索引](../operations/README.md)

候補qualification workflowのdry run成功は、orchestratorと証拠形式の検証です。最終candidateのqualification、operator acceptance、security disposition、documentation walkthrough、72時間soakの代替にはなりません。

## v1.0.0 qualificationの現在地

完了済み:

- gate inventoryとqualification plan
- Rust public API audit
- normative v1 compatibility policy
- soak／telemetry contract
- candidate-bound qualification workflowとdry run
- pre-candidate security diff review
- documentation freeze plan
- documentation freeze file／link integrity check

未完了:

- documentation contradiction dispositionとcandidate walkthrough
- final candidate designation
- candidate-bound qualification rerun
- reference-platform operator acceptance
- final security disposition
- 72時間qualification soak
- version `1.0.0`、release checklist、release notes、changelogの準備
- merged-commit validation、tag、GitHub Release、最終evidence記録

## v0.9.0 release record

v0.9.0は、v1.0 stable single-node contractへ進む前のrelease-candidate hardeningとして、protocol parserのresource boundedness、signature verification temporary workspaceの安全性、public API inventory、security disposition、version整合、bounded soak evidenceを固定しました。

- [v0.9.0 Release Checklist](./RELEASE_0_9_0_CHECKLIST.md)
- [v0.9.0 Release Notes](./RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.9.0 Release Evidence](./V0_9_RELEASE_EVIDENCE.md)
- [v0.9.0 Hardening Plan](./V0_9_HARDENING_PLAN.md)
- [v0.9.0 Security Review](../security/V0_9_SECURITY_REVIEW.md)
- [v0.9.0 Security Findings](../security/V0_9_SECURITY_FINDINGS.md)
- [v0.9.0 Public API Freeze Candidate](../architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md)
- [v0.9.0 Rust API Inventory](../architecture/V0_9_RUST_API_INVENTORY.md)

Publication record:

- PR #108 merged
- merge commit `971155340603afdc0c9c5bd37e596f49c260d15e`
- tag `v0.9.0`
- GitHub Release `v0.9.0` published

## Operator baseline

v1.0.0は、v0.8.0で確立したUbuntu Server 24.04 LTS、x86_64、systemdのsingle-node operator baselineを維持します。

- [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md)
- [Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md)
- [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md)
- [Storage Migration and Upgrade Contract](../operations/STORAGE_MIGRATION_AND_UPGRADE.md)

## 文書の役割

- `V1_0_QUALIFICATION_PLAN.md`: mandatory gate、classification、pass／blocker criteria
- `V1_0_QUALIFICATION_STATUS.md`: 現在の実行状態と次の順序
- `V1_0_DOCUMENTATION_FREEZE_PLAN.md`: freeze対象、walkthrough条件、change control
- `V1_0_DOCUMENTATION_WALKTHROUGH.md`: 文書・手順ごとの静的／実行レビュー記録
- `V1_0_SOAK_PLAN.md`: 72時間soakのworkload、telemetry、停止条件
- `V1_0_RELEASE_EVIDENCE.md`: 最終candidateとpublicationに結び付く証拠正本
- `CURRENT_IMPLEMENTATION_STATUS.md`: 実装済み範囲と作業再開用の全体状態
- `ROADMAP_TO_V1_0.md`: release-level sequenceとv1.0境界

## Release history

- v0.9.0: release-candidate hardening、bounded parser、secure signature workspace、public-contract freeze evidence、bounded soak
- v0.8.0: single-node operational readiness、Ubuntu 24.04 reference platform、operator diagnostics、verified recovery、systemd deployment、fresh-runner acceptance
- v0.7.0: storage-format manifest、deterministic migration、verified backup binding、resume／rollback
- v0.6.0: append-only transitions、durable reevaluation、deterministic effective views、bounded diagnostics
- v0.5.0: versioned normal object lifecycle、deterministic index lifecycle、checkpoint／catch-up、restart／recovery smoke
- v0.4.0: deterministic retention cleanup、proof-bound authorization、verified cleanup transaction、path-level recovery
- v0.3.0: verified replacement transaction、generation publication、recovery、operations hardening
- v0.2.0: persistent quarantine lifecycle、backup／restore、RBAC
- v0.1.0: initial protocol／schema／fixtures／carrier contracts

## 絶対に崩さない境界

- validation未通過objectをcanonical storageへ保存しない
- canonical storageよりindexをsemantic sourceとして優先しない
- conflict時に既存objectを上書きしない
- normal startupでimplicit storage migrationを実行しない
- unknown newer storage formatをmutateしない
- active state／data directoryへrestoreしない
- archive segmentやimmutable evidence ledgerをrewrite／deleteしない
- untrusted JSONを上限なしでrecursive parseしない
- signature verification artifactを既存pathへ上書きしない
- same-host lockをdistributed lockとして扱わない
