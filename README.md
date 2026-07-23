# Lingonberry

[English](#english) | [日本語](#日本語)

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

## English

Lingonberry is a Rust workspace for publishing, validating, storing, retrieving, querying, indexing, and operating canonical knowledge objects. Canonical storage is the source of truth; indexes and effective views are derived, verifiable, and rebuildable.

The workspace also provides persistent quarantine, verified backup and isolated restore, verified replacement workflows, proof-bound retention cleanup, explicit storage-format migration, and a production-oriented single-node operator surface.

### Release status

The latest published release is `v0.9.0`. The stable single-node `v1.0.0` release is still under qualification and has not been published.

The designated pre-version qualification candidate is:

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

Evidence and tooling commits made after that candidate do not redefine the candidate. The formal 72-hour soak, version update, `v1.0.0` tag, and GitHub Release remain pending.

Read these sources first when reviewing or resuming v1.0.0 work:

- [v1.0.0 qualification status](docs/roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 qualification plan](docs/roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1 compatibility policy](docs/architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 security diff review](docs/security/V1_0_SECURITY_DIFF_REVIEW.md)
- [v1.0.0 documentation policy](docs/DOCUMENTATION_POLICY.md)
- [documentation inventory](docs/DOCUMENTATION_INVENTORY.md)
- [v1.0.0 soak plan](docs/roadmap/V1_0_SOAK_PLAN.md)
- [v1.0.0 release evidence](docs/roadmap/V1_0_RELEASE_EVIDENCE.md)

A dry run or virtual-time rehearsal validates tooling and evidence formats only. It is not final release evidence. Final qualification must remain bound to the designated candidate and candidate-built binary digests.

### Safety boundaries

Lingonberry treats ambiguous, incomplete, unsupported, or contradictory state as an error. In particular:

- validation failures do not enter canonical storage;
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
packages/identity     identity primitives
packages/validation   validation rules
packages/core         ingestion, quarantine, replacement, and cleanup logic
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

See [Relay Quickstart](docs/operations/RELAY_QUICKSTART.md) for the complete development path.

### Production-oriented operation

The formal reference platform remains:

```text
Ubuntu Server 24.04 LTS / x86_64 / systemd
```

Production-oriented installation uses release-built binaries and hardened systemd units. Start with:

- [Operations index](docs/operations/README.md)
- [v1.0 Operator Runbook](docs/operations/V1_0_OPERATOR_RUNBOOK.md)
- [Supported Platforms](docs/operations/SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](docs/operations/OPERATOR_CLI_CONTRACT.md)
- [Upgrade and Rollback](docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md)

The v1.0 operator runbook is now the pre-release single-node operating guide. It does not imply that v1.0.0 qualification or publication has completed.

Common storage operator commands include:

```bash
lingonberry-storage config
lingonberry-storage health
lingonberry-storage ready
lingonberry-storage status
lingonberry-storage doctor
lingonberry-storage verify
lingonberry-storage metrics
lingonberry-storage backup create /var/backups/lingonberry/manual-backup
lingonberry-storage backup verify /var/backups/lingonberry/manual-backup
lingonberry-storage restore plan /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
lingonberry-storage restore apply /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
lingonberry-storage index verify
lingonberry-storage index rebuild
```

Storage migration remains separately operator-controlled:

```bash
lingonberry-storage-migrate inspect
lingonberry-storage-migrate plan
lingonberry-storage-migrate apply
lingonberry-storage-migrate status
lingonberry-storage-migrate resume
lingonberry-storage-migrate rollback
```

### Validation

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -A warnings
cargo test --workspace
```

JavaScript contract tests and the external conformance suite are run by `.github/workflows/ci.yml`. Candidate-bound qualification, documentation integrity, and documentation inventory checks are maintained in dedicated workflows.

### Documentation

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

## 日本語

Lingonberryは、canonical knowledge objectの公開、検証、保存、取得、検索、索引、および運用を行うRust workspaceです。canonical storageを正本とし、indexとeffective viewは派生物として検証・再構築できます。

persistent quarantine、検証済みbackupとisolated restore、検証済みreplacement、proof-bound retention cleanup、明示的なstorage-format migration、single-node向けoperator surfaceも提供します。

### リリース状況

最新の公開済みreleaseは`v0.9.0`です。stable single-node releaseである`v1.0.0`は資格確認中であり、まだ公開されていません。

version更新前の指定qualification candidateは次のcommitです。

```text
f9543019f2c219aea3b085ff90f2da201b268a48
```

candidate決定後に追加されたevidenceやtoolingのcommitは、candidate自体を変更しません。正式72時間soak、version更新、`v1.0.0` tag、GitHub Releaseは未実施です。

v1.0.0作業の確認・再開時は、まず次を参照してください。

- [v1.0.0 qualification status](docs/roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 qualification plan](docs/roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1 compatibility policy](docs/architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 security diff review](docs/security/V1_0_SECURITY_DIFF_REVIEW.md)
- [v1.0.0 documentation policy](docs/DOCUMENTATION_POLICY.md)
- [documentation inventory](docs/DOCUMENTATION_INVENTORY.md)
- [v1.0.0 soak plan](docs/roadmap/V1_0_SOAK_PLAN.md)
- [v1.0.0 release evidence](docs/roadmap/V1_0_RELEASE_EVIDENCE.md)

dry runやvirtual-time rehearsalはtoolingとevidence形式の確認であり、最終release evidenceではありません。最終qualificationは、指定candidateとcandidateからbuildしたbinary digestに結び付いている必要があります。

### 安全境界

Lingonberryは、曖昧、不完全、未対応、または矛盾した状態をerrorとして扱います。特に次を守ります。

- validationに失敗したobjectをcanonical storageへ保存しない
- conflictで既存canonical recordを上書きしない
- Transition Objectで元のKnowledge Objectを書き換えたり削除したりしない
- 未許可または未知のtransitionをeffective viewへ反映しない
- 複数のauthorized headをtimestampや任意のidentifier順で解決しない
- target不在transitionはevidenceとして保持し、reevaluationまで適用しない
- 派生処理だけが失敗した場合にcanonical storage commitを失敗へ書き換えない
- stale workerが新しいderived checkpointを上書きできない
- 不完全なevidenceでlast-known-good semantic checkpointを上書きしない
- stale effective viewをcurrentと表示しない
- 通常起動時にimplicit storage migrationを行わない
- 未知の新しいstorage formatを変更しない
- 非空legacy migrationは、確認対象source stateに結び付いたverified backup evidenceなしで開始しない
- durable verification成功前にtarget formatをcommitしない
- public diagnosticsへstorage path、row ID、stack trace、不安定な実装errorを出さない
- backupとrestoreでsymbolic linkや危険なtarget再利用を拒否する
- restoreでactive state／data directoryを上書きしない
- cleanupでarchive segmentやimmutable evidence ledgerを書き換えない
- untrusted JSONをrecursive parse前に制限する
- signature verification artifactを排他的に作成し、通常の成功・失敗経路でcleanupする
- same-host lockをdistributed lockとして扱わない
- secure eraseを保証しない

### Workspace

```text
packages/protocol     canonical protocol modelとbounded JSON parser
packages/identity     identity primitive
packages/validation   validation rule
packages/core         ingestion、quarantine、replacement、cleanup logic
packages/indexer      index lifecycle、checkpoint、verification、catch-up
packages/relay        CLI、HTTP relay、Transition、effective view、reevaluation
packages/storage      File／SQLite backend、diagnostics、recovery、migration runtime
```

### 開発用Quickstart

必要なものはGit、現在のRust toolchain、Cargo、`curl`などのHTTP clientです。

```bash
git clone https://github.com/nkkmd/lingonberry.git
cd lingonberry
cargo run -p lingonberry-relay -- capabilities
cargo run -p lingonberry-relay -- serve-http 127.0.0.1:8787
```

別のterminalで確認します。

```bash
curl -sS http://127.0.0.1:8787/v1/capabilities
curl -sS http://127.0.0.1:8787/v1/ready
```

開発用の完全な手順は[Relay Quickstart](docs/operations/RELAY_QUICKSTART.md)を参照してください。

### Production向け運用

正式reference platformは引き続き次の構成です。

```text
Ubuntu Server 24.04 LTS / x86_64 / systemd
```

production向け導入ではrelease build済みbinaryとhardened systemd unitを使用します。次から確認してください。

- [Operations index](docs/operations/README.md)
- [v1.0 Operator Runbook](docs/operations/V1_0_OPERATOR_RUNBOOK.md)
- [Supported Platforms](docs/operations/SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](docs/operations/OPERATOR_CLI_CONTRACT.md)
- [Upgrade and Rollback](docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md)

v1.0 operator runbookは、現在のpre-release single-node運用ガイドです。これはv1.0.0のqualificationや公開が完了したことを意味しません。

代表的なstorage operator commandは次のとおりです。

```bash
lingonberry-storage config
lingonberry-storage health
lingonberry-storage ready
lingonberry-storage status
lingonberry-storage doctor
lingonberry-storage verify
lingonberry-storage metrics
lingonberry-storage backup create /var/backups/lingonberry/manual-backup
lingonberry-storage backup verify /var/backups/lingonberry/manual-backup
lingonberry-storage restore plan /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
lingonberry-storage restore apply /var/backups/lingonberry/manual-backup /var/lib/lingonberry/restore-candidate
lingonberry-storage index verify
lingonberry-storage index rebuild
```

storage migrationは通常運用から分離され、operatorが明示的に制御します。

```bash
lingonberry-storage-migrate inspect
lingonberry-storage-migrate plan
lingonberry-storage-migrate apply
lingonberry-storage-migrate status
lingonberry-storage-migrate resume
lingonberry-storage-migrate rollback
```

### 検証

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -A warnings
cargo test --workspace
```

JavaScript contract testとexternal conformance suiteは`.github/workflows/ci.yml`で実行します。candidate-bound qualification、documentation integrity、documentation inventoryは専用workflowで確認します。

### 文書

- [Documentation policy](docs/DOCUMENTATION_POLICY.md)
- [Documentation inventory](docs/DOCUMENTATION_INVENTORY.md)
- [Operations index](docs/operations/README.md)
- [Roadmap index](docs/roadmap/README.md)
- [Current implementation status](docs/roadmap/CURRENT_IMPLEMENTATION_STATUS.md)
- [Roadmap to v1.0](docs/roadmap/ROADMAP_TO_V1_0.md)
- [Changelog](CHANGELOG.md)

### License

適用される条件は、各package metadataとrepository内のlicense fileを参照してください。
