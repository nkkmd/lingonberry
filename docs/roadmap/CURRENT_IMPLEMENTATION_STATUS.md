# 現在の実装状況

**Status: v0.6.0 in progress** | **Last updated: 2026-07-20**

この文書は、Lingonberryの実装作業を中断・再開するときの引き継ぎ用正本です。

## 1. Release state

```text
released version: 0.5.0
v0.6.0 parent issue: #97
v0.6.0 foundation PR: #98
v0.6.0 working branch: agent/v0.6.0-protocol-contract-foundation
publication state: v0.6.0 implementation in progress
```

Latest published releaseはv0.5.0です。

## 2. v0.6.0で進行中

| 項目 | 状態 |
|---|---|
| external protocol contract | PR #98で追加済み |
| independent version axes／compatibility matrix | PR #98で追加済み |
| versioned conformance manifest／runner | PR #98で追加済み |
| canonicalization／identity v1・v2 fixtures | PR #98で追加済み |
| HTTP publish signature fixtures | PR #98で追加済み |
| index generation digest／timestamp fixtures | PR #98で追加済み |
| producer／consumer／internal suite separation | PR #98で追加済み |
| standalone JavaScript producer → real HTTP publish | PR #98で追加済み |
| conformance manifest integrity check | PR #98で追加済み |
| relation／lineage identity fixture | PR #98で追加済み |
| dedicated append-only transition object decision | B案で確定 |
| transition schema／identity rule | PR #98で追加済み |
| replace／withdraw valid・invalid fixtures | PR #98で追加済み |
| relay transition acceptance／effective-view projection | authorization判断待ち |
| release checklist／CHANGELOG／version update | 未着手 |

## 3. v0.6.0 fixed contract

- canonical JSON rule: `lb.canonical.json.v1`
- knowledge identity rules: `lb.identity.key.v1`、`lb.identity.key.v2`
- transition identity rule: `lb.transition.identity.v1`
- HTTP publish signature rule: `lb.http.publish.signature.v1`
- timestamp rule: `lb.timestamp.rfc3339.utc.v1`
- index generation rule: `lb.index.generation.v1`（内部派生状態。暗号用途ではない）
- replacement／withdrawalは元objectを変更しない専用append-only transition objectで表現する
- replaceは`replacementId`必須、withdrawは`replacementId`禁止
- transitionは独立したID、identity、provenance、rawRef、publisher署名を持つ
- structural validityとauthorizationを分離する
- unknown／unsupported versionは既知versionへfallbackしない

## 4. External producer guarantee

CIはJavaScript producerから実relay HTTP publishまでをRust内部APIなしで検証します。

## 5. Next implementation order

1. transition issuer authorization modelを決定する
2. authorized／unauthorized／unknown-authority fixtureを追加する
3. relayでtransition objectのvalidate／append-only storeを有効化する
4. effective-view projectionとconflicting transition classificationを実装する
5. compatibility matrixを完成させる
6. v0.6.0 release checklist／CHANGELOG／version更新へ進む

## 6. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. transition受理時もtarget objectを変更・削除しない
3. unauthorized transitionをeffective viewへ適用しない
4. conflict時に既存canonical recordを上書きしない
5. canonical storage commit後のindex failureを保存失敗へ書き換えない
6. corruptionとI/O errorをnot-foundやsuccessへ変換しない
7. inconsistent index resultからcheckpointを更新しない
8. corrupt／unsupported／ambiguous stateを自動修復しない
9. archive segmentとimmutable evidence ledgerを変更しない
10. canonicalization、digest、signature対象bytesを暗黙に変更しない
11. unknown ruleを既知versionとして解釈しない
12. fixtureと実装が不一致の場合、fixtureを自動更新して成功扱いしない
