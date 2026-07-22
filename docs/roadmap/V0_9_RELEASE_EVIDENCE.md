# v0.9.0 Release Evidence

**Status: release-ready** | **Target: v0.9.0** | **Last updated: 2026-07-22**

この文書は、v0.9.0 hardeningで得られた検証証跡、release disposition、残存リスクを一か所に集約する正本です。

## Release commits

- Protocol hardening implementation: `fe23c523f358cfa62aea396ec7481778a0915c2c`
- Signature workspace regression tests: `1083ab0348881aabba924f102151c5d4ed3da292`
- Security finding closure: `50d6165d5012d65b56a723972531283046bd6620`
- Workspace version 0.9.0 and bounded soak: `e5b308e54c5ed888dd3b162c37e70fb6bfd48c42`

## Standard CI evidence

### Established baseline

- CI run 1112: Rust formatting、clippy、workspace tests、JavaScript tests、external conformance suiteが成功。
- CI run 1114: security remediation仕様とRust API inventory追加後も全job成功。
- CI run 1115: parser baseline regression追加後も全job成功。
- CI run 1121: parser canonical round-trip、determinism、depth 64 regression追加後も全job成功。

### Post-hardening validation

- CI run 1141: parser／signature hardeningとsecurity finding closure後、Rust checks、JavaScript tests、external conformance suiteが成功。
- Release-preparation workflow run 1 (`29898586767`): 全workspace packageの0.9.0 metadata、標準Rust gate、JavaScript tests、external conformance、bounded soakが成功。

最終release documentation commit後に実行される標準CIをpublication recordへ追記する。

## Security disposition

`docs/security/V0_9_SECURITY_FINDINGS.md`を正本とする。

- Open Critical findings: **0**
- Open High findings: **0**
- Open release-blocking Medium findings: **0**
- Closed findings: **2**

### LB-SEC-009-001

Signature verification workspaceを次のcontractへ変更した。

- exclusive directory creation
- Unix owner-only permission (`0o700`)
- create-new artifact files
- PID／timestamp／atomic counterによるconcurrent isolation
- RAII cleanup on normal success and error returns
- non-sensitive generic errors

Regression coverage:

- workspace cleanup
- owner-only permission
- existing artifact collision rejection
- concurrent workspace isolation and cleanup

### LB-SEC-009-002

Protocol JSON parserへ次のboundedness contractを導入した。

- maximum input: **1 MiB**
- maximum nesting depth: **128**
- oversized inputのearly fail-closed rejection
- depth超過のpanic-free `JsonError`
- object／array共通depth accounting

Regression coverage:

- 1 MiB超過拒否
- 1 MiB boundary受理
- depth 128受理
- depth 129拒否
- mixed nesting拒否

## Parser regression coverage

- `packages/protocol/tests/parser_baseline.rs`
  - trailing content rejection
  - truncated structure rejection
  - invalid number rejection
  - canonical object-key sorting
  - canonical round-trip idempotence
  - repeated parse determinism
  - depth 64 compatibility
- `packages/protocol/tests/parser_limits.rs`
  - input-size and nesting boundaries
- protocol unit tests
  - signature workspace security contracts

## Contract and review artifacts

- `docs/roadmap/RELEASE_0_9_0_CHECKLIST.md`
- `docs/roadmap/V0_9_HARDENING_PLAN.md`
- `docs/security/V0_9_SECURITY_REVIEW.md`
- `docs/security/V0_9_SECURITY_FINDINGS.md`
- `docs/security/V0_9_SIGNATURE_WORKSPACE_REMEDIATION.md`
- `docs/architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md`
- `docs/architecture/V0_9_RUST_API_INVENTORY.md`

## Bounded hardening soak

Release-preparation workflowは次の組合せを5回連続実行し、すべて成功した。

1. protocol parser baseline and boundary tests
2. signature workspace unit tests
3. quarantine replacement crash-point JavaScript matrix

Pass criteria:

- iteration failure: 0
- panic／abort: 0
- parser boundary regression: 0
- signature workspace regression: 0
- replacement crash-matrix regression: 0

これはCI上のbounded soakであり、長時間のproduction-like運転を置き換えるものではない。長時間soak、実機resource telemetry、disk-pressure／power-loss injectionはv1.0 stable release gateとして継続する。

## Compatibility disposition

- Protocol and schema version remain `0.1.0`; v0.9.0はwire-format breaking changeを導入しない。
- Storage format、migration journal、backup archive、replacement／cleanup proof contractは変更しない。
- 全Rust workspace packageと`Cargo.lock`は0.9.0へ統一した。
- New parser limits are intentional fail-closed resource bounds and are documented public constants.
- Unknown、corrupt、contradictory、unsupported durable stateは引き続きfail closedで扱う。

## Residual risks

- process crash、SIGKILL、host power lossではRAII cleanupが動かず、signature workspaceが残る可能性がある。OS temporary lifecycleまたは将来のstale-workspace cleanup policyで扱う。
- bounded CI soakは長時間resource telemetryを提供しない。
- multi-node coordination、distributed locking、replicationはv0.9.0のsingle-node release scope外。

## Release-ready decision

次を満たしたため、v0.9.0 release PRはmerge／tag／GitHub Release作成へ進められる。

- public contracts inventoried and classified
- protocol／API freeze candidates recorded
- zero open Critical／High／release-blocking Medium findings
- parser and signature security regressions green
- standard Rust、JavaScript、external conformance green
- migration、backup／restore、replacement／cleanupの既存workspace tests green
- bounded hardening soak green
- workspace packages versioned as 0.9.0
- release notes and changelog prepared
