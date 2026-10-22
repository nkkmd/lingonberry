# Knowledge Workflow Adoption / 知識活用方針の採用設計

[English](#english) | [日本語](#japanese)

> English is normative for this design proposal. Japanese is a synchronized translation. This document does not override existing protocol or release contracts.
>
> この設計提案では英語を正本とし、日本語を同期翻訳とします。既存のプロトコル仕様や公開条件を変更する文書ではありません。

**Status: proposed adoption design; implementation not started** | **Last updated: 2026-10-22**

## English

### 1. Purpose and decision scope

Prioritize a complete knowledge workflow: record a question, preserve evidence, inspect reasoning, revise a judgment, and reproduce the record elsewhere. Build on Lingonberry's existing single-node foundation. The long-term distributed-commons direction remains, while the next practical milestone is one useful local application.

This proposal adopts a development direction, not a replacement protocol. The [gap analysis](../roadmap/KNOWLEDGE_WORKFLOW_GAP_ANALYSIS.md) records current capabilities and uncertainties. The [research-record pilot](../roadmap/RESEARCH_RECORD_PILOT_PLAN.md) defines the first bounded evaluation. Approval or merge of these documents is not implementation completion or a release decision.

### 2. Responsibilities

| Layer | Responsibility | Boundary |
|---|---|---|
| Protocol and canonical storage | Validation, canonical bytes, identity, authenticated publication, immutable records, documented transitions and recovery | Does not decide scientific truth or application endorsement |
| Research application profile | Research vocabulary, additional validation, source mapping, judgment and limitation records | Uses existing fields and types; does not redefine canonical identity or signatures |
| Client and reader | Entry forms or CLI, evidence navigation, history, search, clear status display | Uses supported interfaces; does not edit storage files directly |
| Derived processing | Rebuildable indexes, optional summaries and semantic search | Preserves source references and exposes freshness; never becomes the canonical source |

Toitoi remains an independent profile example. Bao-specific rules, win-rate interpretation and experiment identifiers stay in the research profile. The pilot must not require Toitoi to run.

### 3. Principles to retain and extend

1. Preserve accepted objects. Corrections create new objects and explicit history. Authorized Transition Objects alone govern core effective-view changes; a `revises` lineage edge is not transition authorization.
2. Distinguish structural validity, signature verification, authority, source availability and content truth. Opposing claims with different IDs may coexist; an immutable-ID content conflict remains an error.
3. Record application endorsement separately from core acceptance and effective-view state. A pilot judgment record names its scope, evidence and decision-maker. Multiple unresolved judgments remain visible; timestamps do not silently select a winner.
4. Preserve provenance and `rawRef`. A source locator alone does not retain or verify source bytes. The pilot keeps permitted source snapshots and explicit hashes outside the protocol object, with a manifest linking them to objects.
5. Keep canonical storage authoritative. Record persistence and derived processing outcomes separately. A failed derived operation must not relabel a durable write as absent.
6. Make export, isolated restore and index reconstruction part of the user workflow. Archive replay and node backup are different operations with different coverage.

### 4. Identity and version mapping

The initial profile uses existing canonical object IDs for immutable versions. A revised interpretation receives a new ID and a `lineage` edge. A stable research-record grouping key may be carried in a namespaced `contexts` entry after profile review; it is an application grouping key, not a new canonical identifier. Do not place judgment semantics only in non-semantic `meta`.

Evidence manifests record SHA-256 over explicitly named bytes: complete canonical object UTF-8 bytes or original source-file bytes. They are not the existing semantic `identityKey`. Do not replace the current default identity rule, recompute old IDs, or infer semantic equivalence from hash similarity. Exact citation uses the immutable object ID plus its checked digest, not a moving “latest” alias.

The existing required fields, object types, schema versions and canonicalization remain unchanged for the pilot. Namespaced profile values require fixtures and validation before use; this design does not add runtime support by declaring them.

### 5. Implementation direction

Reuse the existing Rust workspace and File/SQLite backend interfaces. The first client uses documented publication/retrieval and narrowly scoped read integration. Do not consolidate all storage into one database or remove existing binaries merely to imitate a greenfield design.

The storage/derived-work boundary requires a separate review: transition append and reevaluation-intent append are currently separate writes. Compare an atomic outbox with durable-log reconciliation for each affected backend, including restart and retry behavior. Select a remedy only after that review; no cross-file atomicity is assumed. This is not a prerequisite for planning the read-oriented pilot, but unresolved partial persistence prevents a stronger recovery claim.

Start with type/relationship/history lookup and a small reader. Full-text search, SDK packaging and larger datasets follow measured usability needs. AI is optional future work: generated records must identify source versions and generation settings, be marked as generated, and become visibly stale when relevant source or policy inputs change. AI output cannot silently endorse or overwrite knowledge.

### 6. Access, withdrawal and deletion

The pilot uses only public, redistributable source material, disposable signing keys, a local listener and an isolated state directory. Administrative RBAC is not evidence of per-object read authorization. Multi-user private use must wait for an explicit access design covering retrieval, search, attachments, export and AI input.

Withdrawal changes applicability or visibility under its contract; it is not secure erasure. Design physical deletion, retention and backup handling separately before accepting data that requires those guarantees. The pilot does not modify immutable ledgers or exercise deletion of production data.

### 7. Compatibility and release tracks

The active pre-version v1.0 candidate remains `8c6b48082205a3af555130eec1f3e7d2ac8811fe`; the [current implementation status](../roadmap/CURRENT_IMPLEMENTATION_STATUS.md) is the release-state authority. Reference-host preflight, disk-pressure rehearsal and formal 72-hour soak remain on hold. This proposal does not resume them.

Documentation and isolated application experiments can proceed separately from release qualification. A runtime, protocol, durable-format, API, CLI or default-behavior change included in the release requires explicit candidate selection and fresh candidate-bound evidence under the [release roadmap](../roadmap/ROADMAP_TO_V1_0.md). Do not reuse previous evidence as proof for changed software.

The [v1 compatibility policy](./V1_COMPATIBILITY_POLICY.md) governs changes. Prefer additive clients/profiles and reviewed optional capabilities. Breaking canonicalization, identity or signature changes need a separately versioned proposal and migration assessment; after v1.0 publication, normal breaking changes belong to a later major version. No version number or release date is assigned here.

### 8. Next decisions and completion

1. Review these three documents and their inventory/navigation entries.
2. Before implementation, create a scoped tracking issue with the pilot dataset, API mapping, safety conditions and acceptance tests.
3. Implement and evaluate the pilot on an isolated branch/environment, then record pass, fail or blocked for every criterion.
4. Promote only demonstrated shared needs into core development. Treat domain-only needs as profile work and record deferred work explicitly.

Success means a reader can trace a judgment to exact evidence, understand a correction and reproduce the same records after transfer/recovery. It does not mean that the research conclusion is true, that the game is balanced, or that v1.0 is qualified.

<a id="japanese"></a>

## 日本語

### 1. 目的と今回決める範囲

問いを記録し、根拠を保存し、判断の理由を読み、判断を更新し、別環境で記録を再現するまでの流れを優先します。既存の単一ノード基盤を活用します。分散知識コモンズという長期方針を維持しながら、次の実用上の目標を「ローカルで役立つ一つのアプリケーション」に置きます。

これは開発方針の採用提案であり、プロトコルの置換ではありません。[差分表](../roadmap/KNOWLEDGE_WORKFLOW_GAP_ANALYSIS.md)で現在の機能と未確認点を整理し、[研究記録の最小実証計画](../roadmap/RESEARCH_RECORD_PILOT_PLAN.md)で最初の評価範囲を定めます。文書の承認・統合は、実装完了やリリースの決定を意味しません。

### 2. 役割分担

| 層 | 担うこと | 境界 |
|---|---|---|
| プロトコルと正本ストレージ | 検証、正規化されたバイト列、識別、認証付き公開、不変の記録、仕様に沿った遷移と復旧 | 科学的な真偽やアプリでの採用判断を決めない |
| 研究用アプリケーションプロファイル | 研究の語彙、追加検証、出典との対応、判断・留保の記録 | 既存フィールドと型を使い、canonical identityや署名を再定義しない |
| クライアントと閲覧機能 | 入力画面またはCLI、根拠の閲覧、履歴、検索、状態の明示 | 対応するインターフェースを使い、保存ファイルを直接編集しない |
| 派生処理 | 再構築できる索引、任意の要約・意味検索 | 出典参照と鮮度を示し、正本にはならない |

Toitoiは独立したプロファイルの例として維持します。Baoのルール、勝率の解釈、実験識別子は研究プロファイル側で扱い、実証の実行にToitoiを必須としません。

### 3. 維持・拡張する原則

1. 受理したオブジェクトを保持します。訂正は新しいオブジェクトと明示した履歴で表します。中核の有効ビューを変更するのは、認可されたTransition Objectです。`revises`という履歴の関係だけでは、遷移は認可されません。
2. 形式の妥当性、署名検証、権限、出典の利用可能性、内容の真偽を区別します。異なるIDの対立する主張は共存できます。同じ不変IDに異なる内容を保存する競合は、引き続きエラーです。
3. アプリでの採用判断を、中核の受理や有効ビューの状態と分けて記録します。実証の判断記録には対象範囲、根拠、判断者を示します。未解決の判断が複数ある場合は併記し、時刻で勝手に一つを選びません。
4. `provenance`と`rawRef`を維持します。出典の場所だけでは、元のバイト列の保存・検証を保証できません。実証では保存が許される元資料の写しとハッシュ値をプロトコルオブジェクトの外に保持し、対応表で結び付けます。
5. 正本ストレージを基準にします。保存結果と派生処理の結果を別々に扱い、派生処理の失敗を理由に、永続化済みの記録を未保存と表示しません。
6. 書き出し、分離した環境への復元、索引の再構築を利用手順に含めます。アーカイブの再取込とノードのバックアップは、保存範囲が異なる別の操作です。

### 4. 識別子と版の対応

初期プロファイルでは、既存のcanonical object IDを不変の版の参照に使います。解釈を改訂するときは新しいIDを発行し、`lineage`で結びます。研究記録をまとめる固定キーは、プロファイルの確認後に名前空間を設けた`contexts`の項目へ記録できます。これはアプリのグループ化キーであり、新しい正規識別子ではありません。採用判断の意味を、非意味的な`meta`だけに置きません。

検証用の対応表には、完全な正規オブジェクトのUTF-8バイト列、または元資料ファイルのバイト列など、対象を明示したSHA-256を記録します。既存の意味上の`identityKey`とは区別します。現在の既定識別規則や過去のIDを変更せず、ハッシュの類似から意味の一致を推定しません。正確な引用には、不変のオブジェクトIDと検証したダイジェストを使い、移動する「最新版」だけに依存しません。

実証では既存の必須フィールド、型、スキーマバージョン、正規化を維持します。名前空間を設けるプロファイル値も、使用前にサンプルと検証が必要です。設計文書に書くだけで実装済みにはなりません。

### 5. 実装の進め方

既存のRustワークスペースとFile/SQLiteバックエンドの境界を活用します。最初のクライアントは、文書化された公開・取得と、範囲を限定した読取連携を使います。ゼロベース案に合わせるためだけに保存先を一つのDBへ集約したり、既存バイナリを廃止したりしません。

保存と派生処理の境界は別途確認します。現在、遷移の追記と再評価要求の追記は別々の書込です。対象バックエンドごとに、同じトランザクションで後続作業を登録する方式と、永続ログから未処理分を照合する方式を、再起動・再試行も含めて比較します。確認後に対策を選び、複数ファイルへの書込を不可分とはみなしません。閲覧中心の実証計画は先に進められますが、部分保存の問題が未解決なら、それを超える復旧保証は主張できません。

まず型・関係・履歴の検索と、小さな閲覧機能を整えます。全文検索、SDKの配布、大規模データへの対応は、利用上の必要性を測って追加します。AIは将来の任意機能です。生成物に出典の版と生成設定を記録し、生成物であると表示し、関係する出典や判断規則が変わったら古いことを示します。AIが知識を暗黙に採用・上書きすることは認めません。

### 6. アクセス、撤回、削除

実証は公開・再配布可能な資料、実証専用の署名鍵、ローカル接続、分離した保存領域だけで行います。管理APIのRBACは、オブジェクトごとの読取権限を保証しません。複数利用者の非公開運用には、取得、検索、添付、書き出し、AI入力を一貫して制御する設計が必要です。

撤回は仕様に従って適用や可視状態を変えるもので、安全な完全消去ではありません。物理削除、保存期限、バックアップの扱いは、その保証が必要なデータを受け入れる前に別途設計します。実証では不変の台帳を変更せず、本番データの削除も行いません。

### 7. 互換性とリリースの扱い

バージョン更新前の公開候補は`8c6b48082205a3af555130eec1f3e7d2ac8811fe`を維持し、リリースの状態は[現在の実装状況](../roadmap/CURRENT_IMPLEMENTATION_STATUS.md)を正本とします。基準ホストでの事前検証、ディスク圧迫試験の予行演習、正式72時間試験は保留中です。この提案は再開指示ではありません。

文書作業と分離したアプリ実証は、公開に向けた検証とは別に進められます。実行時の動作、プロトコル、永続形式、API、CLI、既定動作を変えたものを公開対象に含める場合は、[公開ロードマップ](../roadmap/ROADMAP_TO_V1_0.md)に従い候補を明示的に選定し、その候補の検証記録を新たに取得します。過去の検証を、変更後のソフトウェアの証拠として流用しません。

変更には[v1互換性方針](./V1_COMPATIBILITY_POLICY.md)を適用します。追加型のクライアント・プロファイルと、確認済みの任意機能を優先します。正規化、識別、署名を壊す変更には、独立したバージョン付き提案と移行評価が必要です。v1.0公開後の通常の破壊的変更は、後のメジャーバージョンで扱います。ここではバージョン番号や公開日を決めません。

### 8. 次の判断と完了条件

1. 3文書と文書一覧・案内への登録を確認します。
2. 実装前に、実証資料、APIとの対応、安全条件、受入試験を定めた作業Issueを作成します。
3. 分離したブランチ・環境で実証し、各条件の合格・不合格・実行不能を記録します。
4. 実証で必要と分かった共通機能だけを中核へ取り込みます。分野固有の要件はプロファイルに置き、見送る作業も明記します。

成功とは、読者が判断から正確な根拠へ戻れ、訂正理由を理解でき、移送・復旧後も同じ記録を再現できることです。研究結論の真偽、ゲームの公平性、v1.0の公開適格性を証明するものではありません。
