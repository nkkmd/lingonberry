# 分散知識コモンズ・アーキテクチャ草案

**Status: draft** | **Last updated: 2026-06-19**

## 1. 目的

この文書は、Toitoi 的な問い循環モデルを、より広い **分散知識コモンズ** に一般化するための最初のアーキテクチャ草案です。

この基盤は Toitoi を載せられることを重要な設計条件とします。ただし、Toitoi に特化した protocol にはしません。Toitoi の inquiry 循環、provenance、append-only replay、分散 relay モデルは、この基盤の上の application profile として表現します。

もう 1 つの重要な設計条件は、**transport と protocol の完全同一化**を目標にすることです。transport は protocol の外側にある単なる配送手段ではなく、protocol object が wire 上で成立する正規の実装形として扱います。

このプロトコルは、農業に限らず、あらゆる分野の知識を扱うことを目的とします。

設計目標は次の通りです。

- 誰でも relay や storage node を立てられること
- 誰でも knowledge object を publish できること
- knowledge object が replay 可能で provenance を持つこと
- carrier / transport の選択を protocol の正規な実装形として扱えること
- transport と protocol の間に semantic adapter や別 protocol 変換層を置かないこと
- Toitoi のような特定アプリケーションが、この基盤の上に乗れても、protocol core はその分野に縛られないこと

要するに、

> protocol は知識循環の意味を定義し、
> carrier / transport はその意味が wire 上で成立する正規の形を定義します。

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

### 2.6 transport / protocol 同一化

このプロトコルは、transport を protocol の外部にある交換可能な配送層とは見なしません。

carrier は protocol object を wire 上で成立させる正規の実装です。したがって、carrier ごとの差分は次に閉じるべきです。

- framing
- serialization
- routing
- retry / ordering
- storage hint
- capability negotiation
- carrier identity

carrier ごとに semantic を翻訳したり、別の protocol object へ変換したりしてはいけません。

validate / normalize / finalize は、外部形式を Lingonberry に変換する処理ではなく、同じ protocol object を canonical な knowledge object として確定する処理です。

### 2.7 Toitoi を載せられるが、Toitoi に特化しない

Toitoi は、この基盤に対する最初の重要な利用例です。

そのため、core protocol は次を自然に支えられる必要があります。

- inquiry
- observation
- evidence
- synthesis
- revision / lineage
- local context
- provenance tracking
- distributed relay
- replay

ただし、Toitoi 固有の語彙、農業固有の context、Toitoi 専用 UI の都合は core protocol に入れません。それらは application profile、vocabulary、または edge application の責務として扱います。

### 2.8 provenance 優先

どの canonical object についても、どこから来たか、誰が主張したか、いつ現れたか、どう変換されたかを残すべきです。

commons は content の保管庫であるだけでなく、lineage の保管庫でもあります。

## 3. システム概要

アーキテクチャは 5 層に分かれます。

