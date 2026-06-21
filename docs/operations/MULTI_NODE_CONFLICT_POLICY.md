# Multi-node Conflict Policy

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、Lingonberry の複数ノード運用における conflict / duplicate / revision の扱いを定義します。  
Phase 11 の Issue 11.3 に対応し、同一 object 群を複数 node で扱うときに、上書きではなく append-only と lineage で扱う前提を固定します。

## 範囲

この文書で扱うのは次です。

- exact duplicate の扱い
- identity collision の扱い
- conflicting re-publish の扱い
- revision / supersession の扱い
- unresolved conflict の保管方法
- operator / application への返し方

この文書で扱わないものは次です。

- node discovery
- node 間同期の具体契約
- capacity placement
- profile 固有の trust rule
- domain-specific merge rule

## 1. 基本方針

- conflict は protocol semantic ではない
- 同一性が揺れる入力を無理に 1 つへ潰さない
- append-only を壊す修正は受け入れない
- provenance と rawRef を失わない
- lineage は修正や派生を表す正規の経路として使う
- 可能な限り exact duplicate は冪等に扱う

## 2. 用語

### 2.1 exact duplicate

同じ canonical identity を持ち、正規化後の内容も一致する object です。  
この場合は、同一内容の再送として扱い、冪等に吸収できます。

### 2.2 conflicting re-publish

同じ canonical identity を持つが、正規化後の内容が異なる object です。  
この場合は、同一 object の単純な再送ではなく conflict として扱います。

### 2.3 identity collision

異なる canonical id なのに、同じ identity key や同じ外部主張に収束してしまう状態です。  
この場合は、どちらかを自動的に勝たせず、検証可能な分岐として保持します。

### 2.4 revision

`revises`、`supersedes`、`derived_from` などで明示された派生関係です。  
revision は conflict ではなく lineage の一部として扱います。

## 3. 判定順

同期や publish の結果を判定するときは、次の順で見るのが基本です。

1. canonical id が同じか
2. normalized payload が同じか
3. identity key が同じか
4. lineage 関係が明示されているか
5. provenance と rawRef が整合しているか

この順を崩すと、単なる再送と semantic collision を混同しやすくなります。

## 4. 取り扱い

### 4.1 exact duplicate

exact duplicate は受け入れます。  
ただし保存は冪等にし、観測上は duplicate として記録してよいです。

期待する性質:

- 既存 object を壊さない
- publish / sync の再試行で重複が増殖しない
- carrier identity の違いだけで semantic を変えない

### 4.2 conflicting re-publish

conflicting re-publish は、自動上書きしません。  
新しい revision を作らない限り、既存 object の semantic を置き換えません。

処理方針:

- 受け入れは拒否または隔離とする
- conflict 記録を残す
- operator または application へ再送理由を返す
- 必要なら新しい revision を作るよう促す

### 4.3 identity collision

identity collision は、自動マージしません。  
`identity claim`、`provenance`、`lineage` を見て、別 object として保持します。

処理方針:

- 両方を保持する
- collision の根拠を記録する
- どちらを canonical と決めるかを運用で決める
- 必要なら application profile 側で追加の検証を行う

### 4.4 revision

revision は conflict ではありません。  
`revises`、`supersedes`、`derived_from` が明示されているなら、派生として扱います。

処理方針:

- lineage graph に接続する
- 旧 object を破壊しない
- 置換ではなく関係として表す

## 5. 受け入れ判定

### Accept

- exact duplicate
- 明示された revision
- provenance と rawRef が整合している object

### Reject

- 同じ canonical id で内容が変わっているが revision がない object
- provenance が破損していて再構成できない object
- identity collision を隠蔽しようとする入力

### Quarantine

- identity collision の疑いがある object
- provenance はあるが、運用上の解決が必要な object
- carrier 側の再送か semantic conflict か判断が要る object

## 6. 返却と観測

同期や publish の結果では、次を分けて返せるとよいです。

- accepted
- duplicate
- conflict
- collision
- deferred

observability では、少なくとも次が追える必要があります。

- duplicate flooding
- identity collision
- conflicting re-publish
- unresolved conflict

## 7. 運用上の原則

- conflict を見たらまず同期契約と version を疑う
- exact duplicate を conflict と混同しない
- revision は運用上の解決策であって、隠れた上書きではない
- unresolved conflict は可視化して残す
- 競合の最終判断は domain truth と profile policy で行い、core semantic に埋め込まない

## 8. Phase 11 との関係

この文書は Phase 11 の Issue 11.3 に対応します。  
Issue 11.4 以降では、この conflict policy を前提に capacity 分散と runbook 反映を行います。

## 9. 関連

- [Multi-node Sync Contract](./MULTI_NODE_SYNC_CONTRACT.md)
- [Multi-node Discovery and Topology](./MULTI_NODE_DISCOVERY_AND_TOPOLOGY.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Migration and Schema Versioning](./MIGRATION_AND_SCHEMA_VERSIONING.md)
- [GLOSSARY](../concepts/GLOSSARY.md)
- [CONCEPT_MODEL](../concepts/CONCEPT_MODEL.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
