# AGENTS.md

このリポジトリで作業するエージェント向けの運用ルールです。日本語主体で書いています。

## プロジェクト基本情報

- プロジェクト名: `Lingonberry`
- ルート: `.`
- 主要言語: `mixed-ja-en`
- リポジトリ種別: プロトコル仕様、概念文書、JSON Schema
- パッケージマネージャー: 現時点では未定義

## 用語の扱い

- 標準用語: `knowledge object`
- 標準用語: `canonical identity`
- 標準用語: `carrier`
- 標準用語: `protocol object`
- `carrier` は protocol の外側にある単なる transport ではなく、protocol object を wire 上で運ぶ正規の実装として扱う
- `wire` と `canonical` は別プロトコルではなく、同じ protocol object の別表現として扱う

## リポジトリ構成

- `docs/concepts/`: 中核概念、用語、識別子、carrier などの定義
- `docs/architecture/`: 分散知識コモンズとしての設計、Toitoi 参照時の観点
- `docs/operations/`: 技術選定、ADR、carrier / storage の決定メモ
- `docs/roadmap/`: 実装ロードマップと backlog
- `docs/protocols/`: protocol-native な wire format などの仕様
- `docs/profiles/`: application profile の定義
- `schemas/`: protocol-native な JSON Schema
- `fixtures/`: サンプル JSON と検証用データ

## 実装配置の基準

- 実装を追加するときは、`docs/architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md` の `## 13. 推奨リポジトリ構成` を基準にする
- 新しい実装は、原則として `packages/` 配下の責務別ディレクトリに置く
- 既存の `docs/`、`schemas/`、`fixtures/` は、それぞれ仕様、schema、検証サンプルの役割に保つ
- 配置が迷うときは、まず構成章に照らして役割を分け、必要なら文書側の責務を見直す

## 変更前に読むもの

- `README.md`
- `docs/concepts/GLOSSARY.md`
- `docs/concepts/CONCEPT_MODEL.md`
- `docs/concepts/CARRIER.md`
- `docs/profiles/TOITOI_APPLICATION_PROFILE.md`
- protocol / wire format を変更するとき: `docs/protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md`
- schema を変更するとき: `schemas/README.md` と該当する schema ファイル
- 実装計画を確認するとき: `docs/roadmap/IMPLEMENTATION_ROADMAP.md` と `docs/roadmap/IMPLEMENTATION_BACKLOG.md`
- operational readiness を確認するとき: `docs/roadmap/OPERATIONAL_READINESS_ROADMAP.md` と `docs/roadmap/OPERATIONAL_READINESS_BACKLOG.md`
- 技術選定や carrier / storage 方針を確認するとき: `docs/operations/README.md`
- Toitoi との対応関係を確認するとき: `docs/architecture/TOITOI_REFERENCE_CHECKLIST.md`
- アーキテクチャ全体に触れるとき: `docs/architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md`
- 必要に応じて、関連する `docs/` の個別文書も確認する

## よく使うコマンド

- ファイル一覧: `rg --files`
- テキスト検索: `rg "検索語"`
- 変更確認: `git status --short`
- 差分確認: `git diff`
- 標準の lint / test / build コマンド: 現時点では未定義

## 運用ルール

- 変更は最小単位で行う
- 既存の書き方、命名、配置を優先する
- 検索は `rg` と `rg --files` を使う
- ファイル編集は `apply_patch` を優先する
- 文書を更新するときは関連文書も同時に見直す
- 文書を更新するときは、見出しの `Status` と `Last updated` を合わせて更新する
- `.gitignore` の対象にした方がよい生成物やローカル一時ファイルを新しく作る場合は、同時に `.gitignore` へ追記する
- 動作や仕様の意味を変えるなら、近い位置の概念文書、protocol 文書、schema も整合させる
- JSON Schema を変更するときは、概念文書と protocol 文書の用語とずれていないか確認する
- `knowledge object`、`canonical identity`、`provenance`、`raw reference`、`lineage`、`carrier` の意味を曖昧に広げない
- Toitoi は参照元または application profile の例として扱い、Lingonberry の core protocol を Toitoi 固有の都合に縛りつけない
- 分野固有の語彙は core protocol へ直接入れず、原則として application profile 側の拡張点として扱う
- 完了済みの backlog issue は見出しに `（完了済み）` を付けて統一する
- 完了済みの roadmap phase も見出しに `（完了済み）` を付けて統一する

## 進め方

- 明示されていない破壊的な `git` 操作はしない
- 自分が触っていない変更は巻き戻さない
- 迷ったら最小限の仮定で進める
- 仮定は報告時に明示する
- 判断の影響が大きいときだけ、短く確認を取る
- 回答は要点から先に、必要十分な範囲で簡潔にまとめる
- 作業は必要以上に引き延ばさず、完了条件を満たしたらそこで区切って完了として報告する
- コミット文を求められた場合は、未コミット部分の作業をまとめた内容を英語で返す
