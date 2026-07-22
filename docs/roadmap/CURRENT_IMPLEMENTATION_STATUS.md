# 現在の実装状況

**Status: v0.9.0 release-ready** | **Latest published release: v0.8.0** | **Next publication target: v0.9.0** | **Last updated: 2026-07-22**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## Release state

```text
latest published release: 0.8.0
release candidate version: 0.9.0
v0.9.0 parent issue: #107
v0.9.0 release PR: #108 (open, draft at last synchronization)
v0.9.0 release branch: release/v0.9.0-release-candidate-hardening
v0.9.0 tag: pending
GitHub Release: pending
formal reference platform: Ubuntu Server 24.04 LTS, x86_64, systemd
publication state: release-ready, merge and publication pending
```

## v0.9.0で完成した範囲

### Protocol parser hardening

- JSON input size limit: 1 MiB
- array／object共通nesting depth limit: 128
- oversized inputをrecursive parse前にfail closedで拒否
- depth 128を受理し、depth 129以上をpanicせず拒否
- mixed object／array nestingへ同じ上限を適用
- trailing content、truncated structure、invalid numberの拒否契約を維持
- canonical object-key sortingとround-trip idempotenceを維持
- repeated parseのdeterministic behaviorを回帰テストで固定

### Signature verification workspace hardening

- PID、timestamp、atomic counterを組み合わせたworkspace候補名
- exclusive directory creation
- Unix reference platformでowner-only `0o700` permission
- artifact fileの`create_new(true)`による既存path上書き拒否
- success／verification failure／command failure／write failureの通常return pathでRAII cleanup
- payload、signature、temporary pathを含まないgeneric error
- workspace cleanup、permission、collision、concurrent isolationのunit test

### Public contract freeze evidence

- Rust public API inventory
- public API freeze candidate
- security reviewとfinding ledger
- parser／signature hardeningのrelease evidence
- Critical finding: 0
- High finding: 0
- release-blocking Medium finding: 0

### Version and release preparation

- 全Rust workspace packageを`0.9.0`へ更新
- internal path dependency versionを`0.9.0`へ更新
- `Cargo.lock`を`0.9.0`へ更新
- `CHANGELOG.md`へ0.9.0 entryを追加
- v0.9.0 release notesを追加
- release checklistとrelease evidenceをrelease-readyへ更新

## Preserved v0.8.0 operational baseline

### Operator diagnostics and configuration

- `config`、`health`、`ready`、`status`、read-only `doctor`、strict `verify`、bounded-cardinality `metrics`
- stable machine-readable diagnostic codes
- canonical JSON output and documented exit-code contract
- configuration precedence: `defaults < config file < environment < CLI`
- storage format、migration journal、raw log、catalog、generation pointer、index、backup inventory、workspace、disk capacityのread-only inspection
- symlink、unknown-newer、corrupt、contradictory stateのfail-closed判定

### Backup, restore, index, and disaster recovery

- verified `backup create` / `backup verify`
- non-mutating `restore plan`
- explicit empty isolated targetへの`restore apply`
- active state／data directory、symlink、non-empty target、partial archiveの拒否
- restored-record read verification
- deterministic `index verify` / `index rebuild`
- isolated restore DR drill
- duplicate-safe write-path verification
- interrupted restore failure injection and partial-state cleanup

### Linux operations

- formal reference platform: Ubuntu Server 24.04 LTS、x86_64、systemd
- hardened systemd units
- non-root service user、environment file、filesystem ownership contract
- release-built binaries installed under `/usr/local/bin`
- clean fresh-runner operator acceptance
- process-restart persistence verification
- v0.7.0からv0.8.0へのupgrade and compatible rollback procedure

## Fixed safety model

- validation未通過objectをcanonical storageへ保存しない
- conflict時に既存canonical objectを上書きしない
- original Knowledge ObjectをTransition Objectでrewrite／deleteしない
- incomplete evidenceでlast-known-good semantic checkpointを上書きしない
- ordinary startupでimplicit migrationやdestructive repairを実行しない
- unknown、corrupt、contradictory stateを成功扱いしない
- restoreはactive state／data directoryを上書きしない
- restore targetはexplicit、empty、isolated、non-symlinkを要求する
- canonical storageを正本とし、indexは検証・再構築可能な派生状態とする
- protocol、storage format、proof、replacement、cleanup contractを弱めない
- untrusted JSONをsize／depth上限なしでrecursive parseしない
- signature verification artifactを既存pathへ上書きしない
- normal return pathでsignature verification workspaceを残留させない
- same-host lockをdistributed lockとして扱わない

## Validation record

v0.9.0 release branchで次を確認済みです。

- standard CI run 1141: Rust formatting、library／binary／test-target Clippy、workspace tests、JavaScript tests、external conformanceが成功
- parser hardening workflow: formatting、Clippy、workspace testsが成功
- signature workspace security regression workflow: formatting、Clippy、workspace testsが成功
- v0.9.0 release preparation workflow: package metadata、標準Rust gate、JavaScript tests、external conformanceが成功
- bounded hardening soak: parser limits、signature workspace tests、quarantine replacement crash matrixを5反復し成功

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

## Publication work remaining

1. final documentation synchronization後のstandard CIを確認する
2. PR #108をReady for reviewへ変更する
3. PR #108を`main`へmergeする
4. tag `v0.9.0`を作成する
5. GitHub Release `v0.9.0`を公開する
6. issue #107をcloseする
7. publication recordをmerge commit、tag、release URLで更新する

## Next step after publication

v0.9.0公開後は`docs/roadmap/ROADMAP_TO_V1_0.md`に従い、v1.0.0 stable single-node release gateの最終確認へ進みます。
