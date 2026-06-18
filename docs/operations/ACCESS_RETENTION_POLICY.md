# Access and Retention Policy

**Status: draft** | **Last updated: 2026-06-18**

## 目的

この文書は、Lingonberry における access policy と retention policy の運用境界を整理します。

ここで扱う policy は protocol semantic ではありません。  
carrier、relay、storage node、operator が運用上どう扱うかを定義する補助層です。

## 原則

- access policy は公開範囲を決める
- retention policy は保存期間と削除手順を決める
- 物理削除や scrub は protocol core の責務ではない
- private / encrypted object は core の初期版に含めない
- public / curated / private の区分は運用ポリシーとして扱う

## 1. Access policy

Access policy は、どの object をどの carrier / relay / API が受け付けるかを決めます。

### 1.1 Public

- 署名と形式が正しい public object を受け入れる
- 内容の真偽は保証しない
- 原則として canonical view を公開できる

### 1.2 Curated

- 特定 type や特定 profile の object のみ受け入れる
- 運用者またはコミュニティの curation rule を適用できる
- protocol semantic ではなくローカル運用の判断として扱う

### 1.3 Private

- 限定メンバーにのみ公開する
- 暗号化オブジェクトや非公開配布を許容する場合がある
- ただし、core protocol の初期版では private / encrypted object を前提にしない

## 2. Retention policy

Retention policy は、保存の寿命と削除相当の扱いを決めます。

### 2.1 基本方針

- append-only を壊さない
- replay 可能性を壊さない
- canonicalization の再実行に必要な情報を残す

### 2.2 保持対象

少なくとも次を retention の対象として扱います。

- raw log
- canonical catalog
- replay metadata
- archive manifest

### 2.3 削除の扱い

- `delete` は tombstone 化として扱う
- 物理削除は protocol core の意味論にしない
- scrub が必要な場合は storage / operator policy として実施する

### 2.4 既定の運用方針

初期運用では、次の考え方を既定とします。

- public object は基本的に永続保存を前提にする
- curated object は運用者が retention を短く設定してもよい
- private object を扱う場合は、core ではなく policy / carrier 側で別扱いにする
- export 可能性は削らない

### 2.5 退役と移行

storage node や relay を退役させる場合は、次を満たすようにします。

- export で archive を作れる
- archive から replay できる
- retention に基づく削除と、operator の都合による退役を区別する

## 3. Authentication / Authorization

Authentication / authorization は、必要なら運用層で定義します。

- protocol core では必須要素にしない
- HTTP carrier では将来の拡張点として扱える
- carrier ごとの差分は policy と capability に閉じる

## 4. 運用モデル

### Public relay

- public object を受け付ける
- 署名と形式の妥当性を確認する
- 内容の真偽は保証しない

### Curated relay

- public relay より狭い受け入れ条件を持てる
- access policy を運用上の許可条件として使う

### Archive / storage

- 長期保存を前提に retention を設定する
- replay のための情報を残す
- 退役時も export 可能性を考える

## 5. 判定ルール

- `public / curated / private` の分類は protocol object の semantic ではない
- retention の設定差は carrier capability と operator policy で表す
- authn/authz の有無は protocol compatibility とは分ける

## 6. carrier ごとの既定値

### 6.1 HTTP carrier

- default access: public
- curated: 任意で有効化できる
- private: core の初期版では既定で無効
- default retention: public object は永続保存を基本とする
- authn/authz: 必須ではなく、必要なら運用層で追加する

### 6.2 file / archive carrier

- default access: export/import 可能な object を広く受ける
- curated: 取り込み時の制約として扱える
- private: policy / carrier 側で別扱いにする
- default retention: archive 自体は長期保管を前提にする
- scrub: export 時に operator policy で適用可

### 6.3 relay / storage carrier

- default access: public relay は public object を受ける
- curated: curated relay として狭められる
- private: 初期版では別運用として切る
- default retention: replay 可能性を壊さない範囲で長期保存する
- scrub: storage / operator policy でのみ実施する

## 7. 未決事項

次は実装と運用に応じて詰めます。

1. carrier 別の access policy 表現
2. retention の既定値
3. export 時の scrub 方針
4. authn/authz をどこまで共通化するか
5. public / curated / private の具体的な carrier 既定値

## 関連

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
