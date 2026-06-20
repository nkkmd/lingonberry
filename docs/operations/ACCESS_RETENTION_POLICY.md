# Access and Retention Policy

**Status: draft** | **Last updated: 2026-06-20**

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
- carrier ごとの既定値は capability と contract で公開し、semantic には持ち込まない

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

### 1.4 Carrier への適用

access policy は carrier ごとの既定値として公開できますが、どの carrier でも同じ意味ではありません。

- HTTP carrier は公開探索と publish の入口として public を既定にする
- archive carrier は export / import の入口として public object を広く受け、private / curated は policy で制御する
- relay / storage carrier は replay 可能性を壊さない範囲で public を既定にする
- curated は運用者が狭めるための上書きであり、protocol semantic ではない
- private は初期版では別運用として扱い、core の必須条件にしない

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
- 退役時は少なくとも `manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json`、`resolved-config.json` を保持する
- `tempDir` 配下の一時ファイルや再生成可能なキャッシュは retention ではなく operator policy で削除できる

### 2.6 backup と restore

- backup は、restore 可能性を失わない保存単位として扱う
- backup の中核は raw log、canonical catalog、replay metadata、archive manifest です
- backup bundle では、`manifest.json`、`wire-log.jsonl`、`canonical-catalog.sqlite3`、`replay-metadata.json` を基本要素として扱う
- restore では、backup の内容を上書き修正せず、再構成のための入力として扱う
- backup と archive は別概念だが、運用上は archive を backup の運搬形式として使える
- backup の保持期間は retention policy に従い、operator policy で短縮してもよいが、replay 可能性は壊さない
- backup の作成中に一時生成した退避物は、完了後に `backupDir` へまとめる

### 2.7 監査時の確認項目

監査時は、少なくとも次を確認します。

1. access scope の既定値が carrier ごとに説明できる
2. retention hint の既定値が carrier ごとに説明できる
3. raw log、canonical catalog、replay metadata、archive manifest の保持方針が一致している
4. scrub が operator policy に閉じている
5. backup / restore / retirement と export / import の責務が分かれている
6. authn/authz が必要になった場合の注入経路が secret management と分離されている

### 2.8 監査の参照順

監査や変更確認では、次の順で文書を突き合わせます。

1. [Access and Retention Audit Checklist](./ACCESS_RETENTION_AUDIT_CHECKLIST.md) で実行項目を確認する
2. [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md) で運用方針を確認する
3. [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md) と [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md) で carrier ごとの公開値を確認する
4. [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md) で export / import と scrub の扱いを確認する
5. [Node Lifecycle Runbook](./NODE_LIFECYCLE_RUNBOOK.md) で backup / restore / retirement の手順を確認する
6. [Secret Management](./SECRET_MANAGEMENT.md) で authn/authz の注入経路を確認する

## 3. Authentication / Authorization

Authentication / authorization は、必要なら運用層で定義します。

- protocol core では必須要素にしない
- HTTP carrier では将来の拡張点として扱える
- carrier ごとの差分は policy と capability に閉じる
- secret の保管と注入は [Secret Management](./SECRET_MANAGEMENT.md) に分離する

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
- backup / restore の実行時は、archive manifest と raw log の整合性を優先する

## 5. 判定ルール

- `public / curated / private` の分類は protocol object の semantic ではない
- retention の設定差は carrier capability と operator policy で表す
- authn/authz の有無は protocol compatibility とは分ける
- 監査の基準は policy 文書、carrier contract、runbook の 3 点を突き合わせて確認する

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
- differential export は既定ではなく、必要な場合にのみ operator policy で採用する

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
6. backup / restore の運用上の既定値
7. 監査のチェックリストを runbook に分離するかどうか
8. 監査の参照順を別紙に切り出すかどうか

## 関連

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [Secret Management](./SECRET_MANAGEMENT.md)
