# 分散知識コモンズ・アーキテクチャ草案

**Status: draft** | **Last updated: 2026-06-16**

## 1. 目的

この文書は、Toitoi 的な問い循環モデルを、より広い **分散知識コモンズ** に一般化するための最初のアーキテクチャ草案です。

このプロトコルは、農業に限らず、あらゆる分野の知識を扱うことを目的とします。

設計目標は次の通りです。

- 誰でも relay や storage node を立てられること
- 誰でも knowledge object を publish できること
- knowledge object が replay 可能で provenance を持つこと
- carrier の選択を semantic core の一部として扱えること
- Toitoi のような特定アプリケーションが、この基盤の上に乗れても、プロトコル自体はその分野に縛られないこと

要するに、

> プロトコルは知識循環の意味を定義し、
> carrier はその意味をどう運ぶかを定義します。

## 2. 設計原則

### 2.1 分野非依存

このプロトコルは、ひとつの対象分野を前提にしてはいけません。

次のような知識型を支えられる必要があります。

- inquiry
- observation
- claim
- evidence
- annotation
- synthesis
- translation
- reference
- concept

分野固有の語彙は、外付け・差し替え可能・任意であるべきです。

### 2.2 append-only セマンティクス

公開された知識オブジェクトは、破壊的には上書きしません。

修正、更新、再解釈は、新しいオブジェクトか、明示的な関係として表現します。たとえば:

- `revises`
- `supersedes`
- `derives_from`
- `translates`
- `cites`

### 2.3 canonical-first アーキテクチャ

このプロトコルは、次を区別します。

- carrier identity
- canonical identity
- provenance
- wire reference
- derived index

carrier の識別子は、ルーティングと重複排除のために使います。
canonical 識別子は、commons 内の意味的 identity に使います。

### 2.4 replay 可能性

どのノードも、wire archive から canonical state を再構築できる必要があります。

そのためには、次が必要です。

- wire event の保持
- 決定的な validate / normalize
- versioned な schema 規則
- 明示的な migration 経路

### 2.5 carrier 一体性

このプロトコルは、複数の carrier で同じ wire semantics を保てる必要があります。

例:

- relay ベースの pub/sub
- HTTP publish API
- file/archive ingest
- 将来の federated sync や offline sync carrier

carrier 固有の規則は framing と capability negotiation に閉じ、semantic model には持ち込みません。

### 2.6 provenance 優先

どの canonical object についても、どこから来たか、誰が主張したか、いつ現れたか、どう変換されたかを残すべきです。

commons は content の保管庫であるだけでなく、lineage の保管庫でもあります。

## 3. システム概要

アーキテクチャは 5 層に分かれます。

```text
Authoring / Edge
  ↓
Transport / Relay
  ↓
Canonicalization
  ↓
Indexer / Commons Memory
  ↓
API / Viewer
```

### 3.1 Authoring / Edge 層

この層は、利用者、ローカルエージェント、エディタ、またはアプリケーションが知識オブジェクトを草稿する場所です。

責務:

- ローカル文脈を集める
- knowledge object を草稿する
- raw なローカルデータの外部露出を最小化する
- 著者オブジェクトに署名または証明を付ける
- carrier 用にオブジェクトを梱包する

この層はアプリケーション固有であるべきです。
Toitoi はここに載る一例ですが、プロトコルそのものではありません。

### 3.2 Transport / Relay 層

この層は、protocol object を保存し、転送します。

責務:

- publish された protocol object を受け取る
- wire-level の構造を検証する
- 必要に応じて carrier 署名を認証・検証する
- carrier identity で重複排除する
- append-only log を保持する
- 購読対象のオブジェクトを配信する

この層は、意味の定義を変更してはいけません。

### 3.3 Canonicalization 層

この層は、wire 上の protocol object を canonical な knowledge object として確定します。

責務:

- wire input を解析する
- 必須の semantic field を検証する
- 順序と構造を正規化する
- canonical identity を解決する
- provenance を保持する
- wire reference を付与する
- 決定的な canonical view を生成する

### 3.4 Indexer / Commons Memory 層

この層は、canonical object から検索可能で辿れる構造を作ります。

責務:

- 時系列 index
- type index
- author index
- relation graph
- lineage graph
- provenance graph
- text 検索と facet 検索
- 任意の semantic embedding index

indexer は、canonical な意味を変えてはいけません。

### 3.5 API / Viewer 層

