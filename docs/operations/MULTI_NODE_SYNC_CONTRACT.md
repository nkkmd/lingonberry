# Multi-node Sync Contract

**Status: draft** | **Last updated: 2026-06-22**

## 目的

この文書は、Lingonberry の複数ノード運用における node 間同期の運用正本を定義します。  
Phase 11 の Issue 11.2 に対応し、relay 間、storage node 間、archive との同期を、`subscription`、`replay`、`export / import` に分けて扱います。

## 範囲

この文書で扱うのは次です。

- relay 間の近接同期
- storage node 間の再構成同期
- archive を介した持ち運び同期
- 同期方式ごとの役割分離
- capability による同期可否の説明

この文書で扱わないものは次です。

- conflict resolution の詳細
- capacity placement の詳細
- discovery の詳細
- profile 固有の routing rule
- semantic translation

## 1. 基本方針

- 同期は protocol semantic ではない
- 同期方式ごとに役割を分ける
- 同じ object 群を複数 node に運ぶときも、意味の翻訳はしない
- 失敗時は再試行可能であることを優先する
- 互換境界は capability と version で説明する
- 中央 registry に依存して同期先を決めない

## 2. 同期方式

### 2.1 subscription

`subscription` は、relay が保持する wire object の配信経路です。  
近接同期や継続配信に向きます。

使いどころ:

- relay 間の継続的な追従
- publish 直後の反映
- ある条件に合う object 群の配信

期待する性質:

- append-only である
- 冪等 delivery を目指せる
- object type、author、時間範囲、relation target などで絞り込める
- replay とは別の経路として扱える

### 2.2 replay

`replay` は、保存済みの wire log から canonical state を再構成する経路です。  
storage node の再投入、障害復旧、archive からの再構成に向きます。

使いどころ:

- storage node の再構成
- archive からの復元
- 保存状態の整合性確認

期待する性質:

- 決定的である
- validate / normalize / finalize の順を壊さない
- raw log と provenance を失わない
- 同じ入力から同じ canonical state に着地する

### 2.3 export / import

`export / import` は、node 間で bundle を持ち運ぶ経路です。  
archive carrier と整合する形で、離れた node へ移送するときに使います。

使いどころ:

- オフライン移送
- 長期保管
- node 入れ替え
- 退役後の再投入

期待する性質:

- bundle の境界が明示される
- manifest、wire-log、replay metadata を説明できる
- capability で受け入れ可否を判断できる
- scrub や retention は policy 側で決める

## 3. 役割分離

### 3.1 relay 間

relay 間では、主に `subscription` を使います。  
これは継続配信と近接追従のためです。

relay 間同期では次を守ります。

- subscription を semantic translation に使わない
- carrier identity と delivery order を区別する
- public relay、curated relay、archive relay の役割差を保つ

### 3.2 storage node 間

storage node 間では、主に `replay` と `export / import` を使います。  
保存状態の再構成や移送に向いています。

storage node 間同期では次を守ります。

- raw log を正本として扱う
- canonical catalog は派生物として扱う
- replay を壊す同期を採用しない

### 3.3 archive との間

archive との同期では、主に `export / import` を使います。  
archive は replay 可能性を保った持ち運び形式として扱います。

archive との同期では次を守ります。

- archive version と protocol version を明示する
- import 前に manifest を確認する
- import 後に replay で canonical state を確認する

## 4. 同期可否の説明

同期可否は、node 名ではなく capability で説明します。  
最低限、次が分かる必要があります。

- どの carrier を受けられるか
- どの schema version に対応するか
- replay が必要かどうか
- archive 互換があるか
- subscription が使えるか

capability が曖昧な場合は、推測で同期を始めず fail closed にします。

## 5. 失敗時の扱い

- subscription 失敗は配信遅延として扱う
- replay 失敗は保存状態か input bundle の不整合として扱う
- export / import 失敗は manifest、wire-log、replay metadata のどこで崩れたかを順に切り分ける
- 同期失敗は semantic conflict と即断しない
- 同期先が不明な場合は discovery と capability を見直す

## 6. 切り分け順

1. まず capability を確認する
2. 次に carrier kind を確認する
3. 次に version と bundle 形式を確認する
4. 次に subscription / replay / export / import のどれで失敗したかを確認する
5. 最後に raw log、canonical catalog、replay metadata、manifest の順で見る

## 7. Phase 11 との関係

この文書は Phase 11 の Issue 11.2 に対応します。  
Issue 11.3 以降では、同期で起こりうる duplicate や conflict を `canonical identity`、`provenance`、`lineage` の側から扱います。

## 8. 関連

- [Multi-node Discovery and Topology](./MULTI_NODE_DISCOVERY_AND_TOPOLOGY.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Multi-node Conflict Policy](./MULTI_NODE_CONFLICT_POLICY.md)
- [Migration and Schema Versioning](./MIGRATION_AND_SCHEMA_VERSIONING.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
