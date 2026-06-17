# Storage Backend Decision Memo

**Status: draft** | **Last updated: 2026-06-17**

## 目的

この文書は、Lingonberry の `relay` と `storage node` に使う storage backend の考え方を整理します。  
ここでの目的は、最初から巨大な分散 DB を決めることではなく、**MVP で壊れにくく、replay-friendly で、運用しやすい構成** を選ぶことです。

## 先に結論

現時点の第一候補は次です。

- **append-only log**: local filesystem
- **canonical object catalog**: SQLite
- **derived index / metadata**: SQLite
- **archive / export**: filesystem 由来の snapshot または object storage への退避

要するに、**filesystem-first + SQLite catalog** のハイブリッドです。

この構成は次の理由で相性がよいです。

- append-only と replayable を保ちやすい
- 単一ノードで立てやすい
- 運用負荷が低い
- canonical object と derived index を分けやすい

## 技術選定の軸

### 1. replay 可能性

Lingonberry では、保存は単なる永続化ではなく、再構成のための履歴保持です。  
そのため、wire input をそのまま追記できる log と、そこから canonical state を再構成できる仕組みが必要です。

### 2. 運用の軽さ

誰でも relay や storage node を立てられることが目的なので、外部依存が少ない方がよいです。  
単一バイナリの Rust 実装と、ローカルで完結する storage は相性がよいです。

### 3. 役割分離

次を分けて持つと、設計が崩れにくいです。

- raw wire log
- canonical object store
- derived index
- replay metadata

### 4. 将来の拡張性

初期版は軽くしてよいですが、後で object storage や Postgres を選べる余地は残したいです。  
そのため、storage backend は「実装を差し替えやすい」形にします。

## 候補比較

### Local filesystem only

向いている点:

- もっとも単純
- replay の見通しがよい
- デバッグしやすい

注意点:

- index や検索を別に持ちたくなる
- メタデータ管理を手でやりやすい

### SQLite only

向いている点:

- 単一ファイルで運用しやすい
- index と metadata をまとめやすい
- 実装が比較的簡単

注意点:

- append-only log を表現するには工夫が必要
- raw log と canonical store を混ぜると責務が曖昧になりやすい

### PostgreSQL

向いている点:

- 複数プロセスや運用機能を載せやすい
- 検索や管理機能を拡張しやすい

注意点:

- MVP としては重い
- 誰でも立てられるという目標には少し過剰になりやすい
- replay-friendly な raw log を別途どう持つかを考える必要がある

### Object storage

向いている点:

- archive relay や長期保管と相性がよい
- スナップショットや export に向く

注意点:

- 単独では catalog と query の役割を満たしにくい
- 低レイテンシな運用には向かないことがある

## 推奨構成

### 1. raw log

wire object を append-only に保存します。  
最初は local filesystem 上の log file で十分です。

### 2. canonical catalog

canonical object の索引、取得、identity 解決に SQLite を使います。  
ここには `id`、`identity key`、`provenance` の参照、`rawRef`、`status` などを持たせると扱いやすいです。

### 3. derived index

検索用の derived index も SQLite か別の軽量 index store に置きます。  
ただし、semantic source はあくまで canonical object であり、index は再構築可能であるべきです。

### 4. snapshot / compaction

log が増えたら、定期的に snapshot を作って compact します。  
古い log を消すかどうかは retention policy に従って決めます。

## なぜこの構成か

### 1. 実装しやすい

Rust から filesystem と SQLite を扱うのは比較的素直です。  
最初の publish 経路と replay 経路を、あまり複雑な依存なしで作れます。

### 2. replay と forensic inspection に向く

raw log が残っていれば、canonicalization ルールを変えた後でも再解析しやすいです。  
これは provenance と rawRef を重視する Lingonberry と合っています。

### 3. relay / storage node を分離しやすい

relay は raw log に追記し、storage node は catalog と snapshot を維持する、という役割分担が作りやすいです。

## 代替案

### 代替案 A: PostgreSQL を default にする

これは、最初から強い query 機能や運用機能が必要な場合の案です。  
ただし、MVP では重くなりやすいので、第一候補にはしません。

### 代替案 B: filesystem のみで完結する

これは最小実装としては魅力的です。  
ただし、検索や metadata 管理が膨らむと、後で SQLite か別の catalog を足したくなります。

### 代替案 C: object storage を default にする

これは archive relay には向いています。  
一方で、日常的な publish / replay / query の基盤としては、単独ではやや重いです。

## 採用の判断

この memo では、MVP の default として次を採用します。

- raw log: filesystem
- canonical catalog: SQLite
- derived index: SQLite

## 保留事項

次は別途決めます。

1. log record の具体フォーマット
2. snapshot の間隔
3. retention policy
4. archive relay で object storage を使うかどうか
5. PostgreSQL へ移行する条件

## 見直し条件

この判断は、次のときに見直します。

- 複数 writer の高い並行性が必要になったとき
- 1 台の node で扱うデータ量が SQLite の運用限界を超えたとき
- search / graph の要件が急に大きくなったとき
- archive relay を強く重視する設計に変わったとき

