# 現在の実装状況

**Status: v0.5.0 release candidate merged to main** | **Last updated: 2026-07-19**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.5.0の機能実装、end-to-end smoke scenario、package version更新、release checklist、release notesはmainへmerge済みです。READMEと関連index文書をv0.5.0へ同期した後、最新main commitのCI成功を確認してtagとGitHub Releaseを公開します。

```text
candidate version: 0.5.0
parent issue: #76
release hardening PR: #94
release hardening merge commit: 747bc4bbefee75bc2c48187638c683487273cb42
release checklist: docs/roadmap/RELEASE_0_5_0_CHECKLIST.md
release notes: docs/roadmap/RELEASE_0_5_0_RELEASE_NOTE.md
publication state: tag and GitHub Release pending final documentation sync and main-CI confirmation
```

Latest published releaseはv0.4.0です。

## 2. v0.5.0で実装済み

| 項目 | 状態 |
|---|---|
| versioned publish ingestion contract | 実装済み |
| CLI／HTTP共通ingestion orchestrator | 実装済み |
| deterministic duplicate／conflict classification | 実装済み |
| versioned object retrieval contract | 実装済み |
| versioned basic query contract | 実装済み |
| deterministic index generation and content digest | 実装済み |
| rebuild／verification／atomic checkpoint | 実装済み |
| checkpoint-driven catch-up | 実装済み |
| corrupt／unsupported／ambiguous state fail-closed | 実装済み |
| restart／recovery／ambiguity smoke coverage | 実装済み |
| workspace package version 0.5.0 | mainへmerge済み |
| release checklist／release notes／CHANGELOG | mainへmerge済み |

## 3. Index lifecycle model

Canonical storageを正本とし、indexは検証・再構築可能な派生状態として扱います。

- generationはcanonical ID集合とrecord contentから決定的に生成する
- checkpointはconsistentなrebuild resultからのみatomicに保存する
- missingまたはstale checkpointはcatch-up可能
- corrupt、unsupported、ambiguous checkpoint／index stateは自動上書きしない
- contradictory stateを成功扱いしない

## 4. End-to-end保証

CIは次の経路を実binaryで検証します。

```text
publish
→ validate
→ store
→ retrieve
→ query
→ process restart
→ retrieve／query
→ rebuild／consistency verification
→ checkpoint catch-up
```

さらにduplicate、conflict、defer、validation reject、corrupt checkpoint、ambiguous index rejectionを固定しています。

## 5. Compatibility

- v0.4.0までのquarantine／backup／replacement／cleanup安全性を維持
- canonical storageとarchive／immutable evidenceをindex lifecycleから書き換えない
- File／SQLite backendでduplicate／conflictとindex lifecycleのparityを検証
- multi-node consistency、vector search、AI integrationはv0.5.0の非スコープ

## 6. Release gate

完了済み：

- v0.5.0 feature implementation
- Phase 1〜5 contract／smoke coverage
- package version `0.5.0`
- `Cargo.lock` synchronization
- release checklist／release notes／CHANGELOG
- release hardening PR #94の全CI成功
- release hardening PRのmain merge

未完了：

- README／roadmap／operations indexのv0.5.0同期
- 同期後の最新main CI成功確認
- annotated tag `v0.5.0`
- GitHub Release `Lingonberry v0.5.0`
- release checklistへのpublication記録

## 7. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. conflict時に既存canonical recordを上書きしない
3. canonical storage commit後のindex failureを保存失敗へ書き換えない
4. corruptionとI/O errorをnot-foundやsuccessへ変換しない
5. inconsistent index resultからcheckpointを更新しない
6. corrupt／unsupported／ambiguous stateを自動修復しない
7. archive segmentとimmutable evidence ledgerを変更しない
8. same-host lockをdistributed lockとして扱わない
9. metricsへpath、identifier、digest、record ID、free-form errorを出さない
