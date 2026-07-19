# Lingonberry v0.5.0 Release Roadmap

**Status: released** | **Target: v0.5.0** | **Published: 2026-07-19**

## 1. 目的

v0.5.0は通常のknowledge object lifecycleを単一ノードでend-to-endに統合しました。

```text
receive
→ validate
→ duplicate／conflict classification
→ canonical storage または quarantine
→ retrieve／query
→ restart
→ index rebuild／verification／catch-up
```

Canonical storageを正本とし、indexは検証・再構築可能な派生状態として扱います。

## 2. Phase 1: Ingestion contract

Status: **completed**

- [x] versioned publish ingestion result
- [x] shared CLI／HTTP ingestion orchestrator
- [x] stable machine code and transport mappings
- [x] stored／duplicate／deferred／rejected／conflict／failed classification
- [x] process-level CLI／HTTP contract tests

## 3. Phase 2: Duplicate／conflict rules

Status: **completed**

- [x] exact duplicateをidempotent successとして固定
- [x] same ID different contentをconflictとして固定
- [x] same carrier identity different contentをconflictとして固定
- [x] live publish／retry／archive import／quarantine promotionへ共通規則を適用
- [x] File／SQLite parity tests

## 4. Phase 3: Public read／write API

Status: **completed**

- [x] publish response contract version `1`
- [x] object retrieval contract version `1`
- [x] basic query contract version `1`
- [x] HTTP status and CLI exit mappings
- [x] real-binary publish→GET／query tests

Basic query orderingは`canonicalId-ascending`です。

## 5. Phase 4: Durable index lifecycle

Status: **completed**

- [x] canonical storageからの正式rebuild API
- [x] deterministic generation、ID digest、content digest
- [x] storage／index count・ID set・content comparison
- [x] machine-readable consistency report
- [x] atomic checkpoint version `1`
- [x] checkpoint-driven catch-up
- [x] corrupt／unsupported／ambiguous stateのfail-closed
- [x] inconsistent resultからのcheckpoint更新拒否

## 6. Phase 5: End-to-end smoke scenario

Status: **completed**

- [x] fresh state publish
- [x] canonical storage確認
- [x] ID retrieval／basic query
- [x] process restart
- [x] restart後のGET／query
- [x] rebuild and consistency verification
- [x] missing／stale checkpointからのcatch-up
- [x] corrupt checkpoint rejection and byte preservation
- [x] ambiguous index rejection and checkpoint preservation
- [x] duplicate／conflict／defer／validation reject

## 7. Phase 6: Release hardening and publication

Status: **completed**

- [x] all Rust workspace packages version `0.5.0`
- [x] `Cargo.lock` synchronization
- [x] release checklist／release notes／CHANGELOG
- [x] `CURRENT_IMPLEMENTATION_STATUS.md` synchronization
- [x] release hardening PR #94 CI
- [x] merge PR #94 to main
- [x] README／documentation sync PR #95 CI
- [x] merge PR #95 to main
- [x] post-merge main CI
- [x] tag `v0.5.0`
- [x] GitHub Release `Lingonberry v0.5.0`

Publication record:

- Release: https://github.com/nkkmd/lingonberry/releases/tag/v0.5.0
- Tag: `v0.5.0`
- Release target commit: `bf8176da0d992152fb116ca0c45177904d1aa61c`
- Tag/main comparison at publication: identical
- Main CI: successful, confirmed in the GitHub Actions UI
- Published: 2026-07-19

## 8. Safety invariants

1. validation未通過objectをcanonical storageへ保存しない
2. conflict時に既存canonical recordを変更しない
3. canonical storage commit後のindex failureを保存失敗へ書き換えない
4. corruption／I/O errorをnot-foundまたはsuccessへ変換しない
5. inconsistent index resultからcheckpointを更新しない
6. corrupt／unsupported／ambiguous stateを自動修復しない
7. archive segmentとimmutable evidence ledgerを変更しない
8. metricsへpath、identifier、digest、record ID、free-form errorを出さない

## 9. 非スコープ

- separately persisted searchable index database
- multi-node consistency／distributed consensus
- vector search／AI integration
- protocol conformance suite completion
- storage migration framework

## 10. 関連Issue／PR

- #76: v0.5.0 parent tracking issue
- #77: ingestion result contract（completed）
- #80: duplicate／conflict parity（completed）
- #83: object retrieval contract（completed）
- #85: basic query contract（completed）
- #87: durable index lifecycle（merged）
- #88: checkpoint-driven catch-up（merged）
- #89: ambiguous index guard（merged）
- #90: restart smoke scenario（completed）
- #91: restart smoke PR（merged）
- #92: recovery smoke PR（merged）
- #93: ambiguity smoke PR（merged）
- #94: release hardening PR（merged）
- #95: README／documentation synchronization PR（merged）