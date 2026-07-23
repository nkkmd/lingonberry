# Lingonberry Documentation Policy / Lingonberry 文書方針

**Status: normative**  
**Applies from: v1.0.0 documentation freeze**

> English is the normative language for Lingonberry documentation. Japanese translations are provided for accessibility. If the English and Japanese texts differ, the English text takes precedence.
>
> Lingonberryの文書では英語を正本とします。日本語は利用しやすさのために提供される翻訳です。英語と日本語の内容に差異がある場合は、英語を優先します。

## English

### 1. Purpose

This policy defines which Lingonberry documents are bilingual, which remain English-only, how bilingual files are structured, and what documentation work must be completed before v1.0.0 is released.

The goals are to:

- provide an accessible entry point for English- and Japanese-speaking users;
- keep protocol, implementation, and maintenance terminology consistent with source code and public interfaces;
- prevent Japanese-only requirements or behavior from becoming an unofficial parallel specification;
- limit translation maintenance to documents where it materially improves installation, operation, security, or release adoption;
- make documentation completeness an explicit v1.0.0 release gate.

### 2. Normative language

English is normative for all Lingonberry documentation, specifications, compatibility statements, command contracts, and release evidence.

Japanese text in a bilingual document is a translation of the English section. It must not introduce requirements, exceptions, procedures, or guarantees that do not appear in the English section.

Source code identifiers, CLI commands, configuration keys, API fields, event names, error codes, file paths, and machine-readable values must not be translated.

### 3. Bilingual document format

A bilingual document must contain English first and Japanese second in the same file.

Recommended structure:

```markdown
# Document Title / 文書タイトル

> English is normative. Japanese is a translation.
> 英語を正本とし、日本語はその翻訳です。

## English

...

---

## 日本語

...
```

The English and Japanese sections must use equivalent heading order and semantic scope. Commands and configuration examples must remain technically identical unless a locale-specific path or explanatory note is explicitly required.

### 4. Document classification

#### Tier 1: bilingual required

The following must be bilingual before v1.0.0:

- `README.md`;
- `SECURITY.md`;
- project overview or introduction document;
- Getting Started guide;
- installation and initial configuration guide;
- standard start, stop, restart, health, and readiness procedures;
- backup and restore runbook;
- upgrade and rollback runbook;
- basic troubleshooting runbook;
- v1.0.0 release summary and operator-visible migration notes;
- this documentation policy.

#### Tier 2: bilingual, scoped to essential content

The following should provide a bilingual high-level section, while detailed technical material may remain English-only:

- `CONTRIBUTING.md`;
- high-level architecture overview;
- compatibility and support policy;
- public roadmap summary;
- migration overview.

#### Tier 3: English only

The following are English-only unless a later decision explicitly changes their classification:

- detailed architecture and storage internals;
- protocol, API, CLI, and configuration reference material;
- source-code comments and generated reference documentation;
- developer setup, CI, test-matrix, and debugging documentation;
- qualification, formal-soak, crash-matrix, disk-pressure, and reference-host procedures;
- release evidence, manifests, machine-readable reports, and internal maintainer runbooks;
- issue templates, pull-request templates, commit messages, and release automation documentation.

Formal-soak and qualification documents are not part of the bilingual minimum operator runbook. A bilingual overview may link to them, but the executable and normative procedures remain English-only.

### 5. Minimum bilingual operator runbook

The bilingual operator surface must let a new operator perform the following without reading internal qualification material:

1. understand what Lingonberry is and its supported deployment boundary;
2. install the supported package or binaries;
3. create and validate the minimum configuration;
4. start, stop, and restart the supported service;
5. check health, readiness, and logs;
6. publish and retrieve a minimal object where applicable;
7. create and verify a backup;
8. perform or plan an isolated restore;
9. verify storage and index state;
10. upgrade from the supported previous release;
11. roll back or stop safely after a failed upgrade;
12. perform basic failure diagnosis and know when to stop and escalate.

### 6. Translation synchronization

A change to the English section of a Tier 1 bilingual document must update the corresponding Japanese section in the same pull request.

If an exceptional pull request cannot update the translation, the document must carry an explicit stale-translation warning, and a blocking follow-up issue must be opened before release. v1.0.0 must not be released with a stale warning in a Tier 1 document.

English and Japanese sections must agree on:

- version numbers;
- support status;
- command names and arguments;
- file paths;
- configuration keys and precedence;
- compatibility promises;
- safety warnings;
- backup, restore, upgrade, and rollback order;
- release-blocking conditions.

