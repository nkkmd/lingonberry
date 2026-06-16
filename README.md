# Lingonberry

**分散知識コモンズ・プロトコル**

Lingonberry は、分散的に運営されるリレー群のあいだで知識オブジェクトを循環させるためのプロトコルです。

これは、まず第一にソーシャルネットワークのプロトコルではありません。
**知識基盤のプロトコル** です。

目的は、誰でもサーバーを立て、知識オブジェクトを保存し、複製し、検索できるようにすることです。
しかも、その中核プロトコルを農業、医療、法律、教育、研究などの特定分野に縛りつけません。

## このリポジトリに含めるもの

- プロトコルの概念と用語
- 正規化されたデータモデル
- リレーの責務
- identity、provenance、revision の規則
- protocol-native な index と API 参照面
- ドメイン語彙とアプリケーション・プロファイルの拡張点

## 中核の考え方

Lingonberry は、知識を append-only で、replay 可能で、provenance を保持するものとして扱います。

wire 上の protocol object は正本です。
正本は canonical な knowledge object です。

WebSocket、HTTP、ファイル archive、将来の federated carrier は、同じ protocol を運ぶ carrier 実装です。

## まず読む場所

- [アーキテクチャ草案](./docs/architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md)
- [Toitoi 参照チェックリスト](./docs/architecture/TOITOI_REFERENCE_CHECKLIST.md)
- [概念](./docs/concepts/README.md)
- [Protocols](./docs/protocols/README.md)
- [Schemas](./schemas/README.md)
