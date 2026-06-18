# Identity and Provenance

**Status: draft** | **Last updated: 2026-06-18**

## 目的

この文書は、Lingonberry における `identity key`、`identity claim`、`provenance`、`rawRef` の役割分担を、実装可能な最小単位で固定します。

## 1. Identity key

`identity key` は、carrier をまたいで同じ semantic object を比較するための照合キーです。

### 1.1 Rule version

Phase 3 の初期実装では、`lb.identity.key.v1` を使います。

### 1.2 Derived basis

`identity key` は次の semantic field から導出します。

- `type`
- `createdAt`
- `body`
- `contexts`
- `relations`
- `status`
- `lineage`
- `attachments`
- `labels`

次は含めません。

- `id`
- `provenance`
- `rawRef`
- `identityClaims`
- `meta`

### 1.3 Encoding

導出 basis を canonical JSON にし、`fnv1a64` で fingerprint 化します。

形式は次の通りです。

```text
lb:key:lb.identity.key.v1:fnv1a64:<hex>
```

### 1.4 性質

- carrier-neutral である
- relay URL に依存しない
- storage hint に依存しない
- deterministic である

## 2. Identity claim

`identity claim` は、`identity key` と canonical id の対応を示す検証可能な主張です。

### 2.1 必須項目

- `schemaVersion`
- `claimType`
- `ruleVersion`
- `identityKey`
- `canonicalId`
- `issuer`
- `issuedAt`
- `verification`

### 2.2 追加ルール

- `claimType` は `identity`
- `ruleVersion` は `lb.identity.key.v1`
- `canonicalId` は enclosing object の `id` と一致する
- `identityKey` は導出済み `identity key` と一致する
- `issuer` は publisher / relay / other attestor を表せる
- `verification` は署名や検証状態を保持できる

### 2.3 位置づけ

`identity claim` は provenance ではありません。

- claim: 同一性の根拠
- provenance: 来歴と変換履歴

## 3. Provenance

`provenance` は、knowledge object がどこから来て、誰が主張し、どう変換されたかを記録します。

### 3.1 必須の観点

- source
- author / actor
- observed / issued time
- transform chain
- verification state

### 3.2 位置づけ

`provenance` は内容そのものではなく、履歴の情報です。

## 4. rawRef

`rawRef` は、carrier 上の raw object または raw payload への参照です。

### 4.1 役割

- 再取得
- 再 canonicalize
- 監査
- 再現

### 4.2 位置づけ

`rawRef` は provenance とは別責務です。

## 5. Validation

identity / provenance の検証では、少なくとも次を確認します。

- identity claim の `canonicalId` が object の `id` と一致する
- identity claim の `identityKey` が導出結果と一致する
- `provenance` と `rawRef` が失われていない
- `rawRef` から raw payload を再取得できる

## 6. Fixtures

- [knowledge-object/with-identity-claim.json](../../fixtures/knowledge-object/with-identity-claim.json)
- [knowledge-object/invalid-identity-claim-mismatch.json](../../fixtures/knowledge-object/invalid-identity-claim-mismatch.json)
- [http-publish-request/with-identity-claim.json](../../fixtures/http-publish-request/with-identity-claim.json)
- [http-publish-request/invalid-identity-claim-mismatch.json](../../fixtures/http-publish-request/invalid-identity-claim-mismatch.json)
