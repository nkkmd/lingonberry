# Carrier

**Status: draft** | **Last updated: 2026-06-16**

## 目的

この文書は、Lingonberry における `carrier` の意味を、protocol と同一視する前提で定義します。

## 定義

carrier とは、protocol object を wire 上で運ぶための実装です。

このリポジトリでは、carrier は protocol の外側にある変換対象ではなく、protocol の正規な実装形です。

## carrier の責務

- protocol object を受け取る
- wire-level の構造を保持する
- 必要なら framing、署名、再送、順序保証を提供する
- carrier identity を付与する
- storage と replay を支える

## carrier がやらないこと

- semantic を carrier ごとに別物へ翻訳すること
- protocol の意味を carrier 固有の都合で変更すること
- canonical object と wire object を別プロトコルとして扱うこと

## 主要な識別子

### carrier identity

carrier が内部で使う識別子です。

例:

- relay event id
- record URI
- archive object key
- stream offset

### protocol object

carrier 上に表現された protocol の実体です。

知識オブジェクトは、carrier によって意味が変わるのではなく、同じ意味を保ったまま表現されるべきです。

## 設計原則

- carrier と protocol を分離しすぎない
- carrier の違いは framing と capability に閉じる
- semantic は protocol object 側に置く
- wire と canonical は、別プロトコルではなく別表現として扱う

## 関連

- [概念モデル](./CONCEPT_MODEL.md)
- [用語集](./GLOSSARY.md)
- [protocol-native wire format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
