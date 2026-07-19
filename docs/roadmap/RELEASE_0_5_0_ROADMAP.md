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

### 3.1 Publish orchestration

Phase 1で、validation、acceptance decision、quarantine append、finalization、storage append、duplicate／conflict分類を`packages/core`の共通orchestratorへ統合しました。

CLIとHTTPは同じversioned result contractとrelay adapterを使用します。

### 3.2 Result contract

Phase 1で次の状態をmachine-readableに固定しました。

- stored
- duplicate
- deferred
- rejected
- conflict
- failed

Phase 3でobject retrievalのfound／invalid-request／not-found／failedもversioned contractへ固定しました。

`stored-but-index-pending`はPhase 4のdurable index lifecycleで追加します。

### 3.3 Index lifecycleが未統合

canonical storageを正本とし、index update、partial failure、catch-up、rebuild、consistency verificationを通常publish経路へ統合する必要があります。

### 3.4 End-to-end restart scenarioが不足

個別testではなく、publishからrestart後のqueryとconsistency verificationまでを単一scenarioとして保証します。

## 4. 実装工程

## Phase 1: Ingestion result contractと共通orchestrator

Status: **completed**

- [x] `packages/core`にversioned publish ingestion result型を追加
- [x] `stored`／`duplicate`／`deferred`／`rejected`／`conflict`／`failed`を型として固定
- [x] validation、acceptance、quarantine、finalization、storage appendを共通orchestratorへ統合
- [x] stable machine codeとhuman-readable errorsを分離
- [x] conflictを既存recordの上書きなしで共通resultへ変換
- [x] relay CLI publishを共通orchestratorへ移行
- [x] relay HTTP publishを共通orchestratorへ移行
- [x] CLI／HTTP process-level contract testを追加

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

process-level contract testは、実binaryと実TCP HTTP serverを起動し、署名検証を含めてstored、duplicate、deferred、rejected、conflictを確認します。

## Phase 2: Duplicate／conflict規則の固定

Status: **completed**

- [x] canonical ID、carrier identity、canonical contentの比較規則を文書化
- [x] exact duplicateをidempotent successとして固定
- [x] same ID different contentをconflictとして固定
- [x] same carrier identity different contentをconflictとして固定
- [x] live publish／retry／archive import／quarantine promotionへ同じ規則を適用
- [x] File／SQLite parity fixtureとcontract testを追加
- [x] archive re-importとconflict safety boundaryを検証
- [x] active single／batch quarantine promotion CLIをclassified APIへ接続
- [x] active archive import CLIをclassified APIへ接続
- [x] `replay()`が読み取り専用であり、現行のmutating restore経路ではないことを確認
- [x] File backend内部の判定を共通classifierへ置換
- [x] SQLite backend内部の判定を共通classifierへ置換

## Phase 3: Public read／write API

Status: **in progress**

- [x] publish responseをversioned contractへ統一
- [x] ID取得responseをversioned contractへ統一
- [x] basic query responseをversioned contractへ統一
- [x] HTTP statusとmachine codeのmappingを固定
- [x] CLI exit codeとmachine codeのmappingを固定
- [x] 実binaryによるpublish→GET found／not-found process-level contract testを追加

Basic query contract version `1`は`success`／`empty`／`invalid-request`／`failed`を固定し、返却順序を`canonicalId-ascending`として明示します。

Object retrieval contract version `1`は次の状態を固定します。

| 状態 | Machine code | HTTP |
|---|---|---:|
| found | `LB_OBJECT_FOUND` | 200 |
| invalid-request | `LB_CANONICAL_ID_REQUIRED` | 400 |
| not-found | `LB_OBJECT_NOT_FOUND` | 404 |
| failed | 元のstable `LB_*` codeを保持 | 500 |

## Phase 4: Durable index lifecycle

- [x] canonical storageからindexをrebuildする正式APIを固定
- [x] index generationまたはcheckpointを導入
- [x] storageとindexのrecord count／ID set／digestを比較
- [x] partial index updateを検出
- [x] catch-upを実装
- [x] consistency verification reportをmachine-readable化
- [x] corrupt／ambiguous indexをfail closedで扱う

## Phase 5: End-to-end smoke scenario

- [x] fresh stateでpublish
- [x] canonical storage確認
- [x] ID取得
- [x] basic query
- [x] process restart
- [x] restart後のID取得／query
- [x] index consistency verification
- [x] duplicate publish
- [x] conflict publish
- [x] defer／quarantine
- [x] validation reject
- [x] partial index updateからのcatch-up
- [ ] ambiguous stateを成功扱いしないことの確認

## Phase 6: Release hardening

- [ ] package versionを`0.5.0`へ更新
- [ ] `Cargo.lock`同期
- [x] Rust format／Clippy／workspace tests（Phase 1）
- [x] JavaScript contract tests（Phase 1）
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
- #77: ingestion result contract and common orchestrator（completed）
- #83: object retrieval response contract
