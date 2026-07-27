# 現在の実装状況

**Status: v1.0.0 active-candidate reference-host preflight ready** | **Latest published release: v0.9.0** | **Next release target: v1.0.0** | **Last updated: 2026-07-27**

この文書は、Lingonberryの実装・qualification・release作業を中断／再開するときの引き継ぎ用正本です。

## Release state

```text
latest published release: 0.9.0
next release target: 1.0.0
active pre-version candidate: 8c6b48082205a3af555130eec1f3e7d2ac8811fe
reference platform: Ubuntu Server 24.04 LTS, x86_64, systemd
candidate qualification: passed and independently inspected
documentation walkthrough: passed and independently inspected
repository-side preflight: complete
privileged reference-host preflight: ready to execute
formal 72-hour soak: blocked until reference-host preflight passes
v1.0.0 version/tag/GitHub Release: pending
```

Evidence／documentation commitをcandidate決定後に追加しても、固定candidateは変更しません。

## Active candidate identities

```text
candidate commit:
8c6b48082205a3af555130eec1f3e7d2ac8811fe

lingonberry-storage SHA-256:
737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507

lingonberry-relay SHA-256:
23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

Superseded candidate `f9543019f2c219aea3b085ff90f2da201b268a48`と、そのcandidateに結び付くbinary、walkthrough、soak、release authorization evidenceは歴史記録です。active runtimeの承認には使用しません。

## Verified candidate evidence

### Candidate qualification

- workflow run: `30238378797`
- artifact ID: `8642393171`
- artifact digest: `sha256:c30a0472f6ea07f3e395c9a27c67d1460b8f35a13a7afd397bd0e5895cb93b3e`
- candidate SHA、binary SHA-256、全`SHA256SUMS` entryを独立検証済み
- 12 qualification gate: all passed

### Documentation walkthrough

- workflow run: `30239602412`
- artifact ID: `8642773653`
- artifact digest: `sha256:9b954ada86f86e5da4966951039af9dddc2eddb3d49c996d09256e4cad598338`
- 16 procedures: all passed
- 34 `SHA256SUMS` entries: all verified
- valid、tampered、malformed、verifier failureのsignature pathを確認
- duplicate／conflict pathがpublisher authenticationを迂回しないことを確認

## v1.0.0で固定済みの主要範囲

### Canonical storage and derived state

- canonical storageをsemantic source of truthとして維持
- index／effective viewは検証・再構築可能な派生状態
- deterministic index verify／rebuild、checkpoint、catch-up
- stale worker／incomplete evidenceによるlast-known-good state上書きを拒否

### Authenticated publishing

- Ed25519 publisher signatureをJSON parse直後に検証
- acceptance policy、quarantine、duplicate／conflict判定、raw append、canonical storageより前にfail closed
- malformed encoding: `LB_PUBLISH_SIGNATURE_MALFORMED`
- cryptographically invalid signature: `LB_PUBLISH_SIGNATURE_INVALID`
- verifier execution failure: `LB_PUBLISH_SIGNATURE_VERIFIER_ERROR`
- valid signed fixture、tampered signature、duplicate／conflict non-bypassを検証済み

### Operations and recovery

- Ubuntu Server 24.04 LTS、x86_64、systemd
- release-built binaries under `/usr/local/bin`
- hardened systemd units
- health／ready／status／doctor／verify／metrics
- verified backup and isolated restore
- deterministic index verify／rebuild
- explicit migration、resume、rollback
- persistent quarantine、verified replacement、proof-bound cleanup

### Input and workspace hardening

- JSON input size limit: 1 MiB
- array／object共通nesting depth limit: 128
- oversized／over-nested inputをrecursive parse前に拒否
- signature verification workspaceのexclusive creation、owner-only permission、RAII cleanup

## Current GO / NO-GO boundary

### GO

- repository-side implementation and evidence review
- exact-candidate qualification
- candidate documentation walkthrough
- privileged reference-host preflight preparation

### NO-GO

次はreference-host preflightがPASSするまで開始しません。

- formal 72-hour soak
- version `1.0.0` update
- release checklist／release notes／CHANGELOGの最終確定
- merged release commit validation
- annotated tag `v1.0.0`
- GitHub Release publication

## Reference-host preflight inputs

- candidateと2つのbinary digestを変更しない
- Ubuntu Server 24.04 LTS、x86_64、systemdを使用
- service user、directory、ownership、environment file、systemd unitを記録
- evidence／journalをdisk-pressure対象とは別filesystemへ置く
- startup、restart、signed publish、persistence、diagnostics、backup／restore、indexを実行
- malformed、invalid signature、verifier failure、duplicate、conflictのfail-closed behaviorを実行
- disk-pressure rehearsalとstop conditionを確認
- UTC timestamp、operator identity、host provenance、deviation、artifact digestを記録

Tracking issue: [#343](https://github.com/nkkmd/lingonberry/issues/343)

## Canonical documents

再開時は次の順に確認します。

1. [v1.0.0 Qualification Status](./V1_0_QUALIFICATION_STATUS.md)
2. [v1.0.0 Release Evidence](./V1_0_RELEASE_EVIDENCE.md)
3. [v1.0.0 Documentation Walkthrough](./V1_0_DOCUMENTATION_WALKTHROUGH.md)
4. [v1.0.0 Security Diff Review](../security/V1_0_SECURITY_DIFF_REVIEW.md)
5. [v1.0.0 Qualification Plan](./V1_0_QUALIFICATION_PLAN.md)
6. [v1.0.0 Soak Plan](./V1_0_SOAK_PLAN.md)
7. [v1 Compatibility Policy](../architecture/V1_COMPATIBILITY_POLICY.md)
8. [v1.0 Operator Runbook](../operations/V1_0_OPERATOR_RUNBOOK.md)
9. [Roadmap to v1.0](./ROADMAP_TO_V1_0.md)

## Next step

Issue #344で入口文書と実行identityを整合させた後、Issue #343に従ってactive candidateのprivileged reference-host preflightを実行します。preflightがPASSした場合のみ、同じcandidate／binary identityで正式72時間soakを開始します。
