# 運用前提メモ

**Status: active** | **Last updated: 2026-06-19**

## 目的

この文書は、[運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md) のフェーズ 0 で固定する運用前提をまとめる、正本メモです。  
以後のフェーズで迷いやすい境界を先に確定し、実装や運用の判断をぶらさないことを目的にします。

## 決定事項

- `knowledge object` は append-only とする
- `wire` と `canonical` は別プロトコルではなく、同じ protocol object の別表現とする
- `carrier` は protocol object を wire 上で運ぶ正規の実装とする
- `relay` と `storage node` は責務を分離する
- core protocol は分野中立を保ち、分野固有の語彙や運用差分は profile / policy 側に寄せる
- secret は設定ファイルに平文で置かず、deployment 側の secret store や注入経路で扱う

## 1. relay と storage node の責務境界

### relay の責務

relay は入口として振る舞います。

- ingress
- schema / framing / carrier identity の validation
- routing
- publish の受け口

relay は次を持ちません。

- 永続化の内部構造
- raw log の保管方式の詳細
- canonical catalog の実装詳細
- domain truth の判定

### storage node の責務

storage node は保持と再構成を担います。

- append
- replay
- retrieve
- export

storage node は次を担います。

- raw log の保持
- canonical store の保持
- replay の再構成
- export / import の基盤

### 境界の含意

- relay は受け口であり、保存方式を知りすぎない
- storage node は保存と再構成を担い、入口の transport 詳細を持ち込まない
- 既存の HTTP publish 経路は、この責務分離に合わせて所属を固定する

## 2. public / private の扱い

フェーズ 0 では、core protocol を public object 前提で固定します。  
private / encrypted object の扱いは、core へ持ち込まず、application profile または policy 拡張で扱います。

この方針により、次を避けます。

- core schema の複雑化
- carrier ごとの差分の肥大化
- 公開範囲の判断を protocol semantic に混ぜること

### 含意

- core は公開可能な知識オブジェクトの共通面に集中する
- secret の扱いは profile / policy の責務に分ける
- secret は profile / policy / deployment の責務に分ける
- carrier 固有の公開範囲は protocol semantic にしない

## 3. 監視対象としないもの

### 監視対象

フェーズ 0 で監視対象にするのは、運用成立に直接効くものだけです。

- relay と storage node の起動可否
- publish の受け付け可否
- append / replay / retrieve の基本健全性
- 明示した carrier contract に対する整合性

### 監視対象としないもの

- 内容の真偽
- domain-specific な妥当性
- profile 固有の trust rule
- UI や表示順序の都合
- federation 全体の完全な可観測性

## 4. core と profile の境界

### core に残すもの

- knowledge object の基本構造
- canonical identity
- provenance
- rawRef
- carrier framing
- validate / normalize / finalize
- replay 可能性

### profile に寄せるもの

- object subtype の使い分け
- relation vocabulary
- context vocabulary
- curation rule
- display / query priority
- profile 固有の trust rule

### 境界の含意

- core は意味の共通面に集中する
- profile は分野差分と運用差分を受け持つ
- 追加の語彙は core に逆流させない

## 5. 実装配置の前提

Phase 0 の時点では、配置を新しく増やしません。  
既存の責務別配置を前提に、次の置き場を使います。

- `packages/relay/`
- `packages/storage/`
- `packages/core/`
- `packages/protocol/`
- `packages/api/`
- `packages/cli/`

## 6. Phase 1 への引き継ぎ

### レビュー観点

- [ ] 責務境界が一貫しているか
- [ ] core に入れないものが明確か
- [ ] public / private が profile / policy 側か
- [ ] 監視対象外が明文化されているか
- [ ] profile 側の差分が切り出せているか

### 再利用する判断

- relay と storage の責務境界
- core に入れないもの
- public / private の扱い
- 監視対象としないもの
- profile 側へ逃がすべき差分

## 参照

- [運用準備ロードマップ](../roadmap/OPERATIONAL_READINESS_ROADMAP.md)
- [運用準備バックログ](../roadmap/OPERATIONAL_READINESS_BACKLOG.md)
- [技術決定 ADR](./TECH_DECISION_ADR.md)
- [概念モデル](../concepts/CONCEPT_MODEL.md)
- [Carrier](../concepts/CARRIER.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
