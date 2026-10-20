# Toitoiアプリケーションプロファイル

**Status: draft** | **Last updated: 2026-10-20**

## 目的

この文書は、Lingonberryの共通仕様を使いながら、Toitoi固有の語彙、検証規則、API要件を定めるための草案です。ここに挙げる型やAPIはプロファイルの設計要件であり、現行リレーでの実装完了を示すものではありません。

基本用語は[用語集](../concepts/GLOSSARY.md)を参照してください。

狙いは次のとおりです。

- Toitoi固有の要件を中核プロトコルに埋め込まない
- 問いから観察・根拠へつながる流れに必要な型と関係を、プロファイル側で定義する
- 農業固有の語彙を共通仕様から切り離す
- Toitoiに必要なAPIを、プロファイルの仕様として定める

## 1. 境界

### 1.1 共通仕様に残すもの

Toitoiプロファイルは、次の共通の前提に従います。

- 知識オブジェクトは追記を基本とし、既存の内容を上書きしない
- `canonical identity`は、共通仕様における正規の識別情報である
- `carrier`は、プロトコルオブジェクトを伝送形式で運ぶ正規の実装である
- 伝送形式（`wire`）と正規表現（`canonical`）は、同じプロトコルの異なる表現である
- 来歴（`provenance`）と生データへの参照（`rawRef`）は、共通仕様で扱う
- 識別情報の主張（`identity claim`）は、共通仕様では任意である

### 1.2 プロファイルで定めるもの

Toitoi固有の意味付けは、プロファイル側で定義します。

- オブジェクトの下位分類の使い分け
- 関係を表す語彙
- 文脈情報の語彙
- 情報を選び、整理する規則
- 表示や検索の優先順位
- Toitoi向けAPIの応答形式
- プロファイル固有の設定値と既定値
- プロファイル固有の運用上の違い

### 1.3 carrierとの境界

Toitoiプロファイルでは、carrierの違いによって知識の意味付けを変えません。
HTTPなどのcarrierが定めるのは、受信・検証・公開のための制約です。プロファイルで扱う知識の意味そのものは定めません。

- carrier が変わっても `inquiry`、`observation`、`evidence` の意味は変えない
- carrierごとの違いは `supported content types`、`supported auth modes`、`supported access scopes` など、対応機能を示す範囲にとどめる
- プロファイルの検証は、carrier固有のデータの区切り方ではなく、正規化されたビューへの追加制約として定義する
- `GET /objects/:id` などのAPIの応答形式は、プロファイルの仕様として扱う

### 1.4 共通仕様に入れないもの

次の事項は共通仕様に入れません。

- 農業固有の前提
- Toitoi 専用 UI の都合
- Toitoi専用の状態や処理段階の名称
- プロファイル固有の信頼に関する規則
- プロファイル固有の検索の簡略化
- プロファイル固有の設定ファイル形式
- プロファイル固有の秘密情報の取り扱い

## 2. 基本となるオブジェクトの種類

Toitoiプロファイルで必須とするオブジェクトの種類は、次の3つです。

- `inquiry`
- `observation`
- `evidence`

これらを使い、問いの起点、観察、根拠を表します。

拡張として扱うオブジェクトの種類は、次のとおりです。

- `synthesis`
- `annotation`

これらは必須ではありませんが、Toitoi の実運用では早い段階で必要になる想定です。

### 2.1 inquiry

`inquiry` は問いの起点です。

プロファイルが求める基本的な性質:

- 何についての問いか、主題や対象を明示できる
- 他のオブジェクトとの関係を表せる
- 出所や経緯をたどれる来歴情報を持つ

### 2.2 observation

`observation` は観察・記録です。

プロファイルが求める基本的な性質:

- 何を見たかを記述できる
- 問い（`inquiry`）と結び付けられる
- 根拠（`evidence`）への参照を持てる

### 2.3 evidence

`evidence` は根拠・証拠です。

プロファイルが求める基本的な性質:

- 観察（`observation`）や外部の情報源と関係付けられる
- 主張（`claim`）の支持や反証に使える

### 2.4 synthesis

`synthesis` は複数のオブジェクトをまとめた結果です。

プロファイルが求める基本的な性質:

- 複数の情報源を参照できる
- 派生の履歴（`lineage`）を通じて、元となった情報をたどれる

### 2.5 annotation

`annotation` は オブジェクトへの注釈です。

プロファイルが求める基本的な性質:

- 対象のオブジェクトを明示できる
- コメントと評価を分けて扱える

## 3. 関係を表す語彙

Toitoiプロファイルは、共通仕様の関係表現（statement-based relation）を使い、用途固有の語彙を定義できます。

必須の関係は次のとおりです。

- `asks`
- `responds_to`
- `supports`
- `cites`

拡張の関係は次のとおりです。

- `refutes`
- `annotates`
- `derived_from`
- `revises`

### 3.1 使い分け

- `asks`: 問いが何を対象とするかを示す
- `responds_to`: 観察や統合結果が問いに答える
- `supports`: 根拠が主張や統合結果を支える
- `refutes`: 根拠が主張に反論する
- `cites`: 参照関係
- `annotates`: 対象への注釈を示す
- `derived_from`: 派生元を示す
- `revises`: 修正版を示す

## 4. 文脈情報の語彙

