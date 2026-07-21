# v0.8.0 Release Checklist

**Status: active** | **Target: v0.8.0** | **Last updated: 2026-07-22**

## Release objective

開発者ではないoperatorが、Ubuntu Server 24.04 LTSの単一ノード環境で、文書に従ってLingonberryを導入、起動、診断、バックアップ、隔離復元、索引検証、障害復旧できる状態を完成させる。

Formal reference platform:

- Ubuntu Server 24.04 LTS
- x86_64 / amd64
- systemd
- Rust stable source build

その他のsystemdベースLinuxはbest-effort supportとする。保存形式、protocol、public APIをUbuntu固有にはしない。

## 1. Operator diagnostics and configuration

- [x] `status` command
- [x] read-only `doctor` command
- [x] strict `verify` command
- [x] severity model: `ok` / `warning` / `failed`
- [x] stable machine-readable diagnostic codes
- [x] configuration validation
- [x] state / data / backup / temporary directory validation
- [x] storage format inspection
- [x] migration journal inspection
- [x] raw log and catalog inspection
- [x] symlink fail-closed behavior
- [x] unknown-newer and corrupt storage fail-closed behavior
- [x] configuration precedence: defaults < config file < environment < CLI
- [x] effective configuration output without secrets
- [ ] generation pointer inspection
- [ ] index consistency inspection from `doctor`
- [ ] archive / backup evidence inspection
- [ ] replacement / cleanup evidence inspection
- [ ] maintenance workspace inspection
- [ ] real disk-capacity / disk-condition inspection
- [ ] deprecated configuration warnings connected to the v0.7.0 policy

## 2. Observability

- [x] process-level `health`
- [x] storage-aware `ready`
- [x] failed readiness returns a failure exit code
- [x] bounded-cardinality `metrics`
- [x] systemd journal-based diagnosis procedure
- [ ] correlation information contract for operator-visible failures
- [ ] explicit degraded-state test coverage beyond current doctor warnings

## 3. Backup, restore, index, and disaster recovery

- [x] `backup create`
- [x] automatic isolated verification of every created backup
- [x] `backup verify`
- [x] non-mutating `restore plan`
- [x] isolated `restore apply`
- [x] refusal to restore over active state or data directories
- [x] refusal of symlink backup and restore paths
- [x] refusal of non-empty restore targets
- [x] restored index consistency verification
- [x] `index verify`
- [x] `index rebuild`
- [x] automated isolated restore drill
- [x] mandatory cleanup of the drill target
- [ ] restored storage read / write verification in the automated drill
- [ ] interrupted restore / failure-injection coverage

## 4. Linux deployment

- [x] formal reference platform fixed to Ubuntu Server 24.04 LTS x86_64
- [x] hardened storage readiness systemd unit
- [x] hardened long-running relay systemd unit
- [x] environment-file examples
- [x] non-root service user
- [x] filesystem ownership and directory layout
- [x] install / start / status / restart / stop procedure
- [x] `systemd-analyze verify`
- [x] operator acceptance pinned to `ubuntu-24.04`
- [x] CI assertion for Ubuntu 24.04, x86_64, and systemd
- [ ] upgrade from v0.7.0 under systemd
- [ ] binary rollback and compatible-backup restore procedure
- [ ] production-like reboot / restart persistence scenario

## 5. Integrated operator surface

- [x] storage configuration and diagnostics commands
- [x] backup / restore commands
- [x] index lifecycle commands
- [x] migration remains explicit in `lingonberry-storage-migrate`
- [ ] command / exit-code / JSON-output contract document
- [ ] quarantine inspection integrated or explicitly routed from the operator entrypoint
- [ ] replacement operations integrated or explicitly routed from the operator entrypoint
- [ ] cleanup operations integrated or explicitly routed from the operator entrypoint
- [ ] migration CLI responsibility and long-term routing policy documented
- [ ] human-readable output policy

## 6. Documentation

- [x] supported-platform contract
- [x] Ubuntu installation procedure
- [x] configuration and startup procedure
- [x] status / doctor / metrics diagnosis procedure
- [x] backup / verify procedure
- [x] isolated restore procedure
- [x] index verification / rebuild procedure
- [x] disaster-recovery drill procedure
- [x] failure diagnosis procedure
- [ ] complete operator CLI reference
- [ ] quarantine inspection procedure connected to the v0.8.0 runbook
- [ ] replacement / cleanup procedure connected to the v0.8.0 runbook
- [ ] v0.7.0 to v0.8.0 upgrade procedure
- [ ] rollback procedure
- [ ] release notes

## 7. Automated acceptance

- [x] reference-platform assertion
- [x] Rust formatting, Clippy, and workspace tests
- [x] systemd unit verification against built binaries
- [x] configuration / health / status / doctor / metrics
- [x] publish and list
- [x] backup create / verify
- [x] restore plan / apply
- [x] index verify / rebuild
- [x] isolated restore drill
- [x] standard CI passes on the release branch
- [x] Ubuntu 24.04 operator acceptance passes on the release branch
- [ ] quarantine inspection included in the acceptance scenario
- [ ] fail-closed fixtures for corrupt / contradictory / partial operational state
- [ ] fresh-machine run performed only from README and runbook

## 8. Release gate

The release can proceed only when:

- [ ] every required item above is complete or explicitly deferred with rationale
- [ ] `cargo fmt --all -- --check` passes
- [ ] all Clippy checks pass with warnings denied
- [ ] `cargo test --workspace` passes
- [ ] JavaScript tests and external conformance suite pass
- [ ] Ubuntu 24.04 operator acceptance passes
- [ ] no temporary workflow or test-only deployment file remains
- [ ] PR is reviewed and no release-blocking issue remains
- [ ] package versions and `Cargo.lock` are set to `0.8.0`
- [ ] current implementation status and operations index are synchronized
- [ ] annotated tag and GitHub Release are prepared

## Current evidence

At commit `472643e55bd86a10babeeedd4bc1036b09c6f22b`:

- standard CI run `29844895503`: success
- Ubuntu 24.04 operator acceptance run `29844895652`: success

This evidence proves the currently implemented operator path. It does not mark the remaining unchecked release requirements complete.
