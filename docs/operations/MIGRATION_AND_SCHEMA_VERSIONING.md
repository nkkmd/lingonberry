# Migration and Schema Versioning

**Status: draft** | **Last updated: 2026-06-18**

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

### 1.3 Carrier version

carrier 固有の framing や response contract の version です。

- HTTP request / response contract
- archive layout
- discovery payload

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

## 3. Migration の適用先

### 3.1 Wire object

wire object の migration は、parse 時の schema compatibility と normalize 時の表現調整です。

### 3.2 Storage

storage migration は、raw log、canonical catalog、replay metadata の更新です。

### 3.3 Archive

archive migration は、manifest version と wire-log 互換性の更新です。

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

## 関連

- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Carrier Capability Negotiation](./CARRIER_CAPABILITY_NEGOTIATION.md)
- [Technical Decision ADR](./TECH_DECISION_ADR.md)
