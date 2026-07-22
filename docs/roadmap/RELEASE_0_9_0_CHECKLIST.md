# Lingonberry v0.9.0 Release Checklist

**Status: in progress** | **Target: v0.9.0** | **Release type: release-candidate hardening** | **Last updated: 2026-07-22**

## 1. Release objective

v0.9.0では新機能追加を原則停止し、v1.0で公開するprotocol、public API、storage format、operator contractのfreeze candidateを固定する。そのうえでsecurity review、fuzzing／property testing、production-like acceptance、long-running soakを実施し、critical／high severityのrelease blockerが残っていないことを証明する。

## 2. Release scope

### 2.1 Rust public API audit

- [ ] workspace全crateの`pub`／`pub(crate)`／re-exportをinventory化した
- [ ] intended public APIとimplementation detailを分類した
- [ ] accidental public surfaceを縮小した
- [ ] public error type、trait bound、feature flag、re-exportの互換性影響を確認した
- [ ] v1.x semver policyを文書化した

### 2.2 Freeze candidates

- [ ] protocol freeze candidateを記録した
- [ ] public API freeze candidateを記録した
- [ ] storage format freeze candidateを記録した
- [ ] protocol、schema、API、storage、journal、proofのversion軸を照合した
- [ ] compatibility matrixを最新化した
- [ ] unknown newer version／formatがfail closedで拒否されることを確認した
- [ ] freeze後のbreaking changeをrelease blockerとして扱う手順を固定した

### 2.3 Security review

- [ ] path traversal
- [ ] symlink handling
- [ ] oversized input
- [ ] deeply nested input
- [ ] malformed serialization
- [ ] signature verification bypass
- [ ] authorization ordering
- [ ] information leakage
- [ ] TOCTOU
- [ ] disk-full／I/O failure
- [ ] findingごとにseverity、owner、disposition、regression testを記録した
- [ ] unresolved critical／high findingがゼロである

### 2.4 Fuzzing／property testing

- [ ] parser
- [ ] validator
- [ ] identifier
- [ ] digest verifier
- [ ] journal parser
- [ ] recovery classifier
- [ ] index／segment reader
- [ ] crash、panic、unbounded allocation、unbounded recursionを検出対象にした
- [ ] minimized corpusをregression fixtureとして保存した
- [ ] bounded CI regressionと長時間manual／scheduled fuzzの責務を分離した

### 2.5 Production-like acceptance and soak

- [ ] reference topologyを固定した
- [ ] publish／read／query／restartの継続負荷を実行した
- [ ] backup create／verify／isolated restoreを反復した
- [ ] index verify／rebuildを反復した
- [ ] quarantine／replacement／cleanup／migration／recoveryの代表経路を実行した
- [ ] disk pressure／I/O failure／process interruptionを注入した
- [ ] memory、file descriptor、disk growth、latency、error rateを記録した
- [ ] soak durationとpass criteriaを満たした
- [ ] test artifactと残存リスクを保存した

### 2.6 Supported platform and packaging

- [ ] Ubuntu Server 24.04 LTS、x86_64、systemdを再検証した
- [ ] clean hostへrelease candidate artifactをinstallできた
- [ ] checksumを検証した
- [ ] `/usr/local/bin`、service unit、environment file、ownership contractを確認した
- [ ] v0.8.0からupgradeできた
- [ ] compatibility範囲内でrollbackできた
- [ ] unsupported／best-effort platform表記を確定した

### 2.7 Documentation freeze

- [ ] installation
- [ ] configuration
- [ ] protocol
- [ ] public API
- [ ] security
- [ ] upgrade／rollback
- [ ] backup／restore
- [ ] operations
- [ ] troubleshooting
- [ ] compatibility policy
- [ ] known issues
- [ ] release notes

## 3. Mandatory validation

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] protocol conformance suite
- [ ] supported legacy-state migration suite
- [ ] object lifecycle end-to-end smoke
- [ ] replacement／cleanup crash matrix
- [ ] operator acceptance
- [ ] isolated disaster-recovery drill
- [ ] fuzz／property regression suite
- [ ] production-like soak

## 4. Severity and release-blocker policy

| Severity | Definition | v0.9.0 disposition |
|---|---|---|
| Critical | signature／authorization bypass、canonical data corruption、silent destructive operation、remote code execution相当 | 必ず修正。未解決ならrelease禁止 |
| High | fail-open、durability violation、unbounded resource exhaustion、reliable sensitive information disclosure | 原則必ず修正。例外承認なし |
| Medium | bounded denial of service、operator recovery阻害、限定的な契約不整合 | 修正または明示的known issueと期限付き追跡 |
| Low | defense-in-depth、診断品質、文書上の曖昧さ | risk acceptance可能。ただし記録必須 |

severityは影響、悪用可能性、再現性、検出可能性、回復可能性を根拠に決定する。テスト未整備のsecurity findingは、コード修正だけで完了扱いにしない。

## 5. Freeze policy

freeze candidate確定後、protocol、public API、storage formatに対するbreaking changeは次を必須とする。

1. release blocker issue
2. compatibility impact analysis
3. migrationまたは互換shimの要否判定
4. specification、fixture、test、documentationの同時更新
5. v1.0 release gateへの明示的な再承認

## 6. Completion criteria

次をすべて満たした場合のみv0.9.0をrelease-readyと判定する。

```text
public contracts inventoried
→ freeze candidates recorded
→ security review completed
→ fuzz / property regression green
→ production-like acceptance green
→ soak criteria satisfied
→ zero critical / high release blockers
→ release candidate documents frozen
```

## 7. Publication record

- Release PR: pending
- Merge commit: pending
- Tag: pending
- GitHub Release: pending
- Standard CI run: pending
- Reference-platform acceptance run: pending
- Soak result: pending
