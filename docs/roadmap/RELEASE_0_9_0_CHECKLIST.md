# Lingonberry v0.9.0 Release Checklist

**Status: release-ready** | **Target: v0.9.0** | **Release type: release-candidate hardening** | **Last updated: 2026-07-22**

## 1. Release objective

v0.9.0は新機能を追加せず、v1.0へ向けたprotocol、public API、storage、operator contractのfreeze candidateを記録し、securityとresource boundednessを強化するreleaseです。

## 2. Release scope completion

### 2.1 Rust public API audit

- [x] workspace全crateのexported surfaceをinventory化した
- [x] freeze candidate、behavior-frozen、workspace-internal、implementation-detailへ分類した
- [x] public error、trait、re-export、version constantの互換性影響を確認した
- [x] accidental public surfaceをrelease blockerとして扱う方針を固定した
- [x] v1.x semver／compatibility方針を文書化した

Evidence:

- `docs/architecture/V0_9_RUST_API_INVENTORY.md`
- `docs/architecture/V0_9_PUBLIC_API_FREEZE_CANDIDATE.md`

### 2.2 Freeze candidates

- [x] protocol freeze candidateを記録した
- [x] public API freeze candidateを記録した
- [x] storage formatとdurable proof contractへの非変更を確認した
- [x] protocol、schema、package、storageのversion軸を照合した
- [x] unknown newer／corrupt／contradictory stateのfail-closed contractを維持した
- [x] freeze後のbreaking changeをrelease blockerとして扱う手順を固定した

Disposition:

- wire protocol／schema: `0.1.0`維持
- Rust workspace packages: `0.9.0`
- storage format: 変更なし
- archive／journal／proof formats: 変更なし

### 2.3 Security review

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
- [x] findingごとにseverity、owner、disposition、regression evidenceを記録した
- [x] unresolved Critical／High findingがゼロである
- [x] unresolved release-blocking Medium findingがゼロである

Finding summary:

- LB-SEC-009-001: fixed and regression-tested
- LB-SEC-009-002: fixed and regression-tested

### 2.4 Parser／property regression

- [x] malformed parser inputsを固定した
- [x] canonical round-trip idempotenceを固定した
- [x] repeated parse determinismを固定した
- [x] input-size boundaryを固定した
- [x] array／object mixed nesting boundaryを固定した
- [x] depth超過をpanic-free rejectionとして固定した
- [x] bounded CI regressionと長時間fuzz／soakの責務を分離した

### 2.5 Production-like acceptance and soak

- [x] v0.8.0 reference-platform acceptanceを継承できる非packaging変更であることを確認した
- [x] publish／storage／migration／backup／restore／index／replacement／cleanupのworkspace regressionがgreen
- [x] parser、signature workspace、replacement crash matrixのbounded soakを5反復実行した
- [x] bounded soak pass criteriaを満たした
- [x] bounded soakと長時間実機soakの差異を残存リスクとして記録した

Long-running resource telemetry、disk-pressure／power-loss injection、実機長時間運転はv1.0 stable gateへ継続する。v0.9.0ではbounded CI soakをrelease evidenceとする。

### 2.6 Supported platform and packaging

- [x] formal reference platformをUbuntu Server 24.04 LTS、x86_64、systemdのまま維持した
- [x] v0.8.0のinstall／systemd／backup／restore acceptance contractを変更していない
- [x] 全workspace packageとlockfileを0.9.0へ統一した
- [x] v0.8.0からのupgradeにstorage migrationを追加しないことを確認した
- [x] compatible binary rollback／verified backup restore contractを維持した
- [x] unsupported／best-effort platform表記を維持した

### 2.7 Documentation freeze

- [x] release checklist
- [x] hardening plan
- [x] security review and findings
- [x] public API inventory and freeze candidate
- [x] signature workspace remediation contract
- [x] release evidence
- [x] release notes
- [x] changelog
- [x] known limitations and residual risks

## 3. Mandatory validation

- [x] `cargo fmt --all -- --check`
- [x] library clippy with warnings denied
- [x] binary clippy with established dead-code allowance
- [x] test-target clippy
- [x] `cargo test --workspace`
- [x] JavaScript test suite
- [x] external protocol conformance suite
- [x] parser baseline and limit regressions
- [x] signature workspace security regressions
- [x] replacement／cleanup crash matrix
- [x] bounded hardening soak, 5 consecutive iterations

Evidence is recorded in `V0_9_RELEASE_EVIDENCE.md`.

## 4. Release-blocker disposition

| Severity | Open count | Disposition |
|---|---:|---|
| Critical | 0 | release allowed |
| High | 0 | release allowed |
| Release-blocking Medium | 0 | all fixed and tested |
| Low／residual | documented | accepted for v0.9.0 |

## 5. Compatibility and safety declaration

v0.9.0は次を変更しません。

- canonical object model
- signature payload definition
- protocol／schema version
- storage format
- migration journal
- backup archive
- replacement／cleanup proof contract
- authorization ordering

新しいparser limitはavailabilityを守るfail-closed boundaryです。silent relaxationは行わず、変更時はcompatibility reviewとregression updateを必須とします。

## 6. Completion decision

```text
public contracts inventoried
→ freeze candidates recorded
→ security findings fixed and tested
→ parser / signature regressions green
→ standard CI and conformance green
→ bounded hardening soak green
→ workspace version 0.9.0
→ release documents prepared
→ release-ready
```

## 7. Publication record

- Release PR: #108
- Source hardening commit: `fe23c523f358cfa62aea396ec7481778a0915c2c`
- Security regression-test commit: `1083ab0348881aabba924f102151c5d4ed3da292`
- Version preparation commit: `e5b308e54c5ed888dd3b162c37e70fb6bfd48c42`
- Post-hardening standard CI: run 1141, success
- Versioning／conformance／bounded-soak workflow: run `29898586767`, success
- Final standard CI: pending final documentation commit
- Merge commit: pending
- Tag: pending
- GitHub Release: pending