この層は、canonical view をアプリケーションや人間に公開します。

責務:

- canonical id で取得する
- 条件を指定して一覧する
- キーワードや facet で検索する
- relation を辿る
- provenance を確認する
- revision と lineage を確認する
- capability を公開する

API は、明示的に求められない限り carrier 固有の詳細を隠すべきです。

## 4. 中核データモデル

このプロトコルの意味的中核は **Knowledge Object** です。

### 4.1 Knowledge Object

knowledge object とは、canonical で、append-only で、provenance を持つ知識循環の単位です。

最小共通フィールド:

| フィールド | 役割 |
|---|---|
| `id` | canonical identity |
| `type` | semantic type |
| `body` | 主となる自然言語内容 |
| `language` | 言語タグ |
| `contexts` | 抽象化された local context |
| `relations` | semantic relation |
| `status` | ライフサイクル状態 |
| `lineage` | 派生と revision の履歴 |
| `provenance` | 来歴と変換履歴 |
| `rawRef` | carrier/wire payload への参照 |
| `claims` | 検証可能な identity または authorship claim |
| `rights` | ライセンスと再利用条件 |
| `visibility` | public / restricted / private の方針 |
| `attachments` | 任意の関連アーティファクト |
| `schemaVersion` | versioned schema contract |

### 4.2 推奨オブジェクト型

このプロトコルは、ひとつの狭い概念だけではなく、小さな core semantic type 群を支えるべきです。

- `inquiry`
- `observation`
- `claim`
- `evidence`
- `annotation`
- `synthesis`
- `translation`
- `reference`
- `concept`

アプリケーションは、このうち必要なものだけを使ってもかまいません。

### 4.3 Context モデル

`contexts` は、抽象化された locality や状況情報を表します。

例:

- 気候帯
- 研究分野
- 臨床環境
- 法域
- 教育段階
- 計測器クラス
- データセット provenance クラス

重要な点:

- contexts は raw な秘密情報ではない
- contexts は必ずしも普遍的な ontology 用語ではない
- contexts は、局所的な意味を潰さずに cross-context translation を助けるためのもの

### 4.4 Relation モデル

Relation は、意味レベルのつながりを表すべきです。

中核となる relation family:

- `related_to`
- `supports`
- `refutes`
- `cites`
- `derived_from`
- `revises`
- `supersedes`
- `translates`
- `annotates`
- `synthesizes`

relation は graph edge としても statement object としても保存できますが、canonical model は決定的でなければなりません。

### 4.5 Lineage モデル

Lineage は、オブジェクトがどう生まれたかを記録します。

これは一般的な relation とは別です。

例:

- このオブジェクトは別のオブジェクトから派生した
- このオブジェクトは複数のオブジェクトから統合された
- このオブジェクトは別オブジェクトの翻訳である
- このオブジェクトは前の版を修正している

Lineage は、replay、traceability、trust 評価に不可欠です。

## 5. Identity アーキテクチャ

このプロトコルは identity の責務を分離しなければなりません。

### 5.1 Carrier identity

Carrier identity は、特定の carrier が使う識別子です。

例:

- relay event id
- record URI
- archive の file hash
- storage bucket の object key

Carrier identity は routing、duplicate check、source lookup に使います。

### 5.2 Canonical identity

Canonical identity は、commons 内で使う semantic identity です。

性質:

- carrier をまたいでも、同じ semantic object を指すなら安定している
- carrier semantics に強く依存しない opaque な値である
- 決定的な規則から replay 可能である
- relay endpoint address に依存しない

推奨形式:

- `kc:obj:<opaque-id>` のような opaque canonical id

具体的なエンコードは後で確定できます。

### 5.3 Identity key

Identity key は、validate / normalize 規則から決定的に導出される carrier-neutral な値です。

用途:

- carrier 間で object を比較する
- canonical identity を解決する
- third-party verification を支える

Identity key は次に依存してはいけません。

- relay URL
- storage hint
- semantic identity に含めない provenance メタデータ

### 5.4 Identity claim

Identity claim は、identity key と canonical identity の対応を示す検証可能な主張です。

含めるべきもの:

- schema version
- claim type
- rule version
- identity key
- canonical id
- issuer
- issued time
- verification data

Claim は provenance と同じではありません。
Claim は identity resolution と verification のためのものです。
Provenance は origin と履歴のためのものです。

### 5.5 Provenance

