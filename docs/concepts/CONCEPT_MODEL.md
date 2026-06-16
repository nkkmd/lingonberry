# 概念モデル

**Status: draft** | **Last updated: 2026-06-16**

## 目的

この文書は、Lingonberry の中核になる概念の関係を、最小限の形で整理します。

## 中心構造

```text
carrier identity
  ↓
protocol object
  ↓ validate / normalize
knowledge object
  ↓ index / API
canonical view
```

## 主要な分離

### 1. carrier と protocol

- carrier は protocol の wire 実装です
- protocol は意味とルールです
- carrier と protocol を分離しすぎません

### 2. raw と canonical

- raw は carrier 上の未検証表現です
- canonical は commons 内での正規表現です
- 両方を保持できるようにします

### 3. identity と provenance

- identity は「それが何か」を扱います
- provenance は「どう来たか」を扱います
- 役割を分けます

### 4. content と lineage

- content は知識の本体です
- lineage は知識の生まれ方や変化です
- 修正は上書きではなく lineage で表現します

## knowledge object の基本像

knowledge object は次の性質を持ちます。

- canonical である
- append-only である
- provenance を持つ
- raw reference を持つ
- relation と lineage を持てる
- domain-neutral である

## context の役割

context は、対象分野ごとの local な状況を抽象化するためのものです。

重要なのは、context が raw data の漏えい先にならないことです。

## application profile の役割

application profile は、core protocol を壊さずに分野ごとの差分を載せる層です。

例:

- Toitoi のような inquiry 中心の profile
- 医療研究向けの profile
- 法律知識向けの profile

## まとめ

Lingonberry の core は、知識を「保存する」だけでなく、**再構成できる形で循環させる**ことを目指します。
