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
| transition supersession `lb.transition.supersession.v1` | PR #98で追加済み |
| multi-parent atomic fork merge | PR #98で追加済み |
| parent-set identity normalization | PR #98で追加済み |
| cycle／missing／cross-target／unauthorized-parent fixtures | PR #98で追加済み |
| protocol identifier rule `lb.protocol.id.ascii.v1` | PR #98で追加済み |
| ASCII／Unicode／length-boundary ID fixtures | PR #98で追加済み |
| relay transition append-only storage／effective-view projection | 未着手 |
| release checklist／CHANGELOG／version update | 未着手 |

## 3. Fixed transition and identifier contract

- 元Knowledge Objectは変更・削除しない
- replacement／withdrawalは専用Transition Objectとしてappend-only保存する
- structurally validで署名済みのtransitionはauthorityにかかわらず保持する
- authorityは`authorized`／`unauthorized`／`unknown`として派生判定する
- effective viewへ適用できるのはauthorized transitionのみ
- 複数authorized transitionをtimestampやID順で自動解決しない
- `supersedesTransitionIds`で複数のauthorized headを原子的に解消する
- 全headを列挙しない部分解消は`ambiguous`のままとする
- duplicate parent、self-reference、missing parent、cross-target、unauthorized parent、cycleはfail closedにする
- `ambiguous`／`invalid-transition-graph`では元objectを隠さず、replacementを選択しない
- parent ID配列はidentity derivation時だけASCII byte ascendingでsortする
- stored Transition Objectや一般のcanonical JSON配列順は書き換えない
- duplicate parentは正規化で除去せずinvalidとする
- protocol IDは`A-Z a-z 0-9 . _ ~ : -`のみを許可する
- object／transition／identity keyのprefixを固定し、IDはcase-sensitiveとする
- object IDとtransition IDはprefix込み最大255 ASCII bytesとする
- identity keyはprefix込み最大512 ASCII bytesとする
- valid IDをtrim、truncate、case-convert、percent-decode、Unicode-normalizeしない
- legacy Unicode／over-limit IDは証拠として保持できるが、v0.6 conforming IDとして再発行・parent参照しない

## 4. Next implementation order

1. relay publish APIでKnowledge ObjectとTransition Objectを同一endpointのunionとして扱うか、別endpointへ分離するか決定する
2. 決定したAPI envelope／response code／duplicate・conflict semanticsをfixture化する
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
