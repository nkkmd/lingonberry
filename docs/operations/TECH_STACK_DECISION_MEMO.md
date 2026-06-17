# 技術選定メモ

**Status: draft** | **Last updated: 2026-06-17**

## 目的

この文書は、Lingonberry の `relay` と `storage node` を実装するための技術選定の考え方を整理します。  
ここでの役割は、最終決定を先に固定することではなく、**どこをどの基準で決めるべきか** を明確にすることです。

## 先に結論

現時点でのおすすめは次です。

- **core protocol / relay / storage node:** Rust
- **CLI / 運用ツール / 変換補助:** Rust か Go
- **Toitoi の application profile / edge UI:** Toitoi 側の既存スタックに合わせる

この選択は、`append-only`、`replay`、`canonicalization`、`identity`、`provenance` を安全に扱いやすいことを重視したものです。  
ただし、これはまだ最終決定ではなく、実装方針の候補です。

## 技術選定の軸

### 1. 正確性

Lingonberry は、同じ wire object から同じ canonical object を再構成できることが重要です。  
そのため、次を扱いやすい言語が向きます。

- 決定的な serialize / deserialize
- schema validation
- canonicalization
- append-only replay
- identity / provenance の厳密な取り扱い

### 2. 運用性

誰でも relay や storage node を立てられることを目標にするなら、次が重要です。

- 単一バイナリで配布できる
- 依存が少ない
- デプロイしやすい
- 容量や永続化の挙動が予測しやすい

### 3. 実装速度

最初の MVP を早く作ることも重要です。  
そのため、CLI や profile 周辺は、core よりも開発速度を優先してよいです。

### 4. 既存資産との接続

Toitoi が既に持っているコードベースや UI があるなら、その周辺は無理に言語を揃えない方がよい場合があります。  
core と edge を同じ言語で統一するより、**責務で分ける** 方が自然なこともあります。

## 候補比較

### Rust

向いている点:

- 型で不変条件を表しやすい
- 低レベルの永続化や wire 処理と相性がよい
- 1 バイナリで配布しやすい
- relay / storage node の中核に向く

注意点:

- 開発初速は Go や TypeScript より遅くなりやすい
- profile や UI まで全部 Rust に寄せる必要はない

### Go

向いている点:

- `relay` や `storage node` のようなサーバ実装を素早く作りやすい
- 配布と運用が簡単
- 並行処理やネットワークサービスとの相性がよい

注意点:

- 複雑な canonicalization や強い型制約では Rust より弱いことがある
- 細かな不変条件を設計で補う必要がある

### TypeScript / Node.js

向いている点:

- Toitoi の edge や UI と近いなら接続しやすい
- 実装速度が速い
- profile や API クライアントには便利

注意点:

- relay / storage node の基盤としては、長期運用時の厳密性を別途かなり意識する必要がある
- canonicalization や replay の核に置くかは慎重に決めた方がよい

## 推奨方針

### 推奨 1: core は Rust

次を最重要にするなら、core は Rust が第一候補です。

- canonicalization の厳密さ
- replay の決定性
- relay / storage node の安全性
- 将来の carrier 拡張

### 推奨 2: edge は Toitoi 側に合わせる

Toitoi が既に持つ UI、API、エディタ、ワークフローには、Toitoi 側の技術を使ってよいです。  
Lingonberry の core と edge を無理に一体化させる必要はありません。

### 推奨 3: CLI は最小限でよい

CLI は、`validate`、`inspect`、`replay`、`migrate` を支える用途に限定するとよいです。  
ここは core と同じ言語にしておくと、実装と保守が楽です。

## 具体的な構成案

### 案 A: Rust 中心

- `packages/protocol`: Rust
- `packages/core`: Rust
- `packages/codecs`: Rust
- `packages/relay`: Rust
- `packages/indexer`: Rust
- `packages/cli`: Rust
- `packages/api`: Rust でサーバ、または別の薄いフロント

評価:

- 正確性と配布性が高い
- 最初の実装コストはやや高い

### 案 B: Go 中心

- `packages/relay`: Go
- `packages/core`: Go
- `packages/codecs`: Go
- `packages/cli`: Go
- `packages/api`: Go

評価:

- 立ち上げが速い
- 運用しやすい
- ただし、複雑な型安全性は設計で補う必要がある

### 案 C: ハイブリッド

- core / relay / storage: Rust
- profile / UI / edge: Toitoi 側の言語
- CLI: Rust

評価:

- このリポジトリの目的にはかなり合う
- 責務分離が明確
- 学習コストと運用コストのバランスがよい

## どこで決めるか

最終決定は、次の順で行うのがよいです。

1. **Phase 0**
   - `relay` の検証範囲
   - 最初の正規 carrier
   - public relay の trust model
2. **Tech ADR**
   - core 言語
   - storage backend
   - CLI / API の実装分担
3. **Phase 1 実装開始**
   - 最初の publish 経路に必要な最小スタック

## 判断基準

次のどれを最優先するかで、選ぶ言語は変わります。

- **正確性を最優先する**: Rust
- **速度と運用の軽さを最優先する**: Go
- **Toitoi との近さを最優先する**: Toitoi 側の既存言語

## このメモの使い方

この文書は、まだ最終 ADR ではありません。  
実際には、Phase 0 の未決事項を埋めたうえで、次の 1 枚を作るのがよいです。

- `TECH_DECISION_ADR.md`

そこでは、`relay`、`storage node`、`CLI`、`API` について、採用理由と不採用理由を確定します。

