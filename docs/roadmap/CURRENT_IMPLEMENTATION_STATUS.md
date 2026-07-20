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
| HTTP publish signature rule／valid・invalid・malformed fixtures | PR #98で追加済み |
| index generation digest rule／fixture | PR #98で追加済み |
| timestamp semantics／fixtures | PR #98で追加済み |
| producer／consumer／internal suite separation | PR #98で追加済み |
| standalone JavaScript minimal producer | PR #98で追加済み |
| non-Rust producer → real HTTP publish integration | PR #98で追加済み |
| conformance manifest integrity check | PR #98で追加済み |
| relation／lineage identity fixture | PR #98で追加済み |
| replacement／withdrawal schema and fixtures | 設計判断待ち |
| release checklist／CHANGELOG／version update | 未着手 |

## 3. v0.6.0 fixed contract

- canonical JSON rule: `lb.canonical.json.v1`
- identity rules: `lb.identity.key.v1`、`lb.identity.key.v2`
- HTTP publish signature rule: `lb.http.publish.signature.v1`
- timestamp rule: `lb.timestamp.rfc3339.utc.v1`
- index generation rule: `lb.index.generation.v1`（内部派生状態。暗号用途ではない）
- signature target: `publisher.signature`だけを除いたrequest全体のcanonical UTF-8 bytes
- signature algorithm: Ed25519、pre-hashなし
- public key: raw 32 bytesのlowercase hex
- signature: raw 64 bytesのlowercase hex
- timestampはcanonicalizationで変換せず、producerはUTC `Z`形式を使用する
- relationとlineageは別概念として保持し、identity v2のsemantic basisに含める
- protocol／schema／canonicalization／identity／signature／API／storage／journal／proof versionを独立軸として扱う
- unknown／unsupported versionは既知versionへfallbackしない

## 4. External producer guarantee

CIは次の経路をRust内部APIを使わずに検証します。

```text
JavaScript producer
→ Knowledge Object生成
→ canonical signature target生成
→ Ed25519署名
→ HTTP publish request出力
→ lingonberry-relay実binary
→ POST /v1/objects
→ validate
→ signature verify
→ canonical storage
→ LB_OBJECT_STORED
```

## 5. Next implementation order

1. replacement／withdrawalを既存knowledge objectのstatus／lineageで表現するか、専用transition objectを導入するか決定する
2. 決定したschemaのvalid／invalid／conflict fixtureを追加する
3. compatibility matrixを完成させる
4. v0.6.0 release checklist／CHANGELOG／version更新へ進む

## 6. 絶対に崩さない安全性ルール

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
