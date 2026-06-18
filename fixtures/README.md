# Fixtures

このディレクトリには、protocol / schema / roadmap の説明で参照する最小サンプルを置きます。

## 文書で使うもの

- [knowledge-object/minimal-wire-object.json](./knowledge-object/minimal-wire-object.json)
- [knowledge-object/with-identity-claim.json](./knowledge-object/with-identity-claim.json)
- [knowledge-object/invalid-identity-claim-mismatch.json](./knowledge-object/invalid-identity-claim-mismatch.json)
- [knowledge-object/invalid-missing-rawref.json](./knowledge-object/invalid-missing-rawref.json)
- [http-publish-request/minimal-request.json](./http-publish-request/minimal-request.json)
- [http-publish-request/with-identity-claim.json](./http-publish-request/with-identity-claim.json)
- [http-publish-request/invalid-identity-claim-mismatch.json](./http-publish-request/invalid-identity-claim-mismatch.json)
- [http-publish-request/invalid-missing-signature.json](./http-publish-request/invalid-missing-signature.json)

## Validation の進め方

Phase 1 では、fixture を次の順で回して validate の土台を確認します。

1. `fixtures/knowledge-object/minimal-wire-object.json` を knowledge object として validate する
2. `fixtures/knowledge-object/invalid-missing-rawref.json` を不正例として reject する
3. `fixtures/http-publish-request/minimal-request.json` を publish request として validate する
4. `fixtures/http-publish-request/invalid-missing-signature.json` を不正例として reject する

Phase 3 では、identity claim を含む fixture も追加して、identity key と canonical id の対応を確認します。

CLI で確認する場合は、リポジトリルートから次のように実行します。

```bash
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/minimal-wire-object.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-missing-rawref.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/with-identity-claim.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-identity-claim-mismatch.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-missing-signature.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/with-identity-claim.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-identity-claim-mismatch.json
node packages/cli/lingonberry.mjs publish fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs get lb:obj:example-0001
node packages/cli/lingonberry.mjs list
node packages/cli/lingonberry.mjs identity-key fixtures/knowledge-object/with-identity-claim.json
```

publish の最小スキャフォールドは、`http-publish-request` を入力として受け取り、`object` を canonical 化した結果を返す形で進めます。
同一 `id` で内容が同じ場合は idempotent、内容が異なる場合は conflict として扱います。

## Phase 1 の手動テスト

Phase 1 の動作確認は、リポジトリルートから次の順で行えます。

```bash
rm -rf .lingonberry
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/minimal-wire-object.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-missing-rawref.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/with-identity-claim.json
node packages/cli/lingonberry.mjs validate fixtures/knowledge-object/invalid-identity-claim-mismatch.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/minimal-request.json
node packages/cli/lingonberry.mjs validate fixtures/http-publish-request/invalid-missing-signature.json
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
