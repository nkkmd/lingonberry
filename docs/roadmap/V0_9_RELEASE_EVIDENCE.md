# v0.9.0 Release Evidence

**Status: active** | **Target: v0.9.0 release candidate** | **Last updated: 2026-07-22**

この文書は、v0.9.0 hardeningで得られた検証証跡と未完了blockerを一か所に集約する正本です。

## Established evidence

### CI baseline

- CI run 1112: Rust formatting、clippy、workspace tests、JavaScript tests、external conformance suiteが成功。
- CI run 1114: security remediation仕様とRust API inventory追加後も全job成功。
- CI run 1115: parser baseline regression test追加後も全job成功。
- CI run 1121: parser canonical round-trip、determinism、depth 64 regression追加後も全job成功。

### Contract and review artifacts

- `docs/roadmap/RELEASE_0_9_0_CHECKLIST.md`
- `docs/roadmap/V0_9_HARDENING_PLAN.md`
- `docs/security/V0_9_SECURITY_REVIEW.md`
- `docs/security/V0_9_SECURITY_FINDINGS.md`
- `docs/security/V0_9_SIGNATURE_WORKSPACE_REMEDIATION.md`
- `docs/architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md`
- `docs/architecture/V0_9_RUST_API_INVENTORY.md`

### Parser regression coverage

`packages/protocol/tests/parser_baseline.rs`は次の既存契約を固定する。

- trailing content rejection
- truncated structure rejection
- invalid number rejection
- canonical object-key sorting
- canonical round-trip idempotence
- repeated parse determinism
- depth 64 compatibility

Malformed-input panic freedomは、LB-SEC-009-002のdepth limit実装時に境界テストとして追加する。

## Open release blockers

### LB-SEC-009-001

Signature verification temporary workspaceのexclusive creation、owner-only permission、create-new artifacts、全return path cleanup、regression coverageが未実装。

### LB-SEC-009-002

Protocol JSON parserの1 MiB input limitとdepth 128 limit、およびboundary regression coverageが未実装。

## Required implementation evidence

各blockerをclosedにする前に、最低限次を記録する。

1. source commit SHA
2. affected public behavior
3. regression test path
4. formatting／clippy／workspace test result
5. external conformance result
6. security finding state transition
7. residual riskまたはoperational note

## Implementation application constraint

現在利用可能なGitHub connectorは、既存UTF-8 fileの部分patchを提供せず、更新時にcomplete replacement contentを要求する。`packages/protocol/src/lib.rs`は約1,800行の単一fileであり、truncated connector responseから再構築した内容で全置換するとsilent truncationまたは欠落の危険がある。

このため、次のいずれかの安全な経路が利用可能になるまで、同fileへの書き込みは行わない。

1. authenticated local checkoutで通常のpatch、rustfmt、clippy、testsを実行する。
2. connectorが完全なblob contentを再利用可能なfile referenceとして返す。
3. connectorが既存fileへのvalidated partial patchを提供する。

この制約は実装上のrelease blockerを解除するものではない。安全でない全置換を避けるための変更管理上の制約である。

## RC completion rule

次をすべて満たすまでv0.9.0 RCを完了扱いにしない。

- Critical／High findingがゼロ
- release-blocking Medium findingが修正済みまたは証拠付きで明示的にdisposition済み
- parser／signature hardening regression testsがgreen
- storage、migration、backup／restore、replacement／cleanupの既存CIがgreen
- public API freeze candidateに未分類exportが残っていない
- production-like soak evidenceが記録されている
