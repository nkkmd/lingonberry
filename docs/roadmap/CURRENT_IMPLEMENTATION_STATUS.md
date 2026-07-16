# 現在の実装状況

**Status: v0.4.0 release candidate merged to main** | **Last updated: 2026-07-17**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.4.0の実装、package version更新、release checklist、release notesはmainへmerge済みです。`v0.4.0` tagとGitHub Releaseは、merge後のmain branch CI成功を確認してから公開します。

```text
candidate version: 0.4.0
candidate commit: f304564a963f6e65470c3e2dead9906bf49ee9aa
release issue: #62
release readiness issue: #68
release readiness PR: #74
release checklist: docs/roadmap/RELEASE_0_4_0_CHECKLIST.md
release notes: docs/roadmap/RELEASE_0_4_0_RELEASE_NOTE.md
publication state: tag and GitHub Release pending main-CI confirmation
```

Latest published releaseはv0.3.0です。

## 2. v0.4.0で実装済み

| 項目 | 状態 |
|---|---|
| deterministic retention policy evaluator | 実装済み |
| retained previous-generation floor | 実装済み |
| durable terminal completion evidence | 実装済み |
| canonical cleanup plan／proof | 実装済み |
| atomic JSON／digest artifact publication | 実装済み |
| current-state reconstruction and stale-proof verifier | 実装済み |
| exact eligible-set binding | 実装済み |
| path traversal／glob／duplicate rejection | 実装済み |
| symlink／special-file／unexpected-entry rejection | 実装済み |
| versioned cleanup transaction journal | 実装済み |
| same-filesystem tomb preparation | 実装済み |
| sealed canonical tomb inventory | 実装済み |
| deterministic managed-path processing order | 実装済み |
| durable path-level progress | 実装済み |
| idempotent resume | 実装済み |
| rollback before irreversible processing | 実装済み |
| recovery-required／partially-deleted classification | 実装済み |
| explicit two-stage operator authorization | 実装済み |
| terminal workspace evidence retention | v0.4.0方針として固定 |
| operator runbook／failure inventory／crash matrix | 実装済み |
| workspace package version 0.4.0 | mainへmerge済み |

## 3. Cleanup safety model

Cleanup対象はoperatorがexactに指定し、verified proofのcomplete subject setと一致しなければなりません。

Categorically ineligible:

- active generation
- incomplete transaction generation
- orphan or unreferenced generation
- corrupt or ambiguous state
- legacy-root layout
- unverified generation or completion evidence
- durable age evidenceがない、またはminimum age未達のsubject
- retained-generation floorを侵害するsubject

Filesystem timestampsはretention ageの正本ではありません。

## 4. Cleanup transaction states

```text
prepared
revalidated
renaming-to-tomb
tomb-sealed
deleting
committed
recovery-required
rolled-back
partially-deleted
```

Rollbackはirreversible processing開始前だけ利用できます。開始後の中断はresume、`recovery-required`、または`partially-deleted`として扱い、自動rollbackや成功推定を行いません。

## 5. Evidence model

v0.4.0は次をdurable evidenceとして保持します。

- replacement terminal completion evidence JSON／digest
- cleanup plan／proof JSON／digest
- cleanup transaction journal／digest
- sealed tomb inventory／digest
- deterministic path-level progress
- terminal `committed`／`rolled-back`／`partially-deleted` state

terminal cleanup transaction workspaceはv0.4.0で自動削除しません。将来のretention policyはIssue #72で別versionとして扱います。

## 6. Compatibility

- pointerがないlegacy root layoutは引き続き読み取り可能
- legacy-root stateはimplicit cleanup対象にならない
- generation layoutはverified active pointerとgeneration metadataを使用
- existing replacement apply／resume／rollback、backup、index、segment verification behaviorを維持
- archive segmentとimmutable evidence ledgerはcleanupでrewrite／deleteしない

## 7. 正本文書

```text
docs/operations/QUARANTINE_REPLACEMENT_RETENTION_POLICY.md
docs/operations/QUARANTINE_REPLACEMENT_CLEANUP_RUNBOOK.md
docs/operations/quarantine-replacement-cleanup-failure-points.v1.json
docs/operations/quarantine-replacement-cleanup-crash-matrix.v1.json
docs/operations/QUARANTINE_REPLACEMENT_V0_4_0_SMOKE_TEST.md
docs/roadmap/RELEASE_0_4_0_ROADMAP.md
docs/roadmap/RELEASE_0_4_0_CHECKLIST.md
docs/roadmap/RELEASE_0_4_0_RELEASE_NOTE.md
CHANGELOG.md
```

## 8. Release gate

完了済み：

- package version `0.4.0`
- `Cargo.lock` synchronization
- release notes and changelog
- PR-side Rust format、clippy、workspace tests、JavaScript tests
- release readiness PR merge

未完了：

- merge後main branch CI成功の確認
- annotated tag `v0.4.0`
- GitHub Release `Lingonberry v0.4.0`
- release checklistへのpublication記録

## 9. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. corruptionとI/O errorを黙って無視しない
3. same-host lockをdistributed lockとして扱わない
4. archive segmentを上書き・変更・削除しない
5. immutable evidence ledgerを変更しない
6. active generationをcleanup対象にしない
7. filesystem timestampだけでeligibilityを判断しない
8. wildcard、implicit-all、partial selectionをapplyに使わない
9. proofとcurrent stateが一致しなければmutationを開始しない
10. symbolic linkやunsupported entry typeをfollow／acceptしない
11. contradictory recovery stateを自動修復または成功扱いしない
12. irreversible processing開始後にrollbackを案内しない
13. scheduled／unattended cleanupを導入しない
14. terminal cleanup workspaceをv0.4.0で自動削除しない
15. metricsへpath、identifier、digest、record ID、free-form errorを出さない
