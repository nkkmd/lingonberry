# 技術決定 ADR

**Status: draft** | **Last updated: 2026-06-19**

## 目的

この文書は、Lingonberry の `relay`、`storage node`、`API`、`CLI` に関する技術選定を最終決定する ADR です。  
あわせて `storage backend` の方針も扱い、実装境界を 1 枚にまとめます。

## 前提

Lingonberry は次を満たす必要があります。

- 誰でも `relay` を立てられる
- 誰でも `storage node` を立てられる
- 誰でも `knowledge object` を publish できる
- append-only と replayable を壊さない
- canonicalization と provenance を決定的に扱う

そのため、実装言語と基盤技術には、単なる開発速度だけでなく、保存・再構成・検証の厳密さが求められます。

## 決定

### 採用するもの

- **core protocol / relay / storage node:** Rust
- **CLI:** Rust
- **API サーバ:** Rust
- **最初の正規 carrier:** HTTP publish API
- **Toitoi の application profile / edge UI:** Toitoi 側の既存スタック
- **raw log:** local filesystem
- **canonical catalog:** SQLite
- **derived index:** SQLite

### 保留するもの

- `indexer` の内部実装
  - core と独立した派生構造として扱い、後で最適化する
- `storage backend` の詳細パラメータ
  - log record の具体フォーマット
  - snapshot の間隔
  - retention policy
  - archive relay で object storage を使うかどうか
  - PostgreSQL へ移行する条件
- `access / retention policy` の詳細
  - carrier ごとの公開境界
  - retention の既定値
  - export 時の scrub 方針

### 採用しないもの

- `relay` と `storage node` の中核を TypeScript/Node.js に置く案
  - 実装速度は魅力だが、長期運用での厳密性と replay の統制を優先した
- core を Toitoi 側の技術に寄せ切る案
  - Toitoi を application profile に留める設計方針と合わないため
- PostgreSQL を MVP の default storage backend にする案
  - 将来の選択肢としては残すが、初期運用には重い

## 決定理由

### 1. canonicalization を厳密に扱うため

Lingonberry の core では、同じ wire object から同じ canonical object を再構成できることが重要です。  
Rust は型と所有権の仕組みを使って、不変条件を保ちやすいです。

### 2. relay / storage node の運用性を確保するため

誰でもノードを立てられることが目標なので、配布しやすく、依存が少ない実行形が向いています。  
Rust は単一バイナリ運用と相性がよく、server runtime としても扱いやすいです。

### 3. replay と provenance を壊しにくいため

append-only log、identity resolution、rawRef、provenance の取り扱いは、後から曖昧にすると取り返しがつきにくいです。  
core を厳密に実装できる言語を最初に選ぶ方が安全です。  
storage も raw log と catalog を分けて持つと、再構成の見通しがよくなります。

### 4. Toitoi を application profile として保つため

Toitoi は重要な利用例ですが、core protocol そのものではありません。  
edge や UI は Toitoi 側の都合に合わせ、core は Lingonberry の共通基盤として保ちます。

### 5. MVP の storage を軽く保つため

filesystem-first + SQLite catalog は、replay 可能性、運用の軽さ、実装しやすさのバランスがよいです。  
raw log、canonical store、derived index を分けやすく、初期の publish 経路を小さく始められます。

## 実装範囲

### Rust で持つもの

- `packages/protocol/`
- `packages/core/`
- `packages/codecs/`
- `packages/relay/`
- `packages/indexer/`
- `packages/api/`
- `packages/cli/`

### storage backend で持つもの

- raw wire log
- canonical object catalog
- derived index
- replay metadata
- snapshot / compaction

### Toitoi 側に残すもの

- UI
- application profile
- domain-specific vocabulary
- domain-specific curation / trust rule

## 代替案

### 代替案 A: Go 中心

Go は、`relay` や `storage node` の初速を上げやすいです。  
ただし、Lingonberry では canonicalization、identity、provenance、replay の決定性を強く求めるため、今回の ADR では第一選択にしません。

### 代替案 B: TypeScript/Node.js 中心

TypeScript は Toitoi 側との接続がしやすい一方で、core の保存・再構成・検証の中核に置くには、運用上の注意が増えます。  
そのため、API クライアントや edge では有用でも、relay / storage node の中心技術にはしません。

### 代替案 C: PostgreSQL を default にする

複数プロセスや強い query 要件には向きますが、MVP では重くなりやすいです。  
まずは filesystem + SQLite で始め、必要時に移行条件を明確化します。

## 影響

### 良い影響

- 実装の不変条件をコードで守りやすい
- relay と storage node の責務が明確になる
- replay と canonicalization のテストが組みやすい
- 長期運用での事故を減らしやすい
- storage backend の初期運用が軽い

### 注意点

- 初期実装の学習コストはやや高い
- Toitoi 側の技術スタックとの境界設計が必要
- carrier の拡張順や追加 carrier の要件は別文書で詰める必要がある

## 関連する未決事項

この ADR で決めても、次の点はまだ別途決める必要があります。

1. `storage backend` の詳細パラメータ
2. `indexer` の保存方式
3. `api` の公開範囲
4. `access / retention policy` の carrier 別既定値

## 実施計画

1. 初期の未決事項を確定する
2. Rust で最小 publish 経路を実装する
3. relay / storage node を分離する
4. identity / provenance を実用化する
5. access / retention policy を運用層として切り出す
6. 必要なら Go や TypeScript を周辺ツールに限定して使う

## 関連

- [Access and Retention Policy](./ACCESS_RETENTION_POLICY.md)

## 見直し条件

この ADR を見直す条件は次です。

- core の不変条件を Rust で保ちづらいことが判明したとき
- Toitoi 側との統合で別言語の方が明らかに優位になったとき
- carrier や storage の要件が大きく変わったとき
