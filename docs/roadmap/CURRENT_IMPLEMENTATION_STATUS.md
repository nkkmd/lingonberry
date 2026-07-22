# 現在の実装状況

**Status: v0.9.0 released** | **Latest published release: v0.9.0** | **Next release target: v1.0.0** | **Last updated: 2026-07-22**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## Release state

```text
latest published release: 0.9.0
next release target: 1.0.0
v0.9.0 parent issue: #107 (closed, completed)
v0.9.0 release PR: #108 (merged)
v0.9.0 merge commit: 971155340603afdc0c9c5bd37e596f49c260d15e
v0.9.0 tag: v0.9.0
GitHub Release: published
formal reference platform: Ubuntu Server 24.04 LTS, x86_64, systemd
publication state: released
```

## v0.9.0で完成した範囲

### Protocol parser hardening

- JSON input size limit: 1 MiB
- array／object共通nesting depth limit: 128
- oversized inputをrecursive parse前にfail closedで拒否
- depth 128を受理し、depth 129以上をpanicせず拒否
- mixed object／array nestingへ同じ上限を適用
- canonical ordering、round trip、deterministic parse契約を維持

### Signature verification workspace hardening

- PID、timestamp、atomic counterを組み合わせたworkspace候補名
- exclusive directory creation
- Unix reference platformでowner-only `0o700` permission
- artifact fileの`create_new(true)`による既存path上書き拒否
- normal success／failure return pathでRAII cleanup
- payload、signature、temporary pathを含まないgeneric error
- cleanup、permission、collision、concurrent isolationのunit test

### Public contract freeze evidence

- Rust public API inventory
- public API freeze candidate
- security reviewとfinding ledger
- Critical finding: 0
- High finding: 0
- release-blocking Medium finding: 0

### Version and publication

- 全Rust workspace packageと`Cargo.lock`を`0.9.0`へ更新
- `CHANGELOG.md`へ0.9.0 entryを追加
- release checklist、release notes、release evidenceを確定
- PR #108をmainへmerge
- tag `v0.9.0`を作成
- GitHub Release `v0.9.0`を公開
- issue #107をcompletedとしてclose

## Preserved v0.8.0 operational baseline

- Ubuntu Server 24.04 LTS、x86_64、systemd
- release-built binaries under `/usr/local/bin`
- hardened systemd units
- read-only `doctor`、strict `verify`、health／ready／status／metrics
- verified backup and isolated restore
- deterministic index verify／rebuild
- isolated DR drill
- explicit migration and compatible rollback

## Fixed safety model

- validation未通過objectをcanonical storageへ保存しない
- conflict時に既存canonical objectを上書きしない
- ordinary startupでimplicit migrationやdestructive repairを実行しない
- unknown、corrupt、contradictory stateを成功扱いしない
- restoreはactive state／data directoryを上書きしない
- canonical storageを正本とし、indexは検証・再構築可能な派生状態とする
- untrusted JSONをsize／depth上限なしでrecursive parseしない
- signature verification artifactを既存pathへ上書きしない
- normal return pathでsignature verification workspaceを残留させない
- same-host lockをdistributed lockとして扱わない

## Validation and publication record

- post-hardening CI run 1141: success
- release-preparation workflow `29898586767`: success
- final standard CI run 1152: success
- Operator acceptance run 74: success
- bounded hardening soak: parser limits、signature workspace、replacement crash matrixを5反復し成功
- PR #108 merged
- merge commit `971155340603afdc0c9c5bd37e596f49c260d15e`
- tag `v0.9.0`
- GitHub Release `v0.9.0` published

## Canonical documents

- [v0.9.0 Release Checklist](./RELEASE_0_9_0_CHECKLIST.md)
- [v0.9.0 Release Notes](./RELEASE_0_9_0_RELEASE_NOTE.md)
- [v0.9.0 Release Evidence](./V0_9_RELEASE_EVIDENCE.md)
- [v0.9.0 Hardening Plan](./V0_9_HARDENING_PLAN.md)
- [v0.9.0 Security Review](../security/V0_9_SECURITY_REVIEW.md)
- [v0.9.0 Security Findings](../security/V0_9_SECURITY_FINDINGS.md)
- [v0.9.0 Public API Freeze Candidate](../architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md)
- [v0.9.0 Rust API Inventory](../architecture/V0_9_RUST_API_INVENTORY.md)
- [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md)
- [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md)
- [Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md)

## Next step

`docs/roadmap/ROADMAP_TO_V1_0.md`に従い、v1.0.0 stable single-node release gateの最終確認へ進みます。
