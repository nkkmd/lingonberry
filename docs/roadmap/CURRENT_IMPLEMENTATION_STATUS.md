# 現在の実装状況

**Status: v0.8.0 in development** | **Latest published release: v0.7.0** | **Last updated: 2026-07-22**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## Release state

```text
released version: 0.7.0
next release target: 0.8.0
parent issue: #105 (open)
release PR: #106 (draft, open)
release branch: release/v0.8.0-operational-readiness
formal reference platform: Ubuntu Server 24.04 LTS, x86_64, systemd
latest implementation commit: ca0342a2b659c147ab6510180c3ed5e464c85373
publication state: not released
```

## v0.8.0で実装済み

### Operator diagnostics and configuration

- read-only storage doctor model
- `status`、`doctor`、strict `verify`
- `ok`／`warning`／`failed` severity
- stable machine-readable diagnostic codes
- configuration、state/data/backup/temp directory、storage format、migration journal、raw log、catalog checks
- symlink、unknown-newer format、corrupt formatのfail-closed判定
- configuration precedence: `defaults < config file < environment < CLI`
- secretを含まないeffective configuration出力

### Observability

- process-level `health`
- storage-aware `ready`
- failed readinessの非zero exit
- bounded-cardinality `metrics`

### Backup, restore, index, and DR

- `backup create`と自動isolated verification
- `backup verify`
- read-only `restore plan`
- explicit empty isolated targetへの`restore apply`
- active state/data directory、symlink、non-empty targetの拒否
- restored index consistency verification
- `index verify`／`index rebuild`
- isolated restore DR drillとmandatory cleanup

### Linux operations

- formal reference platform: Ubuntu Server 24.04 LTS、x86_64、systemd
- storage readiness gate用systemd oneshot unit
- relay用long-running systemd unit
- non-root service user、environment file、filesystem ownership contract
- Ubuntu install／start／stop／restart／diagnosis runbook
- `ubuntu-24.04`に固定したoperator acceptance workflow

## Fixed safety model

- ordinary startup never performs implicit migration or destructive repair
- `doctor` is read-only
- unknown、corrupt、contradictory state is not treated as success
- restore never overwrites the active state or data directory
- restore target must be explicit, empty, isolated, and not a symbolic link
- every created backup is verified through an isolated import before success is reported
- canonical storage remains authoritative; index remains derived and rebuildable
- protocol、storage format、proof、replacement、cleanup contracts are not weakened
- the Ubuntu reference platform does not make durable data or public contracts Ubuntu-specific

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
- [Supported Platforms](../operations/SUPPORTED_PLATFORMS.md)
- [v0.8.0 Operator Runbook](../operations/V0_8_OPERATOR_RUNBOOK.md)

## Validation state

At commit `472643e55bd86a10babeeedd4bc1036b09c6f22b`:

- standard CI run `29844895503`: success
- Ubuntu 24.04 operator acceptance run `29844895652`: success

The validated path includes:

- Rust formatting, Clippy, and workspace tests
- JavaScript tests and external conformance suite
- Ubuntu 24.04 / x86_64 / systemd assertions
- systemd unit verification against built binaries
- configuration、health、status、doctor、metrics
- publish and list
- backup create / verify
- restore plan / apply
- index verify / rebuild
- isolated restore drill

## Remaining v0.8.0 work

- extend `doctor` to generation pointer、index、archive/evidence、workspace、real disk condition
- connect deprecated configuration warnings to the v0.7.0 policy
- complete operator-visible correlation and degraded-state contracts
- add restored read/write and interrupted-restore coverage
- document v0.7.0 → v0.8.0 systemd upgrade and rollback
- integrate or explicitly route quarantine、replacement、cleanup operations
- fix command、exit-code、machine-readable output、human-readable output contracts
- add quarantine and fail-closed operational fixtures to acceptance
- perform a fresh-machine acceptance using only README and runbook
- prepare package version bump、release notes、tag、GitHub Release

## Known limitations

- v0.8.0 is not released yet.
- Automatic downgrade is not supported.
- Other systemd Linux distributions are best-effort rather than formal release-validation targets.
- Complete generation pointer、evidence/workspace、and disk-condition doctor coverage is not yet implemented.
- Integrated quarantine、replacement、and cleanup routing remains incomplete.
- Multi-node coordination and distributed locking remain outside v1.0 scope.
