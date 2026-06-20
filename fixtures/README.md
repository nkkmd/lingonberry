# Fixtures

このディレクトリには、protocol / schema / roadmap の説明で参照する最小サンプルを置きます。

## Version baseline

現在の fixture は、次の参照点に合わせています。

- `knowledge-object.schema.json` の `schemaVersion: 0.1.0`
- `identityClaim.schemaVersion: 1`
- `http-publish-request.schema.json` は request envelope として object を包む

schema 変更時は、正例 fixture と不正例 fixture を同時に見直します。
HTTP carrier の capability discovery を調整した場合は、`supported schema versions` の返却と `invalid-schema-version` 系 fixture を合わせて見直します。
`invalid-schema-version` 系 fixture は、`publish` に進む前の `validate` で reject されることを確認するためのものです。

## 文書で使うもの

- [knowledge-object/minimal-wire-object.json](./knowledge-object/minimal-wire-object.json)
- [knowledge-object/with-identity-claim.json](./knowledge-object/with-identity-claim.json)
- [knowledge-object/invalid-identity-claim-mismatch.json](./knowledge-object/invalid-identity-claim-mismatch.json)
- [knowledge-object/invalid-missing-rawref.json](./knowledge-object/invalid-missing-rawref.json)
- [knowledge-object/invalid-schema-version.json](./knowledge-object/invalid-schema-version.json)
- [http-publish-request/minimal-request.json](./http-publish-request/minimal-request.json)
- [http-publish-request/with-identity-claim.json](./http-publish-request/with-identity-claim.json)
- [http-publish-request/invalid-identity-claim-mismatch.json](./http-publish-request/invalid-identity-claim-mismatch.json)
- [http-publish-request/invalid-missing-signature.json](./http-publish-request/invalid-missing-signature.json)
- [http-publish-request/invalid-schema-version.json](./http-publish-request/invalid-schema-version.json)

## Validation の進め方

最初の validate 段階では、fixture を次の順で回して土台を確認します。

1. `fixtures/knowledge-object/minimal-wire-object.json` を knowledge object として validate する
2. `fixtures/knowledge-object/invalid-missing-rawref.json` を不正例として reject する
3. `fixtures/knowledge-object/invalid-schema-version.json` を schema version 不一致として reject する
4. `fixtures/http-publish-request/minimal-request.json` を publish request として validate する
5. `fixtures/http-publish-request/invalid-missing-signature.json` を不正例として reject する
6. `fixtures/http-publish-request/invalid-schema-version.json` を schema version 不一致として reject する

identity 実用化の段階では、identity claim を含む fixture も追加して、identity key と canonical id の対応を確認します。
schema 版管理の段階では、`schemaVersion` の mismatch が validate で拒否されることも合わせて確認します。

CLI で確認する場合は、リポジトリルートから次のように実行します。

```bash
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/minimal-wire-object.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-missing-rawref.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-schema-version.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/with-identity-claim.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-identity-claim-mismatch.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-missing-signature.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-schema-version.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/with-identity-claim.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-identity-claim-mismatch.json
node packages/cli/lingonberry.mjs publish fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs get lb:obj:example-0001
node packages/cli/lingonberry.mjs list
node packages/cli/lingonberry.mjs identity-key fixtures/knowledge-object/with-identity-claim.json
```

publish の最小スキャフォールドは、`http-publish-request` を入力として受け取り、`object` を canonical 化した結果を返す形で進めます。
同一 `id` で内容が同じ場合は idempotent、内容が異なる場合は conflict として扱います。
schema version mismatch は publish まで進めず、validate 段階で落とすのを基本にします。

## 最初の手動テスト

最初の動作確認は、リポジトリルートから次の順で行えます。

```bash
rm -rf .lingonberry
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/minimal-wire-object.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-missing-rawref.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-schema-version.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/with-identity-claim.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-identity-claim-mismatch.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-missing-signature.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-schema-version.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/with-identity-claim.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-identity-claim-mismatch.json
node packages/cli/lingonberry.mjs publish fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs get lb:obj:example-0001
node packages/cli/lingonberry.mjs list
node packages/cli/lingonberry.mjs identity-key fixtures/knowledge-object/with-identity-claim.json
node packages/cli/lingonberry.mjs publish fixtures/http-publish-request/minimal-request.json
```

期待結果:

- 正常系の validate が通る
- 不正系の validate が reject される
- schema version mismatch の不正例が reject される
- schema version mismatch は publish に進まず validate で止まる
- publish 後に `get` で canonical view を再取得できる
- `list` で保存済み ID を確認できる
- 同一内容の再 publish は idempotent に扱われる
- identity claim を含む fixture が validate できる
- identity key を CLI で導出できる
- mismatch の identity claim が reject される

## Conflict の確認

同じ `id` で内容だけ変えた JSON を作って `publish` すると、conflict として reject されることを確認できます。

期待結果:

- `LB_OBJECT_CONFLICT` が返る
- `object already exists with different content` を含むエラーになる
