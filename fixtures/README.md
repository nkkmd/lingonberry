# サンプルデータ案内

このディレクトリには、プロトコルやスキーマの説明、検証に使うJSONサンプルを置いています。正常例と不正例を比較すると、必須フィールドや識別情報の規則を確認できます。

## サンプルの基準

- 知識オブジェクトの`schemaVersion`は`0.1.0`です。
- 識別情報の主張の`schemaVersion`は`1`です。
- HTTP公開要求は、`object`に知識オブジェクトを入れ、`publisher`に公開者の公開鍵と署名を付けます。

構造の説明は[スキーマ案内](../schemas/README.md)、用語は[用語集](../docs/concepts/GLOSSARY.md)を参照してください。

## 主なサンプル

| 確認したいこと | 知識オブジェクト | HTTP公開要求 |
|---|---|---|
| 最小の正常例 | [minimal-wire-object.json](./knowledge-object/minimal-wire-object.json) | [minimal-request.json](./http-publish-request/minimal-request.json) |
| 識別情報の主張を含む例 | [with-identity-claim.json](./knowledge-object/with-identity-claim.json) | [with-identity-claim.json](./http-publish-request/with-identity-claim.json) |
| 識別情報が一致しない不正例 | [invalid-identity-claim-mismatch.json](./knowledge-object/invalid-identity-claim-mismatch.json) | [invalid-identity-claim-mismatch.json](./http-publish-request/invalid-identity-claim-mismatch.json) |
| 必須情報がない不正例 | [invalid-missing-rawref.json](./knowledge-object/invalid-missing-rawref.json) | [invalid-missing-signature.json](./http-publish-request/invalid-missing-signature.json) |
| スキーマバージョンが違う不正例 | [invalid-schema-version.json](./knowledge-object/invalid-schema-version.json) | [invalid-schema-version.json](./http-publish-request/invalid-schema-version.json) |

## 1. 保存せずに検証する

Git、Rustツールチェーン、Cargoを用意し、リポジトリのルートから実行します。`validate`は検証用のコマンドです。公開・保存の成功を示すものではありません。

まず正常例を確認します。

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/knowledge-object/minimal-wire-object.json
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/http-publish-request/minimal-request.json
```

次に不正例を確認します。以下は**検証に失敗することが期待されるコマンド**です。

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/knowledge-object/invalid-missing-rawref.json
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/knowledge-object/invalid-schema-version.json
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/http-publish-request/invalid-missing-signature.json
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/http-publish-request/invalid-schema-version.json
```

期待する結果は、正常例が検証に通り、必須情報の欠落や未対応のスキーマバージョンを含む例が拒否されることです。

## 2. 識別情報の対応を確認する

識別情報の主張を含む例を検証し、識別キーを導出します。

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/knowledge-object/with-identity-claim.json
cargo run -p lingonberry-relay --bin lingonberry-relay -- identity-key fixtures/knowledge-object/with-identity-claim.json
```

`identity-key`の結果には`canonicalId`と`identityKey`が含まれます。対応しない識別情報を含む次の例は、検証で拒否されることを確認します。

```bash
cargo run -p lingonberry-relay --bin lingonberry-relay -- validate fixtures/knowledge-object/invalid-identity-claim-mismatch.json
```

## 3. 署名付きサンプルを公開して取得する

公開処理では、構造の検証に加え、公開者の署名検証や受入方針、重複・競合の判定が行われます。付属の`minimal-request.json`は署名済みです。署名対象の内容を変更した場合は、署名を作り直す必要があります。

既存の作業データと混ざらないよう、新しい一時ディレクトリを作り、サブシェル内で作業します。`LINGONBERRY_STATE_DIR`は、そのサブシェル内だけで設定されます。終了後も試行データは一時ディレクトリに残ります。

```bash
(
  fixture_trial_dir=$(mktemp -d) || exit 1
  export LINGONBERRY_STATE_DIR="$fixture_trial_dir/state"
  cargo run -p lingonberry-relay --bin lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
  cargo run -p lingonberry-relay --bin lingonberry-relay -- get lb:obj:example-0001
  cargo run -p lingonberry-relay --bin lingonberry-relay -- raw lb:obj:example-0001
  cargo run -p lingonberry-relay --bin lingonberry-relay -- list
  cargo run -p lingonberry-relay --bin lingonberry-relay -- publish fixtures/http-publish-request/minimal-request.json
)
```

構成や受入方針によっては隔離・拒否になるため、公開コマンドの構造化された結果を確認してください。

- `stored`: 新しいオブジェクトを保存した
- `duplicate`: 同じ内容が既にあり、重複として成功した
- `deferred`: 隔離された。正本となるストレージには保存されていない
- `rejected`、`conflict`、`failed`: 拒否、競合、処理失敗。`code`と`errors`などで原因を確認する

正常に保存された場合は、`get`でオブジェクトを、`raw`で受信した要求の記録を、`list`で保存済みの識別子を確認できます。同じ署名付き要求の再送は重複として扱われ、同じオブジェクトを追加保存しません。

HTTP経由の公開や、再起動後の取得確認は[リポジトリ付属のサンプルを使った公開手順](../docs/developers/REPOSITORY_PUBLISH_WALKTHROUGH.md)を参照してください。独自のデータと署名を作る場合は[公開クライアントの入門ガイド](../docs/developers/PUBLISHER_QUICKSTART.md)を参照してください。

## 競合と署名の確認

同じ正規識別子に異なる内容を公開すると、競合として拒否されます。ただし、署名済みのJSONを書き換えただけでは署名が無効になるため、競合判定に到達しません。競合を試す場合も、変更後の内容に対する有効な署名が必要です。

## サンプルを変更するとき

- スキーマを変更したら、正常例と不正例の両方を見直します。
- 対応スキーマバージョンの案内を変えたら、返却される対応一覧と、バージョン不一致の不正例を揃えます。
- 公開要求の内容を変えたら、署名の再生成が必要か確認します。
- `validate`の成功と、公開時の署名検証・認可・保存の成功を区別します。
