# Lingonberry Documentation Policy / Lingonberry 文書方針

**Status:** normative for the v1.0.0 documentation set  
**English is the normative language. Japanese is a synchronized translation.**

> If the English and Japanese sections differ, the English section takes precedence.
>
> 英語部分と日本語部分に差異がある場合は、英語部分を優先します。

## English

### 1. Purpose

This policy defines documentation language, classification, synchronization, inventory, and release-gate rules for Lingonberry.

Its goals are to:

- provide accessible entry points for English- and Japanese-speaking users;
- keep normative terminology aligned with source code, schemas, commands, APIs, diagnostics, and durable formats;
- prevent Japanese-only requirements from becoming a parallel specification;
- limit translation maintenance to documents where it materially improves adoption or routine operation;
- make unresolved documentation work visible in the generated inventory.

### 2. Normative language

English is normative for specifications, compatibility statements, command contracts, operational procedures, and release evidence.

Japanese text in a bilingual document is a translation. It must not introduce a requirement, exception, procedure, safety guarantee, or release claim that is absent from the English section.

Source identifiers, commands, configuration keys, API fields, event names, error codes, file paths, version strings, hashes, and machine-readable values are not translated.

### 3. Bilingual format

A bilingual document must place English first and Japanese second in the same file. The sections must have equivalent semantic scope and heading order.

Commands, paths, configuration examples, candidate identifiers, version numbers, and safety conditions must remain technically identical in both sections.

A Tier 1 change to English requires the corresponding Japanese update in the same pull request. A stale-translation warning is release-blocking until removed.

### 4. Classification

The generated inventory is the authoritative per-file classification and progress ledger.

#### `BILINGUAL_REQUIRED`

Use for repository entry points and routine operator documents whose complete operational scope must be available in English and Japanese.

Current required set is maintained by `scripts/generate-documentation-inventory.py`. A required document may be marked `KEEP_BILINGUAL` only after scope and synchronization review.

#### `BILINGUAL_SCOPED`

Use when the high-level or user-facing portion must be bilingual but detailed technical material may remain English-only. A reviewed file may be marked `KEEP_BILINGUAL` without changing its classification.

#### `ENGLISH_ONLY`

Use for detailed protocol, architecture, security-review, qualification, soak, evidence, CI, debugging, generated-reference, and maintainer-only material unless an explicit decision requires translation.

Historical documents may remain English-only or retain clearly non-normative historical Japanese content when the inventory records that disposition.

### 5. Minimum bilingual operator surface

Taken together, the reviewed bilingual documents must allow a new single-node operator to:

1. understand the product and supported deployment boundary;
2. install or build the supported binaries;
3. create and validate minimum configuration;
4. start, stop, and restart the services;
5. inspect health, readiness, logs, and diagnostics;
6. publish and retrieve a minimal object where applicable;
7. create and verify a backup;
8. plan or perform an isolated restore;
9. verify storage and index state;
10. upgrade from the supported previous release;
11. roll back or stop safely after a failed upgrade;
12. identify conditions that require stopping and escalation.

These capabilities may be distributed across the root README, operations index, quickstart, operator runbook, and upgrade/rollback guide. The policy does not require duplicate procedures in every document.

### 6. Inventory and completion state

`docs/DOCUMENTATION_INVENTORY.md` is generated from tracked Markdown files. Manual edits to the generated inventory are not authoritative.

The generator must preserve classification separately from completion state. For example, a reviewed Tier 1 file remains `BILINGUAL_REQUIRED` while its action becomes `KEEP_BILINGUAL` and its blocker becomes `no`.

A path listed in a classification set but absent from tracked files is not an inventory entry and is not, by that fact alone, a release blocker. Adding a new required document requires adding the file and reviewing its classification and synchronization.

### 7. Automated checks

The repository maintains checks for:

- generated inventory freshness;
- required bilingual section markers and synchronization-sensitive files;
- documentation-freeze boundaries;
- standard Rust and JavaScript validation;
- candidate-bound documentation walkthrough evidence when required by the documentation workflow.

