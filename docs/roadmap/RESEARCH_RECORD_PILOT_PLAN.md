# Research Record Pilot Plan / 研究記録を使う最小実証計画

[English](#english) | [日本語](#japanese)

> English is normative for this plan. Japanese is a synchronized translation. Planned checks below have not been executed.
>
> この計画では英語を正本とし、日本語を同期翻訳とします。以下の実証試験は未実施です。

**Status: planned; dataset not selected; implementation and execution not started** | **Last updated: 2026-10-22**

## English

### 1. Goal, scope and non-goals

Demonstrate one end-to-end research-record workflow on Lingonberry: ingest, retrieve, follow evidence, inspect a revision and its limitations, transfer records, and recover them in isolation. Follow the [adoption design](../architecture/KNOWLEDGE_WORKFLOW_ADOPTION.md) and [gap analysis](./KNOWLEDGE_WORKFLOW_GAP_ANALYSIS.md).

Use an existing Bao research report and a related earlier/later record from `nkkmd/bao-nakakamado`, selected and pinned during step S1. This is a read-only use of existing research, not a new balance experiment, proof search, result revision or write to that repository. Do not infer research conclusions from chat memory. If the selected sources cannot support a genuine revision/clarification, choose another documented pair or mark that criterion blocked; do not fabricate a changed conclusion.

Initial scope: one study/question, approximately 10 Knowledge Objects, at most 20 source files totaling 5 MiB, one local node and a second isolated destination. Each publish request must satisfy the current input bounds, including the 1 MiB limit. Large result bundles remain out of scope. Multi-node sync, AI, full-text ranking, confidential data, production deployment, destructive testing and v1.0 qualification are excluded.

### 2. Dataset and representation

The following labels are planning labels, not actual canonical IDs or completed research claims. The exact count and mapping are frozen in S1 before execution; deviations need a recorded reason.

| Label | Existing type | Intended content |
|---|---|---|
| Q1 | `inquiry` | One research question as stated by the source |
| P1 | `reference` | Rule/software version and experiment conditions |
| E1 | `evidence` | Initial result source and exact location |
| O1 | `observation` | A bounded observation taken from E1 |
| C1 | `claim` | The initial interpretation, preserving uncertainty |
| L1 | `annotation` | Scope limits, unresolved questions and cautions |
| E2 | `evidence` | Later or supplementary result source |
| O2 | `observation` | Observation supported by E2 |
| C2 | `claim` | Documented revision or clarification of C1 |
| D1 | `annotation` | Pilot reader's explicit display/adoption decision, with scope and evidence; not an attribution to the original researcher |

Use `body.text` and `body.language`, retain required `provenance` and `rawRef`, and use `relations`/`lineage` within the current schema. Proposed profile namespace: `contexts["research-record-pilot"]`, containing a profile version and research grouping information; finalize its allowed keys and fixtures in S2. Do not introduce new root fields or new core types. Domain meaning belongs in the profile, not non-semantic `meta` alone.

Every relation records explicit `source`, `target` and `kind`; in the pilot, the source is the containing object's ID. Candidate kinds are `cites`, `supports`, `responds_to` and `annotates`. Use `lineage.type = revises` from C2 to C1 only when supported by the source. An evidence object's outgoing `supports` relation can be found from the claim through inbound navigation; source order must not be mistaken for causal order.

D1 names its decision-maker, scope, selected object IDs, reason and policy/profile version. It is a new pilot annotation, clearly distinguished from imported historical statements. It does not authorize a core Transition Object. The minimal main dataset uses no transitions; transition authority/partial-write remediation remains separate follow-up work. A second incompatible display decision is tested only as synthetic data and must produce an unresolved selection.

### 3. Source manifest and citation rules

S1 creates a versioned source manifest containing repository URL, immutable commit SHA, path, source-file SHA-256, selected heading/line span, license/redistribution disposition, retrieval time, and mapping to each imported object. Preserve permitted source bytes locally; explicitly mark anything omitted. Record any extraction, summary or translation and its author, rather than claiming a verbatim copy.

After publication, record the actual canonical ID and SHA-256 of complete canonical UTF-8 object bytes, with canonicalization rule version and digest scope. Never use the default semantic `identityKey` as a substitute for that digest. The reader shows the immutable source and object version; original research observation times and the new import/creation time remain distinct.

Source snapshots and this manifest are pilot artifacts, not automatically managed attachments or verified external sources under core `rawRef`. No network fetch is triggered merely by displaying a locator. Keep source authorship distinct from the pilot publisher's signature.

### 4. Work sequence and deliverables

All steps below are pending. Before coding, create a tracking issue with these boundaries and record the selected base commit. Implement additions on a separate branch. Placement follows existing responsibilities: profile documentation in `docs/profiles/`, sample objects in `fixtures/`, integration tests in `tests/`, application code under a reviewed responsibility-specific `packages/` path. These are proposed placements, not files already delivered.

| Step | Work | Deliverable / exit condition |
|---|---|---|
| S1 | Recheck Lingonberry main and release hold; select public research sources | Frozen baseline, licensed source manifest, dataset mapping and expected graph |
| S2 | Define profile and active interface map | Schema-valid objects, profile checks, exact commands/routes/library entry points and signing method; record gaps before claiming support |
| S3 | Publish and retrieve in isolated state | Signed requests, structured outcomes, canonical digests; duplicate/invalid/conflict results |
| S4 | Build a small local reader and complete a walkthrough | Question list, record detail, one-hop inbound/outbound evidence, history, limitations, explicit display selection and status labels |
| S5 | Test portability and recovery independently | Archive round-trip, source-package checks, verified backup/isolated restore, index-rebuild comparisons |
| S6 | Review evidence and report | Pass/fail/blocked per acceptance criterion, limitations, usability observations and scoped follow-up issues |

The reader may start as a CLI plus generated local HTML, without a hosted site or permanent service. It must obtain knowledge through supported read interfaces and render untrusted text as text. Browser-visible output must not contain signing keys or admin credentials. If a required read route is unavailable, use a documented CLI/library adapter or propose an additive API separately; do not silently read internal storage layouts.

### 5. Acceptance criteria

| ID | Check | Required result and retained evidence |
|---|---|---|
| A01 | Source fidelity | Every imported observation/claim maps to a pinned source location; qualifiers and uncertainty preserved; D1 clearly marked as a pilot decision |
| A02 | Validation and signature | All intended valid requests accepted under the selected policy; invalid signature and missing `rawRef` rejected with no canonical insertion |
| A03 | Duplicate and conflict | Identical signed resend is duplicate; separately re-signed different content under the same ID conflicts; original canonical bytes unchanged |
| A04 | Evidence navigation | Expected inbound/outbound links and types match a frozen graph; a missing target is shown as unresolved, not silently complete |
| A05 | Revision and judgment | C1 and C2 remain independently retrievable with original digests; reader explains revision and L1; D1 does not mutate them; incompatible synthetic decisions remain unresolved |
| A06 | Retrieval and restart | After graceful stop/restart, all expected canonical IDs and digests match; no implicit claim that indexes are current |
| A07 | Index reconstruction | Rebuild derived state using supported commands; compare fixed type/graph queries before/after; keep command outputs and freshness/verification results |
| A08 | Archive portability | Export with `export-archive`, import into empty isolated state with `import-archive`; IDs, canonical bytes and graph agree; repeat import is duplicate; source files are verified separately |
| A09 | Backup and recovery | Inventory selected storage/state paths; create and verify backup, restore to an unused destination using documented operations; compare all dataset IDs/digests and rebuilt queries; list uncovered state explicitly |
| A10 | Practical use | A reviewer follows the written instructions to find C2, name its exact evidence, explain the earlier interpretation and limitation, and identify unresolved data; record steps, elapsed time and assistance required |

Use a separate disposable dataset/directory for synthetic missing-target, conflicting-decision and rejection tests. They are software tests and must not be mixed into the research result. A successful HTTP status alone is not a stored-object assertion: retain structured status/code and readback evidence. Do not deliberately corrupt storage, exhaust disks or inject crashes in this pilot.

A09 cannot pass solely because A08 passed. Archive import does not cover all node state, and external source files require their own retained package/checksums. A backup's coverage must be verified for the chosen backend; uncovered material needed for the workflow must be preserved and checked separately or the criterion remains blocked. No transition-state recovery guarantee is inferred from a dataset without transitions.

### 6. Environment, stopping and evidence

Use fresh explicit directories for source material, live pilot state, archive destination, backup, restore destination and evidence. Bind locally, use disposable keys and keep secrets outside version control and published evidence. Start without privilege or systemd installation; record OS, tool versions, build commit, binary hashes, backend/configuration and command results. Confirm command support before execution; never aim restore/import at production state.

Stop on unexplained canonical-byte changes, a source/license mismatch, a storage/signature inconsistency, ambiguous recovery coverage, unexpected writes outside the pilot, or a need to change frozen protocol semantics. Preserve evidence and open a scoped follow-up rather than weakening a criterion. Ordinary bounded local restart/recovery checks do not authorize the held reference-host, disk-pressure or formal soak work.

The result bundle contains the source manifest and permitted snapshots, profile/dataset version, signed requests without private keys, expected graph, command/HTTP records, object digests, archive and backup coverage manifests, query comparisons, walkthrough notes and a final report. Store runtime outputs outside tracked source paths; if future tooling creates repository-local outputs, add appropriate ignore rules with that implementation. Use durable artifact storage and record its location, retention and checksums before calling the pilot reproducible.

### 7. Completion and next decision

The pilot passes only when A01–A10 pass with retained evidence. Missing user/reviewer observation or an unresolved implementation gap is **blocked**, not an automatic pass. A failed or blocked pilot may still yield a useful report, but is not complete success. The implementation owner assembles the evidence; a designated reviewer assesses it, with roles and any self-review limitation recorded.

After the report, decide separately which gaps justify shared core changes, profile/client improvements or deferral. Do not assign a release version until compatibility and scope are reviewed. The fixed candidate `8c6b48082205a3af555130eec1f3e7d2ac8811fe`, held real-host tests, and unpublished v1.0 status remain governed by [current implementation status](./CURRENT_IMPLEMENTATION_STATUS.md). This plan and a future successful pilot do not satisfy those release gates.

<a id="japanese"></a>

## 日本語

### 1. 目的、範囲、対象外

Lingonberryで、研究記録の取込・取得、根拠の閲覧、改訂と留保の確認、移送、分離した環境への復旧を一通り実証します。[採用設計](../architecture/KNOWLEDGE_WORKFLOW_ADOPTION.md)と[差分表](./KNOWLEDGE_WORKFLOW_GAP_ANALYSIS.md)に従います。

`nkkmd/bao-nakakamado`の既存研究報告と、関連する前後の記録をS1で選び、版を固定します。既存研究の読取利用であり、新しいバランス試験、必勝探索、研究結果の変更、元リポジトリへの書込は行いません。チャットの記憶から研究結論を推測しません。実際の改訂・補足を示す資料がなければ別の記録を選ぶか、その条件を実行不能とし、結論の変更を作りません。

初期範囲は、一つの研究・問い、約10件のKnowledge Object、合計5 MiB以下・20ファイル以下の元資料、ローカルノード一つと分離した移送先一つです。各公開要求は1 MiBを含む現行の入力上限を守ります。大きな結果一式は対象外とします。複数ノード同期、AI、全文検索の順位付け、機密データ、本番配置、破壊試験、v1.0の適格性検証も対象外です。

### 2. 資料と表現方法

以下は計画上のラベルであり、実際の正規IDや確定済みの研究主張ではありません。実行前のS1で件数と対応を固定し、変更には理由を残します。

| ラベル | 既存の型 | 記録する内容 |
|---|---|---|
| Q1 | `inquiry` | 元資料に書かれた一つの研究上の問い |
| P1 | `reference` | ルール・ソフトウェアの版と試験条件 |
| E1 | `evidence` | 最初の結果の出典と正確な記載箇所 |
| O1 | `observation` | E1から取り出した範囲を限定した観測 |
| C1 | `claim` | 不確実性も維持した当初の解釈 |
| L1 | `annotation` | 適用範囲、未解決事項、注意点 |
| E2 | `evidence` | 後続または補足結果の出典 |
| O2 | `observation` | E2に支えられる観測 |
| C2 | `claim` | 記録に基づくC1の改訂または補足 |
| D1 | `annotation` | 実証の読者による明示的な表示・採用判断と、その範囲・根拠。元の研究者の判断と偽らない |

`body.text`と`body.language`を使い、必須の`provenance`と`rawRef`を維持し、現行スキーマ内の`relations`と`lineage`を使います。プロファイルの名前空間案は`contexts["research-record-pilot"]`で、プロファイルの版と研究のグループ情報を含めます。使用できるキーとサンプルはS2で確定します。ルートフィールドや中核の型は追加しません。分野固有の意味を、非意味的な`meta`だけに置きません。

関係には`source`、`target`、`kind`を明記します。実証では`source`を、その関係を含むオブジェクト自身のIDにします。候補語彙は`cites`、`supports`、`responds_to`、`annotates`です。C2からC1への`lineage.type = revises`は、元資料に根拠がある場合だけ使います。根拠オブジェクトから出る`supports`は、主張側から入方向をたどって見つけられます。資料の並び順を因果関係とみなしません。

D1には判断者、対象範囲、選んだオブジェクトID、理由、判断規則・プロファイルの版を記録します。実証で新たに作った注釈として、過去の研究記録から区別します。中核のTransition Objectを認可するものではありません。最小の本データには遷移を使わず、遷移の権限や部分保存への対処は後続作業とします。矛盾する二つ目の表示判断は合成データだけで試し、未解決と表示させます。

### 3. 出典対応表と引用規則

S1で作るバージョン付き対応表には、リポジトリURL、不変のコミットSHA、パス、元ファイルのSHA-256、対象の見出し・行範囲、ライセンスと再配布可否、取得時刻、取込オブジェクトとの対応を記録します。保存可能な元のバイト列を保持し、省略したものを明記します。抽出、要約、翻訳をした場合は、その処理と担当者を記録し、原文そのものと偽りません。

公開後は、実際の正規IDと、完全な正規オブジェクトのUTF-8バイト列に対するSHA-256を、正規化規則の版・ダイジェスト対象とともに記録します。既定の意味上の`identityKey`で代用しません。閲覧機能では不変の出典・オブジェクトの版を示し、研究時の観測日時と、新たな取込・作成日時を区別します。

元資料の写しと対応表は実証成果物です。中核の`rawRef`によって自動管理される添付や、検証済みの外部資料ではありません。参照先の表示だけでネットワーク取得は行いません。元資料の著者と、実証で公開する署名者も区別します。

### 4. 作業順序と成果物

以下の工程はすべて未実施です。実装前に対象範囲を記したIssueを作成し、基準コミットを記録します。追加作業は別ブランチで行います。配置は既存の役割に従い、プロファイル文書は`docs/profiles/`、サンプルは`fixtures/`、統合試験は`tests/`、アプリのコードは責務を確認した`packages/`配下とします。これは配置案であり、今回作成済みのファイルではありません。

| 工程 | 作業 | 成果物・次へ進む条件 |
|---|---|---|
| S1 | Lingonberryのmainと試験保留を再確認し、公開研究資料を選ぶ | 固定した基準、利用条件付きの出典対応表、データ対応表、期待する関係図 |
| S2 | プロファイルと実際のインターフェースを定める | スキーマに適合したオブジェクト、追加検証、正確なコマンド・HTTP・ライブラリの対応、署名方法。不足を対応済みとせず記録する |
| S3 | 分離した保存領域へ公開・取得する | 署名済み要求、構造化された結果、正規データのダイジェスト、重複・不正・競合の結果 |
| S4 | 小さなローカル閲覧機能で手順を通す | 問いの一覧、詳細、入方向・出方向の1段階の根拠、履歴、留保、明示的な表示選択、状態表示 |
| S5 | 持ち運びと復旧を別々に試す | アーカイブ往復、元資料一式の照合、検証済みバックアップ・分離復元、索引再構築の比較 |
| S6 | 検証記録を確認して報告する | 各受入条件の合格・不合格・実行不能、限界、使いやすさの観察、範囲を絞った後続Issue |

閲覧機能はCLIと生成したローカルHTMLから始められ、公開サイトや常設サービスを必須としません。対応する読取インターフェースで記録を取得し、信頼できない本文を文字列として表示します。ブラウザーに見える出力に署名秘密鍵や管理資格情報を含めません。必要な読取経路がなければ、文書化したCLI・ライブラリのアダプターを使うか、追加APIを別途提案します。保存領域の内部構造を暗黙に直接読みません。

### 5. 受入条件

| ID | 確認すること | 必要な結果と保存する証拠 |
|---|---|---|
| A01 | 出典への忠実さ | 全ての取込観測・主張を固定した出典箇所に対応付け、不確実性を維持する。D1は実証での判断と明記する |
| A02 | 検証と署名 | 正常要求が選択した方針で受理され、不正署名と`rawRef`欠落は拒否され、正本へ挿入されない |
| A03 | 重複と競合 | 同じ署名済み要求の再送は重複。同じIDで異なる内容を改めて有効に署名した要求は競合。元の正規バイト列は不変 |
| A04 | 根拠をたどる操作 | 入・出方向の関係と型が固定した関係図に一致する。参照先欠落を未解決と表示し、完全とみなさない |
| A05 | 改訂と判断 | C1・C2を元のダイジェストで個別取得でき、改訂理由とL1を読める。D1は元記録を変えず、合成した対立判断は未解決になる |
| A06 | 取得と再起動 | 通常の停止・再起動後に全対象IDとダイジェストが一致する。索引が最新と暗黙に主張しない |
| A07 | 索引の再構築 | 対応コマンドで再構築し、固定した型・グラフ検索の前後を比較する。出力、鮮度、検証結果を残す |
| A08 | アーカイブの移送 | `export-archive`で書き出し、空の分離領域へ`import-archive`で取り込む。ID・正規バイト列・関係が一致し、再取込は重複になる。元資料は別に検証する |
| A09 | バックアップと復旧 | 選んだ保存・状態パスを棚卸しし、バックアップを作成・検証して、文書に沿って未使用の場所へ復元する。全対象ID・ダイジェスト・再構築後の検索を比較し、含まれない状態を明記する |
| A10 | 実際の使いやすさ | 確認担当が手順を読み、C2と正確な根拠を見つけ、過去の解釈・留保・未解決情報を説明できる。操作、所要時間、必要だった支援を記録する |

参照先欠落、対立判断、拒否の合成試験は、独立した使い捨てデータ・ディレクトリで行います。ソフトウェア試験であり、研究結果へ混ぜません。HTTPの成功だけを保存成功とせず、構造化された状態・コードと再取得の証拠を残します。この実証では保存領域の故意の破損、ディスク圧迫、クラッシュ注入は行いません。

A08の合格だけでA09を合格にできません。アーカイブはノードの全状態を含まず、外部の元資料には別の保存一式とチェックサムが必要です。バックアップの範囲は選んだバックエンドで確認します。必要な資料が含まれなければ、別途保存・検証するか実行不能とします。遷移を含まないデータから、遷移状態の復旧保証を推定しません。

### 6. 環境、停止条件、検証記録

元資料、実証の稼働領域、アーカイブ先、バックアップ、復元先、検証記録は、新しい明示的なディレクトリに分けます。ローカル接続と実証専用鍵を使い、秘密情報をGitや公開検証記録に入れません。管理者権限やsystemdへの導入なしで開始し、OS、ツールの版、ビルドコミット、バイナリハッシュ、バックエンド・設定、コマンド結果を記録します。実行前にコマンドの対応を確認し、本番領域を復元・取込先にしません。

説明できない正規バイト列の変化、出典・ライセンスの不一致、保存・署名の不整合、曖昧な復旧範囲、実証領域外への予期しない書込、固定プロトコルの意味変更が必要になった場合は停止します。条件を緩めず、証拠を保持して後続作業へ切り出します。範囲を限定した通常の再起動・復旧確認は、保留中の基準ホスト検証・ディスク圧迫試験・正式連続稼働試験の再開許可ではありません。

成果物には、出典対応表と保存可能な元資料、プロファイル・データの版、秘密鍵を含まない署名済み要求、期待する関係図、コマンド・HTTP記録、オブジェクトのダイジェスト、アーカイブ・バックアップ範囲の明細、検索結果の比較、操作観察、最終報告を含めます。実行時出力は追跡対象ソースの外に保存します。将来のツールがリポジトリ内に出力する場合は、その実装で適切な除外規則を追加します。再現可能と判断する前に、永続的な成果物保存先、保持期間、チェックサムを記録します。

### 7. 完了と次の判断

A01〜A10がすべて、保存した証拠付きで合格した場合だけ実証成功とします。利用者・確認担当の操作観察が未実施の場合や、実装上の不足が未解決の場合は**実行不能**とし、自動的に合格としません。不合格・実行不能でも報告書は作れますが、実証成功とは区別します。実装担当が証拠をまとめ、指定した確認担当が評価し、役割と自己確認になった場合の限界を記録します。

報告後に、中核の共通機能へ進めるもの、プロファイル・クライアントを改善するもの、見送るものを別々に判断します。互換性と範囲の確認前に公開バージョンを割り当てません。固定候補`8c6b48082205a3af555130eec1f3e7d2ac8811fe`、実機試験の保留、v1.0未公開の状態は[現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)に従います。この計画や将来の実証成功は、それらの公開条件を満たすものではありません。
