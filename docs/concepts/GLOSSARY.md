# 用語集

**Status: draft** | **Last updated: 2026-06-19**

この文書は、Lingonberry で頻出する概念の意味をそろえるための共通辞書です。

## knowledge object

- 知識の流通単位です。
- append-only で扱います。
- canonical identity を持ちます。
- provenance と raw reference を保持します。
- 特定分野に依存しない、再利用可能な意味単位です。
- carrier 上では protocol object として表現されます。
- 初期仕様の最小必須構造は `id`, `schemaVersion`, `type`, `createdAt`, `body`, `provenance`, `rawRef` です。

## canonical identity

- プロトコル内部で使う主識別子です。
- carrier ごとの id とは役割を分けて扱います。
- semantic identity を表します。

## wire

- protocol object が carrier 上で受け渡される生の表現です。
- canonical 化の前後をまたいで再解析や再構成の対象になります。
- raw reference の参照先として扱います。

## carrier

- protocol object を wire 上で運ぶ正規の実装です。
- relay、HTTP、file/archive など、具体的な transport / framing を担います。
- routing、serialization、retry、ordering などの carrier 固有差分を含みます。

## carrier identity

- relay、record、archive など carrier 固有の識別子です。
- ルーティング、重複排除、再取得に使います。
- semantic identity とは別です。

## identity key

- canonical identity を解決するための照合キーです。
- 決定的で再計算可能であることを目指します。
- carrier endpoint には依存しません。

## identity claim

- identity key と canonical identity の対応を示す検証可能な主張です。
- 第三者が検証できることを目指します。
- core では必須に固定せず、identity 実用化の段階で扱います。

## provenance

- ある knowledge object が、どこから来たか、誰が主張したか、どう変換されたかを示す来歴です。
- provenance は内容そのものではなく、履歴の情報です。

## raw reference

- carrier 上の raw object または raw payload への参照です。
- 再取得、再 canonicalize、監査のために使います。

## lineage

- オブジェクト同士の派生関係や修正関係です。
- `derived_from`、`revises`、`supersedes`、`translates` などを表します。

## context

- オブジェクトが成立した状況や文脈の抽象化です。
- 生データそのものではありません。
- 分野ごとに語彙を差し替えられるべきです。

## relation

- knowledge object 間の意味的なつながりです。
- 検索やグラフ探索の基礎になります。

## commons

- 分散的に保存・配信・再構成される知識基盤全体です。
- 単なる保管庫ではなく、知識の継承と循環の仕組みです。

## relay

- knowledge object を受け取り、保存し、配信するノードです。
- protocol carrier としての責務を担います。

## indexer

- canonical object から検索可能な構造を派生させる仕組みです。
- semantic source そのものにはなりません。

## canonical view

- raw carrier 表現ではなく、共通の見方で整えた参照面です。
- API や UI が扱うための表示形です。

## application profile

- 特定分野向けに、core protocol の上へ載せる追加ルールのまとまりです。
- 例: Toitoi のような分野特化アプリケーション。
