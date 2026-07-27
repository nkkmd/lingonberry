# ロードマップ

**Status: v1.0.0 reference-host preflight ready** | **Latest published release: v0.9.0** | **Next release target: v1.0.0** | **Last updated: 2026-07-27**

このディレクトリには、Lingonberryの実装・運用準備・qualification・releaseに関するroadmap、checklist、release note、release evidence、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

現在のv1.0.0作業では、次の順に確認します。

1. [v1.0.0 Qualification Status](./V1_0_QUALIFICATION_STATUS.md)
2. [v1.0.0 Release Evidence](./V1_0_RELEASE_EVIDENCE.md)
3. [v1.0.0 Documentation Walkthrough](./V1_0_DOCUMENTATION_WALKTHROUGH.md)
4. [v1.0.0 Security Diff Review](../security/V1_0_SECURITY_DIFF_REVIEW.md)
5. [v1.0.0 Qualification Plan](./V1_0_QUALIFICATION_PLAN.md)
6. [v1 Compatibility Policy](../architecture/V1_COMPATIBILITY_POLICY.md)
7. [v1 Rust Public API Audit](../architecture/V1_0_RUST_API_AUDIT.md)
8. [v1.0.0 Soak Plan](./V1_0_SOAK_PLAN.md)
9. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
10. [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md)
11. [運用文書索引](../operations/README.md)

Dry runやvirtual-time rehearsalはtoolingとevidence形式の検証です。candidate-bound qualification、reference-host preflight、正式72時間soakの代替にはなりません。

## Active candidate

```text
candidate:
8c6b48082205a3af555130eec1f3e7d2ac8811fe

lingonberry-storage SHA-256:
737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507

lingonberry-relay SHA-256:
23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

Evidence／documentation commitを追加してもcandidateは移動しません。旧candidate `f9543019f2c219aea3b085ff90f2da201b268a48`に結び付く実行証拠は歴史記録です。

## v1.0.0 qualificationの現在地

### 完了済み

- gate inventoryとqualification plan
- Rust public API audit
- normative v1 compatibility policy
- soak／telemetry contract
- candidate designationとexact SHA freeze
- Ed25519 publisher-signature enforcement
- candidate-delta security reviewとfinal security disposition
- exact-candidate qualification runとartifact独立検証
- documentation freeze／inventory／bilingual／link integrity checks
- 16-procedure candidate documentation walkthrough
- qualification／walkthrough binary identityと全bundle checksumの独立検証
- repository-side pre-real-host preparation
- reference-host checker、command map、disk-pressure contractのactive identity alignment

### 現在実行する項目

- privileged reference-host preflight（Issue #343）
- startup、restart、signed publish、persistence、diagnostics、backup／restore、index checks
- malformed／invalid-signature／verifier-failure／duplicate／conflict fail-closed checks
- isolated disk-pressure rehearsal
- host provenance、command map、threshold、UTC timeline、deviation、artifact digestの固定

### Reference-host preflight PASS後に実行する項目

- formal 72-hour qualification soak
- soak artifactとresource-growth dispositionの独立検証
- version `1.0.0`、release checklist、release notes、CHANGELOGの準備
- release PR review
- merged-commit CI／qualification
- annotated tag `v1.0.0`
- GitHub Release
- published artifact digestと最終release evidenceの確定

## GO / NO-GO

```text
GO:
- privileged reference-host preflight

NO-GO:
- formal 72-hour soak until preflight passes
- version/tag/publication until soak and release validation pass
```

## Evidence record

### Candidate qualification

- run `30238378797`
- artifact `8642393171`
- digest `sha256:c30a0472f6ea07f3e395c9a27c67d1460b8f35a13a7afd397bd0e5895cb93b3e`

### Documentation walkthrough

- run `30239602412`
- artifact `8642773653`
- digest `sha256:9b954ada86f86e5da4966951039af9dddc2eddb3d49c996d09256e4cad598338`

詳細は[Release Evidence](./V1_0_RELEASE_EVIDENCE.md)を正本とします。

## Operator baseline

v1.0.0は、v0.8.0で確立したUbuntu Server 24.04 LTS、x86_64、systemdのsingle-node operator baselineを維持します。

- [v1.0 Operator Runbook](../operations/V1_0_OPERATOR_RUNBOOK.md)
- [Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md)
- [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md)
- [Storage Migration and Upgrade Contract](../operations/STORAGE_MIGRATION_AND_UPGRADE.md)

## 文書の役割

- `V1_0_QUALIFICATION_PLAN.md`: mandatory gate、classification、pass／blocker criteria
- `V1_0_QUALIFICATION_STATUS.md`: 現在の実行状態、identity、GO／NO-GO、次の順序
- `V1_0_DOCUMENTATION_FREEZE_PLAN.md`: freeze対象、walkthrough条件、change control
- `V1_0_DOCUMENTATION_WALKTHROUGH.md`: 文書・手順ごとの静的／実行レビュー記録
- `V1_0_SOAK_PLAN.md`: 72時間soakのworkload、telemetry、停止条件
- `V1_0_RELEASE_EVIDENCE.md`: candidate、artifact、binary、publicationに結び付く証拠正本
- `CURRENT_IMPLEMENTATION_STATUS.md`: 実装済み範囲と作業再開用の全体状態
- `ROADMAP_TO_V1_0.md`: release-level sequenceとv1.0境界

## v0.9.0 release record

v0.9.0は、v1.0 stable single-node contractへ進む前のrelease-candidate hardeningとして、protocol parserのresource boundedness、signature verification temporary workspaceの安全性、public API inventory、security disposition、version整合、bounded soak evidenceを固定しました。

- [v0.9.0 Release Checklist](./RELEASE_0_9_0_CHECKLIST.md)
- [v0.9.0 Release Notes](./RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.9.0 Release Evidence](./V0_9_RELEASE_EVIDENCE.md)
- [v0.9.0 Hardening Plan](./V0_9_HARDENING_PLAN.md)

Publication record:

- PR #108 merged
- merge commit `971155340603afdc0c9c5bd37e596f49c260d15e`
- tag `v0.9.0`
- GitHub Release `v0.9.0` published

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

- validation／publisher authentication未通過objectをcanonical storageへ保存しない
- canonical storageよりindexをsemantic sourceとして優先しない
- conflict時に既存objectを上書きしない
- normal startupでimplicit storage migrationを実行しない
- unknown newer storage formatをmutateしない
- active state／data directoryへrestoreしない
- archive segmentやimmutable evidence ledgerをrewrite／deleteしない
- untrusted JSONを上限なしでrecursive parseしない
- signature verification artifactを既存pathへ上書きしない
- same-host lockをdistributed lockとして扱わない
