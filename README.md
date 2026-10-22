# Lingonberry

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

**Last updated: 2026-10-22**

## English

Lingonberry is a Rust workspace for publishing, validating, storing, retrieving, querying, indexing, and operating canonical knowledge objects. Canonical storage is the source of truth; indexes and effective views are derived, verifiable, and rebuildable.

The workspace also provides persistent quarantine, verified backup and isolated restore, verified replacement workflows, proof-bound retention cleanup, explicit storage-format migration, Ed25519-authenticated publishing, and a production-oriented single-node operator surface.

### Release status

**Execution hold — 2026-10-20:** At the maintainer’s request, privileged reference-host preflight, disk-pressure rehearsal, and the formal 72-hour soak are deferred. Repository preparation and recorded candidate results remain intact; this hold is neither a test failure nor a passed gate. Resume only after an explicit maintainer request, rechecking the fixed candidate, binary identities, retained artifacts, and host prerequisites. Version 1.0.0, tagging, and publication remain pending.

The latest published release is `v0.9.0`. The stable single-node `v1.0.0` release is under final qualification and has not been published.

The fixed pre-version candidate is:

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

Candidate-built binary identities:

```text
lingonberry-storage  737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507
lingonberry-relay    23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

Candidate qualification and the 16-procedure documentation walkthrough have passed and their artifacts were independently verified. Repository-side preparation is complete. Privileged reference-host preflight is currently deferred, and the formal 72-hour soak remains blocked until that preflight passes.

Evidence and documentation commits after the candidate do not redefine it. Version update, the `v1.0.0` tag, GitHub Release, and final publication validation remain pending.

Read these sources first when reviewing or resuming v1.0.0 work:

- [v1.0.0 qualification status](docs/roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 release evidence](docs/roadmap/V1_0_RELEASE_EVIDENCE.md)
- [v1.0.0 documentation walkthrough](docs/roadmap/V1_0_DOCUMENTATION_WALKTHROUGH.md)
- [v1.0.0 qualification plan](docs/roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1 compatibility policy](docs/architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 security diff review](docs/security/V1_0_SECURITY_DIFF_REVIEW.md)
- [v1.0.0 soak plan](docs/roadmap/V1_0_SOAK_PLAN.md)
- [documentation policy](docs/DOCUMENTATION_POLICY.md)
- [documentation inventory](docs/DOCUMENTATION_INVENTORY.md)

A dry run or virtual-time rehearsal validates tooling and evidence formats only. It is not final release evidence. Qualification must remain bound to the fixed candidate and recorded binary digests.

### Safety boundaries

Lingonberry treats ambiguous, incomplete, unsupported, unauthenticated, or contradictory state as an error. In particular:

- validation failures do not enter canonical storage;
- publisher signature verification occurs before acceptance, duplicate/conflict classification, quarantine, raw append, or canonical storage;
- malformed, invalid, or unverifiable signatures fail closed;
- conflicts do not overwrite canonical records;
- original Knowledge Objects are not rewritten or deleted by Transition Objects;
- unauthorized or unknown transitions do not affect the effective view;
- multiple authorized heads are not resolved by timestamps or arbitrary identifier order;
- missing-target transitions remain evidence but are not applied until reevaluated;
- canonical storage commits are not rewritten as failures when only derived processing fails;
- stale workers cannot overwrite newer derived checkpoints;
- incomplete evidence cannot overwrite the last-known-good semantic checkpoint;
- stale effective views are never labeled current;
- ordinary startup never performs implicit storage migration;
- unknown newer storage formats are never mutated;
- non-empty legacy migration requires verified backup evidence bound to the inspected source state;
- target format is not committed before durable verification succeeds;
- public diagnostics exclude storage paths, row IDs, stack traces, and unstable implementation errors;
- backup and restore reject symbolic links and unsafe target reuse;
- restore never overwrites active state or data directories;
- cleanup never rewrites archive segments or immutable evidence ledgers;
- untrusted JSON is bounded before recursive parsing;
- signature-verification artifacts are created exclusively and cleaned on normal success and failure paths;
- same-host locking is not a distributed lock;
- secure erase semantics are not promised.

### Workspace

```text
packages/protocol     canonical protocol model and bounded JSON parser
packages/identity     identity and signature primitives
packages/validation   validation rules
packages/core         authenticated ingestion, quarantine, replacement, and cleanup
packages/indexer      index lifecycle, checkpoints, verification, and catch-up
packages/relay        CLI, HTTP relay, Transition, effective-view, and reevaluation surfaces
packages/storage      File and SQLite backends, diagnostics, recovery, and migration runtime
```

### Development quickstart

Prerequisites: Git, a current Rust toolchain, Cargo, and an HTTP client such as `curl`.

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

In another terminal:

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
```

Choose the path that matches your role:

- [Relay Quickstart](docs/operations/RELAY_QUICKSTART.md)
- [Publisher Quickstart](docs/developers/PUBLISHER_QUICKSTART.md)
- [Storage Node Quickstart](docs/operations/STORAGE_NODE_QUICKSTART.md)
- [Developer Documentation](docs/developers/README.md)

