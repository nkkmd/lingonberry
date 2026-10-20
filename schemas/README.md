# スキーマ案内

このディレクトリには、Lingonberryのプロトコル固有のJSON Schemaを置いています。知識オブジェクトの本体と、それをHTTPで公開するときの要求を定義します。

## 最初に確認する2つのスキーマ

| ファイル | 定義するもの |
|---|---|
| [knowledge-object.schema.json](./knowledge-object.schema.json) | 知識オブジェクト本体の構造 |
| [http-publish-request.schema.json](./http-publish-request.schema.json) | オブジェクト本体と、公開者の公開鍵・署名をまとめたHTTP公開要求 |

公開要求の`object`は、知識オブジェクトのスキーマを参照します。要求全体の構造と、内側のオブジェクトの両方を検証します。

## 知識オブジェクトの構造

`knowledge-object.schema.json`は、正規化の対象となる知識単位の最小構造と拡張点を定めます。現在のスキーマバージョンは`0.1.0`です。

### 必須フィールド

| フィールド | 意味 |
|---|---|
| `id` | プロトコル上の正規識別子。`lb:obj:`で始まる文字列 |
| `schemaVersion` | オブジェクトのスキーマバージョン。現在は`0.1.0` |
| `type` | 問い、観察、根拠など、知識オブジェクトの種別 |
| `createdAt` | UTCで表す作成時刻 |
| `body` | 本文の`text`と言語タグの`language` |
| `provenance` | 出所や発行者などの来歴。`sources`に少なくとも1件の参照を含む |
| `rawRef` | 受信した生データへの参照。`protocol`と`sourceId`を含む |

### 任意フィールド

| フィールド | 意味 |
|---|---|
| `contexts` | 抽象化した文脈情報 |
| `relations` | 他のオブジェクトとの意味上の関係 |
| `status` | オブジェクトの状態 |
| `lineage` | 派生、改訂、置換、翻訳などの履歴上の関係 |
| `identityClaims` | 識別キーと正規識別子の対応を示す、検証可能な主張 |
| `attachments` | 補助的な添付情報 |
| `labels` | 検索や分類のためのラベル |
| `meta` | 意味内容に含めない、実装上の補助情報 |

フィールドの型、使用できる値、追加フィールドの可否は、スキーマ本体で確認してください。

## HTTP公開要求の構造

`http-publish-request.schema.json`は、知識オブジェクトをHTTPで送るための外側の入れ物（request envelope）を定義します。

| フィールド | 意味 |
|---|---|
| `object` | 公開する知識オブジェクト本体 |
| `publisher.publicKey` | 公開者のEd25519公開鍵。32バイトを小文字の16進数64文字で表す |
| `publisher.signature` | 公開要求に対するEd25519署名。64バイトを小文字の16進数128文字で表す |

`publisher`はHTTP公開に必要な付帯情報であり、内側の知識オブジェクト本体には含まれません。

署名対象は、要求全体から`publisher.signature`だけを除き、`lb.canonical.json.v1`で正規化したUTF-8バイト列です。`object`だけを署名する方式ではありません。構造や符号化形式の検証に通っても、暗号学的に有効な署名であるとは限りません。署名対象の正確な規則は[HTTP公開要求の署名仕様](../docs/protocols/HTTP_PUBLISH_SIGNATURE.md)、生成手順は[公開クライアントの入門ガイド](../docs/developers/PUBLISHER_QUICKSTART.md)を参照してください。

## バージョンと変更時の確認

- `schemaVersion`は、オブジェクト本体が従う仕様のバージョンです。
- 公開要求のスキーマは外側の入れ物を定めるため、内側の`schemaVersion`とは別に文書としてバージョンを管理します。
- スキーマを変更するときは、[移行とスキーマのバージョン管理](../docs/operations/MIGRATION_AND_SCHEMA_VERSIONING.md)と、[正常例・不正例のサンプル](../fixtures/README.md)を併せて見直します。

初めて読む場合は、知識オブジェクトのスキーマで本体を理解してから、HTTP公開要求のスキーマで送信時の付帯情報を確認してください。
