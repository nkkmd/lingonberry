# File / Archive Carrier Contract

**Status: draft** | **Last updated: 2026-06-20**

## 目的

この文書は、長期保管と持ち運びのための file / archive carrier を定義します。

ここでの `archive` は、protocol object を固めて持ち運べる正規の carrier 表現です。  
単なるバックアップ形式ではなく、replay 可能性を保つ保存形式として扱います。

## 基本方針

- archive は replay 可能である
- archive は raw log を失わない
- archive は canonical projection を再構築できる
- archive は protocol version と carrier capability を明示する
- archive の物理コンテナは固定しすぎない

## 論理レイアウト

archive は、ディレクトリ構造でも tar でも zip でもよいですが、論理的には次の要素を持ちます。

### 1. `manifest.json`

archive 全体のメタデータです。

最低限の情報:

- archive version
- protocol version
- carrier kind
- createdAt
- item count
- wire log の所在
- 署名や hash があればその参照

### 2. `wire-log.jsonl`

append-only の wire event 列です。

最低限の性質:

- 順序が安定している
- 1 行 1 object である
- replay の唯一の入力として扱える
- rawRef を保持できる

### 3. `snapshot/` もしくは `catalog/`

canonical projection や検索補助を載せるための任意領域です。

ここは再生成可能な補助であり、semantic source にはしません。

## Replay

archive からの replay は次の順で行います。

1. `manifest.json` を読む
2. archive version と protocol version を確認する
3. `wire-log.jsonl` を先頭から順に読む
4. validate / normalize / finalize を通す
5. canonical state を再構成する

## Import / Export

### Export

- relay / storage の保存内容から archive を生成する
- `storage node` の backup 単位は `dataDir` を起点とし、そこから `manifest.json` と `wire-log.jsonl` に相当する情報を固める
- export は原則として full bundle を基本とし、差分 bundle は必要な場合にのみ運用層で採用する
- replay metadata を含める場合は `replay-metadata.json` として扱う
- raw log と provenance を失わない
- 再構築に必要な情報を落とさない
- differential export を使う場合でも、manifest で差分種別と基準点を明示する
- export 時に scrub が必要なら operator policy に従う
- scrub は archive の semantic ではなく、運用上の変換として扱う
- public object を前提にした export を基本形にする

### Import

- archive を受け取って replay する
- canonical state を復元する
- 必要なら canonical catalog を再生成する
- private / curated object の import 可否は policy と capability で決める

## Carrier への期待

archive carrier は、少なくとも次を満たします。

- 長期保管できる
- 持ち運べる
- 再検証できる
- carrier 間で semantics を変えない

## 運用メモ

- 物理削除や scrub は archive carrier の semantic ではない
- retention は storage / operator policy の責務として切る
- archive 形式の変更は versioned に扱う
- export 時の scrub 方針は manifest か別 policy 参照で明示できるようにする

## 関連

- [Carrier Decision Memo](./CARRIER_DECISION_MEMO.md)
- [Protocol-Native Wire Format](../protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md)
- [Technical Decision ADR](./TECH_DECISION_ADR.md)
- [storage node runtime](./STORAGE_NODE_RUNTIME.md)
