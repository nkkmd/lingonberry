# Migration and Schema Versioning

**Status: draft** | **Last updated: 2026-06-20**

## 目的

この文書は、Lingonberry における migration と schema versioning の運用方針を定義します。

ここでの migration は、protocol semantic を壊さずに wire / storage / archive の表現を更新していくための手順です。  
schema versioning は、その更新を追跡可能にするための version contract です。

## 原則

- schema version は versioned である
- migration は決定的である
- canonicalization と replay を壊さない
- carrier ごとの差分は framing と capability に閉じる
- schema の更新は protocol semantic の更新と同じではない

## 0. 現行ベースライン

現在の参照点は次の通りです。

- `knowledge-object.schema.json` の `schemaVersion` は `0.1.0`
- `http-publish-request.schema.json` は request envelope として別 schema document で管理する
- `identityClaim.schemaVersion` は `1` で、object schema 内の別契約として扱う

この文書では、`schemaVersion` を payload 側の contract version として扱い、`$id` や file path の version は schema document の所在を示す補助情報として扱います。

### 0.1 照合項目

schema version の baseline を更新するときは、次を同時に見直します。

- `schemas/knowledge-object.schema.json`
- `schemas/http-publish-request.schema.json`
- `schemas/README.md`
- `fixtures/README.md`
- `fixtures/knowledge-object/*.json`
- `fixtures/http-publish-request/*.json`

## 1. Version 層

### 1.1 Protocol version

protocol 全体の互換境界です。

- wire semantics の大きな変化を表す
- carrier 間の互換性判断に使う
- backwards compatibility の前提を明示する

### 1.2 Schema version

個別 schema の contract を表します。

- `knowledge-object` schema の version
- `http-publish-request` schema の version
- archive manifest の version
- capability manifest の version

schema version は原則として semver 相当の bump で扱います。

- backward-compatible な追加は minor bump
- breaking change は major bump
- 説明文の補足や例の更新だけでは bump しない

判断例:

- `contexts` に新しい任意キーを足しても、既存 object がそのまま通るなら minor bump
- `rawRef` を必須から外す、または `type` の enum を縮めるなら major bump

`schemaVersion` を payload に持たない envelope は、文書単位の version として `$id` と file path を追跡します。

### 1.3 Carrier version

carrier 固有の framing や response contract の version です。

- HTTP request / response contract
- archive layout
- discovery payload

### 1.4 Archive version

archive carrier の version です。

- archive manifest の contract
- archive layout の contract
- replay 互換の境界

## 2. Migration policy

### 2.1 変換の原則

- validate -> normalize -> finalize の順を壊さない
- 変換で semantic を足しすぎない
- lossless でない migration は明示する

### 2.2 破壊的変更

破壊的変更を入れる場合は、次を明示します。

- どの version からどの version へ移すか
- 既存 object を replay できるか
- canonical id / identity key に影響があるか
- rawRef / provenance に影響があるか

### 2.3 後方互換

後方互換を維持する場合は、古い object を受け入れたあとに新しい canonical 表現へ正規化します。

- 旧 schema を受け入れる
- normalize で新しい representation に揃える
- finalize 後の canonical state は新しい contract に従う

後方互換として許容するのは、原則として次のような変更です。

- 任意 field の追加
- 既存 field の意味を壊さない補助情報の追加
- validate で古い表現と新しい表現の両方を受けられる調整

許容しない変更は、次のようなものです。

- 必須 field の削除
- 既存 field の意味変更
- enum の縮小
- replay や provenance を壊す表現変更

## 3. Migration の適用先

### 3.1 Wire object

wire object の migration は、parse 時の schema compatibility と normalize 時の表現調整です。

wire object の変更は、最初に schema で受けられるかを確認し、次に normalize で canonical 表現へ寄せます。

### 3.2 Storage

storage migration は、raw log、canonical catalog、replay metadata の更新です。

storage migration は、wire object の contract と独立に、保存形式の更新として扱います。

### 3.3 Archive

archive migration は、manifest version と wire-log 互換性の更新です。

archive migration は、archive bundle の再投入と replay が壊れないことを最優先にします。

## 4. Replay 要件

migration は replay を壊してはいけません。

- 古い archive から canonical state を再構成できること
- 変換履歴を追えること
- `rawRef` が有効であること
- provenance が保持されること

## 5. Capability との関係

carrier capability は、利用可能な version と migration 境界を公開するために使えます。

返すべき情報の例:

- supported protocol version
- supported schema versions
- supported archive versions
- supported migration path
- archive version に対する互換境界

## 6. 運用手順

1. 互換境界を version で明示する
2. 新旧 version の両方を受ける期間を決める
3. normalize で canonical 表現へ寄せる
4. replay で旧 archive を再構成できるか確認する
5. 非互換のタイミングを capability に反映する

## 7. 未決事項

次は実装と運用の進行に応じて詰めます。

1. protocol major version の上げ方
2. schema version の命名規則
3. archive migration の保持期間
4. deprecated schema の受け入れ終了条件

### 補足方針

- protocol major version は semantic change の互換境界に使う
- schema version は individual schema の contract 変更に使う
- archive migration の保持期間は policy と storage cost を見て決める
- deprecated schema の受け入れ終了は capability と runbook に反映する

### deprecated schema の終了条件

deprecated schema を終了する場合は、次の順で運用します。

1. capability で deprecated 状態を明示する
2. 既存 client に移行期間を与える
3. validate では受けても、finalize では新しい contract に寄せる期間を決める
4. 移行期間が終わったら `supported schema versions` から削除する
5. 削除後は旧 version を fail closed にする

終了時には、少なくとも次を確認します。

- 旧 version の publish / import が残っていない
- replay と archive 再投入に必要な互換性が保たれている
- capability と runbook が同じ結論を返す
- 旧 version を受ける理由が policy 上も残っていない

## 関連

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Technical Decision ADR](./TECH_DECISION_ADR.md)