### Production-oriented operation

The reference platform is:

```text
Ubuntu Server 24.04 LTS / x86_64 / systemd
```

Production-oriented installation uses release-built binaries and hardened systemd units. Start with:

- [Operations index](docs/operations/README.md)
- [v1.0 Operator Runbook](docs/operations/V1_0_OPERATOR_RUNBOOK.md)
- [Supported Platforms](docs/operations/SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](docs/operations/OPERATOR_CLI_CONTRACT.md)
- [Upgrade and Rollback](docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md)

The v1.0 operator runbook is the pre-release single-node operating guide. It does not imply that v1.0.0 publication is complete.

### Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -A warnings
cargo test --workspace
```

JavaScript contract tests and the external conformance suite run in `.github/workflows/ci.yml`. Candidate qualification, documentation integrity, documentation inventory, walkthrough, and soak rehearsal use dedicated workflows.

### Documentation

For the proposed knowledge workflow direction, see the [adoption design](docs/architecture/KNOWLEDGE_WORKFLOW_ADOPTION.md), [feature gap analysis](docs/roadmap/KNOWLEDGE_WORKFLOW_GAP_ANALYSIS.md), and [research-record pilot plan](docs/roadmap/RESEARCH_RECORD_PILOT_PLAN.md). Pilot dataset selection, implementation, and execution have not started; the fixed release candidate and execution hold remain unchanged.

- [Documentation policy](docs/DOCUMENTATION_POLICY.md)
- [Documentation inventory](docs/DOCUMENTATION_INVENTORY.md)
- [Operations index](docs/operations/README.md)
- [Roadmap index](docs/roadmap/README.md)
- [Current implementation status](docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
- [Roadmap to v1.0](docs/roadmap/ROADMAP_TO_V1_0.md)
- [Changelog](CHANGELOG.md)

### License

See the package metadata and repository license files for applicable terms.

---

<a id="japanese"></a>

## 日本語

Lingonberryは、正規化された知識オブジェクト（canonical knowledge object）の公開、検証、保存、取得、検索、インデックス作成と運用を支える、Rustのワークスペースです。保存した知識オブジェクトを正本とし、インデックスと遷移を反映した有効ビューは派生物として検証・再構築できます。

データの永続的な隔離管理、検証済みバックアップと分離した環境への復元、検証済み置換処理、証明データに基づく保存期限後の整理、明示的な保存形式の移行、Ed25519認証付き公開、単一ノード向け運用コマンド群も提供します。

### リリース状況

**実機試験の保留（2026-10-20）:** 管理者の指示により、実機での事前検証、ディスク圧迫試験、正式72時間連続稼働試験を一旦保留します。準備と記録済みの候補検証結果は保持します。保留を試験失敗や合格として扱いません。明示的な再開指示後に、固定候補・バイナリ識別値・保存済み証拠・ホスト条件を再確認します。v1.0.0への更新・タグ・公開は未完了です。

最新の公開版は`v0.9.0`です。単一ノード運用向けの安定版となる`v1.0.0`は公開に向けた最終検証中であり、まだ公開されていません。

バージョン更新前の検証対象として固定した公開候補は次のコミットです。

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

公開候補からビルドしたバイナリの識別情報は次のとおりです。

```text
lingonberry-storage  737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507
lingonberry-relay    23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

公開候補の検証と文書に記載した16手順の動作確認は合格し、成果物も独立した方法で検証済みです。リポジトリ側の準備は完了しており、基準ホストでの管理者権限による事前検証は現在保留しています。ただし正式な72時間連続稼働試験は基準ホストでの事前検証に合格するまで開始しません。

公開候補の決定後に追加する検証記録や文書のコミットは公開候補自体を変更しません。バージョン更新、`v1.0.0` タグ、GitHub Release、公開後の最終確認は未完了です。

v1.0.0作業の確認・再開時は、まず次を参照してください。

- [v1.0.0 検証状況](docs/roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 リリースの検証記録](docs/roadmap/V1_0_RELEASE_EVIDENCE.md)
- [v1.0.0 文書に沿った手順の検証記録](docs/roadmap/V1_0_DOCUMENTATION_WALKTHROUGH.md)
- [v1.0.0 適格性検証計画](docs/roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1 互換性方針](docs/architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 セキュリティ差分の確認記録](docs/security/V1_0_SECURITY_DIFF_REVIEW.md)
- [v1.0.0 連続稼働試験計画](docs/roadmap/V1_0_SOAK_PLAN.md)
- [文書方針](docs/DOCUMENTATION_POLICY.md)
- [文書一覧](docs/DOCUMENTATION_INVENTORY.md)

試行実行や仮想時間を使う予行演習はツールと検証記録形式の確認であり、最終的なリリース判定の根拠にはなりません。適格性検証は固定した公開候補と記録済みバイナリのハッシュ値に結び付いている必要があります。

### 安全境界

Lingonberryは、曖昧、不完全、未対応、未認証、または矛盾した状態をエラーとして扱います。特に次を守ります。

