# Lingonberry v1.0までのロードマップ

**Status: active** | **Starting point: v0.4.0** | **Last updated: 2026-07-17**

## 1. 目的

この文書は、v0.4.0以降のLingonberryを「一応の完成」と呼べるv1.0.0へ進めるためのrelease-level roadmapです。

ここでいうv1.0.0は、次の状態を指します。

> 単一ノード構成でcanonical knowledge objectを受信、検証、保存、索引、取得し、quarantine、backup／restore、世代置換、retention cleanup、障害回復、upgradeを第三者が文書に従って継続運用できる安定版。

この文書は各機能の詳細設計を置き換えません。詳細な機能フェーズは[実装ロードマップ](./IMPLEMENTATION_ROADMAP.md)、運用面は[運用準備ロードマップ](./OPERATIONAL_READINESS_ROADMAP.md)、直近の実装状態は[現在の実装状況](./CURRENT_IMPLEMENTATION_STATUS.md)を正本とします。

## 2. 現在地: v0.4.0

v0.4.0までに、次の基盤が成立しています。

- canonical protocol、identity、validation、core、indexer、relay、storageのRust workspace分離
- persistent quarantine lifecycle
- verified backup／restore
- generation-based replacement transaction
- durable journal、resume、rollback、recovery classification
- deterministic retention evaluation
- canonical cleanup plan／proof
- exact subject bindingとcurrent-state revalidation
- same-filesystem tomb preparationとsealed inventory
- path-level durable progressを持つverified cleanup transaction
- operator-controlled double opt-in authorization
- runbook、failure-point inventory、crash matrix、smoke procedure

この段階で、破壊的処理と障害回復の安全性は大きく前進しています。一方、v1.0に向けては通常利用経路、外部契約、upgrade、導入・運用体験、release hardeningを統合して完成させる必要があります。

## 3. v1.0の境界

### 3.1 v1.0で保証するもの

- canonical knowledge objectの受信とdeterministic validation
- identity、signature、digestの検証
- durable canonical storage
- ID取得とbasic query／index
- duplicate／conflictの明示的分類
- persistent quarantineとoperator review
- verified backup／restore
- index verification／rebuild
- verified generation replacement
- proof-bound retention cleanup
- crash recoveryとcontradictory-state rejection
- versioned storage migration
- documented installation、configuration、operation、upgrade
- protocol v1、storage format v1、public APIの互換性方針

### 3.2 v1.0の必須条件にしないもの

次はv1.0以降の独立した機能として扱います。

- distributed lock、consensus、multi-node strong consistency
- Kubernetes operator
- OAuth／OIDCの完全統合
- per-record ACL
- remote backup service
- secure erase
- vector searchやAI integration
- 複数carrier／transportの完全対応
- terminal cleanup workspaceの自動retention処理

これらをv1.0のrelease gateへ含めず、単一ノードの安定運用を先に完成させます。

## 4. Release roadmap

## v0.5.0: 通常のobject lifecycle

### 目的

quarantineとmaintenanceに偏らず、通常のpublish、validate、store、index、readをend-to-endで完成させます。

### 主な工程

1. ingestion pipelineを固定する
   - receive
   - parse
   - schema validation
   - semantic validation
   - identity／signature verification
   - duplicate／conflict classification
   - canonical storageまたはquarantine
   - index update
2. public read／write APIを整理する
3. validation failureをversioned machine-readable error codeにする
4. duplicateとconflictの規則を固定する
5. storageを正本としたindex rebuild／catch-up／consistency verificationを実装する
6. restartとpartial index updateを含むend-to-end smoke testを追加する

### 完了条件

```text
publish
→ validate
→ store
→ retrieve
→ query
→ restart
→ query
→ consistency verification
```

この一連の処理が単一の自動化されたsmoke scenarioで成功し、失敗時は曖昧な状態を成功扱いしないこと。

## v0.6.0: Protocol contractとconformance

### 目的

Rust実装内部ではなく、第三者が互換実装を作れる外部契約を固定します。

### 主な工程

1. protocol specificationを整理する
   - canonical envelope
   - canonical serialization
   - identifier規則
   - digest対象
   - signature対象
   - timestamp semantics
   - relation／lineage
   - replacement／withdrawal
   - validation levels
2. version軸を分離する
   - protocol version
   - schema version
   - storage format version
   - journal version
   - proof version
   - API version
3. compatibility matrixを作成する
4. valid、invalid、boundary、signature、digest、conflict、legacy fixtureを固定する
5. JavaScript contract testsを外部実装向けconformance suiteへ発展させる

### 完了条件

Rust以外の最小クライアントが、仕様書とfixtureのみを使ってLingonberryが受理するobjectを生成し、digestとsignatureの期待値を再現できること。

## v0.7.0: Storage migrationとupgrade保証

### 目的

新規導入だけでなく、既存データを保ったまま継続的に更新できる状態を作ります。

### 主な工程

1. data directoryのstorage format manifestを導入する
2. migration frameworkを実装する
   - inspect
   - plan
   - verified backup
   - migrate
   - verify
   - commit
   - resume／rollback
3. upgrade／downgrade policyを文書化する
4. unknown newer formatをfail closedで拒否する
5. v0.4.0相当のfixtureからupgradeするintegration testを追加する
6. deprecated configurationの廃止予定と移行手順を固定する