Passing these checks does not complete the formal soak, privileged reference-host qualification, version preparation, release PR, tag, GitHub Release, or publication evidence.

### 8. v1.0.0 release gate

Documentation readiness requires:

- every tracked Markdown file is present in the generated inventory;
- every inventory row marked as a v1.0 blocker is resolved;
- reviewed bilingual files contain synchronized English and Japanese sections;
- no Tier 1 file contains a stale-translation warning;
- routine installation, configuration, backup, restore, upgrade, and rollback instructions do not contradict one another;
- English-only normative files contain no Japanese-only requirement;
- release-facing documents accurately distinguish completed candidate evidence from pending release gates.

The fixed pre-version candidate is `8c6b48082205a3af555130eec1f3e7d2ac8811fe`. Documentation and tooling commits after that commit do not redefine it. Executable evidence bound to the previous candidate `f9543019f2c219aea3b085ff90f2da201b268a48` is historical. Formal 72-hour soak, privileged reference-host qualification, version preparation, release PR, tag, GitHub Release, and final publication evidence remain separate gates.

### 9. Change control

For v1.x:

- new user-facing or routine-operator documents must be classified when introduced;
- Tier 1 English changes require synchronized Japanese changes;
- internal technical documents default to English-only;
- classification or completion-state changes require generator and inventory updates;
- English remains normative until a future major-version policy explicitly changes it.

---

## 日本語

### 1. 目的

本方針は、Lingonberryの文書で使う言語、文書の分類、翻訳の同期、文書一覧、公開前の必須確認事項を定めます。

目的は次のとおりです。

- 英語と日本語のどちらの利用者にも、利用を始めるための分かりやすい入口を提供する
- 仕様上の用語を、ソースコード、スキーマ、コマンド、API、診断、永続保存形式と一致させる
- 日本語にしかない要件が、英語とは別の仕様になるのを防ぐ
- 翻訳の保守対象を、導入や通常運用に役立つ文書に絞る
- 残っている文書作業を、自動生成の文書一覧で確認できるようにする

### 2. 正本となる言語

仕様、互換性に関する記述、コマンドの仕様、運用手順、リリースの検証記録では、英語を正本とします。

英日併記文書の日本語部分は翻訳です。英語部分にない要件、例外、手順、安全性の保証、リリースに関する主張を、日本語部分だけに追加してはいけません。

ソースコード内の識別子、コマンド、設定キー、APIフィールド、イベント名、エラーコード、ファイルパス、バージョン文字列、ハッシュ値、機械可読の値は翻訳しません。

### 3. 英日併記の形式

英日併記文書では、同じファイル内で英語を先に、日本語を後に配置します。両言語で、説明する範囲と見出しの順序を一致させます。

コマンド、パス、設定例、公開候補の識別情報、バージョン番号、安全条件は、両言語で技術的に同一でなければなりません。

Tier 1文書の英語部分を変更する場合は、同じPRで日本語部分も更新します。翻訳が未更新であることを示す警告が残っている間は、リリースできません。

### 4. 分類

自動生成の文書一覧を、ファイルごとの分類と進捗を管理する正本とします。

#### `BILINGUAL_REQUIRED`

リポジトリの入口や通常運用の手順など、運用に必要な範囲全体を英語と日本語で提供する文書に適用します。

必須文書の一覧は`scripts/generate-documentation-inventory.py`で管理します。対象範囲と翻訳の同期をレビューした文書だけを`KEEP_BILINGUAL`にできます。

#### `BILINGUAL_SCOPED`

概要や利用者向けの部分は英日併記とし、技術的な詳細は英語のみでもよい文書に適用します。レビュー済みの文書は、分類を変えずに`KEEP_BILINGUAL`にできます。

#### `ENGLISH_ONLY`

プロトコル、設計、セキュリティ確認、適格性検証、連続稼働試験、検証記録、CI、デバッグ、自動生成の参照資料、保守担当者専用の文書など、技術的な詳細を扱う文書に適用します。翻訳が必要と明示的に決定された場合は、その決定に従います。

