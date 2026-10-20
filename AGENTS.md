# AGENTS.md

このリポジトリで作業するエージェント向けの運用ルールです。日本語を中心に記載しています。

## プロジェクト基本情報

- プロジェクト名: `Lingonberry`
- ルート: `.`
- 最新公開版: `v0.9.0`
- 次期目標: `v1.0.0`（未公開）
- 現在の作業状態: 実機試験は管理者の指示により保留中（2026-10-20）。明示的な再開指示を受けるまで、基準ホストでの事前検証、ディスク圧迫試験の予行演習、正式72時間連続稼働試験を実行しない。現在地は `docs/roadmap/V1_0_QUALIFICATION_STATUS.md` を参照する。
- 主要言語: Rust、JavaScript、Markdown、JSON Schema
- リポジトリ種別: 分散知識コモンズのプロトコル、実装、運用仕様、スキーマ、検証用データ
- Rust workspace: `packages/protocol`、`packages/identity`、`packages/validation`、`packages/core`、`packages/indexer`、`packages/relay`、`packages/storage`
- パッケージのバージョンはワークスペース全体で揃える

## 用語の扱い

- 標準用語: `knowledge object`
- 標準用語: `canonical identity`
- 標準用語: `carrier`
- 標準用語: `protocol object`
- `carrier` は、プロトコルオブジェクトを伝送形式で運ぶ正規の実装として扱う。プロトコルと無関係な単なる伝送手段とみなさない
- `wire`（伝送形式）と`canonical`（正規表現）は、同じプロトコルオブジェクトの異なる表現として扱う

## リポジトリ構成

- `packages/`: Rustワークスペース内の、役割ごとに分けた実装
- `docs/concepts/`: 中核概念、用語、識別子、carrier などの定義
- `docs/architecture/`: 分散知識コモンズとしての設計、Toitoi 参照時の観点
- `docs/operations/`: 技術判断、運用仕様、運用手順書
- `docs/roadmap/`: 実装状況、公開までの計画、確認表、未完了の作業一覧
- `docs/protocols/`: プロトコル固有の伝送形式などの仕様
- `docs/profiles/`: アプリケーションプロファイルの定義
- `schemas/`: プロトコル固有のJSON Schema
- `fixtures/`: サンプル JSON と検証用データ

## 実装配置の基準

- 実装を追加するときは、`docs/architecture/DISTRIBUTED_KNOWLEDGE_COMMONS_ARCHITECTURE.md` の `## 13. 推奨リポジトリ構成` を基準にする
- 新しい実装は、原則として `packages/` 配下の責務別ディレクトリに置く
- 既存の `docs/`、`schemas/`、`fixtures/` は、それぞれ仕様、スキーマ、検証サンプルの役割に保つ
- 配置に迷うときは、まず構成章に照らして役割を分け、必要なら文書側の責務を見直す

## 変更前に読むもの

- `README.md`
- `docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md`
- `docs/roadmap/README.md`
- リリース関連の作業では、対象バージョンの計画、確認表、リリースノート
- 隔離データの置換では、次の正本文書を確認する
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
- プロトコルや伝送形式を変更するとき: `docs/protocols/PROTOCOL_NATIVE_WIRE_FORMAT.md`
- スキーマを変更するとき: `schemas/README.md` と該当するスキーマファイル
- 実装計画を確認するとき: `docs/roadmap/IMPLEMENTATION_ROADMAP.md` と `docs/roadmap/IMPLEMENTATION_BACKLOG.md`
- 運用準備の状況を確認するとき: `docs/roadmap/OPERATIONAL_READINESS_ROADMAP.md` と `docs/roadmap/OPERATIONAL_READINESS_BACKLOG.md`
- 技術選定やcarrier・ストレージの方針を確認するとき: `docs/operations/README.md`
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
- JavaScript tests: リポジトリの既存CIまたはパッケージのスクリプトに従い、正規化、識別、検証、仕様適合のテストをすべて実行する

## 運用ルール