### 完了条件

v0.4.0相当の永続データをv0.7.0へ移行し、read、write、index verification、backup、crash recoveryが成功すること。

## v0.8.0: Operational readiness

### 目的

開発者ではないoperatorが、runbookに従って導入・診断・回復できる状態にします。

### 主な工程

1. operator CLIを統合する
   - `serve`
   - `status`
   - `doctor`
   - `verify`
   - `backup create／verify`
   - `restore plan／apply`
   - `index verify／rebuild`
   - quarantine operations
   - replacement operations
   - cleanup operations
2. read-only `doctor`でconfiguration、permission、storage layout、journal、pointer、index、archive、evidence、workspace、disk conditionを検査する
3. configuration file、environment variable、CLI optionのprecedenceを固定する
4. health、readiness、bounded-cardinality metricsを完成させる
5. isolated restoreを含むdisaster recovery drillを自動化する
6. Linux上の正式な起動方法を少なくとも1つ提供する
   - systemd unit、または
   - container image

### 完了条件

新しいoperatorがREADMEとrunbookのみを使って、install、start、publish、backup、restore、verify、index rebuild、quarantine inspection、failure diagnosisを実行できること。

## v0.9.0: Release candidate hardening

### 目的

新機能追加を停止し、v1.0の公開契約を凍結して安定性を検証します。

### 主な工程

1. Rust public API audit
2. protocol、API、storage formatのfreeze candidate作成
3. security review
   - path traversal
   - symlink handling
   - oversized／deeply nested input
   - malformed serialization
   - signature verification bypass
   - authorization ordering
   - information leakage
   - TOCTOU
   - disk-full／I/O failure
4. fuzzing／property testing
   - parser
   - validator
   - identifier
   - digest verifier
   - journal parser
   - recovery classifier
   - index／segment reader
5. long-running soak test
6. supported platformとpackagingを固定する
7. installation、configuration、protocol、API、security、upgrade、backup／restore、operations、troubleshooting文書をrelease candidateとして凍結する

### 完了条件

v0.9.0-rcを実環境または実環境相当で継続運用し、v1.0を妨げるcritical／high severity defectが残っていないこと。

## v1.0.0: Stable single-node release

### Release gate

次をすべて満たした時点でv1.0.0を公開します。

- object lifecycleのend-to-end testが成功する
- protocol conformance suiteが成功する
- supported legacy stateからのmigration testが成功する
- backup／isolated restore drillが成功する
- index rebuildとconsistency verificationが成功する
- replacement／cleanup crash matrixが成功する
- security reviewのrelease blockerがない
- public protocol、API、storage formatのcompatibility policyが文書化されている
- installation、configuration、operation、upgrade、recovery文書が揃っている
- release candidateのsoak testが完了している

### v1.x compatibility policy

- protocol v1のbreaking changeはv2.0まで導入しない
- storage format v1の読み取り互換性をv1.xで維持する
- breaking migrationは明示的なplan、verified backup、operator authorizationを要求する
- deprecated機能には移行期間と削除versionを設定する
- unknown、corrupt、contradictory stateはfail closedで扱う

## 5. 優先順位

v0.4.0以降の優先順位は次のとおりです。

1. 通常のobject lifecycle
2. protocolと外部APIの固定
3. storage migrationとupgrade
4. installation、configuration、doctor、recovery
5. security review、fuzz、soak test
6. v1.0 compatibility declaration

高度な分散機能、検索機能、AI連携を先行させず、まず単一ノードの安全性と第三者運用可能性を完成させます。

## 6. v1.0以降

### v1.1以降の候補

- terminal cleanup workspace retention policy
- remote backup adapter
- backup encryption／signing
- richer query language
- relation traversal
- alternative storage backend
- additional carrier／transport adapter
- replication experiment

### v2.0候補

- distributed coordination
- multi-node replication
- distributed lockまたはconsensus
- breaking protocol revision
- per-record authorization
- legacy compatibility removal

## 7. 絶対に崩さない安全境界

1. validation未通過objectをcanonical storageへ保存しない
2. corruptionとI/O errorを黙って無視しない
3. same-host lockをdistributed lockとして扱わない
4. archive segmentを上書き、変更、削除しない
5. immutable evidence ledgerを変更しない
6. active generationをcleanup対象にしない
7. filesystem timestampだけでeligibilityを判断しない
8. wildcard、implicit-all、partial selectionをdestructive applyに使わない
9. proofとcurrent stateが一致しなければmutationを開始しない
10. symbolic linkやunsupported entry typeをfollow／acceptしない
11. contradictory recovery stateを自動修復または成功扱いしない
12. irreversible processing開始後にrollbackを案内しない
13. scheduled／unattended destructive cleanupを暗黙に導入しない
14. metricsへpath、identifier、digest、record ID、free-form errorを出さない

## 8. 文書更新規則

- release scopeが変わった場合はこの文書を更新する
- 個別機能の詳細は対応するversion roadmap、operations document、ADRへ記録する
- 実装完了状態は`CURRENT_IMPLEMENTATION_STATUS.md`へ反映する
- issue分解は`IMPLEMENTATION_BACKLOG.md`またはversion-specific issueへ記録する
- v1.0 release gateの判定はrelease checklist、CI、smoke／soak evidenceを正本とする
