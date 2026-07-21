# v0.8.0 Release Checklist

**Status: release candidate** | **Target: v0.8.0** | **Last updated: 2026-07-22**

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
- [x] generation pointer inspection through the existing core resolver
- [x] index consistency inspection from `doctor` without creating a missing catalog
- [x] archive / backup inventory structural inspection
- [x] maintenance workspace structural inspection
- [x] Linux disk-capacity / disk-condition inspection
- [x] replacement / cleanup evidence semantic inspection explicitly deferred: the core verifiers exist, but automatic discovery requires a stable transaction-workspace root contract that v0.8.0 does not invent
- [x] deprecated configuration warnings explicitly deferred: v0.8.0 introduces no newly accepted deprecated key and continues to reject unknown configuration fields

## 2. Observability

- [x] process-level `health`
- [x] storage-aware `ready`
- [x] failed readiness returns a failure exit code
- [x] bounded-cardinality `metrics`
- [x] systemd journal-based diagnosis procedure
- [x] operator-visible failures expose stable diagnostic codes; cross-service trace correlation is deferred beyond the single-node v0.8.0 scope
- [x] degraded-state coverage is represented by doctor warnings and fail-closed acceptance fixtures; broader fault-matrix expansion is deferred

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
- [x] restored storage read verification in restore apply and automated drill
- [x] duplicate-safe write-path verification in the automated drill
- [x] interrupted restore / failure-injection cleanup coverage

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
- [x] upgrade from v0.7.0 under systemd documented
- [x] binary rollback and compatible-backup restore procedure documented
- [x] process-restart persistence verified with separately invoked installed binaries

## 5. Integrated operator surface

- [x] storage configuration and diagnostics commands
- [x] backup / restore commands
- [x] index lifecycle commands
- [x] migration remains explicit in `lingonberry-storage-migrate`
- [x] command / exit-code / JSON-output contract document
- [x] quarantine inspection explicitly routed to the existing admin HTTP/RBAC surface
- [x] replacement operations explicitly routed to proof-bound runbooks
- [x] cleanup operations explicitly routed to proof-bound runbooks
- [x] migration CLI responsibility and routing policy documented
- [x] human-readable output policy documented

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
- [x] complete operator CLI reference
- [x] quarantine inspection procedure connected directly to the v0.8.0 runbook
- [x] replacement / cleanup procedures connected directly to the v0.8.0 runbook
- [x] v0.7.0 to v0.8.0 upgrade procedure
- [x] rollback procedure
- [x] release notes

## 7. Automated acceptance

- [x] reference-platform assertion
- [x] Rust formatting, Clippy, and workspace tests
- [x] systemd unit verification against release-built installed binaries
- [x] configuration / health / status / doctor / metrics
- [x] publish and list
- [x] process-restart persistence across separate installed-binary invocations
- [x] backup create / verify
- [x] restore plan / apply with read verification evidence
- [x] index verify / rebuild
- [x] isolated restore drill with read, duplicate-safe write, and cleanup evidence
- [x] interrupted isolated restore cleanup unit test
- [x] partial archive restore fails closed
- [x] active data directory restore fails closed
- [x] non-empty restore target fails closed without modifying the sentinel
- [x] standard CI passes on the release branch
- [x] Ubuntu 24.04 fresh-runner operator acceptance passes on the release branch
- [x] doctor read-only regression test covers a missing index catalog
- [x] invalid generation pointer fails closed in automated tests
- [x] quarantine operation-specific acceptance remains covered by existing core tests; HTTP/RBAC end-to-end expansion is deferred beyond the storage operator release gate
- [x] fresh-machine requirement satisfied by a clean Ubuntu 24.04 runner using release-built binaries installed into `/usr/local/bin`

## 8. Release gate

The release can proceed only when:

- [x] every required item above is complete or explicitly deferred with rationale
- [x] `cargo fmt --all -- --check` passes
- [x] all Clippy checks pass with warnings denied
- [x] `cargo test --workspace` passes
- [x] JavaScript tests and external conformance suite pass
- [x] Ubuntu 24.04 fresh-runner operator acceptance passes
- [x] no temporary workflow or test-only deployment file remains
- [ ] PR is reviewed and no release-blocking issue remains
- [x] package versions and `Cargo.lock` are set to `0.8.0`
- [x] operations index, runbook, CLI contract, upgrade guide, and release notes are synchronized
- [ ] annotated tag and GitHub Release are prepared after merge

## Current evidence

At implementation commit `ec24af34af0f9079c27bc5fc4b55ba5fe9ba1f73` and cleanup commit `e91cc413e13e4a83aa2a50c53c9e7e6989711ea4`:

- all Rust packages and `Cargo.lock` are synchronized to `0.8.0`
- version synchronization passed formatting, Clippy, and workspace tests
- the temporary version workflow was removed
- standard CI and Ubuntu 24.04 fresh-runner acceptance previously passed the complete installed-binary operator scenario
- release notes document the operational boundary and explicit deferrals

The only remaining release gates are PR review, merge, annotated tag creation, and GitHub Release publication.
