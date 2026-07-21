# ロードマップ

**Status: v0.8.0 released** | **Latest published release: v0.8.0** | **Next release target: v0.9.0** | **Last updated: 2026-07-22**

このディレクトリには、Lingonberryの実装・運用準備・releaseに関するroadmap、checklist、release note、および作業再開用の現在地文書を置きます。

## 再開時に最初に読む文書

1. [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)
2. [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md)
3. [v0.8.0 Release Checklist](./RELEASE_0_8_0_CHECKLIST.md)
4. [v0.8.0 Release Notes](./RELEASE_0_8_0_RELEASE_NOTE.md)
5. [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md)
6. [v0.8.0 Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md)
7. [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md)
8. [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md)
9. [運用文書索引](../operations/README.md)
10. [実装バックログ](./IMPLEMENTATION_BACKLOG.md)

`CURRENT_IMPLEMENTATION_STATUS.md`は中断後に作業を再開するための引き継ぎ用正本です。`ROADMAP_TO_V1_0.md`はrelease-level roadmapです。v0.8.0の実装・検証・公開記録は、release checklist、release notes、operator runbook、CLI contract、upgrade／rollback guideを正本とします。

## v0.8.0の到達点

v0.8.0では、Ubuntu Server 24.04 LTS、x86_64、systemdを正式なreference platformとして、single-node operatorが導入、起動、診断、backup、isolated restore、index lifecycle、DR drill、upgrade、rollbackを文書と機械検証に従って実行できる状態を完成させました。

```text
release-built binaries
→ installed under /usr/local/bin
→ hardened systemd units
→ read-only diagnostics
→ verified backup
→ isolated restore
→ deterministic index verification / rebuild
→ read / duplicate-safe write / cleanup DR drill
→ documented upgrade / rollback
```

主な到達点:

- Ubuntu Server 24.04 LTS、x86_64、systemdの正式なreference platform契約
- release-built binaryと`/usr/local/bin`導入を使うfresh-runner acceptance
- `config`、`health`、`ready`、`status`、read-only `doctor`、strict `verify`、`metrics`
- stable diagnostic code、canonical JSON output、exit-code contract
- generation pointer、index、backup inventory、workspace、disk capacityを含むread-only診断
- verified backup create／verify
- non-mutating restore planとisolated restore apply
- restored-record read verificationとindex consistency verification
- deterministic index verify／rebuild
- read verification、duplicate-safe write verification、cleanup verificationを含むisolated DR drill
- interrupted restore failure injectionとpartial-state cleanup
- symlink、active target、non-empty target、partial archive、unknown-newer／corrupt stateのfail-closed rejection
- v0.7.0からv0.8.0へのsystemd upgradeとcompatible rollback手順
- workspace package version `0.8.0`
- tag `v0.8.0`とGitHub Release公開

v0.8.0は新しいstorage format、implicit migration、destructive in-place restore、Ubuntu固有のdata contractを導入しません。canonical storageは引き続き正本であり、indexは検証・再構築可能な派生状態です。

## 文書の役割

- [現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md): 現在のrelease state、実装済み機能、安全境界、次の作業
- [v1.0までのロードマップ](./ROADMAP_TO_V1_0.md): release sequence、v1.0の境界、release gate
- [v0.8.0 Release Checklist](./RELEASE_0_8_0_CHECKLIST.md): v0.8.0のrelease gate、検証証跡、公開記録
- [v0.8.0 Release Notes](./RELEASE_0_8_0_RELEASE_NOTE.md): v0.8.0の公開範囲、reference platform、互換性、安全性、既知制約
- [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md): Ubuntu reference platformの導入・起動・診断・backup・restore・DR手順
- [v0.8.0 Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md): command、JSON output、exit code、routing policy
- [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md): v0.7.0からのupgradeとcompatible rollback
- [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md): reference platformとbest-effort support境界

## Release history

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
- same-host lockをdistributed lockとして扱わない
