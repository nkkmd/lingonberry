# Lingonberry v0.9.0 Release Checklist

**Status: released** | **Version: v0.9.0** | **Release type: release-candidate hardening** | **Released: 2026-07-22**

## 1. Release objective

v0.9.0は新機能を追加せず、v1.0へ向けたprotocol、public API、storage、operator contractのfreeze candidateを記録し、securityとresource boundednessを強化するreleaseです。

## 2. Release scope completion

### Rust public API audit

- [x] workspace全crateのexported surfaceをinventory化した
- [x] freeze candidate、behavior-frozen、workspace-internal、implementation-detailへ分類した
- [x] public error、trait、re-export、version constantの互換性影響を確認した
- [x] v1.x semver／compatibility方針を文書化した

Evidence:

- `docs/architecture/V0_9_RUST_API_INVENTORY.md`
- `docs/architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md`

### Freeze candidates

- [x] protocol freeze candidateを記録した
- [x] public API freeze candidateを記録した
- [x] storage formatとdurable proof contractへの非変更を確認した
- [x] protocol、schema、package、storageのversion軸を照合した
- [x] breaking changeをrelease blockerとして扱う手順を固定した

Disposition:

- wire protocol／schema: `0.1.0`維持
- Rust workspace packages: `0.9.0`
- storage format: 変更なし
- archive／journal／proof formats: 変更なし

### Security review

- [x] path traversal
- [x] symlink handling
- [x] oversized input
- [x] deeply nested input
- [x] malformed serialization
- [x] signature verification workspace
- [x] authorization ordering
- [x] information leakage
- [x] TOCTOU
- [x] disk-full／I/O failure contract
- [x] unresolved Critical／High findingがゼロ
- [x] unresolved release-blocking Medium findingがゼロ

Finding summary:

- LB-SEC-009-001: fixed and regression-tested
- LB-SEC-009-002: fixed and regression-tested

### Parser／property regression

- [x] malformed parser inputs
- [x] canonical round-trip idempotence
- [x] repeated parse determinism
- [x] input-size boundary
- [x] array／object mixed nesting boundary
- [x] depth超過のpanic-free rejection

### Production-like acceptance and soak

- [x] v0.8.0 reference-platform acceptance contractを維持
- [x] publish／storage／migration／backup／restore／index／replacement／cleanup regressionがgreen
- [x] parser、signature workspace、replacement crash matrixを5反復
- [x] bounded soak pass criteriaを満たした
- [x] 長時間実機soakとの差異を残存リスクとして記録した

### Supported platform and packaging

- [x] Ubuntu Server 24.04 LTS、x86_64、systemdを正式reference platformとして維持
- [x] 全workspace packageとlockfileを0.9.0へ統一
- [x] v0.8.0からのupgradeにstorage migrationを追加しない
- [x] compatible rollback／verified backup restore contractを維持

### Documentation freeze

- [x] release checklist
- [x] hardening plan
- [x] security review and findings
- [x] public API inventory and freeze candidate
- [x] signature workspace remediation contract
- [x] release evidence
- [x] release notes
- [x] changelog
- [x] root README、roadmap index、operations index、current status

## 3. Mandatory validation

- [x] `cargo fmt --all -- --check`
- [x] library clippy with warnings denied
- [x] binary clippy
- [x] test-target clippy
- [x] `cargo test --workspace`
- [x] JavaScript test suite
- [x] external protocol conformance suite
- [x] parser baseline and limit regressions
- [x] signature workspace security regressions
- [x] replacement／cleanup crash matrix
- [x] bounded hardening soak, 5 consecutive iterations
- [x] final standard CI run 1152
- [x] Operator acceptance run 74

## 4. Release-blocker disposition

| Severity | Open count | Disposition |
|---|---:|---|
| Critical | 0 | release allowed |
| High | 0 | release allowed |
| Release-blocking Medium | 0 | all fixed and tested |
| Low／residual | documented | accepted for v0.9.0 |

## 5. Compatibility and safety declaration

v0.9.0はcanonical object model、signature payload、protocol／schema version、storage format、migration journal、backup archive、replacement／cleanup proof contract、authorization orderingを変更しません。

新しいparser limitはavailabilityを守るfail-closed boundaryです。

## 6. Completion decision

```text
public contracts inventoried
→ freeze candidates recorded
→ security findings fixed and tested
→ parser / signature regressions green
→ standard CI and conformance green
→ bounded hardening soak green
→ workspace version 0.9.0
→ release documents frozen
→ merged and published
```

## 7. Publication record

- Parent issue: #107 (closed, completed)
- Release PR: #108 (merged)
- Source hardening commit: `fe23c523f358cfa62aea396ec7481778a0915c2c`
- Security regression-test commit: `1083ab0348881aabba924f102151c5d4ed3da292`
- Version preparation commit: `e5b308e54c5ed888dd3b162c37e70fb6bfd48c42`
- Merge commit: `971155340603afdc0c9c5bd37e596f49c260d15e`
- Post-hardening standard CI: run 1141, success
- Versioning／conformance／bounded-soak workflow: `29898586767`, success
- Final standard CI: run 1152, success
- Operator acceptance: run 74, success
- Tag: `v0.9.0`
- GitHub Release: `v0.9.0`, published 2026-07-22
