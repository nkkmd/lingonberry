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

## 2. v0.6.0で進行中

| 項目 | 状態 |
|---|---|
| external protocol contract／independent version axes | PR #98で追加済み |
| conformance manifest／runner／integrity check | PR #98で追加済み |
| canonicalization／identity／signature／digest／timestamp fixtures | PR #98で追加済み |
| standalone JavaScript producer → real HTTP publish | PR #98で追加済み |
| dedicated append-only Transition Object | PR #98で追加済み |
| transition identity `lb.transition.identity.v1` | PR #98で追加済み |
| transition authority `lb.transition.authority.v1` | PR #98で追加済み |
| authority authorized／unauthorized／unknown fixtures | PR #98で追加済み |
| transition supersession `lb.transition.supersession.v1` | PR #98で追加済み |
| single head／parallel ambiguity／explicit supersession fixtures | PR #98で追加済み |
| relay transition append-only storage／effective-view projection | 未着手 |
| release checklist／CHANGELOG／version update | 未着手 |

## 3. Fixed transition contract

- 元Knowledge Objectは変更・削除しない
- replacement／withdrawalは専用Transition Objectとしてappend-only保存する
- structurally validで署名済みのtransitionはauthorityにかかわらず保持する
- authorityは`authorized`／`unauthorized`／`unknown`として派生判定する
- effective viewへ適用できるのはauthorized transitionのみ
- 複数authorized transitionをtimestampやID順で自動解決しない
- 明示的な`supersedesTransitionId`がない複数headは`ambiguous`としてfail closedにする
- `ambiguous`／`invalid-transition-graph`では元objectを隠さず、replacementを選択しない
- supersession fieldはtransition identity basisに含める

## 4. Next implementation order

1. forkした複数headを解決するsupersession表現を決定する
2. 決定したgraph modelのcycle／missing parent／cross-target fixtureを追加する
3. relayでtransition validate／append-only storeを有効化する
4. authority classificationとeffective-view projectionを実装する
5. compatibility matrixを完成させる
6. v0.6.0 release checklist／CHANGELOG／version更新へ進む

## 5. 絶対に崩さない安全性ルール

1. validation未通過objectをcanonical storageへ保存しない
2. transition受理時もtarget objectを変更・削除しない
3. unauthorized／unknown transitionをeffective viewへ適用しない
4. ambiguous transition graphから勝者を推測しない
5. conflict時に既存canonical recordを上書きしない
6. canonical storage commit後のindex failureを保存失敗へ書き換えない
7. corruptionとI/O errorをnot-foundやsuccessへ変換しない
8. inconsistent index resultからcheckpointを更新しない
9. corrupt／unsupported／ambiguous stateを自動修復しない
10. canonicalization、digest、signature対象bytesを暗黙に変更しない
11. unknown ruleを既知versionとして解釈しない
12. fixtureと実装が不一致の場合、fixtureを自動更新して成功扱いしない