```text
Authoring / Edge
  ↓
Protocol Carrier / Relay
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
- protocol-native carrier framing に載せる

この層はアプリケーション固有であるべきです。
Toitoi はここに載る一例ですが、プロトコルそのものではありません。

### 3.2 Protocol Carrier / Relay 層

この層は、protocol object を wire 上で成立させ、保存し、転送します。

ここでの transport は protocol の外側にある独立した配送層ではありません。carrier は protocol object の正規な wire 実装であり、relay はその carrier を受け取り、保持し、配信するノードです。

責務:

- publish された protocol object を受け取る
- wire-level の構造を検証する
- schema / framing / carrier identity の妥当性を確認する
- 必要に応じて carrier 署名を認証・検証する
- carrier identity で重複排除する
- append-only log を保持する
- 購読対象のオブジェクトを配信する

Relay は意味論の正しさを決めず、semantic validation は canonicalization 層に寄せます。

この層は、意味の定義を変更してはいけません。また、carrier 固有の semantic adapter を持ってはいけません。

### 3.3 Canonicalization 層

この層は、wire 上の protocol object を canonical な knowledge object として確定します。

これは別 protocol から Lingonberry へ翻訳する層ではありません。carrier 上に存在する protocol object を、決定的な規則で validate / normalize / finalize する層です。

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

初期仕様で固定する core fields は次の通りです。

| フィールド | 必須性 | 役割 |
|---|---|---|
| `id` | 必須 | canonical identity |
| `schemaVersion` | 必須 | versioned schema contract |
| `type` | 必須 | semantic type |
| `createdAt` | 必須 | canonical creation timestamp |
| `body` | 必須 | 主となる自然言語内容 |
| `body.language` | 必須 | 正本の言語タグ。1 object は 1 言語を表す |
| `provenance` | 必須 | 来歴と変換履歴 |
| `rawRef` | 必須 | carrier / wire payload への参照 |
| `contexts` | 任意 | 抽象化された local context |
| `relations` | 任意 | semantic relation |
| `status` | 任意 | ライフサイクル状態 |
| `lineage` | 任意 | 派生と revision の履歴 |
| `identityClaims` | 任意 | 検証可能な identity または authorship claim |
| `attachments` | 任意 | 関連アーティファクト |
| `labels` | 任意 | 検索や facet 用の補助ラベル |
| `meta` | 任意 | 非 semantic な実装メタデータ |

`rights` と `visibility` は core の最小構造には含めず、必要なら application profile または policy 層で拡張します。

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

Toitoi は、少なくとも `inquiry`、`observation`、`evidence`、`synthesis`、`annotation` を中心にした application profile として表現できます。ただし、これらの型は Toitoi 専用ではありません。

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

core では relation を `statement object` として正規化し、`edge` は index / projection 側の派生構造として扱います。

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

relation の semantic source は statement object に置き、graph edge は検索や traverse のための派生物とします。

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

このプロトコルでは、canonical id は **opaque** に固定します。

canonical id は content hash そのものではありません。content-derived な照合は identity key が担い、canonical id は commons 内で解決された object identity を指す安定した opaque identifier として扱います。

推奨形式:

- `lb:obj:<opaque-id>` のような opaque canonical id

具体的な opaque 部分のエンコードは後で確定できますが、content-derived な値をそのまま canonical id にする方針は取りません。

### 5.3 Identity key

Identity key は、validate / normalize 規則から決定的に導出される carrier-neutral な値です。

このプロトコルでは、identity key は **deterministic / content-rule-derived** に固定します。

identity key は、versioned な identity rule に従って、canonicalization 前後で比較可能な semantic content から導出します。どのフィールドを含めるか、どの正規化規則を使うかは rule version によって明示します。

用途:

- carrier 間で object を比較する
- canonical identity を解決する
- third-party verification を支える

Identity key は次に依存してはいけません。

- relay URL
- storage hint
- semantic identity に含めない provenance メタデータ
- canonical id

Identity key と canonical id の対応は identity claim で表現します。

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

このプロトコルは、commons のために複数 relay と複数 carrier を許容すべきです。

ただし、carrier は protocol から分離された transport adapter ではありません。どの carrier も protocol object の正規な wire 実装として振る舞い、semantic model を変えてはいけません。

### 6.1 Relay の責務

Relay は分散配信と永続化のノードです。

すべきこと:

- wire object を受け取る
- wire-level の整合性を検証する
- schema / framing / carrier identity の妥当性を確認する
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
- semantic validation を canonicalization 層より広く肩代わりする

### 6.3 Relay の種類

このプロトコルは、複数の relay profile を支えられます。

- **Public relay**: 公開 publish と retrieval。署名と形式が正しい public object を受け入れるが、その内容の真偽は保証しない
- **Curated relay**: 特定の object type や policy だけを受ける
- **Private relay**: 限定メンバーまたは暗号化オブジェクト
- **Archive relay**: 長期保管と replay に重点を置く
- **Gateway relay**: carrier 間の framing / routing / capability 差分を橋渡しする

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

このパイプラインは、transport 固有表現を別の protocol へ変換するためのものではありません。同じ protocol object を、replay 可能な canonical knowledge object として確定するためのものです。

### 7.1 パイプライン段階

```text
Wire protocol object on carrier
  ↓ deserialize / parse
