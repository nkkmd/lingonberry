# Lingonberry v0.5.0 Release Roadmap

**Status: active** | **Target: v0.5.0** | **Last updated: 2026-07-19**

## 1. 目的

v0.5.0では、quarantine／maintenance系に偏っていた実装を通常のknowledge object lifecycleへ統合し、単一ノードで次の経路をend-to-endに成立させます。

```text
receive
→ parse
→ schema validation
→ semantic validation
→ identity／signature verification
→ duplicate／conflict classification
→ canonical storage または quarantine
→ index update
→ retrieve／query
```

release gateは次の自動化されたsmoke scenarioです。

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

## 2. 現在の実装inventory

### 2.1 既に存在する部品

- `packages/validation`
  - publish request validation
  - knowledge object validation
  - semantic validation
  - identity／signature validation status
  - acceptance policyによるaccept／reject／defer分類
- `packages/core`
  - `StorageBackend`
  - SQLite canonical catalog
  - append-only raw wire log
  - canonical ID取得
  - subscription／replay
  - persistent quarantine
  - duplicate検出
  - canonical IDまたはcarrier identity衝突時の`LB_OBJECT_CONFLICT`
- `packages/indexer`
  - storage replayからのindex構築
  - relation／lineage／provenance projection
- `packages/relay`
  - CLI publish／get／list／subscribe／replay
  - HTTP publish／read経路
  - rebuild-index command
  - quarantine operations

### 2.2 v0.5.0で解消する構造的不足

1. ingestion orchestrationが`packages/relay/src/main.rs`へ集中している
2. CLIとHTTPで結果分類・error responseが共通型として固定されていない
3. validation errorがstableなversioned machine-readable contractになっていない
4. duplicateはbool、conflictはstorage errorであり、lifecycle result taxonomyとして統合されていない
5. storage commitとindex updateのpartial failure semanticsが固定されていない
6. index rebuildは存在するがcatch-up／consistency verificationの契約が不足している
7. restartを含む通常経路の単一E2E smoke scenarioがない

## 3. 固定する責務境界

### 3.1 validationとacceptance

- validation reportは入力の妥当性だけを表す
- acceptance policyは`accept`／`reject`／`defer`を決定する
- validation未通過objectをcanonical storageへ保存しない
- `defer`はpersistent quarantineへ保存し、canonical storageへ保存しない

### 3.2 canonical storageとindex

- canonical storageをsemantic source of truthとする
- indexは再構築可能な派生状態とする
- storage commit成功後のindex update失敗をpublish全体の完全成功として返さない
- partial index stateは検出可能にし、catch-upまたはrebuildを要求する
- corruption、I/O error、contradictory stateを黙って欠落扱いしない

### 3.3 duplicateとconflict

- exact duplicateは成功したidempotent replayとして分類する
- 同一canonical IDでcanonical contentが異なる場合はconflictとする
- 同一carrier identityでcanonical contentが異なる場合はconflictとする
- conflictを既存recordの上書きで解決しない
- duplicate／conflict分類をCLI、HTTP、test fixtureで共通化する

## 4. 実装工程

## Phase 1: ingestion contract

- `packages/core`に通常publish経路のorchestratorを置く
- request parse後のvalidation、acceptance、finalization、storage、quarantineを単一結果型へ統合する
- relayはI/O adapterとしてorchestratorを呼び出す
- CLI／HTTPで同じstatus、code、detailsを返す

想定result分類：

```text
stored
duplicate
deferred
rejected
conflict
failed
```

## Phase 2: versioned error contract

- error contract versionを導入する
- machine-readable `code`とoperator向け`message`を分離する
- validation detailsをboundedなstructured dataとして返す
- protocol／schema／semantic／identity／signature／storage／index failureを分類する
- free-form error textを識別子やmetricsの軸にしない

## Phase 3: index consistency

- storage inventoryとindex inventoryの比較を実装する
- missing／extra／mismatched entryを区別する
- storageからのdeterministic catch-upを実装する
- full rebuildを保持する
- verification中のcorruption／I/O errorをfail closedで扱う

## Phase 4: public API統合

- publish responseを統一する
- ID取得 responseを統一する
- basic query responseを統一する
- not-found、rejected、deferred、duplicate、conflict、partial-index failureを曖昧なく区別する

## Phase 5: end-to-end smoke

次を一時state directory上で自動化します。

1. valid objectをpublish
2. stored resultを確認
3. IDでretrieve
4. basic queryで発見
5. processをrestart
6. ID取得とqueryを再実行
7. storage／index consistencyをverify
8. index欠落を注入
9. catch-upまたはrebuild
10. consistency回復をverify

追加scenario：

- invalid requestのreject
- policy deferとpersistent quarantine
- exact duplicate
- canonical ID conflict
- carrier identity conflict
- malformed raw log
- partial index update
- storage／index I/O failure

## 5. Issue分解

親Issue: #76

推奨実装順：

1. ingestion result／error contractとorchestrator
2. duplicate／conflict contract tests
3. index consistency verification
4. index catch-up
5. CLI／HTTP response統合
6. restartを含むE2E smoke
7. release checklist、release note、CHANGELOG、current status同期

## 6. 非スコープ

- protocol conformance suiteの完成（v0.6.0）
- storage format migration framework（v0.7.0）
- operator CLIの全面統合（v0.8.0）
- vector search／AI integration
- multi-node consistency／distributed coordination
- automatic conflict resolution

## 7. 完了条件

- 通常publish経路が共通orchestratorを通る
- CLIとHTTPが共通のversioned result／error contractを返す
- duplicate／conflictがdeterministicに分類される
- canonical storageを正本としてindex verify／catch-up／rebuildできる
- restartとpartial index updateを含むE2E smokeがCIで成功する
- ambiguous、corrupt、partial stateを成功扱いしない
- `CURRENT_IMPLEMENTATION_STATUS.md`、release checklist、release note、CHANGELOGが同期する

## 8. 絶対に崩さない安全境界

1. validation未通過objectをcanonical storageへ保存しない
2. deferされたobjectをcanonical storageへ保存しない
3. canonical storageよりindexを優先しない
4. conflict時に既存objectを上書きしない
5. corruptionとI/O errorをnot-foundとして扱わない
6. partial index updateを完全成功として返さない
7. archive segmentとimmutable evidence ledgerを変更しない
8. same-host lockをdistributed coordinationとして扱わない
9. metricsへidentifier、digest、record ID、path、free-form errorを出さない
