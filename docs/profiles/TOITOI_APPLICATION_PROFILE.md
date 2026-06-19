# Toitoi Application Profile

**Status: draft** | **Last updated: 2026-06-19**

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
- profile 固有の設定値と既定値
- profile 固有の運用差分

### 1.3 Core に入れないもの

次は core に入れません。

- 農業固有の前提
- Toitoi 専用 UI の都合
- Toitoi 専用 lifecycle 名
- profile 固有の trust rule
- profile 固有の query shortcut
- profile 固有の設定ファイル形式
- profile 固有の secret 取り扱い

## 2. 最小 object set

Toitoi profile の最小必須 object type は次の 3 つです。

- `inquiry`
- `observation`
- `evidence`

これらは inquiry 循環の起点、観察、根拠を表すための profile の基礎になります。

拡張 object type は次の通りです。

- `synthesis`
- `annotation`

これらは必須ではありませんが、Toitoi の実運用では早い段階で必要になる想定です。

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

最小必須 relation は次の通りです。

- `asks`
- `responds_to`
- `supports`
- `cites`

拡張 relation は次の通りです。

- `refutes`
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

初期候補の例:

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
- profile validation は core schema を fork せず、canonical view に対する追加制約として定義する

### 5.2 inquiry の追加ルール

- `body.text` は問いとして成立する文であること
- `body.text` は質問、依頼、問題提起のいずれかとして解釈できること
- `inquiry` は少なくとも 1 つの対象 relation を持てること
- 主要 relation は `asks` で表すこと
- `asks` relation の source は `inquiry` であること
- `asks` relation の target は topic / subject / answer candidate を表せること

### 5.3 observation の追加ルール

- 観察対象を識別できること
- いつ・どこで・誰が、を profile context または provenance で補えること
- `body.text` は観察、記録、報告のいずれかとして解釈できること
- `responds_to` を使って inquiry との接続を表せること
- `responds_to` relation の source は `observation` であること
- `responds_to` relation の target は `inquiry` であること

### 5.4 evidence の追加ルール

- 根拠 source を明示できること
- support / refute のどちらかの関係を表現できること
- 少なくとも `supports` か `cites` のいずれかを表現できること
- `body.text` は根拠の要点を要約していること
- `supports` relation の source は `evidence` であること
- `supports` relation の target は `claim` または `synthesis` を表せること
- `cites` relation の source は `evidence` または `observation` であること

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

## 7. プロファイル境界の要約

Toitoi profile の最小境界は、次のようにまとめられます。

- 必須 object は `inquiry`、`observation`、`evidence`
- 拡張 object は `synthesis`、`annotation`
- 必須 relation は `asks`、`responds_to`、`supports`、`cites`
- 拡張 relation は `refutes`、`annotates`、`derived_from`、`revises`
- context 語彙は profile 側で管理する
- API は canonical view を返し、raw payload を主返却にしない
- profile validation は object type ごとの意味的追加制約として定義する

## 8. 参照関係

- [概念モデル](../concepts/CONCEPT_MODEL.md)
- [用語集](../concepts/GLOSSARY.md)
- [アーキテクチャ草案](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [Toitoi 参照チェックリスト](../architecture/TOITOI_REFERENCE_CHECKLIST.md)

## 9. プロファイル差分の置き場

Toitoi のような application profile の差分は、core の設定や core schema に入れず、profile 文書と profile 専用の運用メモに分けて置きます。

### 9.1 この文書に置くもの

- 必須 object / relation
- context vocabulary
- profile validation
- profile が返す API の最小契約
- profile が core から追加で要求する前提

### 9.2 併設してよいもの

- profile 固有の起動時設定
- profile 固有の curation / routing 規則
- profile 固有の secret や deployment 差分
- profile 固有の runtime メモ

### 9.3 置かないもの

- core の必須フィールド
- core の carrier 契約
- core の設定ファイル形式
- core の secret 取り扱い
