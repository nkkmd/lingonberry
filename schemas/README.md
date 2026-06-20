# Schemas

このディレクトリには、Lingonberry の protocol-native な JSON Schema を置きます。

## 取り扱い

- `schemaVersion` は payload 側の contract version として扱います
- `knowledge-object.schema.json` は現行 baseline として `0.1.0` を使います
- `http-publish-request.schema.json` は request envelope なので、payload の `schemaVersion` とは別に document として version 管理します
- 変更時は [Migration and Schema Versioning](../docs/operations/MIGRATION_AND_SCHEMA_VERSIONING.md) と fixtures を同時に見直します

## 文書

- [knowledge-object.schema.json](./knowledge-object.schema.json)
- [http-publish-request.schema.json](./http-publish-request.schema.json)
