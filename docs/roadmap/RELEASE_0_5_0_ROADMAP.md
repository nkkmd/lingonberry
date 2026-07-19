# Lingonberry v0.5.0 Release Roadmap

**Status: implementation in progress** | **Target: v0.5.0** | **Last updated: 2026-07-19**

## 1. 目的

v0.5.0では、quarantine／maintenance系に偏っていた実装を通常のknowledge object lifecycleへ統合し、単一ノードで次の経路をend-to-endに成立させます。

```text
receive
→ parse
→ schema validation
→ semantic validation
→ identity／signature verification
→ duplicate／conflict classification
→ canonical storage または quarantine
→ index update
→ retrieve／query
```

最終的には、次の一連を単一の自動化されたsmoke scenarioとして成功させます。

```text
publish
→ validate
→ store
→ retrieve
→ query
→ restart
→ query
→ consistency verification
```

## 2. v0.4.0から引き継ぐ基盤

- canonical protocol／identity／validationのRust実装
- SQLite canonical catalog
- append-only raw wire log
- canonical ID取得
- persistent quarantine lifecycle
- verified backup／restore
- verified replacement／cleanup transaction
- storage backendからのindex snapshot／rebuild
- CLI publish／get／query系command
- HTTP publish／get API

## 3. 現在の主要gap

### 3.1 Publish orchestrationの重複

validation、acceptance decision、quarantine append、finalization、storage append、duplicate／conflict responseの組み立てがrelay層へ集中しています。

CLIとHTTP、将来のcarrier adapterが同じ処理を共有できる共通orchestratorが必要です。

### 3.2 Result contractが未固定

次の状態をmachine-readableに統一します。

- stored
- duplicate
- deferred
- rejected
- conflict
- failed
- stored-but-index-pending

### 3.3 Index lifecycleが未統合

canonical storageを正本とし、index update、partial failure、catch-up、rebuild、consistency verificationを通常publish経路へ統合する必要があります。

### 3.4 End-to-end restart scenarioが不足

個別testではなく、publishからrestart後のqueryとconsistency verificationまでを単一scenarioとして保証します。

## 4. 実装工程

## Phase 1: Ingestion result contractと共通orchestrator

Status: **in progress**

- [x] `packages/core`にversioned publish ingestion result型を追加
- [x] `stored`／`duplicate`／`deferred`／`rejected`／`conflict`／`failed`を型として固定
- [x] validation、acceptance、quarantine、finalization、storage appendを共通orchestratorへ統合
- [x] stable machine codeとhuman-readable errorsを分離
- [x] conflictを既存recordの上書きなしで共通resultへ変換
- [ ] relay CLI publishを共通orchestratorへ移行
- [ ] relay HTTP publishを共通orchestratorへ移行
- [ ] CLI／HTTP contract testを追加

初期contract versionは`1`です。

| 状態 | Machine code |
|---|---|
| stored | `LB_OBJECT_STORED` |
| duplicate | `LB_OBJECT_DUPLICATE` |
| empty request | `LB_EMPTY_REQUEST` |
| invalid JSON | `LB_INVALID_JSON` |
| final validation failure | `LB_VALIDATION_FAILED` |
| conflict | `LB_OBJECT_CONFLICT` |
| storage／quarantine failure | 元のstable `LB_*` codeを保持 |

## Phase 2: Duplicate／conflict規則の固定

- [ ] canonical ID、carrier identity、canonical contentの比較規則を文書化
- [ ] exact duplicateをidempotent successとして固定
- [ ] same ID different contentをconflictとして固定
- [ ] same carrier identity different contentをconflictとして固定
- [ ] retry／replay／archive importで同じ規則を適用
- [ ] fixtureとproperty testを追加

## Phase 3: Public read／write API

- [ ] publish responseをversioned contractへ統一
- [ ] ID取得responseをversioned contractへ統一
- [ ] basic query responseを整理
- [ ] HTTP statusとmachine codeのmappingを固定
- [ ] CLI exit codeとmachine codeのmappingを固定

## Phase 4: Durable index lifecycle

- [ ] canonical storageからindexをrebuildする正式APIを固定
- [ ] index generationまたはcheckpointを導入
- [ ] storageとindexのrecord count／ID set／digestを比較
- [ ] partial index updateを検出
- [ ] catch-upを実装
- [ ] consistency verification reportをmachine-readable化
- [ ] corrupt／ambiguous indexをfail closedで扱う

## Phase 5: End-to-end smoke scenario

- [ ] fresh stateでpublish
- [ ] canonical storage確認
- [ ] ID取得
- [ ] basic query
- [ ] process restart
- [ ] restart後のID取得／query
- [ ] index consistency verification
- [ ] duplicate publish
- [ ] conflict publish
- [ ] defer／quarantine
- [ ] validation reject
- [ ] partial index updateからのcatch-up
- [ ] ambiguous stateを成功扱いしないことの確認

## Phase 6: Release hardening

- [ ] package versionを`0.5.0`へ更新
- [ ] `Cargo.lock`同期
- [ ] Rust format／Clippy／workspace tests
- [ ] JavaScript contract tests
- [ ] release checklist／release notes／CHANGELOG
- [ ] `CURRENT_IMPLEMENTATION_STATUS.md`同期
- [ ] merge後main CI
- [ ] annotated tag `v0.5.0`
- [ ] GitHub Release

## 5. Transaction boundary

1. validationとacceptanceが完了するまでcanonical storageを変更しない
2. deferの場合はquarantineへdurable appendし、canonical storageを変更しない
3. acceptの場合はraw wire logとcanonical storageを更新する
4. canonical storage commit後のindex failureはcanonical objectの保存失敗へ書き換えない
5. index未反映はpartial resultまたはcatch-up対象として明示する
6. conflict時は既存canonical recordを変更しない

storageとindexを偽装atomic transactionとして扱いません。canonical storageを正本とし、indexは検証・再構築可能な派生状態として扱います。

## 6. Error contract原則

- machine codeはversionedで安定させる
- human-readable messageは互換性判定に使わない
- validation、identity、signature、conflict、storage、index、quarantineを分類する
- corruptionとI/O errorをnot-foundへ変換しない
- unsupported versionとinvalid inputを区別する
- path、identifier、digest、record ID、free-form errorをmetrics labelへ出さない

## 7. 完了条件

```text
publish
→ validate
→ store
→ retrieve
→ query
→ restart
→ query
→ consistency verification
```

このscenarioが自動化され、CIで成功すること。

さらに次を満たすこと。

- duplicateとconflictがdeterministicに分類される
- validation failureがversioned machine-readable codeで返る
- deferred objectがcanonical storageへ入らない
- indexがcanonical storageからrebuild／catch-upできる
- partial update、corruption、I/O failure、contradictory stateを成功扱いしない

## 8. 非スコープ

- protocol conformance suiteの完成
- storage migration framework
- operator CLI全面統合
- vector search／AI integration
- multi-node consistency
- distributed lock／consensus

## 9. 関連Issue

- #76: v0.5.0 parent tracking issue
- #77: ingestion result contract and common orchestrator
