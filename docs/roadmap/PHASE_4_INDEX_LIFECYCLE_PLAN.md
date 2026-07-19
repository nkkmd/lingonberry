# Phase 4: Durable index lifecycle

## Goal

Canonical storageを正本とし、indexを検証・再構築可能な派生状態として扱う。

## Contract v1

- rebuild result contract version: `1`
- status: `consistent` / `inconsistent` / `failed`
- stable codes: `LB_INDEX_CONSISTENT` / `LB_INDEX_INCONSISTENT`
- storageとindexのrecord count、ID digest、missing／unexpected IDを比較
- generationはcanonical ID集合の昇順FNV-1a digestから導出

## Work items

- [x] 正式rebuild API
- [x] generation／ID digest
- [x] consistency verification result型
- [x] File／SQLite parity test
- [ ] CLI `rebuild-index`接続
- [ ] checkpoint persistence
- [ ] catch-up
- [ ] corrupt／ambiguous indexのfail-closed test
- [ ] release roadmap同期