### 7. Documentation organization

The preferred v1.x documentation layout is:

```text
README.md
SECURITY.md
CONTRIBUTING.md
docs/
├── DOCUMENTATION_POLICY.md
├── OVERVIEW.md
├── getting-started/
│   └── GETTING_STARTED.md
├── runbooks/
│   ├── INSTALLATION.md
│   ├── START_STOP.md
│   ├── HEALTH_CHECK.md
│   ├── BACKUP_RESTORE.md
│   ├── UPGRADE.md
│   └── TROUBLESHOOTING.md
├── architecture/
├── reference/
├── development/
├── roadmap/
├── qualification/
└── releases/
```

Existing files may be retained where renaming would introduce unnecessary release risk, but the documentation inventory must record their target classification and eventual destination.

### 8. v1.0.0 pre-release documentation work

The following work is required before v1.0.0 release publication.

#### Policy and inventory

- [x] Adopt a normative documentation language policy.
- [ ] Create a complete inventory of existing Markdown and operator-facing text documents.
- [ ] Classify every document as `BILINGUAL_REQUIRED`, `BILINGUAL_SCOPED`, `ENGLISH_ONLY`, `MERGE`, `ARCHIVE`, or `DELETE`.
- [ ] Identify duplicate, obsolete, conflicting, and unlinked documents.
- [ ] Record the target path and action for each document.

#### Tier 1 bilingual documents

- [ ] Normalize `README.md` to English-first, Japanese-second form.
- [ ] Normalize `SECURITY.md` to English-first, Japanese-second form.
- [ ] Establish or normalize the bilingual project overview.
- [ ] Establish or normalize the bilingual Getting Started guide.
- [ ] Establish the minimum bilingual installation/configuration runbook.
- [ ] Establish the minimum bilingual start/stop/restart runbook.
- [ ] Establish the minimum bilingual health/readiness/log runbook.
- [ ] Establish the minimum bilingual backup/restore runbook.
- [ ] Establish the minimum bilingual upgrade/rollback runbook.
- [ ] Establish the minimum bilingual troubleshooting runbook.
- [ ] Prepare the bilingual v1.0.0 release summary and migration notes.

#### English-only normalization

- [ ] Convert mixed-language internal architecture and reference documents to English.
- [ ] Convert qualification, soak, evidence, CI, and maintainer-only procedures to English.
- [ ] Remove Japanese-only normative requirements from English-only documents.
- [ ] Preserve historical Japanese material only when it is clearly marked archived and non-normative.

#### Consistency and navigation

- [ ] Add a documentation index linking all supported user, operator, developer, and maintainer documents.
- [ ] Repair links affected by document moves or renames.
- [ ] Standardize terminology, product name, version notation, command spelling, and path notation.
- [ ] Ensure every supported operational action has exactly one normative procedure.
- [ ] Ensure README and Getting Started do not duplicate detailed reference contracts.

#### Automated checks

- [ ] Add CI checks for required bilingual files and language-section markers.
- [ ] Add link checking for release-facing Markdown documents.
- [ ] Add checks for mismatched version numbers and frozen candidate identifiers where applicable.
- [ ] Add a stale-translation warning or review check for Tier 1 files.
- [ ] Include documentation policy and inventory files in the documentation freeze check.

#### Release gate

v1.0.0 documentation readiness requires all of the following:

- all Tier 1 documents are present and synchronized;
- no Tier 1 document contains a stale-translation warning;
- all release-facing links pass validation;
- no contradictory installation, configuration, backup, restore, upgrade, or rollback instructions remain;
- English-only normative documents contain no Japanese-only requirements;
- the documentation inventory has no unresolved `MERGE`, `DELETE`, or release-blocking `ARCHIVE` action;
- the release summary clearly identifies supported platforms, compatibility, known limitations, and upgrade requirements.

### 9. Change control after v1.0.0

For v1.x:

- new user-facing or routine-operator documents must be classified under this policy when introduced;
- Tier 1 changes require synchronized Japanese translation;
- internal technical documents default to English-only;
- changing a document's classification requires a pull request that updates this policy or the documentation inventory;
- English remains normative until a future major-version policy explicitly changes it.

---

## 日本語

### 1. 目的

本方針は、Lingonberryのどの文書を英日併記とし、どの文書を英語のみとするか、英日併記ファイルをどのような形式で管理するか、v1.0.0公開前にどの文書作業を完了させる必要があるかを定めます。

目的は次のとおりです。

