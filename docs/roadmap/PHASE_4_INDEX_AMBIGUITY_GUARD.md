# Phase 4: Index ambiguity guard

## Goal

Canonical ID集合が一致していてもrecord内容が矛盾するindexを検出し、fail closedで扱う。

## Contract

- code: `LB_INDEX_AMBIGUOUS`
- status: `inconsistent`
- `ambiguousIds`で該当canonical IDを列挙
- storage／index双方にcontent digestを含める
- canonical storageを正本とし、ambiguous snapshotからcheckpointを更新しない

## Coverage

- [x] File backend
- [x] SQLite backend
- [x] same ID set／different canonical object
- [ ] full CI
- [ ] Phase 4 roadmap同期
