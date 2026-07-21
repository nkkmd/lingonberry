# 現在の実装状況

**Status: v0.8.0 released** | **Latest published release: v0.8.0** | **Next release target: v0.9.0** | **Last updated: 2026-07-22**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## Release state

```text
released version: 0.8.0
next release target: 0.9.0
v0.8.0 parent issue: #105 (closed, completed)
v0.8.0 release PR: #106 (merged)
v0.8.0 merge commit: 9d34ec54309254e00cb7c0d02c93a98a177496da
v0.8.0 tag: v0.8.0
formal reference platform: Ubuntu Server 24.04 LTS, x86_64, systemd
publication state: released
```

## v0.8.0で完成した範囲

### Operator diagnostics and configuration

- `config`、`health`、`ready`、`status`、read-only `doctor`、strict `verify`、bounded-cardinality `metrics`
- `ok`／`warning`／`failed` severity
- stable machine-readable diagnostic codes
- canonical JSON output and documented exit-code contract
- configuration precedence: `defaults < config file < environment < CLI`
- effective configuration output without secrets
- state／data／backup／temporary directory validation
- storage format、migration journal、raw log、catalog inspection
- generation pointer、index consistency、backup inventory、maintenance workspace、disk capacity inspection
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
- mandatory drill-target cleanup
- interrupted restore failure injection and partial-state cleanup

### Linux operations

- formal reference platform: Ubuntu Server 24.04 LTS、x86_64、systemd
- storage readiness gate用systemd unit
- relay用long-running systemd unit
- non-root service user、environment file、filesystem ownership contract
- Ubuntu install／start／stop／restart／diagnosis runbook
- release-built binaries installed under `/usr/local/bin`
- clean `ubuntu-24.04` fresh-runner operator acceptance
- process-restart persistence verification
- v0.7.0からv0.8.0へのupgrade and compatible rollback procedure

## Fixed safety model

- ordinary startup never performs implicit migration or destructive repair
- `doctor` is read-only
- unknown、corrupt、contradictory state is not treated as success
- restore never overwrites the active state or data directory
- restore target must be explicit、empty、isolated、and not a symbolic link
- every created backup is verified through an isolated import before success is reported
- canonical storage remains authoritative; index remains derived and rebuildable
- protocol、storage format、proof、replacement、cleanup contracts are not weakened
- pointer、journal、manifest、proof、inventory、completion evidence、cleanup evidenceのmanual repairは禁止
- the Ubuntu reference platform does not make durable data or public contracts Ubuntu-specific
- same-host locks are not distributed locks

## Formal operator path

```text
install on Ubuntu Server 24.04 LTS
→ configure
→ doctor / ready
→ start relay with systemd
→ publish / inspect
→ backup create / verify
→ isolated restore plan / apply
→ index verify / rebuild
→ isolated DR drill
→ journalctl / status / doctor / metrics diagnosis
```

Canonical documents:

- [v0.8.0 Release Checklist](./RELEASE_0_8_0_CHECKLIST.md)
- [v0.8.0 Release Notes](./RELEASE_0_8_0_RELEASE_NOTE.md)
- [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md)
- [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md)
- [Operator CLI Contract](../operations/OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 Upgrade and Rollback](../operations/V0_8_UPGRADE_AND_ROLLBACK.md)

## Validation and publication record

Before merge, the final release branch passed:

- standard CI run `29876111064`
- Ubuntu 24.04 fresh-runner operator acceptance run `29876111078`

Publication:

- PR #106 merged into `main`
- issue #105 closed as completed
- tag `v0.8.0` created
- GitHub Release `v0.8.0` published
- root README、CHANGELOG、release notes、release checklist、roadmap index、operations index synchronized after publication

## Explicit v0.8.0 deferrals

- Cross-service trace correlation was not introduced.
- General `doctor` does not automatically discover every historical replacement／cleanup transaction workspace because no stable workspace-root discovery contract exists.
- Quarantine inspection remains on the existing admin HTTP／RBAC surface.
- Replacement and cleanup remain explicit proof-bound operations through operation-specific runbooks and core verifiers.
- Multi-node coordination、distributed locking、Kubernetes operator、remote backup service、secure erase remain outside the v0.8.0 scope.

## Next step

v0.8.0は公開済みです。次の実装作業は`docs/roadmap/ROADMAP_TO_V1_0.md`に従い、v0.9.0のrelease scopeを確認して開始します。
