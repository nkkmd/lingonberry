# Phase 4: Durable index lifecycle

## Goal

Canonical storageを正本とし、indexを検証・再構築可能な派生状態として扱う。

## Contract v1

- rebuild result contract version: `1`
- status: `consistent` / `inconsistent` / `failed`
- stable codes: `LB_INDEX_CONSISTENT` / `LB_INDEX_INCONSISTENT` / `LB_INDEX_AMBIGUOUS`
- storageとindexのrecord count、ID digest、content digest、missing／unexpected／ambiguous IDを比較
- generationはcanonical ID集合の昇順FNV-1a digestから導出
- content digestはcarrier identity、stored timestamp、canonical object JSONから導出
- checkpoint version: `1`
- checkpointは`LINGONBERRY_STATE_DIR/index/checkpoint.json`へatomic renameで保存
- corrupt／unsupported checkpointはfail closed
- catch-up contract version: `1`
- catch-up status: `up-to-date` / `rebuilt` / `failed`
- checkpoint一致時は無変更、不一致または未作成時はcanonical storageから再構築
- corrupt／unsupported checkpointは自動上書きしない
- ID集合一致でもrecord fingerprintが異なるsnapshotはambiguousとして成功扱いしない

## Work items

- [x] 正式rebuild API
- [x] generation／ID digest
- [x] content digest／ambiguous ID検出
- [x] consistency verification result型
- [x] File／SQLite parity test
- [x] CLI `rebuild-index`接続
- [x] checkpoint persistence
- [x] catch-up
- [x] corrupt checkpointのfail-closed test
- [x] ambiguous indexのfail-closed test
- [ ] release roadmap同期
