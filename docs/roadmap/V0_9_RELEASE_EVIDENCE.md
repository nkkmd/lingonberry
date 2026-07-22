# v0.9.0 Release Evidence

**Status: released** | **Version: v0.9.0** | **Released: 2026-07-22**

この文書は、v0.9.0 hardeningで得られた検証証跡、release disposition、publication record、残存リスクを集約する正本です。

## Release commits

- Protocol hardening implementation: `fe23c523f358cfa62aea396ec7481778a0915c2c`
- Signature workspace regression tests: `1083ab0348881aabba924f102151c5d4ed3da292`
- Security finding closure: `50d6165d5012d65b56a723972531283046bd6620`
- Workspace version 0.9.0 and bounded soak: `e5b308e54c5ed888dd3b162c37e70fb6bfd48c42`
- Release merge commit: `971155340603afdc0c9c5bd37e596f49c260d15e`

## CI and acceptance evidence

- CI run 1112: baseline Rust、JavaScript、external conformance success
- CI run 1114: remediation specification and API inventory success
- CI run 1115: parser baseline regression success
- CI run 1121: canonical round-trip、determinism、depth compatibility success
- CI run 1141: post-hardening standard CI success
- Release-preparation workflow `29898586767`: package metadata、Rust gates、JavaScript、external conformance、bounded soak success
- Final standard CI run 1152: success
- Operator acceptance run 74: success

## Security disposition

- Open Critical findings: **0**
- Open High findings: **0**
- Open release-blocking Medium findings: **0**
- Closed findings: **2**

### LB-SEC-009-001

Signature verification workspace contract:

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

Protocol JSON parser boundedness contract:

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

## Regression paths

- `packages/protocol/tests/parser_baseline.rs`
- `packages/protocol/tests/parser_limits.rs`
- protocol unit tests for signature workspace security contracts
- quarantine replacement crash-point JavaScript matrix

## Bounded hardening soak

次の組合せを5回連続実行し、すべて成功した。

1. protocol parser baseline and boundary tests
2. signature workspace unit tests
3. quarantine replacement crash-point matrix

Pass criteria:

- iteration failure: 0
- panic／abort: 0
- parser boundary regression: 0
- signature workspace regression: 0
- replacement crash-matrix regression: 0

## Compatibility disposition

- Protocol and schema version remain `0.1.0`; wire-format breaking changeなし
- Storage format、migration journal、backup archive、replacement／cleanup proof contractは変更なし
- 全Rust workspace packageと`Cargo.lock`は0.9.0
- parser limitsはdocumented fail-closed resource bounds
- unknown、corrupt、contradictory、unsupported durable stateはfail closed

## Publication record

- Parent issue: #107 (closed, completed)
- Release PR: #108 (merged)
- Merge commit: `971155340603afdc0c9c5bd37e596f49c260d15e`
- Tag: `v0.9.0`
- GitHub Release: `v0.9.0`, published 2026-07-22
- Root README、current status、roadmap index、operations index、release checklist、release notes、release evidenceを公開後同期

## Residual risks

- process crash、SIGKILL、host power lossではRAII cleanupが動かず、signature workspaceが残る可能性がある
- bounded CI soakは長時間resource telemetryを提供しない
- disk-pressure／power-loss injectionはv1.0 stable gateへ継続
- multi-node coordination、distributed locking、replicationはv0.9.0 scope外

## Final decision

v0.9.0はrelease gateを満たし、2026-07-22に公開されました。次のrelease targetはv1.0.0 stable single-node releaseです。