Provenance は、canonical object がどこから来て、誰が作り、どう移動したかを記録します。

典型的な provenance フィールド:

- source protocol
- source identity
- source author / actor
- source time
- ingest time
- transform chain
- verification state

### 5.6 Wire reference

Wire reference は、元の carrier object または payload を指します。

目的:

- データの再解析
- schema 変更後の再 canonicalize
- 取り込み履歴の監査
- バグや検証結果の再現

Wire reference は provenance とは分けて保持します。

## 6. Carrier と relay のモデル

このプロトコルは、commons のために複数 relay を許容すべきです。

### 6.1 Relay の責務

Relay は分散配信と永続化のノードです。

すべきこと:

- wire object を受け取る
- wire-level の整合性を検証する
- append-only の wire log を保存する
- 必要なら canonical projection も保存する
- subscription を配信する
- replay と export を支える

### 6.2 Relay がやらないこと

Relay は次をしてはいけません。

- domain truth を決める
- あいまいな意味をひとつに潰す
- provenance を恣意的に書き換える
- 履歴を破壊的に上書きする

### 6.3 Relay の種類

このプロトコルは、複数の relay profile を支えられます。

- **Public relay**: 公開 publish と retrieval
- **Curated relay**: 特定の object type や policy だけを受ける
- **Private relay**: 限定メンバーまたは暗号化オブジェクト
- **Archive relay**: 長期保管と replay に重点を置く
- **Gateway relay**: carrier 間の変換を担う

### 6.4 配信モデル

Publish は append-only であるべきです。Subscription は絞り込み可能であるべきです。Delivery は可能なら冪等であるべきです。

推奨 subscription filter:

- object type
- author または actor
- namespace または collection
- relation target
- provenance source
- 時間範囲
- tag または facet

## 7. Canonicalization パイプライン

Validation / normalization パイプラインは、決定的であるべきです。

### 7.1 パイプライン段階

```text
Raw carrier object
  ↓ parse
Validated carrier object
  ↓ normalize
Normalized semantic object
  ↓ finalize
Canonical knowledge object
  ↓ store + index
Derived memory structures
```

### 7.2 Validation

Validation では次を確認します。

- carrier schema の形
- 必須 semantic field
- 必要なら署名や証明
- relation の形
- 禁止される field の組み合わせ

### 7.3 Normalization

Normalization では次を行います。

- field 順序の正規化
- 言語タグの正規化
- タイムスタンプの正規化
- relation 構造の正規化
- 共通メタデータの抽出

### 7.4 Finalization

Finalization では次を行います。

- canonical id の解決
- provenance の付与
- wire reference の保持
- 決定的な semantic output の生成
- あいまいな入力の無理な merge を避ける

canonical identity が不確かな場合は、無理にひとつへ統合せず、別オブジェクトとして保持すべきです。

## 8. Storage モデル

Storage は append-only かつ replay-friendly であるべきです。

### 8.1 保存するもの

少なくとも次を保存します。

- raw carrier logs
- canonical objects
- ingest logs
- derived indexes または index snapshot
- replay metadata

### 8.2 raw と canonical を分ける理由

raw と canonical を分けることで、次が可能になります。

- 規則変更後の再 canonicalize
- forensic inspection
- carrier migration
- バグ再現
- multi-carrier convergence

### 8.3 Storage backend

このプロトコルは、ひとつの backend を強制しません。

候補:

- local filesystem
- embedded database
- object storage
- append-only JSONL log
- index 用の relational または document store

## 9. Indexing モデル

Indexer は、canonical knowledge を commons memory に変換します。

### 9.1 必須 index

少なくとも次が必要です。

- canonical id lookup
- chronology / timeline
- type index
- author index
- relation index
- lineage index
- provenance index
- text search

### 9.2 任意 index

任意だが有用なもの:

- faceted search
- graph expansion
- vector embedding
- similarity search
- domain-specific conceptual clustering

### 9.3 Indexing の原則

Indexer は、semantic source そのものになってはいけません。
あくまで canonical object から view を派生するだけです。

## 10. API モデル

API は canonical view 層です。

### 10.1 API の目的

API は次を可能にするべきです。

- 1 件のオブジェクトを取得する
- 複数のオブジェクトを検索する
- relation、type、provenance で絞り込む
- lineage と revision history を見る
- capability を確認する

### 10.2 API が返すべきもの