Validated protocol object
  ↓ normalize
Normalized protocol object
  ↓ finalize
Canonical knowledge object
  ↓ store + index
Derived memory structures
```

### 7.2 Validation

Validation では次を確認します。

- protocol object schema の形
- carrier framing の形
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

- wire protocol logs
- canonical objects
- ingest logs
- derived indexes または index snapshot
- replay metadata

### 8.2 raw と canonical を分ける理由

raw と canonical を分けることで、次が可能になります。

ここでの raw は、別 protocol の raw data ではなく、carrier 上で受け取った protocol object の保存表現です。

- 規則変更後の再 canonicalize
- forensic inspection
- carrier framing migration
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

API は application が扱いやすい canonical view を返します。Toitoi のような application はこの API の上に載れますが、API 自体は Toitoi 専用語彙を前提にしません。

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

Toitoi は最初に想定する重要な application profile ですが、core protocol の default semantics ではありません。

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

多言語の正本は、1 つの object に複数言語を詰め込まず、言語ごとに別 object を持ち、`translation` / `translates` relation で結ぶのを基本にします。

これにより、Toitoi はこのプロトコルの 1 つのアプリケーションになれても、プロトコルそのものにはなりません。

Toitoi profile は、core object type と relation を組み合わせて inquiry 循環を定義します。農業固有の context や語彙は Toitoi profile のさらに外側に置き、必要に応じて差し替え可能にします。

## 13. 推奨リポジトリ構成

このリポジトリは、仕様、概念、運用判断、実装計画、実装を役割ごとに分けて保つ構成にします。  
現在は実装が始まっているため、`packages/` を中核の実装置き場として含める前提で整理します。  

```text
lingonberry/
├─ README.md
├─ AGENTS.md
├─ docs/
│  ├─ architecture/
│  ├─ concepts/
│  ├─ operations/
│  ├─ protocols/
│  └─ roadmap/
├─ packages/
│  ├─ protocol/
│  ├─ codecs/
│  ├─ core/
│  ├─ relay/
│  ├─ api/
│  └─ cli/
├─ schemas/
├─ fixtures/
└─ (optional future)
   ├─ packages/
   │  ├─ carriers/
   │  ├─ indexer/
   │  └─ ...
   ├─ examples/
   └─ tests/
