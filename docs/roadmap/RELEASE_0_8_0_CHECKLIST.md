# v0.8.0 Release Checklist

**Status: released** | **Version: v0.8.0** | **Released: 2026-07-22**

## Release objective

開発者ではないoperatorが、Ubuntu Server 24.04 LTSの単一ノード環境で、文書に従ってLingonberryを導入、起動、診断、バックアップ、隔離復元、索引検証、障害復旧できる状態を完成させる。

Formal reference platform:

- Ubuntu Server 24.04 LTS
- x86_64 / amd64
- systemd
- Rust stable source build

その他のsystemdベースLinuxはbest-effort supportとする。保存形式、protocol、public APIをUbuntu固有にはしない。

## 1. Operator diagnostics and configuration

- [x] `config`、`health`、`ready`、`status`、read-only `doctor`、strict `verify`、bounded-cardinality `metrics`
- [x] `ok` / `warning` / `failed` severity model
- [x] stable machine-readable diagnostic codes
- [x] configuration precedence: defaults < config file < environment < CLI
- [x] effective configuration output without secrets
- [x] state / data / backup / temporary directory validation
- [x] storage format、migration journal、raw log、catalog inspection
- [x] generation pointer inspection through the existing core resolver
- [x] index consistency inspection without creating a missing catalog
- [x] archive / backup inventory structural inspection
- [x] maintenance workspace structural inspection
- [x] Linux disk-capacity inspection
- [x] symlink、unknown-newer、corrupt、contradictory stateのfail-closed behavior
- [x] replacement / cleanup evidence semantic inspection explicitly deferred because automatic discovery requires a stable transaction-workspace root contract
- [x] deprecated configuration warnings explicitly deferred because v0.8.0 introduces no newly accepted deprecated key and rejects unknown fields

## 2. Backup, restore, index, and disaster recovery

- [x] verified `backup create`
- [x] `backup verify`
- [x] non-mutating `restore plan`
- [x] isolated `restore apply`
- [x] refusal of active、non-empty、symlink restore targets
- [x] restored-record read verification
- [x] restored index consistency verification
- [x] `index verify` / `index rebuild`
- [x] isolated restore drill
- [x] duplicate-safe write-path verification
- [x] mandatory drill-target cleanup
- [x] interrupted restore / failure-injection cleanup coverage

## 3. Linux deployment

- [x] Ubuntu Server 24.04 LTS x86_64 formal reference platform
- [x] hardened storage-readiness systemd unit
- [x] hardened long-running relay systemd unit
- [x] environment-file examples
- [x] non-root service user
- [x] filesystem ownership and directory layout
- [x] install / start / status / restart / stop procedure
- [x] `systemd-analyze verify`
- [x] v0.7.0 to v0.8.0 upgrade procedure
- [x] binary rollback and compatible-backup restore procedure
- [x] process-restart persistence using separately invoked installed binaries

## 4. Integrated operator surface and documentation

- [x] command / exit-code / canonical JSON-output contract
- [x] migration responsibility retained by `lingonberry-storage-migrate`
- [x] quarantine inspection routed to existing admin HTTP/RBAC surface
- [x] replacement and cleanup routed to proof-bound runbooks and core verifiers
- [x] supported-platform contract
- [x] Ubuntu installation and configuration procedure
- [x] status / doctor / metrics diagnosis procedure
- [x] backup / restore / index / DR procedures
- [x] quarantine / replacement / cleanup routing procedures
- [x] failure diagnosis procedure
- [x] v0.7.0 to v0.8.0 upgrade and rollback documentation
- [x] v0.8.0 release notes

## 5. Automated acceptance

- [x] reference-platform assertion
- [x] Rust formatting、Clippy、workspace tests
- [x] JavaScript tests and external conformance suite
- [x] release-built binary installation into `/usr/local/bin`
- [x] systemd unit verification
- [x] installed-binary operator acceptance
- [x] configuration / health / ready / status / doctor / metrics
- [x] publish / list and process-restart persistence
- [x] backup create / verify
- [x] restore plan / apply with read verification
- [x] index verify / rebuild
- [x] isolated DR drill with read、duplicate-safe write、cleanup evidence
- [x] partial archive、active target、non-empty target fail-closed fixtures
- [x] invalid generation pointer and missing-index doctor regression coverage
- [x] clean Ubuntu 24.04 fresh-runner scenario

## 6. Release gate and publication

- [x] every required item completed or explicitly deferred with rationale
- [x] `cargo fmt --all -- --check`
- [x] all Clippy checks with warnings denied
- [x] `cargo test --workspace`
- [x] JavaScript tests and external conformance suite
- [x] Ubuntu 24.04 fresh-runner operator acceptance
- [x] no temporary workflow or test-only deployment file remains
- [x] package versions and `Cargo.lock` set to `0.8.0`
- [x] operations index、runbook、CLI contract、upgrade guide、release notes synchronized
- [x] PR #106 reviewed and merged into `main`
- [x] issue #105 closed as completed
- [x] tag `v0.8.0` created
- [x] GitHub Release `v0.8.0` published
- [x] root README、CHANGELOG、release notes、roadmap and operations indexes synchronized after publication

## Publication record

```text
release PR: #106
merge commit: 9d34ec54309254e00cb7c0d02c93a98a177496da
tag: v0.8.0
release date: 2026-07-22
publication state: released
```

Release validation before merge:

- standard CI run `29876111064`: success
- Ubuntu 24.04 fresh-runner operator acceptance run `29876111078`: success

The v0.8.0 release is complete. Future work proceeds from the published v0.8.0 baseline toward v0.9.0.
