# AGENTS.md

このリポジトリで作業するエージェント向けの運用ルールです。日本語主体で書いています。

## プロジェクト基本情報

- プロジェクト名: `Lingonberry`
- ルート: `.`
- 最新安定版: `v0.3.0`
- 主要言語: Rust、JavaScript、Markdown、JSON Schema
- リポジトリ種別: 分散知識コモンズ・プロトコル、実装、運用契約、schema、fixture
- Rust workspace: `packages/protocol`、`packages/identity`、`packages/validation`、`packages/core`、`packages/indexer`、`packages/relay`、`packages/storage`
- package versionはworkspace全体で揃える

## 用語の扱い

- 標準用語: `knowledge object`
- 標準用語: `canonical identity`
- 標準用語: `carrier`
- 標準用語: `protocol object`
- `carrier` は protocol の外側にある単なる transport ではなく、protocol object を wire 上で運ぶ正規の実装として扱う
- `wire` と `canonical` は別プロトコルではなく、同じ protocol object の別表現として扱う

## リポジトリ構成

- `packages/`: Rust workspaceの責務別実装
- `docs/concepts/`: 中核概念、用語、識別子、carrier などの定義
- `docs/architecture/`: 分散知識コモンズとしての設計、Toitoi 参照時の観点
- `docs/operations/`: 技術決定、運用契約、operator runbook
- `docs/roadmap/`: 実装状況、release roadmap、checklist、backlog
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
- `docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md`
- `docs/roadmap/README.md`
- release関連作業では、対象versionのroadmap、checklist、release note
- quarantine replacementでは、次を正本として確認する
  - `docs/operations/QUARANTINE_REPLACEMENT_POLICY.md`
  - `docs/operations/QUARANTINE_REPLACEMENT_PREVIEW.md`
  - `docs/operations/QUARANTINE_REPLACEMENT_PREVIEW_RUNBOOK.md`
  - `docs/operations/QUARANTINE_REPLACEMENT_TRANSACTION.md`
  - `docs/operations/QUARANTINE_REPLACEMENT_GENERATION.md`
  - `docs/operations/QUARANTINE_REPLACEMENT_RECOVERY_RUNBOOK.md`
  - `docs/operations/QUARANTINE_REPLACEMENT_OPERATIONS_HARDENING.md`
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
- Rust format: `cargo fmt --all -- --check`
- Rust library Clippy: `cargo clippy --workspace --lib -- -D warnings`
- Rust binary Clippy: `cargo clippy --workspace --bins -- -D warnings -A dead-code`
- Rust test Clippy: `cargo clippy --workspace --tests -- -D warnings -A dead-code -A unused-variables`
- Rust tests: `cargo test --workspace`
- JavaScript tests: repositoryの既存CIまたはpackage scriptsに従い、canonicalization、identity、validation、contract testsをすべて実行する

## 運用ルール

- 変更は最小単位で行う
- 既存の書き方、命名、配置を優先する
- 検索は `rg` と `rg --files` を使う
- ファイル編集は `apply_patch` を優先する
- 文書を更新するときは関連文書も同時に見直す
- release後の文書更新では、`README.md`、`CHANGELOG.md`、roadmap index、operations index、current implementation status、release roadmap、checklist、release note、関連backlogの整合を確認する
- 文書を更新するときは、見出しの `Status` と `Last updated` を合わせて更新する
- release前の `release candidate`、`closure pending`、未完了checklistをrelease後のmainへ残さない
- release tagは公開後に書き換えず、post-release documentationはmainの新規commitとして追加する
- `.gitignore` の対象にした方がよい生成物やローカル一時ファイルを新しく作る場合は、同時に `.gitignore` へ追記する
- 動作や仕様の意味を変えるなら、近い位置の概念文書、protocol 文書、schema も整合させる
- JSON Schema を変更するときは、概念文書と protocol 文書の用語とずれていないか確認する
- `knowledge object`、`canonical identity`、`provenance`、`raw reference`、`lineage`、`carrier` の意味を曖昧に広げない
- Toitoi は参照元または application profile の例として扱い、Lingonberry の core protocol を Toitoi 固有の都合に縛りつけない
- 分野固有の語彙は core protocol へ直接入れず、原則として application profile 側の拡張点として扱う
- 完了済みの backlog issue は見出しに `（完了済み）` を付けて統一する
- 完了済みの roadmap phase も見出しに `（完了済み）` を付けて統一する

## Quarantine replacementの安全境界

- existing ledgerをin-place overwriteしない
- immutable evidence ledgerを変更しない
- archive segmentをrewriteまたはdeleteしない
- verified complete backup v2とverified policy-v2 proofをapplyの必須gateとする
- runtime fingerprint変更時はfail closedで中止する
- reader-visible publicationはcurrent-generation pointerの1回のatomic renameに限定する
- pointerが存在する状態でlegacy root ledgerへfallbackしない
- ambiguous、mixed、contradictory、corrupt stateを成功扱いしない
- committed transactionはterminalとして扱う
- rollbackはcommit前に限定する
- generationやtransaction workspaceを自動削除しない
- retention deletion、deduplication、event collapse、conflict resolution、schema migrationをreplacement transactionへ混在させない
- same-host lockをdistributed lockまたはmulti-node consensusとして扱わない
- metricsやauditへsecret、filesystem path、transaction ID、record ID、free-form errorを出さない

## Issue、PR、CI、releaseの進め方

- 大きな機能はIssueでscope、安全境界、完了条件を定義してから実装する
- 実装PRでは、変更内容、安全境界、検証結果、関連Issueを本文へ記載する
- review commentは妥当性を確認し、修正、返信、thread resolveまで完了させる
- CIはformat、全Clippy、workspace tests、JavaScript testsをrelease gateとして扱う
- PR CI成功だけでrelease完了とせず、merge後のmain branch CIも確認する
- release前にversion、Cargo.lock、release notes、checklist、roadmap、backlog、current implementation statusを揃える
- release commit、tag、GitHub Release title、latest指定を確認する
- release後はchecklistを`released`へ更新し、release commit、main CI、GitHub Release完了をmainへ記録する
- GitHub APIやconnectorでmain CIを確認できない場合は、未確認を成功扱いせず、GitHub Actions画面での手動確認手順を明示する

## コミット方針

- 通常の機能変更はfeature branchとPRを使用する
- userがmainへの直接反映を明示し、変更が限定的な文書更新である場合は、対象ファイルを確認してmainへ直接commitしてよい
- 1つの論理変更は1つのcommitを基本とする
- unrelatedな変更を同じcommitへ混在させない
- commit messageは英語で簡潔に書く
- 公開済みtagをpost-release documentationのために付け替えない

## 進め方

- 明示されていない破壊的な `git` 操作はしない
- 自分が触っていない変更は巻き戻さない
- 迷ったら最小限の仮定で進める
- 仮定は報告時に明示する
- 判断の影響が大きいときだけ、短く確認を取る
- 回答は要点から先に、必要十分な範囲で簡潔にまとめる
- 作業は必要以上に引き延ばさず、完了条件を満たしたらそこで区切って完了として報告する
- コミット文を求められた場合は、未コミット部分の作業をまとめた内容を英語で返す
