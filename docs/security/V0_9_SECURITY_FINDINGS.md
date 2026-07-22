# v0.9.0 Security Findings

**Status: active** | **Release target: v0.9.0** | **Last updated: 2026-07-22**

この文書は、v0.9.0 security reviewで確認したfinding、severity、根拠、修正状態、release dispositionを追跡する正本です。

## Severity and release policy

- **Critical**: exploitまたは安全境界の破壊が現実的で、直ちにreleaseを停止する。
- **High**: confidentiality、integrity、availability、authorization、durabilityの重大な破壊につながり、release前の修正を必須とする。
- **Medium**: 防御層の不足、限定的なresource exhaustion、残留情報、運用上の安全性低下。原則としてv0.9.0で修正する。
- **Low**: defense-in-depthまたはhardening improvement。未修正の場合は明示的なdispositionを必要とする。

Critical／High findingは未解決のままv0.9.0 release candidateを完了扱いにしない。

## Finding LB-SEC-009-001

### Summary

Signature verification temporary artifacts are not removed after verification.

### Status

- Severity: **Medium**
- State: **open**
- Owner: v0.9.0 hardening workstream
- Release blocker: **yes until fixed or explicitly downgraded with evidence**
- Affected component: `packages/protocol/src/lib.rs`
- Affected function: `verify_publish_request_signature_with_openssl`

### Observation

署名検証はOS temporary directory配下に、次の3ファイルを作成します。

- `public-key.der`
- `signature.bin`
- `message.bin`

workspace名は現在時刻のnanosecond値から生成されます。検証成功時と失敗時のどちらにもworkspaceを削除するcleanup pathがありません。

### Security impact

1. canonical publish request payload、公開鍵、署名がtemporary directoryに残留する。
2. 長時間運用または大量の署名検証によりtemporary storageが単調増加する。
3. temporary directoryのpermissionとhost policyによっては、同一host上の別principalに検証材料が観測される可能性がある。
4. timestamp-only workspace nameは通常は衝突しにくいが、exclusive creation contractではなく、衝突をfail-closedに扱う保証が明示されていない。

現時点でsignature verification bypassの証拠はない。ただしinformation leakage、disk pressure、temporary-file handlingのreview itemに該当する。

### Required remediation

- workspaceをexclusiveに作成する。
- file creation時に既存pathを追従または上書きしない。
- Unix reference platformではowner-only permissionを保証する。
- success、verification failure、OpenSSL execution failure、intermediate write failureの全経路でbest-effort cleanupを実行する。
- cleanup failureを署名成功として黙殺する場合の契約を明示し、diagnostic evidenceを残す。
- regression testでsuccess／failure後のworkspace cleanupを確認する。
- process crash後に残り得るworkspaceの運用上のcleanup policyを文書化する。

### Acceptance criteria

- 通常の成功・失敗経路でtemporary verification workspaceが残らない。
- pre-existing workspace、symlink、non-directory collisionをfail closedで拒否する。
- concurrent verificationが同一workspaceを共有しない。
- request payloadやsignature materialをerror messageへ出力しない。
- standard CIとsecurity regression testが成功する。

## Finding template

新しいfindingは次を必ず記録する。

- identifier
- summary
- severity
- state
- affected component
- evidence
- security impact
- remediation
- acceptance criteria
- release disposition