- 英語利用者と日本語利用者の双方に、利用開始のための入口を提供すること。
- protocol、実装、保守に関する用語をsource codeと公開interfaceに一致させること。
- 日本語だけに存在する要件や動作が、非公式の並行仕様になることを防ぐこと。
- 翻訳保守を、導入、通常運用、security、release移行に実質的な価値がある文書へ限定すること。
- 文書の完成をv1.0.0 release gateとして明示すること。

### 2. 正本となる言語

Lingonberryの文書、仕様、互換性宣言、command contract、release evidenceでは、英語を正本とします。

英日併記文書の日本語部分は英語部分の翻訳です。英語部分に存在しない要件、例外、手順、保証を日本語部分だけに追加してはいけません。

source code identifier、CLI command、configuration key、API field、event名、error code、file path、machine-readable valueは翻訳しません。

### 3. 英日併記文書の形式

英日併記文書は、同一ファイル内で英語を先、日本語を後に配置します。

推奨構成:

```markdown
# Document Title / 文書タイトル

> English is normative. Japanese is a translation.
> 英語を正本とし、日本語はその翻訳です。

## English

...

---

## 日本語

...
```

英語部分と日本語部分では、見出しの順序と意味上の範囲を一致させます。commandやconfiguration exampleは、地域固有のpathや説明が明示的に必要な場合を除き、技術的に同一でなければなりません。

### 4. 文書の分類

#### Tier 1: 英日併記必須

v1.0.0公開前に、以下を英日併記にします。

- `README.md`
- `SECURITY.md`
- project概要または紹介文書
- Getting Started guide
- installationおよび初期configuration guide
- 標準的なstart、stop、restart、health、readiness手順
- backup／restore runbook
- upgrade／rollback runbook
- 基本troubleshooting runbook
- v1.0.0 release概要およびoperator向けmigration notes
- 本文書方針

#### Tier 2: 必須部分を絞って英日併記

以下は概要部分を英日併記とし、詳細な技術内容は英語のみでも構いません。

- `CONTRIBUTING.md`
- high-level architecture overview
- compatibility／support policy
- 公開向けroadmap概要
- migration overview

#### Tier 3: 英語のみ

以下は、将来の決定により分類が明示的に変更されない限り、英語のみとします。

- 詳細architectureおよびstorage internals
- protocol、API、CLI、configuration reference
- source code commentおよび自動生成reference
- developer setup、CI、test matrix、debugging文書
- qualification、formal soak、crash matrix、disk pressure、reference host手順
- release evidence、manifest、machine-readable report、maintainer向け内部runbook
- issue template、pull request template、commit message、release automation文書

formal soakおよびqualification文書は、英日併記する最小operator runbookには含めません。英日併記の概要からリンクすることはできますが、実行手順と正本は英語のみとします。

### 5. 英日併記する最小operator runbook

英日併記のoperator向け文書だけで、新しいoperatorが内部qualification文書を読まずに次を実行できる状態にします。

1. Lingonberryの目的とsupported deployment boundaryを理解する。
2. supported packageまたはbinaryをinstallする。
3. 最小configurationを作成して検証する。
4. supported serviceをstart、stop、restartする。
5. health、readiness、logを確認する。
6. 適用可能な場合、最小objectをpublishおよびretrieveする。
7. backupを作成してverifyする。
8. isolated restoreを実行または計画する。
9. storageおよびindex状態をverifyする。
10. supported previous releaseからupgradeする。
11. upgrade失敗時に安全にrollbackまたは停止する。
12. 基本的な障害診断を行い、停止やescalationが必要な条件を判断する。

### 6. 翻訳同期

Tier 1文書の英語部分を変更する場合、同じpull requestで対応する日本語部分も更新しなければなりません。

例外的に翻訳を更新できない場合は、文書へ明示的な翻訳未更新warningを付け、release前に解決するblocking follow-up issueを作成します。Tier 1文書に翻訳未更新warningが残った状態でv1.0.0を公開してはいけません。

英語部分と日本語部分では、以下を一致させます。

- version番号
- support status
- command名とargument
- file path
- configuration keyとprecedence
- compatibility promise
- safety warning
- backup、restore、upgrade、rollbackの順序
- release blocking condition

### 7. 文書構成

v1.xで推奨する文書構成は次のとおりです。