- 検証に失敗したオブジェクトを正本となるストレージへ保存しない
- 公開者の署名を、受入処理、重複・競合の判定、隔離、受信データの追記、正本となるストレージへの保存より前に検証する
- 形式が不正な署名、無効な署名、検証できない署名を拒否する
- 競合で既存の正規化済みレコードを上書きしない
- 遷移オブジェクト（Transition Object）で元の知識オブジェクト（Knowledge Object）を書き換えたり削除したりしない
- 未許可または未知の遷移を有効ビューへ反映しない
- 権限を持つ遷移の先端が複数ある場合、時刻や任意の識別子順で一つに決めない
- 対象が存在しない遷移は証拠として保持し、再評価するまで適用しない
- 派生処理だけが失敗した場合に正本となるストレージへの保存成功を失敗へ書き換えない
- 古い状態で動くワーカーが新しい派生状態のチェックポイントを上書きできない
- 不完全な検証記録で最後に正常と確認された意味状態のチェックポイントを上書きしない
- 古い有効ビューを最新と表示しない
- 通常起動時に暗黙の保存形式変更を行わない
- 未知の新しい保存形式を変更しない
- データがある旧形式からの移行は、検証済みバックアップの証拠なしで開始しない
- 永続化の検証に成功するまで移行先の形式を確定しない
- 公開診断情報へストレージパス、行ID、スタックトレース、実装に依存する不安定なエラー情報を出さない
- バックアップと復元でシンボリックリンクや危険な対象再利用を拒否する
- 復元で稼働中の状態ディレクトリやデータディレクトリを上書きしない
- 整理処理でアーカイブの区画や変更不可の検証記録台帳を書き換えない
- 信頼できないJSONは、再帰的な解析の前にサイズや深さを制限する
- 署名検証用の一時成果物は既存ファイルを上書きせずに作成し、通常の成功時・失敗時には後片付けする
- 同一ホスト内のロックを分散ロックとして扱わない
- 安全な完全消去を保証しない

### ワークスペース

```text
packages/protocol     正規プロトコルのモデルと、入力制限付きJSON解析
packages/identity     識別情報・署名の基本機能
packages/validation   検証規則
packages/core         認証付き取込、隔離、置換、整理
packages/indexer      インデックス管理、チェックポイント、検証、追随処理
packages/relay        CLI、HTTPリレー、遷移、有効ビュー、再評価
packages/storage      ファイル・SQLite保存、診断、復旧、移行処理
```

### 開発用入門ガイド

必要なものはGit、最新のRustツールチェーン、Cargo、`curl`などのHTTPクライアントです。

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

別の端末で確認します。

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
```

担当する役割に応じて次を参照してください。

- [リレーの入門ガイド](docs/operations/RELAY_QUICKSTART.md)
- [公開クライアントの入門ガイド](docs/developers/PUBLISHER_QUICKSTART.md)
- [ストレージノードの入門ガイド](docs/operations/STORAGE_NODE_QUICKSTART.md)
- [開発者向け文書](docs/developers/README.md)

### 本番環境向け運用

正式な基準環境は次の構成です。

```text
Ubuntu Server 24.04 LTS / x86_64 / systemd
```

本番環境向け導入ではリリース用にビルドされたバイナリと安全対策を施したsystemdユニットを使用します。次から確認してください。

- [運用文書案内](docs/operations/README.md)
- [v1.0 単一ノード運用手順書](docs/operations/V1_0_OPERATOR_RUNBOOK.md)
- [対応環境](docs/operations/SUPPORTED_PLATFORMS.md)
- [運用CLIの仕様](docs/operations/OPERATOR_CLI_CONTRACT.md)
- [v0.8.0 更新とロールバックの手順](docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md)

v1.0 運用手順書は公開前の単一ノード運用ガイドです。これはv1.0.0の公開完了を意味しません。

### 検証

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -A warnings
cargo test --workspace
```

JavaScriptの仕様確認テストと外部仕様適合性試験は`.github/workflows/ci.yml`で実行します。公開候補の検証、文書の整合性、文書一覧、手順に沿った確認、連続稼働試験予行演習は専用ワークフローで確認します。

### 文書

知識活用方針の提案は、[採用設計](docs/architecture/KNOWLEDGE_WORKFLOW_ADOPTION.md#japanese)、[現行機能との差分表](docs/roadmap/KNOWLEDGE_WORKFLOW_GAP_ANALYSIS.md#japanese)、[研究記録の最小実証計画](docs/roadmap/RESEARCH_RECORD_PILOT_PLAN.md#japanese)を参照してください。実証の資料選定・実装・実行は未着手であり、固定した公開候補と実機試験の保留は維持しています。

- [文書方針](docs/DOCUMENTATION_POLICY.md)
- [文書一覧](docs/DOCUMENTATION_INVENTORY.md)
- [運用文書案内](docs/operations/README.md)
- [計画・進捗の文書案内](docs/roadmap/README.md)
- [現在の実装状況](docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
- [v1.0公開までの計画](docs/roadmap/ROADMAP_TO_V1_0.md)
- [変更履歴](CHANGELOG.md)

### ライセンス

適用される条件は、各パッケージのメタデータとリポジトリ内のライセンスファイルを参照してください。
