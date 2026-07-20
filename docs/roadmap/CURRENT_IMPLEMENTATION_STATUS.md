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
| transition identity／authority／supersession | PR #98で追加済み |
| multi-parent atomic fork merge／parent-set normalization | PR #98で追加済み |
| cycle／missing／cross-target／unauthorized-parent fixtures | PR #98で追加済み |
| bounded ASCII protocol identifier rule | PR #98で追加済み |
| dedicated `POST /v1/transitions` contract／route isolation | PR #98で追加済み |
| orphan transition rule `lb.transition.orphan.v1` | PR #98で追加済み |
| durable re-evaluation queue `lb.transition.reevaluation.queue.v1` | PR #98で追加済み |
| target-scoped coalescing `lb.transition.reevaluation.coalescing.v1` | PR #98で追加済み |
| enqueue recovery／retry／stale-generation fixtures | PR #98で追加済み |
| relay transition append-only storage／effective-view projection | 未着手 |
| release checklist／CHANGELOG／version update | 未着手 |

## 3. Fixed transition, identifier, HTTP, and queue contract

- 元Knowledge Objectは変更・削除しない
- replacement／withdrawalは専用Transition Objectとしてappend-only保存する
- structurally validで署名済みのtransitionはauthorityにかかわらず保持する
- authorityは`authorized`／`unauthorized`／`unknown`として派生判定する
- effective viewへ適用できるのはauthorized transitionのみ
- 複数authorized transitionをtimestampやID順で自動解決しない
- `supersedesTransitionIds`で複数authorized headを原子的に解消する
- partial merge、duplicate parent、self-reference、missing parent、cross-target parent、unauthorized parent、cycleはfail closedにする
- parent ID配列はidentity derivation時だけASCII byte ascendingでsortする
- protocol IDはASCII-safe grammarとbyte上限を適用する
- Knowledge Objectは`POST /v1/objects`、Transition Objectは`POST /v1/transitions`へ送る
- transition requestのtop-level payload fieldは`transition`とし、route mismatchやmixed envelopeを拒否する
- target不在のvalid signed transitionはorphan evidenceとしてappend-only保存する
- orphan中は`targetStatus=missing`、authorityは`unknown/target-unavailable`、effective view非適用とする
- target到着後はderived stateだけを再評価し、transition bytes／identity／signature evidenceを変更しない
- target Knowledge Objectを先にcanonical storageへcommitし、再評価はdurable queueで非同期処理する
- queue processingはat-least-once、workerはidempotent、checkpointはcurrent snapshotのdurable result後だけ進める
- re-evaluationのlogical subjectはTransition Objectではなく`targetId`とする
- targetごとにcurrent logical intentを最大1件とし、pending workは最新evidence generationへcoalesceする
- running workerのgenerationがcurrent generationと異なる場合、そのresultはstaleとして破棄する
- stale workerはderived viewやcheckpointを更新せず、新しいgenerationのpending intentを消さない
- reconciliationはcurrent evidence generationとderived checkpointの不一致からmissing workを再生成する

## 4. Next implementation order

1. target evidence generationの構成方法を決定する
2. 決定したgeneration digest／component ordering／collision semanticsをfixture化する
3. relayで`POST /v1/transitions`のvalidate／signature verify／append-only storeを有効化する
4. orphan index、durable queue、authority classification／effective-view projectionを実装する
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
13. target到着後のre-evaluation失敗をtarget Knowledge Objectの保存失敗へ書き換えない
14. in-memory taskだけをre-evaluationの唯一の記録にしない
15. stale workerが新しいderived checkpointを上書きしない
16. completed older generationがnewer pending intentを削除しない