- canonical object
- provenance summary
- lineage summary
- relation graph fragment
- 必要に応じた source metadata

### 10.3 API が基本的に返すべきでないもの

- raw protocol payload を主インターフェースとして見せること
- carrier 固有の実装詳細
- ingest 内部の不安定な表現

## 11. Security と trust モデル

このプロトコルは、すべてのデータが同じだけ信頼できるとは仮定しません。

### 11.1 Trust レベル

想定される trust 分類:

- self-authored
- relay-verified
- third-party attested
- community-curated
- machine-generated
- unverified

### 11.2 Verification 層

Verification は複数あります。

- carrier integrity
- author identity
- claim validity
- provenance completeness
- policy compliance

### 11.3 Abuse 対策

Relay と indexer は次に耐えるよう設計すべきです。

- spam
- duplicate flooding
- provenance forgery
- relation pollution
- identity collision
- storage exhaustion

## 12. Domain vocabulary と application profile

プロトコル core は domain neutral のままであるべきです。

分野固有の意味は次に置きます。

- vocabulary
- profile
- ontology
- schema extension

application profile の例:

- 農業向け inquiry profile
- 医療研究 profile
- 法的 reasoning profile
- 教育向け inquiry profile
- 科学ノート profile

各 profile は次を定義できます。

- object subtype の制約
- 必須フィールド
- relation 語彙
- context 語彙
- validation 規則

これにより、Toitoi はこのプロトコルの 1 つのアプリケーションになれても、プロトコルそのものにはなりません。

## 13. 推奨リポジトリ構成

このプロトコルを独立リポジトリにするなら、次のような構成が実用的です。

```text
lingonberry/
├─ README.md
├─ docs/
│  ├─ architecture/
│  ├─ concepts/
│  ├─ protocols/
│  ├─ operations/
│  └─ roadmap/
├─ schemas/
├─ packages/
│  ├─ protocol/
│  ├─ core/
│  ├─ codecs/
│  ├─ carriers/
│  ├─ relay/
│  ├─ indexer/
│  ├─ api/
│  └─ cli/
├─ fixtures/
├─ examples/
└─ tests/
```

### 13.1 package の責務

- `packages/protocol/`
  - descriptor、capability table、registry、versioning
- `packages/core/`
  - canonical model、identity、provenance、relations
- `packages/codecs/`
  - protocol object の validate / normalize / finalize
- `packages/carriers/`
  - carrier 固有の framing と helper
- `packages/relay/`
  - relay runtime と ingest service
- `packages/indexer/`
  - replay、search、graph、derived memory
- `packages/api/`
  - canonical view API
- `packages/cli/`
  - inspect、validate、replay、migrate

## 14. Toitoi からの移行戦略

Toitoi は、このプロトコルの上に乗る 1 つのアプリケーションとして扱えます。
そのとき、現在の inquiry モデルは application profile の 1 つになります。

### 14.1 そのまま残るもの

- inquiry 循環
- provenance tracking
- append-only replay
- relation と lineage の扱い
- 分散 relay モデル

### 14.2 一般化されるもの

- object type
- vocabulary pack
- carrier abstraction
- 分野横断の relation semantics
- API schema と命名

### 14.3 プロトコル core から外すべきもの

- 農業固有の前提
- Toitoi 固有語彙を default semantics にすること
- 1 分野専用 UI の想定

## 15. 早めに決めるべき未決事項

次の点は、後続設計全体に影響するため、早めに決めた方がよいです。

1. canonical id は opaque にするか、それとも content-derived にするか
2. delete は tombstone だけにするか、限定操作として許すか
3. private または encrypted object を初期版に含めるか
4. relation を triple、edge、statement object のどれで表すか
5. identity claim を必須にするか任意にするか
6. relay は最小限の semantic validation を行うか、wire validation のみにするか
7. 最初の carrier は relay-based pub/sub、HTTP publish、file/archive ingest のどれにするか
8. application profile を中央 registry で管理するか、完全分散にするか
9. 多言語の正本をどう表現するか
10. public relay が object を受け入れるための最小 trust model は何か

## 16. 次の推奨作業

このアーキテクチャに問題がなければ、次に書くべき文書は次です。

- プロトコル用語集
- canonical object 仕様
- identity と provenance の仕様
- relay 運用モデル
- carrier schema テンプレート
- Toitoi 的 inquiry object の最初の application profile

これらを書くと、この草案は実装可能なプロトコルスタックになります。
