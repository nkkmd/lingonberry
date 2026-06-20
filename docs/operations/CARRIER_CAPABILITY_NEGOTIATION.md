# Carrier Capability Negotiation

**Status: draft** | **Last updated: 2026-06-20**

## 目的

この文書は、複数 carrier の差分を扱うための capability negotiation を定義します。

capability negotiation は、carrier ごとの違いを semantic model に持ち込まずに、どの実装がどこまで通るかを明示するための仕組みです。

## 原則

- capability は中央 registry に依存しない
- capability は carrier 固有の framing と option に閉じる
- capability は protocol semantic を上書きしない
- capability は advisory であり、意味論の source of truth ではない

## 参照面

capability は、次の場所から取得できる想定です。

- HTTP carrier の `GET /v1/capabilities`
- archive carrier の `manifest.json`
- 署名付き manifest
- relay 上の discovery endpoint

## 最小語彙

### protocol version

protocol の互換境界を表します。

### archive version

archive carrier の論理 layout と replay contract の version を表します。

### carrier kind

`http`、`archive`、`relay` のような carrier 種別を表します。

### supported object types

受け入れ可能な semantic type の一覧です。

### supported schema versions

validate 可能な schema version の一覧です。

### supported auth modes

publish / retrieve に使える認証方式です。

### supported content types

受け入れ可能な media type や framing です。

### replay support

archive や relay log から再構成可能かどうかを表します。

### supported archive versions

受け入れ可能な archive version の範囲です。

## Negotiation の進め方

1. client は自分の必須条件を列挙する
2. server は公開された capability を返す
3. 共通部分を取る
4. 必須条件が欠ける場合は fail closed にする
5. 成立した framing と version を明示する

## 判定ルール

- major protocol version が合わない場合は原則拒否する
- 必須 object type がない場合は拒否する
- replay が必要な場面で replay support がない場合は拒否する
- 必須 archive version がない場合は拒否する
- 互換性が曖昧な場合は、semantic translation ではなく拒否を優先する

## 期待する性質

- carrier が増えても semantic model は変わらない
- capability の不一致が早く見つかる
- offline / archive / HTTP の差分を説明できる

## 関連

- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [HTTP Carrier Contract](./HTTP_CARRIER_CONTRACT.md)
- [File / Archive Carrier Contract](./FILE_ARCHIVE_CARRIER_CONTRACT.md)
- [Distributed Knowledge Commons Architecture](../architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
