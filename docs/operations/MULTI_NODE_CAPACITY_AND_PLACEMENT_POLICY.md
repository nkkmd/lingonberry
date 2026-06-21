# Multi-node Capacity and Placement Policy

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、Lingonberry の複数ノード運用における capacity 分散と placement の運用正本を定義します。  
Phase 11 の Issue 11.4 に対応し、どの node に何を置くか、どの指標を見て増減や移送を判断するかを固定します。

## 範囲

この文書で扱うのは次です。

- role ごとの capacity 観点
- placement の判断材料
- 最小の capacity 指標
- 1 ノード障害時の継続方針
- capacity 逼迫時の優先順位

この文書で扱わないものは次です。

- discovery の詳細
- sync の具体契約
- conflict resolution
- profile 固有の routing rule
- semantic clustering

## 1. 基本方針

- capacity 分散は protocol semantic ではない
- placement は object の意味ではなく、運用上の負荷と責務で決める
- 自動最適化よりも、説明可能で再現可能な配置を優先する
- 余裕がある node に何でも寄せない
- 1 台障害でも全体が止まらない構成を優先する
- public / curated / archive / gateway の役割差を保つ

## 2. 役割ごとの容量観点

### 2.1 public relay

public relay は、publish と retrieval の入口です。  
主に見るべき容量は、受付負荷と配信負荷です。

見る観点:

- concurrent request 数
- publish rate
- validation failure rate
- rate limit hit
- outbound delivery backlog

### 2.2 curated relay

curated relay は、特定 object type や policy に絞った入口です。  
public relay より狭いが、条件付きの配信やチェックが増えることがあります。

見る観点:

- filtered subscription count
- curation rule evaluation cost
- queue depth
- rejection rate

### 2.3 storage node

storage node は、raw log、canonical catalog、replay metadata を保持します。  
主に見るべき容量は、保存領域と replay 負荷です。

見る観点:

- dataDir usage
- raw log growth rate
- canonical catalog growth rate
- replay duration
- append latency
- retrieve latency

### 2.4 archive node

archive node は、export された bundle を保管し、再投入や replay の起点になります。  
主に見るべき容量は、保管量とインデックスの維持コストです。

見る観点:

- archive bundle count
- archive total size
- replay availability
- import latency
- manifest integrity

### 2.5 gateway relay

gateway relay は、carrier 間の差分を橋渡しします。  
主に見るべき容量は、接続先の種類数と橋渡し負荷です。

見る観点:

- connected carrier count
- translation-free routing load
- capability lookup cost
- cross-carrier backlog

## 3. Placement の判断材料

placement は、次の順で判断します。

1. 役割要件
2. 可用性要件
3. 保存要件
4. replay 要件
5. 接続要件
6. 運用者の保守しやすさ

判断で見る項目:

- CPU とメモリの余裕
- disk usage
- network egress / ingress
- replay backlog
- subscription backlog
- archive 保管余力
- 障害時の代替経路

## 4. 最小の capacity 指標

Phase 11 で最低限見るべき指標は次です。

- node count by role
- dataDir usage
- raw log growth
- replay duration
- replay failure count
- subscription backlog
- publish backlog
- archive bundle count
- archive total size
- capacity threshold hit count

これらは高カーディナリティにしすぎず、運用判断に使える粒度で十分です。

## 5. 逼迫時の優先順位

capacity が逼迫したときは、次の順で優先します。

1. 既存 object の replay 可能性を壊さない
2. raw log と provenance を失わない
3. public relay の入口を保つ
4. storage node の保存と再構成を保つ
5. archive の退避先を確保する
6. gateway relay の橋渡しを後回しにする

この順は、意味のある object を失うことを最初に避けるためのものです。

## 6. 障害時の継続方針

### 6.1 public relay 障害

public relay が落ちた場合は、別の public relay または curated relay が入口を引き継げるようにします。  
少なくとも discovery と capability が見つかる状態を保ちます。

### 6.2 storage node 障害

storage node が落ちた場合は、別 storage node または archive から replay できることを優先します。  
raw log と replay metadata を起点に再構成できることを重視します。

### 6.3 archive node 障害

archive node が落ちた場合は、別保管先に bundle を持てるようにします。  
archive 自体の可用性よりも、再投入可能性を優先します。

### 6.4 gateway relay 障害

gateway relay が落ちた場合は、各 carrier を直接使える経路を残します。  
gateway は補助経路なので、単独で全体を支えないようにします。

## 7. 配置原則

- 役割を混ぜすぎない
- public relay と storage node は、可能なら別 node に分ける
- archive は storage とは別の retention 前提で扱う
- gateway は必要な場合だけ置く
- 小さい構成では役割を兼務してもよいが、兼務を前提に仕様を広げない

## 8. 監視との接続

この文書の指標は [Observability](./OBSERVABILITY.md) と対応させます。  
特に次のイベントや確認順とつながります。

- `config_resolved`
- `readiness_checked`
- `replay_completed`
- `append_completed`
- `retrieve_completed`

## 9. Phase 11 との関係

この文書は Phase 11 の Issue 11.4 に対応します。  
Issue 11.5 では、この placement と capacity の判断順を runbook に落とします。

## 10. 関連

- [Multi-node Discovery and Topology](./MULTI_NODE_DISCOVERY_AND_TOPOLOGY.md)
- [Multi-node Sync Contract](./MULTI_NODE_SYNC_CONTRACT.md)
- [Multi-node Conflict Policy](./MULTI_NODE_CONFLICT_POLICY.md)
- [Observability](./OBSERVABILITY.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
- [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
