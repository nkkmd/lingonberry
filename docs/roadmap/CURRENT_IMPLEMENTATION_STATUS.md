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
| multi-parent atomic fork merge／graph fail-closed fixtures | PR #98で追加済み |
| bounded ASCII protocol identifiers | PR #98で追加済み |
| dedicated `POST /v1/transitions` contract | PR #98で追加済み |
| orphan transition rule `lb.transition.orphan.v1` | PR #98で追加済み |
| durable re-evaluation queue／target-scoped coalescing | PR #98で追加済み |
| deterministic evidence generation `lb.transition.evidence-generation.v1` | PR #98で追加済み |
| supported／unsupported／corrupt／unreadable marker vectors | PR #98で追加済み |
| last-known-good effective view／stale read API | PR #98で追加済み |
| stable public diagnostics `lb.http.effective-view.diagnostics.v1` | PR #98で追加済み |
| bounded generation-pinned diagnostic pagination | PR #98で追加済み |
| bounded derived diagnostic retention | PR #98で追加済み |
| relay transition append-only storage／effective-view projection | 未着手 |
| release checklist／CHANGELOG／version update | 未着手 |

## 3. Fixed transition, identifier, HTTP, queue, generation, and read contract

- 元Knowledge Objectは変更・削除しない
- replacement／withdrawalは専用Transition Objectとしてappend-only保存する
- structurally validで署名済みのtransitionはauthorityにかかわらず保持する
- authorityは`authorized`／`unauthorized`／`unknown`として派生判定する
- effective viewへ適用できるのはauthorized transitionのみ
- 複数authorized transitionをtimestampやID順で自動解決しない
- `supersedesTransitionIds`で複数のauthorized headを原子的に解消する
- duplicate parent、self-reference、missing parent、cross-target、unauthorized parent、cycleはfail closedにする
- parent ID配列はidentity derivation時だけASCII byte ascendingでsortする
- protocol IDはbounded ASCII grammarとし、暗黙変換しない
- Knowledge Objectは`POST /v1/objects`、Transition Objectは`POST /v1/transitions`へ送る
- target不在のvalid signed transitionはorphan evidenceとしてappend-only保存する
- target到着後はderived stateだけを再評価し、transition bytes／identity／signature evidenceを変更しない
- target Knowledge Objectを先にcanonical storageへcommitし、再評価はdurable queueで非同期処理する
- queue processingはat-least-once、workerはidempotentとする
- re-evaluationのlogical subjectは`targetId`とし、current intentは最新evidence generationへcoalesceする
- stale workerはderived checkpointを更新できない
- target evidence generationはcanonical evidence basisのSHA-256から決定的に導出する
- evidenceはkind順、ASCII ID順、classification順、digest順でsortする
- exact duplicate carrierはgenerationを変えない
- 同一kind／IDで異なるclassificationまたはdigestが存在する場合はgeneration constructionをfail closedにする
- `unsupported`／`corrupt`／`unreadable` evidenceも分類付きmarkerとしてgeneration basisへ含める
- unusable markerはimmutable carrier digestを必須とし、trusted digestも読めるbytesもない場合はgenerationを捏造しない
- unusable evidenceを含むsnapshotは`incomplete`、authorityは`unknown`、effective view非適用とする
- evidenceがsupportedへ変化・明示的に修復された場合はgenerationを変更して再評価する
- incomplete observation時はlast-known-good semantic viewを維持し、`freshness=stale`として返す
- semantic checkpointとobservation checkpointを分離する
- `GET /v1/effective-objects/{targetId}`はstale viewでも`200 OK`を返し、bodyを正本とする
- public diagnosticsはstable reason codeとprotocol identifierだけを返し、storage path、row ID、stack trace、parser exception、worker IDを公開しない
- diagnosticはkind、ASCII evidence ID、classification、reason codeの順で決定的にsortする
- exact duplicate diagnosticはcollapseし、同一kind／IDの競合diagnosticを暗黙選択しない
- 通常のeffective-view responseで返すdiagnosticは先頭20件までとする
- `diagnosticSummary.total`／`byClassification`は対象generationの完全集合を正確に表す
- 完全一覧は`GET /v1/effective-objects/{targetId}/diagnostics`で取得し、generationを必須にする
- diagnostic paginationのdefault／maximum limitは100とする
- cursorはtarget／generationへbindし、不透明でrelay内部identifierを公開しない
- pagination中に異なるobservation generationを混在させない
- canonical evidenceとimmutable carrier digestはderived snapshot expirationを理由に削除しない
- current observation generationとsemantic checkpoint generationは常にderived snapshot retention対象とする
- unexpired cursor leaseが参照するgenerationは保持する
- cursor leaseは有限であり、永久retention pinにしない
- 保持されていないgenerationのpaginationを別generationへ暗黙切替しない

## 4. Next implementation order

1. 非保護のrecent generation retentionを件数基準、時間基準、またはhybridのどれにするか決定する
2. 決定したretention bound／cursor lease expiry／garbage-collection orderをfixture化する
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
16. incomplete snapshotからauthorized semantic effectを生成しない
17. unreadable evidenceにtrusted carrier digestがない状態で完全なgenerationを発行しない
18. stale viewをcurrentとして返さない
19. public diagnosticsへrelay内部情報や非安定なexception textを漏らさない
20. diagnostic truncationを隠さない
21. pagination中に別generationのdiagnosticを混在させない
22. cursorへstorage path、row ID、ingestion sequenceを埋め込まない
23. derived snapshot garbage collectionでcanonical evidenceを削除しない
24. current observation／semantic checkpoint／active cursor参照generationを回収しない
25. cursor leaseを無期限retentionとして扱わない