過去の文書は、英語のみで維持できます。文書一覧にその扱いを記録すれば、仕様上の正本ではないことを明示した過去の日本語記述を残すこともできます。

### 5. 通常運用に必要な英日併記の範囲

レビュー済みの英日併記文書を通じて、新しく単一ノードを運用する担当者が次を行えるようにします。

1. 製品と対応する運用構成の範囲を理解する
2. 対応するバイナリを導入またはビルドする
3. 最小限の設定を作成して検証する
4. サービスを起動、停止、再起動する
5. 稼働状態、受付準備の状態、ログ、診断結果を確認する
6. 該当する場合に、最小構成のオブジェクトを公開・取得する
7. バックアップを作成して検証する
8. 稼働環境から分離した場所への復元を計画または実行する
9. ストレージとインデックスの状態を検証する
10. 対応する旧バージョンから更新する
11. 更新に失敗した後、安全にロールバックまたは停止する
12. 作業を止めて責任者への確認が必要となる条件を判断する

これらの説明は、ルートのREADME、運用文書案内、入門ガイド、運用手順書、更新・ロールバックの手順書に分けて構いません。同じ手順をすべての文書に重複して記載する必要はありません。

### 6. 文書一覧と完了状態

`docs/DOCUMENTATION_INVENTORY.md`は、Gitで追跡しているMarkdownファイルから生成します。生成結果を手作業で編集しても、正本への変更にはなりません。

生成処理では、分類と完了状態を別々に管理します。例えば、レビュー済みのTier 1文書は`BILINGUAL_REQUIRED`のまま、対応事項が`KEEP_BILINGUAL`、公開を妨げる項目の有無が`no`になります。

分類用の一覧にパスがあっても、Gitで追跡するファイルが存在しなければ文書一覧の項目にはなりません。それだけを理由にリリースを止めることもありません。新しい必須文書を追加するときは、ファイルを追加し、分類と翻訳の同期をレビューします。

### 7. 自動検査

リポジトリでは、次の項目を検査します。

- 自動生成の文書一覧が最新であること
- 必須の英日併記見出しと、翻訳の同期が必要な文書
- 文書の確定範囲
- 標準のRustおよびJavaScript検証
- 文書用ワークフローで必要となる、公開候補に対応した手順の動作確認記録

これらの検査に合格しても、正式な連続稼働試験、管理者権限による基準ホストの検証、バージョン更新の準備、リリースPR、タグ、GitHub Release、公開結果の記録が完了したことにはなりません。

### 8. v1.0.0の公開に必要な文書の条件

文書の準備完了には、次の条件を満たす必要があります。

- Gitで追跡するMarkdownファイルが、すべて自動生成の文書一覧に含まれている
- 文書一覧でv1.0の公開を妨げるとされた項目が、すべて解消されている
- レビュー済みの英日併記文書で、英語と日本語の内容が同期している
- Tier 1文書に、翻訳が未更新であることを示す警告がない
- 導入、設定、バックアップ、復元、更新、ロールバックの手順が互いに矛盾しない
- 英語のみの正本文書に、日本語だけの要件がない
- リリースに関する文書が、完了済みの公開候補の検証記録と、未完了の公開条件を正確に区別している

バージョン更新前の公開候補は`8c6b48082205a3af555130eec1f3e7d2ac8811fe`に固定しています。このコミット以後に文書やツールを変更しても、公開候補は変更しません。旧候補`f9543019f2c219aea3b085ff90f2da201b268a48`に対応する実行結果は過去の記録です。正式72時間連続稼働試験、管理者権限による基準ホストの検証、バージョン更新の準備、リリースPR、タグ、GitHub Release、最終的な公開結果の記録は、それぞれ別の必須工程です。

### 9. 変更管理

v1.xでは、次の規則を適用します。

- 新しい利用者向け文書や通常運用の文書は、追加時に分類する
- Tier 1文書の英語部分を変更したら、日本語部分も同期して変更する
- 内部向けの技術文書は、原則として英語のみとする
- 分類や完了状態を変更したら、生成処理と文書一覧を更新する
- 将来のメジャーバージョンの方針で明示的に変更するまで、英語を正本とする
