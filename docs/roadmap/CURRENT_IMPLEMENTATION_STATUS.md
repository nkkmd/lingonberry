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
| dedicated append-only transition object | B案で確定・追加済み |
| transition schema／identity rule | PR #98で追加済み |
| replace／withdraw valid・invalid fixtures | PR #98で追加済み |
| transition authority classification | C案で確定・fixture追加済み |
| relay transition acceptance／effective-view projection | conflict resolution判断待ち |
| release checklist／CHANGELOG／version update | 未着手 |

## 3. v0.6.0 fixed contract

- canonical JSON rule: `lb.canonical.json.v1`
- knowledge identity rules: `lb.identity.key.v1`、`lb.identity.key.v2`
- transition identity rule: `lb.transition.identity.v1`
- transition authority rule: `lb.transition.authority.v1`
- HTTP publish signature rule: `lb.http.publish.signature.v1`
- timestamp rule: `lb.timestamp.rfc3339.utc.v1`
- index generation rule: `lb.index.generation.v1`（内部派生状態。暗号用途ではない）
- replacement／withdrawalは元objectを変更しない専用append-only transition objectで表現する
- replaceは`replacementId`必須、withdrawは`replacementId`禁止
- transitionは独立したID、identity、provenance、rawRef、publisher署名を持つ
- structural validity、signature validity、authority、effective-view conflict resolutionを分離する
- structurally validかつ署名済みのtransitionはauthorityにかかわらずappend-onlyで保持する
- `authorized`のみeffective viewへ適用する
- `unauthorized`と`unknown`は保持するがeffective viewへ適用しない
- authority classificationはderived stateであり、追加証拠により再評価できる
- unknown／unsupported versionは既知versionへfallbackしない

## 4. Transition authority model

```text
authorized
→ retain
→ apply to effective view

unauthorized
→ retain as evidence
→ do not apply

unknown
→ retain pending evidence
→ do not apply
→ re-evaluate when authority evidence becomes available
```

元publisherと一致する署名鍵は`original-publisher`としてauthorizedです。検証済みかつ有効期間内の委任は`delegated-publisher`としてauthorizedです。

## 5. Next implementation order

1. 複数のauthorized transitionが同一targetへ競合した場合のresolution ruleを決定する
2. 決定したprecedence／ambiguity fixtureを追加する
3. relayでtransition objectのvalidate／append-only storeを有効化する
4. authority classificationをderived stateとして保存・再評価する
5. effective-view projectionを実装する
6. compatibility matrixを完成させる
7. v0.6.0 release checklist／CHANGELOG／version更新へ進む

## 6. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. transition受理時もtarget objectを変更・削除しない
3. unauthorized／unknown transitionをeffective viewへ適用しない
4. authority不明をunauthorizedまたはauthorizedへ推測変換しない
5. conflict時に既存canonical recordを上書きしない
6. canonical storage commit後のindex failureを保存失敗へ書き換えない
7. corruptionとI/O errorをnot-foundやsuccessへ変換しない
8. inconsistent index resultからcheckpointを更新しない
9. corrupt／unsupported／ambiguous stateを自動修復しない
10. archive segmentとimmutable evidence ledgerを変更しない
11. canonicalization、digest、signature対象bytesを暗黙に変更しない
12. unknown ruleを既知versionとして解釈しない
13. fixtureと実装が不一致の場合、fixtureを自動更新して成功扱いしない
