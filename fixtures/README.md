# Fixtures

このディレクトリには、protocol / schema / roadmap の説明で参照する最小サンプルを置きます。

## 文書で使うもの

- [knowledge-object/minimal-wire-object.json](./knowledge-object/minimal-wire-object.json)
- [knowledge-object/invalid-missing-rawref.json](./knowledge-object/invalid-missing-rawref.json)
- [http-publish-request/minimal-request.json](./http-publish-request/minimal-request.json)
- [http-publish-request/invalid-missing-signature.json](./http-publish-request/invalid-missing-signature.json)

## Validation の進め方

Phase 1 では、fixture を次の順で回して validate の土台を確認します。

1. `fixtures/knowledge-object/minimal-wire-object.json` を knowledge object として validate する
2. `fixtures/knowledge-object/invalid-missing-rawref.json` を不正例として reject する
3. `fixtures/http-publish-request/minimal-request.json` を publish request として validate する
4. `fixtures/http-publish-request/invalid-missing-signature.json` を不正例として reject する

CLI で確認する場合は、リポジトリルートから次のように実行します。

```bash
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/minimal-wire-object.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-missing-rawref.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-missing-signature.json
node packages/cli/lingonberry.mjs publish fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs get lb:obj:example-0001
node packages/cli/lingonberry.mjs list
```

publish の最小スキャフォールドは、`http-publish-request` を入力として受け取り、`object` を canonical 化した結果を返す形で進めます。
同一 `id` で内容が同じ場合は idempotent、内容が異なる場合は conflict として扱います。
