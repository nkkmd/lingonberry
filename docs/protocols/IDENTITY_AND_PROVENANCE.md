# Identity and Provenance

**Status: draft** | **Last updated: 2026-07-11**

## 目的

この文書は、Lingonberry における `identity key`、`identity claim`、`provenance`、`rawRef` の役割分担と、identity key v1 から v2 への移行規則を定義します。

## 1. Identity key

`identity key` は、carrier をまたいで同じ semantic object を比較するための照合キーです。Object ID や lineage identity の代替ではありません。

### 1.1 Rule versions

| Rule | Hash | Status |
|---|---|---|
| `lb.identity.key.v1` | FNV-1a 64-bit | legacy / compatibility |
| `lb.identity.key.v2` | SHA-256 | recommended |

v1 の出力は既存 object と fixture の検証に必要なため変更しません。新しく生成する identity claim では v2 を推奨します。

### 1.2 Derived basis

identity key は次の semantic field から導出します。

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
- `schemaVersion`
- `provenance`
- `rawRef`
- `identityClaims`
- `meta`

この分離により、保存場所、carrier、取得元、attestation が異なっても、semantic basis が同じ object を照合できます。

### 1.3 Canonicalization

導出 basis は [`lb.canonical.json.v1`](./CANONICALIZATION.md) により canonical UTF-8 bytes へ変換します。

```text
semantic fields
  -> lb.canonical.json.v1
  -> UTF-8 bytes
  -> versioned hash
  -> identity key
```

### 1.4 v1 encoding

```text
lb:key:lb.identity.key.v1:fnv1a64:<16 lowercase hex>
```

v1 は後方互換性のために維持しますが、暗号学的な衝突耐性を提供しません。

### 1.5 v2 encoding

```text
lb:key:lb.identity.key.v2:sha256:<64 lowercase hex>
```

v2 は canonical bytes の SHA-256 digest を使用します。

### 1.6 性質

- carrier-neutral
- relay URL に非依存
- storage hint に非依存
- deterministic
- rule version と hash algorithm を文字列から識別可能
- v2 は暗号学的衝突耐性を持つ

## 2. Migration

### 2.1 Generation

- 新規実装は v2 を生成する
- 既存 object の v1 claim を書き換えない
- migration のために新しい v2 claim を追加してよい
- v1 と v2 は同じ semantic basis を使用する

### 2.2 Verification

移行期間の verifier は次を行います。

1. `ruleVersion` と identity key prefix の整合を確認する
2. 宣言された rule version で identity key を再計算する
3. 未対応 rule version を「不一致」ではなく「unsupported」として扱えるようにする
4. 複数 claim がある場合は claim ごとに独立して検証する

### 2.3 Deprecation

v1 の読み取りサポートを停止する時期は、protocol version だけで暗黙に決めず、別の migration specification で定義します。

## 3. Identity claim

`identity claim` は、identity key と canonical ID の対応を示す検証可能な主張です。

### 3.1 必須項目

- `schemaVersion`
- `claimType`
- `ruleVersion`
- `identityKey`
- `canonicalId`
- `issuer`
- `issuedAt`
- `verification`

### 3.2 Rules

- `claimType` は `identity`
- `ruleVersion` は対応する identity key rule を示す
- identity key prefix の rule version と `ruleVersion` は一致する
- `canonicalId` は enclosing object の `id` と一致する
- `identityKey` は宣言された rule から導出した値と一致する
- `issuer` は publisher、relay、other attestor を表せる
- `verification` は署名や検証状態を保持できる

### 3.3 責務分離

- identity claim: semantic identity と canonical ID の対応に関する主張
- provenance: 出所と変換履歴
- rawRef: raw payload の再取得・監査参照

## 4. Provenance

`provenance` は、knowledge object がどこから来て、誰が主張し、どう変換されたかを記録します。

最低限、source、author / actor、observed / issued time、transform chain、verification state を表現できる必要があります。

provenance は identity key の導出 basis に含めません。

## 5. rawRef

`rawRef` は carrier 上の raw object または raw payload への参照です。

用途:

- 再取得
- 再 canonicalize
- 監査
- 再現

rawRef は provenance と別責務であり、identity key の導出 basis に含めません。

## 6. Validation

identity / provenance の検証では、少なくとも次を確認します。

- identity claim の `canonicalId` が object の `id` と一致する
- `ruleVersion` と identity key prefix が一致する
- 対応 rule で再計算した identity key と一致する
- `provenance` と `rawRef` が失われていない
- 可能な場合は `rawRef` から raw payload を再取得できる

## 7. Reference implementations

| Language | Location |
|---|---|
| Rust | `packages/identity/src/lib.rs` |
| JavaScript | `packages/identity/identity-key.mjs` |

共通 test vector は `conformance/identity-key-v2/` に置きます。

## 8. Existing fixtures

- [knowledge-object/with-identity-claim.json](../../fixtures/knowledge-object/with-identity-claim.json)
- [knowledge-object/invalid-identity-claim-mismatch.json](../../fixtures/knowledge-object/invalid-identity-claim-mismatch.json)
- [http-publish-request/with-identity-claim.json](../../fixtures/http-publish-request/with-identity-claim.json)
- [http-publish-request/invalid-identity-claim-mismatch.json](../../fixtures/http-publish-request/invalid-identity-claim-mismatch.json)
