# Toitoi Application Profile

**Status: draft** | **Last updated: 2026-06-18**

## 目的

この文書は、Lingonberry core の上に載る Toitoi 用 application profile の最小境界を定義します。

狙いは次の通りです。

- Toitoi を core protocol に埋め込まない
- inquiry 循環に必要な型と relation を profile 側で定義する
- 農業固有の語彙を core から切り離す
- Toitoi が必要とする API だけを profile 契約として公開する

## 1. 境界

### 1.1 Core に残すもの

Toitoi profile は、core の次の前提をそのまま使います。

- `knowledge object` は append-only である
- `canonical identity` は core の共通識別子である
- `carrier` は protocol object を wire 上で運ぶ正規実装である
- `wire` と `canonical` は別プロトコルではなく別表現である
- `provenance` と `rawRef` は core の責務である
- `identity claim` は core では任意である

### 1.2 Profile に寄せるもの

Toitoi 固有の意味は profile 側で定義します。

- object subtype の使い分け
- relation vocabulary
- context vocabulary
- curation rule
- display / query priority
- Toitoi 向け API の返却形

### 1.3 Core に入れないもの

次は core に入れません。

- 農業固有の前提
- Toitoi 専用 UI の都合
- Toitoi 専用 lifecycle 名
- profile 固有の trust rule
- profile 固有の query shortcut

## 2. 最小 object set

Toitoi profile では、少なくとも次の object type を扱います。

- `inquiry`
- `observation`
- `evidence`
- `synthesis`
- `annotation`

これらは Toitoi 専用ではなく、core の上で profile として組み合わせる型です。

### 2.1 inquiry

`inquiry` は問いの起点です。

最低限の profile 期待値:

- 明示的な subject / topic を持てる
- 他の object へ relation を張れる
- 追跡可能な provenance を持つ

### 2.2 observation

`observation` は観察・記録です。

最低限の profile 期待値:

- 何を見たかを記述できる
- inquiry に紐づけられる
- evidence への参照を持てる

### 2.3 evidence

`evidence` は根拠・証拠です。

最低限の profile 期待値:

- observation または external source と関係づけられる
- claim の支持や反証に使える

### 2.4 synthesis

`synthesis` は複数 object の統合結果です。

最低限の profile 期待値:

- 複数の source を参照できる
- lineage を通じて導出元を辿れる

### 2.5 annotation

`annotation` は object への注釈です。

最低限の profile 期待値:

- 対象 object を明示できる
- コメントと評価を分けて扱える

## 3. Relation vocabulary

Toitoi profile は、core の statement-based relation を使いながら、profile 語彙を定義できます。

最小候補:

- `asks`
- `responds_to`
- `supports`
- `refutes`
- `cites`
- `annotates`
- `derived_from`
- `revises`

### 3.1 使い分け

- `asks`: inquiry が対象を問う
- `responds_to`: observation や synthesis が inquiry に答える
- `supports`: evidence が claim や synthesis を支える
- `refutes`: evidence が claim を否定する
- `cites`: 参照関係
- `annotates`: annotation が対象に付く
- `derived_from`: 派生元を示す
- `revises`: 修正版を示す

## 4. Context vocabulary

Toitoi profile の context は、農業固有の意味を core に持ち込まないための外部語彙です。

例:

- field
- crop
- site
- season
- condition
- practice

これらは profile 側で差し替え可能にし、core の標準語彙にはしません。

## 5. Validation rules

Toitoi profile は、core validation に加えて profile validation を持てます。

### 5.1 共通ルール

- `canonical id` は opaque なまま扱う
- `provenance` と `rawRef` は必須のまま維持する
- 1 object は 1 言語を表す

### 5.2 inquiry の追加ルール

- `body.text` は問いとして成立する文であること
- `inquiry` は少なくとも 1 つの対象 relation を持てること

### 5.3 observation の追加ルール

- 観察対象を識別できること
- いつ・どこで・誰が、を profile context または provenance で補えること

### 5.4 evidence の追加ルール

- 根拠 source を明示できること
- support / refute のどちらかの関係を表現できること

## 6. API contract

Toitoi profile が必要とする最小 API は次の通りです。

- `GET /objects/:id`
- `GET /objects?type=...`
- `GET /objects/:id/relations`
- `GET /objects/:id/lineage`
- `GET /objects/:id/provenance`
- `GET /sources/:protocol/:sourceId`

### 6.1 返却方針

- raw payload を主返却にしない
- canonical view を基本返却にする
- graph は 1-hop の fragment から始める
- provenance は identity claim と分けて返す

## 7. 参照関係

- [概念モデル](../concepts/CONCEPT_MODEL.md)
- [用語集](../concepts/GLOSSARY.md)
- [アーキテクチャ草案](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [Toitoi 参照チェックリスト](../architecture/TOITOI_REFERENCE_CHECKLIST.md)