```

### 13.1 ディレクトリの役割

- `README.md`
  - リポジトリ全体の入口
- `AGENTS.md`
  - 作業ルールと参照順序
- `docs/concepts/`
  - `knowledge object`、`canonical identity`、`carrier` などの中核概念
- `docs/architecture/`
  - 全体アーキテクチャ、設計草案、Toitoi 参照情報
- `docs/operations/`
  - 技術選定、ADR、carrier / storage の決定メモ
- `docs/roadmap/`
  - 実装ロードマップと backlog
- `docs/protocols/`
  - protocol-native な wire 仕様
- `schemas/`
  - protocol-native な JSON Schema
- `fixtures/`
  - schema や wire 仕様の検証用サンプル
- `packages/`
  - 実装本体。責務ごとに分離したパッケージを置く

### 13.2 現在の実装配置

- `packages/protocol/`
  - protocol object の parser / validator / canonicalizer
- `packages/codecs/`
  - validate / normalize / finalize の共通処理
- `packages/core/`
  - append-only storage と replay / retrieval
- `packages/indexer/`
  - canonical store から派生する search / graph / view index
- `packages/relay/`
  - relay runtime
- `packages/api/`
  - canonical view の組み立て
- `packages/cli/`
  - validate / publish / get / list の実行入口

### 13.3 実装を追加する場合の責務

- `packages/carriers/`
  - protocol object の正規 wire 実装、carrier 固有の framing と helper
- `packages/indexer/`
  - replay、search、graph、derived memory
- `examples/`
  - 仕様検証や profile 例の最小サンプル
- `tests/`
  - 統合テストや回帰テスト

## 14. Toitoi からの移行戦略

Toitoi は、このプロトコルの上に乗る 1 つのアプリケーションとして扱えます。
そのとき、現在の inquiry モデルは application profile の 1 つになります。

移行の目的は、Toitoi を protocol core に埋め込むことではありません。Toitoi が必要とする問い循環を支えられるだけの core semantics を残し、Toitoi 固有の語彙と運用判断は profile 側へ寄せます。

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
- Toitoi 専用の lifecycle を core status として固定すること
- Toitoi 専用の relation 名を core relation として固定すること

### 14.4 Toitoi profile として定義すべきもの

- 最小必須 object set
- 最小必須 relation set
- inquiry / observation / evidence / synthesis / annotation の profile 上の役割
- Toitoi 内で使う context 語彙
- Toitoi 内での trust / curation rule
- Toitoi UI が必要とする derived view

## 15. 早めに決めるべき未決事項

次の点は、後続設計全体に影響するため、早めに決めた方がよいです。

1. delete は tombstone だけにするか、限定操作として許すか（決定済み: tombstone 限定）
2. private または encrypted object を初期版に含めるか（決定済み: 初期版には含めない）
3. relation を triple、edge、statement object のどれで表すか（決定済み: statement object）
4. identity claim を必須にするか任意にするか（決定済み: 現段階では任意。後から profile / policy で必須化できる余地を残す）
5. relay は最小限の semantic validation を行うか、protocol object schema / carrier framing validation のみにするか（決定済み: schema / framing / carrier identity まで）
6. 最初の正規 carrier は relay-based pub/sub、HTTP publish、file/archive ingest のどれにするか（決定済み: HTTP publish API）
7. carrier capability を中央 registry で管理するか、完全分散にするか（決定済み: 完全分散）
8. application profile を中央 registry で管理するか、完全分散にするか（決定済み: 完全分散）
9. 多言語の正本をどう表現するか（決定済み: 1 object 1 language、translation は別 object + relation）
10. public relay が object を受け入れるための最小 trust model は何か（決定済み: 署名と形式が正しい public object を受け入れるが、内容の真偽は保証しない）
11. Toitoi profile の最小必須 object / relation set は `inquiry` / `observation` / `evidence` を必須、`synthesis` / `annotation` を拡張とし、relation は `asks` / `responds_to` / `supports` / `cites` を必須にする（決定済み）

補足:

- `delete` は core では tombstone 化のみとし、履歴を残したまま可視状態を変える方針を採用します
- 物理削除や scrub が必要な場合は、protocol core ではなく storage / operator policy の責務として扱います
- `private / encrypted object` は初期版には含めず、core は public object 前提で固めます
- 必要になった場合は、後段の application profile または policy 拡張として扱います
- `relation` は core では statement object として正規化し、`edge` は index / projection 側の派生構造とします
- `relay` は schema / framing / carrier identity の妥当性までを担い、semantic validation は canonicalization 層に寄せます
- `public relay` は署名と形式が正しい public object を受け入れるが、内容の真偽は保証しません
- 受け入れ範囲の詳細は運用ポリシーで絞れても、protocol semantic にはしません
- `carrier capability` と `application profile` は中央 registry を前提にせず、完全分散で配布・参照します
- 必要なら署名付き manifest、well-known endpoint、relay 上の discovery、indexer キャッシュで補助します
- public relay の trust model は、署名・形式・運用ポリシーに限定します
- `identity claim` は core では任意とし、後から application profile または policy で必須化できます
- 多言語の正本は 1 object 1 language とし、translation は別 object + relation で表します
- 最初の正規 carrier は HTTP publish API とし、ここから core の wire semantics を固定します

## 16. 次の推奨作業

このアーキテクチャに問題がなければ、次に書くべき文書は次です。

- プロトコル用語集
- canonical object 仕様
- [identity と provenance の仕様](../protocols/IDENTITY_AND_PROVENANCE.md)
- relay 運用モデル
- protocol-native carrier schema テンプレート
- Toitoi 的 inquiry object の最初の application profile

これらを書くと、この草案は実装可能なプロトコルスタックになります。
