# Lingonberry

[English](#english) | [日本語](#japanese)

> English is the normative version of this document. The Japanese section is a translation. If the two sections differ, the English section takes precedence.
>
> 英語版がこの文書の正本です。日本語部分は翻訳です。内容に差異がある場合は英語版を優先します。

## English

Lingonberry is a Rust workspace for publishing, validating, storing, retrieving, querying, indexing, and operating canonical knowledge objects. Canonical storage is the source of truth; indexes and effective views are derived, verifiable, and rebuildable.

The workspace also provides persistent quarantine, verified backup and isolated restore, verified replacement workflows, proof-bound retention cleanup, explicit storage-format migration, Ed25519-authenticated publishing, and a production-oriented single-node operator surface.

### Release status

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

Candidate qualification and the 16-procedure documentation walkthrough have passed and their artifacts were independently verified. Repository-side preparation is complete. Privileged reference-host preflight may proceed, but the formal 72-hour soak remains blocked until that preflight passes.

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

Lingonberryは、canonical knowledge objectの公開、検証、保存、取得、検索、索引、および運用を行うRust workspaceです。canonical storageを正本とし、indexとeffective viewは派生物として検証・再構築できます。

persistent quarantine、検証済みbackupとisolated restore、検証済みreplacement、proof-bound retention cleanup、明示的なstorage-format migration、Ed25519認証付きpublish、single-node向けoperator surfaceも提供します。

### リリース状況

最新の公開済みreleaseは`v0.9.0`です。stable single-node releaseである`v1.0.0`は最終qualification中であり、まだ公開されていません。

固定済みのversion更新前candidateは次のcommitです。

```text
8c6b48082205a3af555130eec1f3e7d2ac8811fe
```

candidateからbuildしたbinary identityは次のとおりです。

```text
lingonberry-storage  737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507
lingonberry-relay    23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c
```

candidate qualificationと16手順のdocumentation walkthroughはPASSし、artifactも独立検証済みです。repository側の準備は完了しており、privileged reference-host preflightへ進めます。ただし正式72時間soakはreference-host preflightがPASSするまで開始しません。

candidate決定後のevidence／documentation commitはcandidate自体を変更しません。version更新、`v1.0.0` tag、GitHub Release、最終publication validationは未完了です。

v1.0.0作業の確認・再開時は、まず次を参照してください。

- [v1.0.0 qualification status](docs/roadmap/V1_0_QUALIFICATION_STATUS.md)
- [v1.0.0 release evidence](docs/roadmap/V1_0_RELEASE_EVIDENCE.md)
- [v1.0.0 documentation walkthrough](docs/roadmap/V1_0_DOCUMENTATION_WALKTHROUGH.md)
- [v1.0.0 qualification plan](docs/roadmap/V1_0_QUALIFICATION_PLAN.md)
- [v1 compatibility policy](docs/architecture/V1_COMPATIBILITY_POLICY.md)
- [v1.0.0 security diff review](docs/security/V1_0_SECURITY_DIFF_REVIEW.md)
- [v1.0.0 soak plan](docs/roadmap/V1_0_SOAK_PLAN.md)
- [documentation policy](docs/DOCUMENTATION_POLICY.md)
- [documentation inventory](docs/DOCUMENTATION_INVENTORY.md)

dry runやvirtual-time rehearsalはtoolingとevidence形式の確認であり、最終release evidenceではありません。qualificationは固定candidateと記録済みbinary digestに結び付いている必要があります。

### 安全境界

Lingonberryは、曖昧、不完全、未対応、未認証、または矛盾した状態をerrorとして扱います。特に次を守ります。

- validationに失敗したobjectをcanonical storageへ保存しない
- publisher signatureをacceptance、duplicate／conflict判定、quarantine、raw append、canonical storageより前に検証する
- malformed、invalid、検証不能なsignatureはfail closedで拒否する
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
- 非空legacy migrationはverified backup evidenceなしで開始しない
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
packages/identity     identity／signature primitive
packages/validation   validation rule
packages/core         authenticated ingestion、quarantine、replacement、cleanup
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

担当する役割に応じて次を参照してください。

- [Relay Quickstart](docs/operations/RELAY_QUICKSTART.md)
- [Publisher Quickstart](docs/developers/PUBLISHER_QUICKSTART.md)
- [Storage Node Quickstart](docs/operations/STORAGE_NODE_QUICKSTART.md)
- [Developer Documentation](docs/developers/README.md)

### Production向け運用

正式reference platformは次の構成です。

```text
Ubuntu Server 24.04 LTS / x86_64 / systemd
```

production向け導入ではrelease build済みbinaryとhardened systemd unitを使用します。次から確認してください。

- [Operations index](docs/operations/README.md)
- [v1.0 Operator Runbook](docs/operations/V1_0_OPERATOR_RUNBOOK.md)
- [Supported Platforms](docs/operations/SUPPORTED_PLATFORMS.md)
- [Operator CLI Contract](docs/operations/OPERATOR_CLI_CONTRACT.md)
- [Upgrade and Rollback](docs/operations/V0_8_UPGRADE_AND_ROLLBACK.md)

v1.0 operator runbookはpre-release single-node運用ガイドです。これはv1.0.0の公開完了を意味しません。

### 検証

```bash
cargo fmt --all -- --check
cargo clippy --workspace --lib -- -D warnings
cargo clippy --workspace --bins -- -D warnings -A dead-code
cargo clippy --workspace --tests -- -A warnings
cargo test --workspace
```

JavaScript contract testとexternal conformance suiteは`.github/workflows/ci.yml`で実行します。candidate qualification、documentation integrity、documentation inventory、walkthrough、soak rehearsalは専用workflowで確認します。

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