- 変更は最小単位で行う
- 既存の書き方、命名、配置を優先する
- 検索は `rg` と `rg --files` を使う
- ファイル編集は `apply_patch` を優先する
- 文書を更新するときは関連文書も同時に見直す
- リリース後の文書更新では、`README.md`、`CHANGELOG.md`、計画・運用の文書案内、現在の実装状況、公開までの計画、確認表、リリースノート、関連する未完了の作業一覧の整合を確認する
- 文書を更新するときは、見出しの `Status` と `Last updated` を合わせて更新する
- リリース前の`release candidate`、`closure pending`、未完了の確認表を、リリース後のmainに残さない
- 公開済みのリリースタグは書き換えず、公開後の文書更新はmainに新しいコミットとして追加する
- `.gitignore` の対象にした方がよい生成物やローカル一時ファイルを新しく作る場合は、同時に `.gitignore` へ追記する
- 動作や仕様の意味を変えるなら、近い位置の概念文書、プロトコル文書、スキーマも整合させる
- JSON Schema を変更するときは、概念文書とプロトコル文書の用語とずれていないか確認する
- `knowledge object`、`canonical identity`、`provenance`、`raw reference`、`lineage`、`carrier` の意味を曖昧に広げない
- Toitoi は参照元、またはアプリケーションプロファイルの例として扱い、Lingonberryの中核プロトコルを Toitoi 固有の都合に縛りつけない
- 分野固有の語彙は中核プロトコルへ直接入れず、原則としてアプリケーションプロファイル側の拡張点で扱う
- 完了済みの作業Issueは見出しに `（完了済み）` を付けて統一する
- 完了済みの計画段階も見出しに `（完了済み）` を付けて統一する

## 隔離データの置換における安全条件

- 既存の台帳を、その場で上書きしない
- 変更不可の検証記録台帳を変更しない
- アーカイブの区画を書き換えたり削除したりしない
- 適用の前提として、完全バックアップv2とpolicy-v2の証明データの検証を必須とする
- 実行時フィンガープリントが変わった場合は、安全側に処理を中止する
- 読み手に見える状態の切り替えは、現行世代ポインターに対する1回の不可分なrenameに限定する
- ポインターが存在する場合は、旧形式のルート台帳へ戻らない
- 曖昧な状態、形式が混在する状態、矛盾や破損のある状態を成功扱いしない
- コミット済みのトランザクションは、終端状態として扱う
- ロールバックはコミット前に限る
- 世代やトランザクションの作業領域を自動削除しない
- 保持期限に基づく削除、重複排除、イベントの集約、競合解決、スキーマ移行を、置換トランザクションに混在させない
- 同一ホスト内のロックを、分散ロックや複数ノード間の合意として扱わない
- メトリクスや監査記録に、秘密情報、ファイルシステムのパス、トランザクションID、レコードID、自由形式のエラーを出力しない

## Issue、PR、CI、リリースの進め方

- 大きな機能はIssueで対象範囲、安全条件、完了条件を定義してから実装する
- 実装PRでは、変更内容、安全条件、検証結果、関連Issueを本文へ記載する
- レビューコメントは妥当性を確認し、修正、返信、スレッドの解決まで完了させる
- フォーマット検査、すべてのClippy検査、ワークスペースのテスト、JavaScriptのテストを、リリース前の必須CIとする
- PRのCIに加え、統合後のmainのCIも確認してからリリース完了と判断する
- リリース前に、バージョン、Cargo.lock、リリースノート、確認表、計画、未完了の作業一覧、現在の実装状況を揃える
- リリースコミット、タグ、GitHub Releaseのタイトル、latestの指定を確認する
- リリース後は確認表を`released`に更新し、リリースコミット、mainのCI、GitHub Releaseの完了をmainに記録する
- GitHub APIやコネクターでmain CIを確認できない場合は、未確認を成功扱いせず、GitHub Actions画面での手動確認手順を明示する

## コミット方針

- 通常の機能変更は作業用ブランチとPRで進める
- 利用者がmainへの直接反映を明示し、変更が限定的な文書更新である場合は、対象ファイルを確認してmainへ直接コミットしてよい
- 1つの論理的な変更は1つのコミットを基本とする
- 関係のない変更を同じコミットに混在させない
- コミットメッセージは英語で簡潔に書く
- 公開後の文書更新のために、公開済みのタグを付け替えない

## 進め方

- 明示されていない破壊的な `git` 操作はしない
- 自分が触っていない変更は巻き戻さない
- 迷ったら最小限の仮定で進める
- 仮定は報告時に明示する
- 判断の影響が大きいときだけ、短く確認を取る
- 回答は要点から先に、必要十分な範囲で簡潔にまとめる
- 作業は必要以上に引き延ばさず、完了条件を満たしたらそこで区切って完了として報告する
- コミット文を求められた場合は、未コミット部分の作業をまとめた内容を英語で返す