```text
README.md
SECURITY.md
CONTRIBUTING.md
docs/
├── DOCUMENTATION_POLICY.md
├── OVERVIEW.md
├── getting-started/
│   └── GETTING_STARTED.md
├── runbooks/
│   ├── INSTALLATION.md
│   ├── START_STOP.md
│   ├── HEALTH_CHECK.md
│   ├── BACKUP_RESTORE.md
│   ├── UPGRADE.md
│   └── TROUBLESHOOTING.md
├── architecture/
├── reference/
├── development/
├── roadmap/
├── qualification/
└── releases/
```

renameによるrelease riskが大きい場合は既存fileを維持して構いません。ただし、documentation inventoryには各文書の分類と将来の移動先を記録します。

### 8. v1.0.0公開前に必要な文書作業

以下はv1.0.0公開前に完了させる必要があります。

#### 方針と棚卸し

- [x] 正本言語と翻訳方針を採用する。
- [ ] 既存のMarkdownおよびoperator向けtext文書をすべて棚卸しする。
- [ ] 各文書を`BILINGUAL_REQUIRED`、`BILINGUAL_SCOPED`、`ENGLISH_ONLY`、`MERGE`、`ARCHIVE`、`DELETE`に分類する。
- [ ] 重複、obsolete、矛盾、未リンクの文書を特定する。
- [ ] 各文書のtarget pathと対応内容を記録する。

#### Tier 1英日併記文書

- [ ] `README.md`を英語先、日本語後の形式に統一する。
- [ ] `SECURITY.md`を英語先、日本語後の形式に統一する。
- [ ] 英日併記のproject overviewを作成または整理する。
- [ ] 英日併記のGetting Started guideを作成または整理する。
- [ ] 英日併記の最小installation／configuration runbookを整備する。
- [ ] 英日併記の最小start／stop／restart runbookを整備する。
- [ ] 英日併記の最小health／readiness／log runbookを整備する。
- [ ] 英日併記の最小backup／restore runbookを整備する。
- [ ] 英日併記の最小upgrade／rollback runbookを整備する。
- [ ] 英日併記の最小troubleshooting runbookを整備する。
- [ ] 英日併記のv1.0.0 release概要とmigration notesを作成する。

#### 英語のみの文書整理

- [ ] 言語が混在している内部architecture／reference文書を英語へ統一する。
- [ ] qualification、soak、evidence、CI、maintainer向け手順を英語へ統一する。
- [ ] 英語のみの文書から、日本語だけに存在するnormative requirementを除去する。
- [ ] 歴史的な日本語資料を残す場合は、archiveかつnon-normativeであることを明記する。

#### 整合性とnavigation

- [ ] user、operator、developer、maintainer向け文書をまとめたdocumentation indexを追加する。
- [ ] 文書の移動やrenameで影響を受けたlinkを修正する。
- [ ] terminology、product名、version表記、command spelling、path表記を統一する。
- [ ] supported operational actionごとに、正本となる手順を1つだけにする。
- [ ] READMEおよびGetting Startedに詳細reference contractを重複記載しない。

#### 自動検査

- [ ] 必須英日併記文書とlanguage section markerを確認するCIを追加する。
- [ ] release-facing Markdown文書のlink checkを追加する。
- [ ] 必要な箇所でversion番号およびfrozen candidate identifierの不一致を検査する。
- [ ] Tier 1文書の翻訳未更新を検知するwarningまたはreview checkを追加する。
- [ ] documentation policyとinventoryをdocumentation freeze checkの対象に加える。

#### Release gate

v1.0.0の文書準備完了には、以下をすべて満たす必要があります。

- Tier 1文書がすべて存在し、英語と日本語が同期している。
- Tier 1文書に翻訳未更新warningが残っていない。
- release-facing linkがすべて検証を通過している。
- installation、configuration、backup、restore、upgrade、rollbackについて矛盾した手順が残っていない。
- 英語のみの正本文書に、日本語だけの要件が存在しない。
- documentation inventoryに未解決の`MERGE`、`DELETE`、またはrelease blockingな`ARCHIVE` actionがない。
- release概要にsupported platform、compatibility、known limitation、upgrade requirementが明記されている。

### 9. v1.0.0以降の変更管理

v1.xでは次の規則を適用します。

- 新しいuser-facing文書または通常operator向け文書を追加する際は、本方針に従って分類する。
- Tier 1の変更では、日本語訳も同期して更新する。
- 内部技術文書は英語のみをdefaultとする。
- 文書分類を変更する場合は、本方針またはdocumentation inventoryを更新するpull requestを必要とする。
- 将来のmajor versionで方針を明示的に変更しない限り、英語を正本とする。
