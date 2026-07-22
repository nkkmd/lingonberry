# v0.9.0 Security Findings

**Status: closed for release** | **Release target: v0.9.0** | **Last updated: 2026-07-22**

この文書は、v0.9.0 security reviewで確認したfinding、severity、根拠、修正状態、release dispositionを追跡する正本です。

## Severity and release policy

- **Critical**: exploitまたは安全境界の破壊が現実的で、直ちにreleaseを停止する。
- **High**: confidentiality、integrity、availability、authorization、durabilityの重大な破壊につながり、release前の修正を必須とする。
- **Medium**: 防御層の不足、限定的なresource exhaustion、残留情報、運用上の安全性低下。原則としてv0.9.0で修正する。
- **Low**: defense-in-depthまたはhardening improvement。未修正の場合は明示的なdispositionを必要とする。

v0.9.0 release candidateに未解決のCritical／High findingはなく、確認されたrelease-blocking Medium findingは修正・回帰テスト済みです。

## Finding LB-SEC-009-001

### Summary

Signature verification temporary artifacts are not removed after verification.

### Status

- Severity: **Medium**
- State: **closed**
- Owner: v0.9.0 hardening workstream
- Release blocker: **resolved**
- Affected component: `packages/protocol/src/lib.rs`
- Affected function: `verify_publish_request_signature_with_openssl`
- Source commit: `fe23c523f358cfa62aea396ec7481778a0915c2c`
- Regression-test commit: `1083ab0348881aabba924f102151c5d4ed3da292`

### Observation

旧実装はOS temporary directory配下に公開鍵、署名、canonical payloadを書き出し、成功・失敗のどちらでもworkspaceを削除していませんでした。

### Remediation

- process ID、timestamp、process-local atomic counterから候補pathを生成する。
- `DirBuilder::create`でworkspaceをexclusiveに作成し、既存pathを再利用しない。
- Unixではworkspaceを`0o700`で作成する。
- artifactは`OpenOptions::create_new(true)`で作成し、既存fileを追従・上書きしない。
- RAII guardの`Drop`でsuccess、verification failure、command failure、intermediate write failureの通常return pathをbest-effort cleanupする。
- payload、signature、host pathをerror messageへ含めない。

### Regression evidence

`packages/protocol/src/lib.rs`のunit testで次を固定しました。

- workspace drop後の削除
- Unix owner-only permission
- existing artifact pathのfail-closed rejection
- concurrent workspaceのpath isolation
- concurrent workspace drop後のcleanup

formatting、library／binary／test-target clippy、workspace testsはtest適用workflowで成功しました。

### Residual risk

process crash、host power loss、kernel terminationではRust `Drop`が実行されないため、workspaceが残る可能性があります。これは通常return pathのcleanupとは別のoperational residual riskであり、OS temporary-directory lifecycleまたは起動時のstale-workspace cleanupで扱います。v0.9.0ではsignature bypassや通常経路の残留を示す証拠はありません。

### Release disposition

**Fixed and regression-tested. Closed for v0.9.0.**

## Finding LB-SEC-009-002

### Summary

The protocol JSON parser has no explicit input-size or nesting-depth limit.

### Status

- Severity: **Medium**
- State: **closed**
- Owner: v0.9.0 hardening workstream
- Release blocker: **resolved**
- Affected component: `packages/protocol/src/lib.rs`
- Affected function: `parse_json`
- Source and test commit: `fe23c523f358cfa62aea396ec7481778a0915c2c`

### Observation

旧実装のrecursive-descent parserには入力byte数上限とarray／object共通のnesting-depth上限がありませんでした。

### Remediation

- `MAX_JSON_INPUT_BYTES = 1 MiB`をpublic constantとして固定した。
- `MAX_JSON_NESTING_DEPTH = 128`をpublic constantとして固定した。
- oversized inputをparser state構築前に`JsonError`で拒否する。
- object／arrayの共通depth counterを導入し、depth超過をpanicせず拒否する。
- parse error後にもdepth counterを確実に復元するinner-method contractを導入した。

### Regression evidence

`packages/protocol/tests/parser_limits.rs`で次を固定しました。

- 1 MiB超過のearly rejection
- 1 MiBちょうどのvalid input受理
- depth 128の受理
- depth 129のpanic-free rejection
- mixed object／array nestingの同一上限

既存の`parser_baseline.rs`ではtrailing content、truncated structure、invalid number、canonical round-trip、determinism、depth 64互換性を継続検証します。

### Release disposition

**Fixed and regression-tested. Closed for v0.9.0.**

## Release summary

- Open Critical findings: **0**
- Open High findings: **0**
- Open release-blocking Medium findings: **0**
- Closed findings: **2**

新しいfindingを追加する場合はidentifier、summary、severity、state、affected component、evidence、security impact、remediation、acceptance criteria、release dispositionを必ず記録します。
