# Lingonberry Documentation Policy / Lingonberry 文書方針

**Status:** normative for the v1.0.0 documentation set  
**English is normative. Japanese is a synchronized translation.**

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

The fixed pre-version candidate is `f9543019f2c219aea3b085ff90f2da201b268a48`. Documentation and tooling commits after that commit do not redefine it. Formal 72-hour soak, privileged reference-host qualification, version preparation, release PR, tag, GitHub Release, and final publication evidence remain separate gates.

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

本方針は、Lingonberryの文書言語、分類、翻訳同期、inventory、およびrelease gateの規則を定めます。

目的は次のとおりです。

- 英語利用者と日本語利用者の双方に利用開始の入口を提供すること。
- 正本となる用語をsource code、schema、command、API、diagnostic、durable formatと一致させること。
- 日本語だけの要件が並行仕様になることを防ぐこと。
- 翻訳保守を、導入または通常運用に実質的な価値がある文書へ限定すること。
- 未解決の文書作業を生成inventoryで可視化すること。

### 2. 正本となる言語

仕様、互換性宣言、command contract、運用手順、release evidenceでは英語を正本とします。

英日併記文書の日本語部分は翻訳です。英語部分に存在しない要件、例外、手順、安全保証、release claimを日本語部分だけに追加してはいけません。

source identifier、command、configuration key、API field、event名、error code、file path、version文字列、hash、machine-readable valueは翻訳しません。

### 3. 英日併記形式

英日併記文書では、同一ファイル内で英語を先、日本語を後に配置します。両sectionの意味上の範囲と見出し順序を一致させます。

command、path、configuration example、candidate identifier、version番号、安全条件は両sectionで技術的に同一でなければなりません。

Tier 1文書の英語部分を変更する場合、同じpull requestで日本語部分も更新します。翻訳未更新warningは、削除されるまでrelease blockerです。

### 4. 分類

生成inventoryを、ファイル単位の分類と進捗の正本とします。

#### `BILINGUAL_REQUIRED`

repositoryの入口および通常operator向け文書のうち、運用範囲全体を英語と日本語で提供する必要があるものに使用します。

現在の必須集合は`scripts/generate-documentation-inventory.py`で管理します。範囲と同期をreviewした文書だけを`KEEP_BILINGUAL`にできます。

#### `BILINGUAL_SCOPED`

概要またはuser-facing部分を英日併記とし、詳細技術部分は英語のみでもよい文書に使用します。review済み文書は分類を変更せず`KEEP_BILINGUAL`にできます。

#### `ENGLISH_ONLY`

詳細protocol、architecture、security review、qualification、soak、evidence、CI、debugging、自動生成reference、maintainer専用文書に使用します。ただし明示的な決定がある場合を除きます。

historical文書は英語のみで維持できます。また、inventoryにその扱いを記録する場合、明確にnon-normativeな歴史的日本語を保持できます。

### 5. 最小英日併記operator surface

review済み英日併記文書全体で、新しいsingle-node operatorが次を実行できる状態にします。

1. productとsupported deployment boundaryを理解する。
2. supported binaryをinstallまたはbuildする。
3. 最小configurationを作成して検証する。
4. serviceをstart、stop、restartする。
5. health、readiness、log、diagnosticを確認する。
6. 適用可能な場合、最小objectをpublishおよびretrieveする。
7. backupを作成してverifyする。
8. isolated restoreを計画または実行する。
9. storageとindex状態をverifyする。
10. supported previous releaseからupgradeする。
11. upgrade失敗後に安全にrollbackまたは停止する。
12. 停止とescalationが必要な条件を判断する。

これらはroot README、operations index、quickstart、operator runbook、upgrade/rollback guideへ分散して構いません。本方針は、同じ手順をすべての文書へ重複記載することを要求しません。

### 6. Inventoryと完了状態

`docs/DOCUMENTATION_INVENTORY.md`は追跡対象Markdownから生成します。生成inventoryの手動編集は正本ではありません。

generatorは分類と完了状態を分離して管理しなければなりません。例えばreview済みTier 1文書は`BILINGUAL_REQUIRED`のまま、actionが`KEEP_BILINGUAL`、blockerが`no`になります。

分類集合に記載されていても追跡ファイルとして存在しないpathはinventory entryではなく、その事実だけではrelease blockerではありません。新しい必須文書を追加する場合は、ファイルを追加し、分類と翻訳同期をreviewします。

### 7. 自動検査

repositoryでは次を検査します。

- 生成inventoryのfreshness。
- 必須英日併記section markerと同期対象文書。
- documentation freeze boundary。
- 標準RustおよびJavaScript validation。
- 文書workflowで必要とされるcandidate-bound documentation walkthrough evidence。

これらのcheck成功は、formal soak、privileged reference-host qualification、version preparation、release PR、tag、GitHub Release、publication evidenceの完了を意味しません。

### 8. v1.0.0 release gate

文書準備完了には次が必要です。

- 追跡対象Markdownがすべて生成inventoryへ掲載されている。
- inventoryでv1.0 blockerとされた項目がすべて解消されている。
- review済み英日併記文書で英語と日本語が同期している。
- Tier 1文書に翻訳未更新warningがない。
- 通常のinstallation、configuration、backup、restore、upgrade、rollback手順が相互に矛盾しない。
- 英語のみの正本文書に日本語だけの要件が存在しない。
- release-facing文書が、完了済みcandidate evidenceと未完了release gateを正確に区別している。

固定pre-version candidateは`f9543019f2c219aea3b085ff90f2da201b268a48`です。このcommit以後の文書・tooling commitはcandidateを再定義しません。formal 72時間soak、privileged reference-host qualification、version preparation、release PR、tag、GitHub Release、最終publication evidenceは別のgateです。

### 9. 変更管理

v1.xでは次を適用します。

- 新しいuser-facing文書または通常operator向け文書を追加する際に分類する。
- Tier 1の英語変更では日本語も同期して変更する。
- 内部技術文書は英語のみをdefaultとする。
- 分類または完了状態の変更ではgeneratorとinventoryを更新する。
- 将来のmajor version方針で明示的に変更しない限り、英語を正本とする。
