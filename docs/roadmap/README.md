# ロードマップ

**Status: v0.9.0 release-ready** | **Latest published release: v0.8.0** | **Next publication target: v0.9.0** | **Last updated: 2026-07-22**

このディレクトリには、Lingonberryの実装・運用準備・releaseに関するroadmap、checklist、release note、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md)
3. [v0.9.0 Release Checklist](./RELEASE_0_9_0_CHECKLIST.md)
4. [v0.9.0 Release Notes](./RELEASE_0_9_0_RELEASE_NOTE.md)
5. [v0.9.0 Release Evidence](./V0_9_RELEASE_EVIDENCE.md)
6. [v0.9.0 Hardening Plan](./V0_9_HARDENING_PLAN.md)
7. [v0.9.0 Security Review](../security/V0_9_SECURITY_REVIEW.md)
8. [v0.9.0 Security Findings](../security/V0_9_SECURITY_FINDINGS.md)
9. [v0.9.0 Public API Freeze Candidate](../architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md)
10. [v0.9.0 Rust API Inventory](../architecture/V0_9_RUST_API_INVENTORY.md)
11. [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md)
12. [Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md)
13. [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md)
14. [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md)
15. [運用文書索引](../operations/README.md)
16. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)

`CURRENT_IMPLEMENTATION_STATUS.md`は中断後に作業を再開するための引き継ぎ用正本です。`ROADMAP_TO_V1_0.md`はrelease-level roadmapです。v0.9.0のhardening、検証、公開準備はrelease checklist、release notes、release evidence、security review、API freeze candidateを正本とします。

## v0.9.0の到達点

v0.9.0では、v1.0 stable single-node contractへ進む前のrelease-candidate hardeningとして、protocol parserのresource boundedness、signature verification temporary workspaceの安全性、public API inventory、security disposition、version整合、bounded soak evidenceを固定しました。

```text
public surface inventory
→ protocol / API freeze candidate
→ security review
→ bounded JSON parser
→ exclusive signature workspace
→ security regression tests
→ standard CI / conformance
→ bounded repeated soak
→ v0.9.0 release-ready
```

主な到達点:

- JSON input 1 MiB limit
- JSON nesting depth 128 limit
- oversized／deeply nested inputのpanic-free fail-closed rejection
- signature workspaceのexclusive creationとowner-only permission
- artifact fileのcreate-new semantics
- normal success／failure pathのRAII cleanup
- workspace cleanup、permission、collision、concurrency regression tests
- Rust public API inventoryとfreeze candidate
- security finding ledgerの全release blocker closure
- workspace package version `0.9.0`
- Rust、JavaScript、external conformanceのgreen evidence
- parser、signature workspace、replacement crash matrixの5反復bounded soak

v0.9.0は新しいstorage format、implicit migration、destructive in-place restore、multi-node coordinationを導入しません。v0.8.0で確立したUbuntu Server 24.04 LTS、x86_64、systemdのsingle-node operator baselineを維持します。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 現在のrelease state、実装済み範囲、検証結果、publication残作業
- [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md): release sequence、v1.0の境界、release gate
- [v0.9.0 Release Checklist](./RELEASE_0_9_0_CHECKLIST.md): v0.9.0 release gateとpublication record
- [v0.9.0 Release Notes](./RELEASE_0_9_0_RELEASE_NOTE.md): v0.9.0の公開範囲、互換性、安全性、既知制約
- [v0.9.0 Release Evidence](./V0_9_RELEASE_EVIDENCE.md): CI、security regression、soak、残存リスクの正本
- [v0.9.0 Hardening Plan](./V0_9_HARDENING_PLAN.md): hardening workstreamとcompletion rule
- [v0.9.0 Security Review](../security/V0_9_SECURITY_REVIEW.md): trust boundaryとreview matrix
- [v0.9.0 Security Findings](../security/V0_9_SECURITY_FINDINGS.md): finding、severity、remediation、release disposition
- [v0.9.0 Public API Freeze Candidate](../architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md): v1.0候補public contract
- [v0.9.0 Rust API Inventory](../architecture/V0_9_RUST_API_INVENTORY.md): Rust public surface inventory
- [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md): Ubuntu reference platformの導入・起動・診断・backup・restore・DR手順
- [Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md): command、JSON output、exit code、routing policy
- [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md): compatible upgrade／rollback baseline
- [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md): reference platformとbest-effort support境界

## Release history

- v0.9.0: release-candidate hardening、bounded parser、secure signature workspace、public-contract freeze evidence、bounded soak。publication pending。
- v0.8.0: single-node operational readiness、Ubuntu 24.04 reference platform、operator diagnostics、verified recovery、systemd deployment、fresh-runner acceptance。
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
- active state／data directoryへrestoreしない
- non-emptyまたはsymlink restore targetをacceptしない
- manifest、journal、pointer、proof、inventory、evidenceを手動修復しない
- archive segmentやimmutable evidence ledgerをrewrite／deleteしない
- untrusted JSONを上限なしでrecursive parseしない
- signature verification artifactを既存pathへ上書きしない
- same-host lockをdistributed lockとして扱わない