Toitoiプロファイルの文脈情報は、農業固有の意味を共通仕様から切り離して扱うための外部語彙です。

初期候補の例:

- `field`: 圃場
- `crop`: 作物
- `site`: 場所
- `season`: 季節
- `condition`: 状態・条件
- `practice`: 実践・作業

これらはプロファイル側で差し替えられるようにし、共通仕様の標準語彙には含めません。

## 5. 検証規則

Toitoiプロファイルは、共通の検証に加えて独自の検証規則を定められます。

### 5.1 共通ルール

- 正規識別子（`canonical id`）は、文字列の内部構造から意味を推測せずに扱う
- `provenance` と `rawRef` は必須のまま維持する
- 1つのオブジェクトは1つの言語を表す
- プロファイルの検証規則は、共通スキーマを分岐させず、正規化されたビューへの追加制約として定義する

### 5.2 inquiry の追加ルール

- `body.text` は問いとして成立する文であること
- `body.text` は質問、依頼、問題提起のいずれかとして解釈できること
- `inquiry` は少なくとも1つ、対象との関係を表せること
- 主要な関係は `asks` で表すこと
- `asks` 関係の始点は `inquiry` であること
- `asks` 関係の終点は 話題、主題、回答候補を表せること

### 5.3 observation の追加ルール

- 観察対象を識別できること
- いつ・どこで・誰が、という情報をプロファイルの文脈情報または来歴情報で補えること
- `body.text` は観察、記録、報告のいずれかとして解釈できること
- `responds_to` を使って問いとのつながりを表せること
- `responds_to` 関係の始点は `observation` であること
- `responds_to` 関係の終点は `inquiry` であること

### 5.4 evidence の追加ルール

- 根拠となる情報源を明示できること
- 支持または反論の関係を表現できること
- 少なくとも `supports` か `cites` のいずれかを表現できること
- `body.text` は根拠の要点を要約していること
- `supports` 関係の始点は `evidence` であること
- `supports` 関係の終点は `claim` または `synthesis` を表せること
- `cites` 関係の始点は `evidence` または `observation` であること

## 6. APIの要件

Toitoiプロファイルが求める最小限のAPIは、次のとおりです。以下はプロファイル上の要件であり、現行リレーのエンドポイント一覧ではありません。

- `GET /objects/:id`
- `GET /objects?type=...`
- `GET /objects/:id/relations`
- `GET /objects/:id/lineage`
- `GET /objects/:id/provenance`
- `GET /sources/:protocol/:sourceId`

### 6.1 返却方針

- 受信した生データを主な返却内容にしない
- 正規化されたビューを基本の返却内容とする
- グラフは、対象から関係を1段階たどった範囲の部分データから始める
- 来歴情報は、識別情報の主張と分けて返す

## 7. プロファイル境界の要約

Toitoiプロファイルで定める最小限の範囲は、次のようにまとめられます。

- 必須のオブジェクトは `inquiry`、`observation`、`evidence`
- 拡張のオブジェクトは `synthesis`、`annotation`
- 必須の関係は `asks`、`responds_to`、`supports`、`cites`
- 拡張の関係は `refutes`、`annotates`、`derived_from`、`revises`
- 文脈情報の語彙はプロファイル側で管理する
- APIは正規化されたビューを返し、生データを主な返却内容にしない
- プロファイルの検証規則は、オブジェクトの種類ごとの意味上の追加制約として定義する

## 8. 参照関係

- [概念モデル](../concepts/CONCEPT_MODEL.md)
- [用語集](../concepts/GLOSSARY.md)
- [アーキテクチャ草案](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [Toitoi 参照チェックリスト](../architecture/TOITOI_REFERENCE_CHECKLIST.md)

## 9. プロファイル差分の置き場

Toitoiのようなプロファイル固有の内容は、共通の設定やスキーマに入れず、プロファイルの文書と専用の運用メモに分けて記載します。

### 9.1 この文書に置くもの

- 必須のオブジェクトと関係
- 文脈情報の語彙
- プロファイルの検証規則
- プロファイルが提供するAPIの最小仕様
- プロファイルが共通仕様に対して追加で要求する前提

### 9.2 併設してよいもの

- プロファイル固有の起動時設定
- プロファイル固有の情報整理・振り分けの規則
- プロファイル固有の秘密情報や配置の違い
- プロファイル固有の実行時の注意事項

### 9.3 置かないもの

- 共通仕様の必須フィールド
- 共通仕様のcarrierの仕様
- 共通の設定ファイル形式
- 共通の秘密情報の取り扱い

### 9.4 carrier 差し替え時の見方

carrier を差し替えるときは、次の順で確認します。

1. `knowledge object` の意味がcarrierによって変わっていないか
2. `supported schema versions` と `validation / finalize constraints` がプロファイルの要件と一致しているか
3. `supported auth modes`、`supported content types`、`supported access scopes`、`supported retention hints` がプロファイルの運用に十分か
4. Toitoi固有の関係や文脈情報の語彙が、carrier側の仕様に入り込んでいないか
5. `GET /objects/:id`、`GET /objects?type=...`、`GET /objects/:id/relations`、`GET /objects/:id/lineage`、`GET /objects/:id/provenance` の応答形式が変わっていないか

carrierが変わっても、プロファイル側で扱う違いは、APIの仕様と、情報の整理・振り分け・検索の優先順位の範囲にとどめます。
