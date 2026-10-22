# Knowledge Workflow Gap Analysis / 知識活用方針と現行機能の差分表

[English](#english) | [日本語](#japanese)

> English is normative for this assessment. Japanese is a synchronized translation.
>
> この評価では英語を正本とし、日本語を同期翻訳とします。

**Status: static baseline assessment; pilot not executed** | **Last updated: 2026-10-22**

## English

### 1. Baseline and assessment method

Reviewed main commit: `81ed02bcd648ea7cfbe3aa4c74be1d7948281131`. This is the documentation/source baseline, not the fixed release candidate. The active candidate is `8c6b48082205a3af555130eec1f3e7d2ac8811fe` as recorded in [current status](./CURRENT_IMPLEMENTATION_STATUS.md).

This assessment uses contracts, schemas and selected active source paths. It is not a new runtime qualification or an exhaustive code audit. “Existing” means the stated bounded capability has a checked-in basis; it does not mean the research workflow has passed. Historical completed backlog entries and capability declarations alone are insufficient evidence for a live endpoint.

Status vocabulary: **Existing** = reusable bounded foundation; **Partial** = some machinery exists but the intended workflow needs work; **Planned** = not established as a supported capability in this review; **Review required** = guarantees depend on path-specific investigation. Priority: **P0** = before/during pilot; **P1** = after pilot findings; **P2** = later independent work.

### 2. Feature differences

| ID | Desired behavior | Current basis and limit | Status | Next action / priority |
|---|---|---|---|---|
| G01 | Preserve and retrieve exact knowledge records | Required fields and nine types exist in the [schema](../../schemas/knowledge-object.schema.json); authenticated ingest and retrieval exist in `packages/core/src/lib.rs` | Existing | Map research records to current types; no root-schema changes / P0 |
| G02 | Distinguish observations, evidence, interpretations and limitations | Types include `inquiry`, `observation`, `evidence`, `claim`, `annotation`; research-specific meaning is not enforced by core | Partial | Define profile constraints and source-to-object mapping / P0 |
| G03 | Follow evidence, revisions and sources | `IndexSnapshot` exposes relation/lineage/provenance fragments in [indexer source](../../packages/indexer/src/lib.rs), including tests; this does not establish every draft profile HTTP route | Existing foundation; partial client | Verify active interface mapping; use a bounded reader / P0 |
| G04 | Show a revised judgment without losing the old one | Immutable objects, lineage and [effective-view contracts](../protocols/EFFECTIVE_VIEW_READ_API.md) exist; lineage and authorized transition effects are distinct | Partial | New IDs plus lineage for the pilot; make application selection explicit / P0 |
| G05 | Preserve decision-maker, scope, evidence and unresolved disagreement | Core acceptance and effective-view authority do not implement a research endorsement workflow | Planned at profile layer | Use separately recorded judgment annotations; reject ambiguous selection / P0 |
| G06 | Cite an exact version and verify source bytes | [Identity/provenance contract](../protocols/IDENTITY_AND_PROVENANCE.md) separates canonical IDs and semantic keys; `rawRef` does not verify or retain external content | Partial | Manifest of IDs, canonical-byte SHA-256 and permitted source snapshots / P0 |
| G07 | Search and read without understanding storage internals | Type and graph lookup exist; [HTTP contract](../operations/HTTP_CARRIER_CONTRACT.md) is narrower than draft profile requirements; full-text ranking/UI are not established here | Partial | Start with type filter and one-hop navigation; treat full-text search/SDK as follow-up / P0 then P1 |
| G08 | Recover derived processing after interruption | Index lifecycle/checkpoints exist; [transition ingestion](../../packages/relay/src/transition_api.rs) writes transition then queue intent separately | Review required | Audit append/retry/reconciliation per backend before promising atomic outbox or automatic completion / P1 |
| G09 | Move a dataset and recover a node | [Archive carrier](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md) replays publish requests; [operator runbook](../operations/V1_0_OPERATOR_RUNBOOK.md) defines verified backup/isolated restore | Existing bounded paths | Test archive replay and backup separately; inventory extra source files and state coverage / P0 |
| G10 | Distinguish authentication, authorization and truth | Publisher-signature checks and separate authority semantics exist; arbitrary issuer claims and source statements are not automatically verified | Existing boundary | Preserve it in labels, negative cases and the report / P0 |
| G11 | Enforce private access across reads/search/export/AI | Admin RBAC exists; complete per-record ACLs and OAuth/OIDC are excluded from the [v1.0 scope](./ROADMAP_TO_V1_0.md) | Planned | Public-source local pilot only; separate end-to-end access design / P2 |
| G12 | Withdraw or physically delete confidential material | Transition withdrawal and operational retention exist; secure erasure is not guaranteed | Review required | Keep withdrawal, retention and physical deletion separate; no confidential pilot data / P2 |
| G13 | Produce optional AI answers with exact sources | AI/vector search are outside v1.0 guarantees | Planned | Add only after traceable citation works; track model/settings/source versions/freshness / P2 |
| G14 | Synchronize across independent nodes | Single-node implementation; distributed documents describe future direction | Planned | Validate portability first; replication/consensus need separate contracts / P2 |

### 3. Active source trace and compatibility disposition

The [relay package manifest](../../packages/relay/Cargo.toml) selects `main_v0_6_final.rs`. It delegates publication/HTTP handling to `main_v0_6_release.rs` and other CLI operations through `main_v0_5_index.rs`. Read the selected chain, not an old file named `main.rs` in isolation. The pilot's first implementation task must record actual commands/routes and their results for its selected revision.

| Change class | Initial disposition |
|---|---|
| Profile, reader, fixtures and source manifest | Prefer additions outside frozen protocol semantics; validate against both schema and active runtime |
| Optional HTTP/query behavior | Separate compatibility review, bounded queries and tests; do not silently claim draft routes exist |
| Outbox/reconciliation or backend changes | Inspect durable formats, partial-write recovery, replay and rollback; assess new-candidate impact |
| Canonical IDs, default identity rule, signing bytes, required fields | Preserve in pilot; any change needs an independently versioned design and migration review |
| Private access, physical deletion, distributed behavior | Independent work; no implied guarantee from this plan |

The [compatibility policy](../architecture/V1_COMPATIBILITY_POLICY.md) and existing contracts govern the actual classification. Priority does not assign a product version or authorize changing the fixed candidate.

### 4. Open questions and next evidence

| Question | Evidence needed | Owner role / decision point |
|---|---|---|
| Can the reader use existing public read surfaces for every task? | Route/CLI/library map and repeatable responses from the active binary | Pilot implementer / before reader implementation |
| Which source files may be retained and redistributed? | Pinned repository commit, paths, license review and file hashes | Dataset curator / before ingestion |
| Does recovery cover the selected backend and every pilot state file? | State inventory, backup manifest and post-restore comparisons | Recovery reviewer / before recovery acceptance |
| How should partial transition persistence be repaired? | Failure/retry matrix and comparison of reconciliation versus atomic registration | Core maintainer / separate follow-up |
| Can a first-time reader explain the evidence and update? | Recorded walkthrough against the pilot acceptance criteria | Reader/reviewer / pilot completion |

Do not label unknowns as absent features or completed capabilities. Update each row with implementation/evidence links when a later task changes the assessment. Overall pilot status remains **not started**.

<a id="japanese"></a>

## 日本語

### 1. 確認対象と評価方法

確認したmainは`81ed02bcd648ea7cfbe3aa4c74be1d7948281131`です。これは文書・ソース確認の基準であり、固定した公開候補ではありません。公開候補は[現在の状態](./CURRENT_IMPLEMENTATION_STATUS.md)に記載した`8c6b48082205a3af555130eec1f3e7d2ac8811fe`です。

仕様、スキーマ、実際に選択される一部のソース経路を確認しました。新たな動作検証やコード全体の監査ではありません。「既存」は記載した範囲の機能に根拠があることを示し、研究記録の実証合格を意味しません。過去の完了済み作業一覧やcapabilityの宣言だけでは、現行エンドポイントの証拠として不十分です。

状態の意味は、**既存**＝活用できる範囲を限定した基盤、**一部あり**＝仕組みはあるが目的の利用には追加作業が必要、**計画対象**＝今回の確認では対応済み機能として確立していないもの、**要確認**＝処理経路ごとの調査で保証範囲を決めるもの、です。優先度は、**P0**＝実証前または実証中、**P1**＝実証結果を踏まえた後続作業、**P2**＝その後の独立した作業、とします。

### 2. 機能ごとの差分

| ID | 目指す動作 | 現行の根拠と限界 | 状態 | 次の作業・優先度 |
|---|---|---|---|---|
| G01 | 正確な知識記録を保存・取得する | [スキーマ](../../schemas/knowledge-object.schema.json)に必須項目と9種類の型があり、`packages/core/src/lib.rs`に認証付き取込・取得がある | 既存 | 研究記録を既存の型へ対応付ける。ルートスキーマは変えない / P0 |
| G02 | 観測・根拠・解釈・留保を区別する | `inquiry`、`observation`、`evidence`、`claim`、`annotation`がある。研究固有の意味は中核で強制しない | 一部あり | プロファイルの制約と出典からの対応表を定める / P0 |
| G03 | 根拠・改訂・出典をたどる | [索引の実装](../../packages/indexer/src/lib.rs)の`IndexSnapshot`に関係・履歴・来歴の取得とテストがある。草案の全HTTP経路が実装済みとは限らない | 基盤は既存、利用側は一部あり | 実際のインターフェースを確認し、範囲を限定した閲覧機能を作る / P0 |
| G04 | 以前の判断を残して改訂を表示する | 不変オブジェクト、履歴、[有効ビューの仕様](../protocols/EFFECTIVE_VIEW_READ_API.md)がある。履歴の関係と認可された遷移の効果は別 | 一部あり | 実証では新しいIDと履歴を使い、アプリ側の選択を明示する / P0 |
| G05 | 判断者・範囲・根拠・未解決の異論を残す | 中核の受理と有効ビューの権限は、研究上の採用判断を実装するものではない | プロファイル層の計画対象 | 判断の注釈を独立して記録し、曖昧な選択を拒む / P0 |
| G06 | 特定の版を引用し、元資料を検証する | [識別・来歴の仕様](../protocols/IDENTITY_AND_PROVENANCE.md)は正規IDと意味上のキーを区別する。`rawRef`は外部資料を保存・検証しない | 一部あり | ID、正規バイト列のSHA-256、保存可能な元資料の写しを対応表にする / P0 |
| G07 | 保存構造を知らなくても検索・閲覧する | 型・グラフの検索はある。[HTTP仕様](../operations/HTTP_CARRIER_CONTRACT.md)はプロファイル草案の要求より狭い。全文検索の順位付けやUIは今回確認できていない | 一部あり | 型による絞込みと1段階の関係表示から始め、全文検索・SDKは後続とする / P0、その後P1 |
| G08 | 中断後に派生処理を復旧する | 索引の管理・チェックポイントはある。[遷移の取込](../../packages/relay/src/transition_api.rs)は遷移とキュー要求を別々に書く | 要確認 | 書込・再試行・未処理照合をバックエンドごとに調べ、不可分な登録や自動完了を先に保証しない / P1 |
| G09 | データを移送し、ノードを復旧する | [アーカイブ](../operations/FILE_ARCHIVE_CARRIER_CONTRACT.md)は公開要求を再取込する。[運用手順](../operations/V1_0_OPERATOR_RUNBOOK.md)に検証済みバックアップ・分離復元がある | 範囲を限定した既存経路 | 再取込とバックアップを別々に試し、元資料と保存範囲を棚卸しする / P0 |
| G10 | 認証・認可・真偽を区別する | 公開者の署名検証と権限の意味が分離されている。任意の発行者の主張や出典記述まで自動検証されるわけではない | 既存の境界 | 表示、異常系、報告書でも区別を維持する / P0 |
| G11 | 読取・検索・書出し・AIまで非公開の権限を守る | 管理RBACはある。完全な個別レコードACLとOAuth/OIDCは[v1.0の範囲](./ROADMAP_TO_V1_0.md)に含まれない | 計画対象 | 実証は公開資料・ローカル限定。権限の一貫した設計は別途行う / P2 |
| G12 | 機密資料を撤回・物理削除する | 遷移による撤回と運用上の保存期限管理はあるが、安全な完全消去は保証しない | 要確認 | 撤回、保存期限、物理削除を分け、機密資料を実証に入れない / P2 |
| G13 | 正確な出典付きのAI回答を任意で使う | AI・ベクトル検索はv1.0の保証対象外 | 計画対象 | 引用を追跡できてから、モデル・設定・出典の版・鮮度を管理して追加する / P2 |
| G14 | 独立した複数ノードを同期する | 実装は単一ノード。分散関連の文書は将来方針を含む | 計画対象 | 先に持ち運びを検証し、複製・合意は別の仕様で扱う / P2 |

### 3. 実行経路と互換性の判断

[リレーのパッケージ設定](../../packages/relay/Cargo.toml)が選ぶ入口は`main_v0_6_final.rs`です。公開・HTTP処理は`main_v0_6_release.rs`、その他のCLIは`main_v0_5_index.rs`経由で処理します。古い`main.rs`だけを読むのではなく、選択される経路を確認します。実証の最初の実装工程で、対象の版で動くコマンド・経路・応答を記録します。

| 変更の種類 | 初期方針 |
|---|---|
| プロファイル、閲覧機能、サンプル、出典対応表 | 固定したプロトコルの意味を変えない追加を優先し、スキーマと実行中の実装の両方で検証する |
| 任意のHTTP・検索機能 | 互換性、検索上限、試験を別途確認し、草案の経路を実装済みとみなさない |
| 後続作業の登録・照合、バックエンド変更 | 永続形式、部分保存からの復旧、再実行、ロールバック、新候補への影響を調べる |
| 正規ID、既定識別規則、署名対象、必須項目 | 実証では維持する。変更には独立したバージョン付き設計と移行評価が必要 |
| 非公開の権限、物理削除、分散動作 | 独立した作業とし、本計画から保証を推定しない |

実際の分類は[互換性方針](../architecture/V1_COMPATIBILITY_POLICY.md)と既存仕様に従います。優先度は製品バージョンの割当てでも、固定した候補を変える承認でもありません。

### 4. 未確認事項と次に必要な証拠

| 確認事項 | 必要な証拠 | 担当する役割・判断時点 |
|---|---|---|
| 閲覧に必要な全操作を既存の公開読取面で実現できるか | 実行対象バイナリのHTTP・CLI・ライブラリ対応表と再現可能な応答 | 実証実装者・閲覧機能の実装前 |
| どの元資料を保存・再配布できるか | 固定したリポジトリコミット、パス、ライセンス確認、ファイルハッシュ | 資料選定担当・取込前 |
| 選択したバックエンドと実証の全状態が復旧範囲に入るか | 状態一覧、バックアップの明細、復元後の比較 | 復旧確認担当・復旧合格の判定前 |
| 遷移が部分保存された場合にどう修復するか | 障害・再試行の表と、照合方式・不可分登録方式の比較 | 中核保守担当・独立した後続作業 |
| 初見の読者が根拠と更新を説明できるか | 実証の受入条件に沿った操作記録 | 読者・確認担当・実証完了時 |

未確認のものを機能欠落や実装完了と断定しません。後続作業で状態が変わったら、実装・検証記録へのリンクを各行へ追加します。実証全体は**未着手**です。
