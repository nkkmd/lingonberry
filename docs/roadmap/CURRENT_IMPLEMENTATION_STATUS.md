# 現在の実装状況

**Status: v0.6.0 in progress** | **Last updated: 2026-07-20**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

v0.5.0の機能実装、end-to-end smoke scenario、package version更新、release hardening、README／関連index文書の同期、main CI確認、tag、GitHub Release公開が完了しました。

```text
released version: 0.5.0
v0.6.0 parent issue: #97
v0.6.0 foundation PR: #98
v0.6.0 working branch: agent/v0.6.0-protocol-contract-foundation
publication state: v0.6.0 implementation in progress
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

| 項目 | 状態 |
|---|---|
| external protocol contract | PR #98で追加済み |
| independent version axes | PR #98で追加済み |
| initial compatibility matrix | PR #98で追加済み |
| versioned conformance manifest | PR #98で追加済み |
| standalone JavaScript conformance runner | PR #98で追加済み |
| canonicalization golden fixture | Rust／JavaScript共有済み |
| identity-key v2 golden fixture | Rust／JavaScript共有済み |
| HTTP publish signature rule v1 | PR #98で追加済み |
| valid Ed25519 signature golden vector | PR #98で追加済み |
| tampered request signature rejection fixture | PR #98で追加済み |
| digest／timestamp／legacy fixture expansion | 未着手 |
| non-Rust producer integration | 未着手 |

## 4. v0.6.0 fixed contract

現在固定した外部契約は次のとおりです。

- canonical JSON rule: `lb.canonical.json.v1`
- identity rule: `lb.identity.key.v2`
- HTTP publish signature rule: `lb.http.publish.signature.v1`
- signature target: `publisher.signature`だけを除いたrequest全体のcanonical UTF-8 bytes
- signature algorithm: Ed25519、pre-hashなし
- public key: raw 32 bytesのlowercase hex
- signature: raw 64 bytesのlowercase hex
- protocol／schema／canonicalization／identity／signature／API／storage／journal／proof versionを独立軸として扱う
- unknown／unsupported versionは既知versionへfallbackしない

## 5. Index lifecycle model

Canonical storageを正本とし、indexは検証・再構築可能な派生状態として扱います。

- generationはcanonical ID集合とrecord contentから決定的に生成する
- checkpointはconsistentなrebuild resultからのみatomicに保存する
- missingまたはstale checkpointはcatch-up可能
- corrupt、unsupported、ambiguous checkpoint／index stateは自動上書きしない
- contradictory stateを成功扱いしない

## 6. End-to-end保証

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

## 7. Next implementation order

1. canonical object／record content digestのrule inventoryとgolden fixture化
2. malformed public key／signature length／hex boundary fixture
3. timestamp valid／invalid／boundary fixture
4. legacy fixtureとcompatibility classification
5. producer／consumer conformance resultの分離
6. non-Rust minimal producerから実publish経路へのintegration test

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
10. canonicalization、digest、signature対象bytesを暗黙に変更しない
11. unknown protocol／schema／identity／signature ruleを既知versionとして解釈しない
12. fixtureと実装が不一致の場合、fixtureを自動更新して成功扱いしない
