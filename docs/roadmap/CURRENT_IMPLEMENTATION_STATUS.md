# 現在の実装状況

**Status: v0.6.0 in progress** | **Latest release: v0.5.0** | **Last updated: 2026-07-20**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.5.0の機能実装、end-to-end smoke scenario、package version更新、release hardening、README／関連index文書の同期、main CI確認、tag、GitHub Release公開が完了しました。

```text
released version: 0.5.0
v0.5.0 parent issue: #76
v0.5.0 release hardening PR: #94
v0.5.0 release documentation sync PR: #95
v0.5.0 release target commit: bf8176da0d992152fb116ca0c45177904d1aa61c
tag: v0.5.0
release: https://github.com/nkkmd/lingonberry/releases/tag/v0.5.0
v0.6.0 parent issue: #97
current release work: protocol contract and conformance
publication state: v0.5.0 released / v0.6.0 in progress
```

Latest published releaseはv0.5.0です。

## 2. v0.5.0で実装済み

| 項目 | 状態 |
|---|---|
| versioned publish ingestion contract | 実装・公開済み |
| CLI／HTTP共通ingestion orchestrator | 実装・公開済み |
| deterministic duplicate／conflict classification | 実装・公開済み |
| versioned object retrieval contract | 実装・公開済み |
| versioned basic query contract | 実装・公開済み |
| deterministic index generation and content digest | 実装・公開済み |
| rebuild／verification／atomic checkpoint | 実装・公開済み |
| checkpoint-driven catch-up | 実装・公開済み |
| corrupt／unsupported／ambiguous state fail-closed | 実装・公開済み |
| restart／recovery／ambiguity smoke coverage | 実装・公開済み |
| workspace package version 0.5.0 | 公開済み |
| release checklist／release notes／CHANGELOG | 公開済み |

## 3. v0.6.0で進行中

v0.6.0は、Rust実装内部の挙動を第三者が再実装できる外部protocol contractとして固定します。

着手済み：

- parent issue #97を作成
- `docs/protocols/PROTOCOL_CONTRACT.md`で外部契約の境界を定義
- `docs/protocols/VERSIONING_AND_COMPATIBILITY.md`でversion軸を分離
- `conformance/manifest.v1.json`でfixture registryをversion化
- `conformance/run.mjs`で依存なしのJavaScript reference runnerを追加
- 既存canonicalization fixtureとidentity-key v2 fixtureを外部runnerから検証

次の実装単位：

1. digest／signature targetの現行実装inventoryとgolden fixture化
2. protocol／schema／API／storage／journal／proof versionの実コード対応表
3. invalid／boundary／timestamp／legacy fixtureの追加
4. producer／consumer conformance結果形式の分離
5. 非Rust最小producerから実publish経路へのcross-implementation test

## 4. Index lifecycle model

Canonical storageを正本とし、indexは検証・再構築可能な派生状態として扱います。

- generationはcanonical ID集合とrecord contentから決定的に生成する
- checkpointはconsistentなrebuild resultからのみatomicに保存する
- missingまたはstale checkpointはcatch-up可能
- corrupt、unsupported、ambiguous checkpoint／index stateは自動上書きしない
- contradictory stateを成功扱いしない

## 5. End-to-end保証

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

## 6. Compatibility

- v0.4.0までのquarantine／backup／replacement／cleanup安全性を維持
- canonical storageとarchive／immutable evidenceをindex lifecycleから書き換えない
- File／SQLite backendでduplicate／conflictとindex lifecycleのparityを検証
- protocol、schema、canonicalization、identity、signature、API、storage、journal、proofのversionを独立軸として扱う
- unknown／unsupported versionを既知versionへfallbackしない
- multi-node consistency、vector search、AI integrationはv0.6.0の非スコープ

## 7. Release gate

v0.5.0は完了済みです。

v0.6.0のrelease gate：

- [ ] normative protocol specificationがcanonical envelopeとserializationを再実装可能な粒度で固定されている
- [ ] version軸とcompatibility matrixが公開されている
- [ ] valid、invalid、boundary、signature、digest、conflict、legacy fixtureが固定されている
- [ ] standalone conformance suiteがCIで成功する
- [ ] Rustと非Rust実装が同じcanonical bytes、identifier、digest、signature targetを再現する
- [ ] 非Rust最小clientが生成したobjectをLingonberryが受理する
- [ ] release checklist／release notes／CHANGELOG／package versionが同期する

## 8. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. conflict時に既存canonical recordを上書きしない
3. canonical storage commit後のindex failureを保存失敗へ書き換えない
4. corruptionとI/O errorをnot-foundやsuccessへ変換しない
5. inconsistent index resultからcheckpointを更新しない
6. corrupt／unsupported／ambiguous stateを自動修復しない
7. archive segmentとimmutable evidence ledgerを変更しない
8. same-host lockをdistributed lockとして扱わない
9. metricsへpath、identifier、digest、record ID、free-form errorを出さない
10. canonical bytes、digest target、signature targetを同一rule version内で変更しない
11. unknown protocol／schema／identity／signature versionをfallbackで受理しない
12. fixtureと実装が不一致の場合にfixtureを自動更新して成功扱いしない
