# Multi-node Discovery and Topology

**Status: draft** | **Last updated: 2026-06-21**

## 目的

この文書は、Lingonberry の複数ノード運用における discovery と topology の運用正本を定義します。  
Phase 11 の Issue 11.1 に対応し、中央 registry に依存しない前提で、relay と storage node をどう見つけ、どう同一運用単位として扱うかを固定します。

## 範囲

この文書で扱うのは次です。

- node discovery
- node topology
- 参照補助手段
- ノード種別ごとの役割差
- 複数ノード運用での見つけ方の原則

この文書で扱わないものは次です。

- node 間同期の具体契約
- conflict resolution
- capacity placement の詳細
- profile 固有の routing rule
- authn/authz の最終方針

## 1. 基本方針

- discovery は protocol semantic ではない
- topology は carrier 実装や運用配置の問題であり、knowledge object の意味そのものではない
- 中央 registry は前提にしない
- 必要なら補助手段として signed manifest、capability endpoint、relay discovery、indexer cache を使う
- 1 つの topology は単一の実装だけでなく、複数の relay / storage node の組み合わせを表せる
- node の役割は、carrier や運用上の責務で説明する

## 2. 用語

### 2.1 node discovery

運用者または client が、ある node の存在、役割、接続先、能力を見つけることです。  
ここでの discovery は、知識 object の semantic lookup ではなく、運用上の到達可能性の確認です。

### 2.2 topology

複数 node の関係の取り方です。  
少なくとも、どの node が publish 入口で、どの node が保存、どの node が replay、どの node が補助 discovery を担うかを表します。

### 2.3 node group

同じ運用意図の下で扱う node のまとまりです。  
node group は必ずしも物理的なクラスタ実装を意味しません。

### 2.4 discovery surface

node が自分の存在や capability を外へ示す面です。  
HTTP の capability endpoint、署名付き manifest、well-known endpoint、relay 上の discovery endpoint などが該当します。

## 3. Node の種別

### 3.1 relay

relay は、publish の入口、wire-level validation、routing、subscription の配信を担います。  
複数 relay がある場合は、同じ node group の中で役割分担してよいですが、semantic model は変えません。

### 3.2 storage node

storage node は、raw log、canonical catalog、replay metadata を保持し、replay と retrieve を支えます。  
複数 storage node がある場合は、保存の冗長化や再構成のために組み合わせて扱えます。

### 3.3 archive node

archive node は、export された archive を保持し、必要に応じて再投入や replay の起点になります。  
archive は backup の代替ではなく、持ち運び可能な carrier 表現です。

### 3.4 gateway relay

gateway relay は、carrier 間の差分を橋渡しする relay です。  
ただし semantic translation は行わず、framing / routing / capability の差を運用上つなぐ役割に閉じます。

## 4. discovery の補助手段

### 4.1 signed manifest

signed manifest は、node が公開する役割、識別子、参照先を検証可能に示すための補助手段です。  
中央 registry の代わりではなく、自己記述的な discovery の材料として使います。

### 4.2 capability endpoint

capability endpoint は、HTTP carrier などが公開する機能一覧です。  
node discovery では、単に「そこに到達できる」だけでなく、「何ができるか」を知るために使います。

### 4.3 relay discovery

relay discovery は、relay が自分の接続情報や役割を示すための面です。  
public relay、curated relay、archive relay、gateway relay の区別を補助できます。

### 4.4 indexer cache

indexer cache は、既知の node 情報を補助的に保持するためのキャッシュです。  
source of truth にはせず、失効や更新がありうる前提で扱います。

## 5. topology の最小表現

Phase 11 の最小 topology は、次の 4 つの観点で表せる必要があります。

- entry point
- storage target
- replay target
- discovery helper

例:

- `public relay` が entry point になる
- `storage node` が storage target になる
- `archive node` が replay target になる
- `signed manifest` と `capability endpoint` が discovery helper になる

この表現で十分な理由は、初期の複数ノード運用では、複雑なクラスタ管理よりも、役割の分離と到達可能性の説明が先に必要だからです。

## 6. 役割差の原則

- `public relay` は公開 publish と retrieval の入口として扱う
- `curated relay` は特定 object type や policy に寄せてよい
- `archive relay` は長期保管と replay に寄せてよい
- `gateway relay` は carrier 差分の橋渡しに寄せてよい
- `storage node` は保存と再構成に寄せる
- `archive node` は持ち運び可能な replay 単位に寄せる

役割差は運用語彙として扱い、knowledge object の semantic type と混同しません。

## 7. 期待する運用フロー

1. operator は node group ごとの role を決める
2. node は signed manifest か capability endpoint で自分を名乗る
3. client は discovery helper をたどって接続先を見つける
4. publish や replay が必要な場合は、role に応じた node に接続する
5. 失敗時は node group の中で代替候補を探す

## 8. 判定ルール

- 中央 registry に依存する設計は採用しない
- discovery 情報が古い可能性は常にある
- discovery できたからといって semantic correctness が保証されるわけではない
- node の役割は capability によって確認し、名前だけで決めない
- topology の変更は protocol semantic の変更として扱わない

## 9. Phase 11 との関係

この文書は Phase 11 の Issue 11.1 に対応します。  
Issue 11.2 以降では、この discovery / topology を前提に、node 間同期、競合解決、capacity 分散、runbook 反映を詰めます。

## 10. 関連

- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](../roadmap/OPERATIONAL_READINESS_BACKLOG.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
